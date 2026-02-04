use num_bigint::BigUint;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};

/// A single token in a version spec. For example, "1" or "foo".
#[derive(Debug, Hash, Clone)]
pub(super) enum Item {
    Int(u32),
    BigInt(BigUint),
    String(String),
}

impl Item {
    pub fn from_str(s: &str, followed_by_digit: bool) -> Self {
        let s = &s[s.bytes().take_while(|c| c == &b'0').count()..s.len()];

        match (followed_by_digit, s) {
            (true, "a") => Item::String("alpha".to_string()),
            (true, "b") => Item::String("beta".to_string()),
            (true, "m") => Item::String("milestone".to_string()),
            (_, "ga") => Item::String("".to_string()),
            (_, "final") => Item::String("".to_string()),
            (_, "release") => Item::String("".to_string()),
            (_, "cr") => Item::String("rc".to_string()),
            (_, _) => Item::String(s.to_string()),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Item::Int(0) => true,
            Item::String(s) if s.is_empty() => true,
            _ => false,
        }
    }

    /// Where this item stands in comparison to no item at all. `more_segments` is whether there are
    /// more segments after the one containing this item on the version spec we're comparing with.
    /// Yes, this is really weird and specific. It's What Maven Does™.
    pub fn better_than_nothing(&self, more_segments: bool) -> Ordering {
        match self {
            Item::Int(_) if more_segments => Ordering::Greater,
            Item::Int(0) => Ordering::Equal,
            Item::Int(_) => Ordering::Greater,
            Item::BigInt(_) => Ordering::Greater,
            Item::String(_) if more_segments => Ordering::Less,
            Item::String(s) => match s.as_str() {
                "alpha" | "beta" | "milestone" | "rc" | "snapshot" => Ordering::Less,
                "" => Ordering::Equal,
                _ => Ordering::Greater,
            },
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Item::Int(i) => f.write_str(&i.to_string()),
            Item::BigInt(i) => f.write_str(&i.to_string()),
            Item::String(s) => f.write_str(s),
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Item::Int(i), Item::Int(j)) => i == j,
            (Item::String(s), Item::String(t)) => s == t,
            _ => false,
        }
    }
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        fn ranking(s: &str) -> i8 {
            match s {
                "alpha" => 0,
                "beta" => 1,
                "milestone" => 2,
                "rc" => 3,
                "snapshot" => 4,
                "" => 5,
                "sp" => 6,
                _ => 7,
            }
        }

        match (self, other) {
            (Item::Int(i), Item::Int(j)) => i.cmp(j),
            (Item::BigInt(i), Item::BigInt(j)) => i.cmp(j),
            (Item::String(s), Item::String(t)) => {
                let s_rank = ranking(s);
                let t_rank = ranking(t);

                if s_rank == 7 && t_rank == 7 {
                    s.cmp(t)
                } else {
                    s_rank.cmp(&t_rank)
                }
            }
            (Item::Int(_), Item::BigInt(_))
            | (Item::String(_), Item::Int(_))
            | (Item::String(_), Item::BigInt(_)) => Ordering::Less,
            (Item::BigInt(_), Item::Int(_))
            | (Item::Int(_), Item::String(_))
            | (Item::BigInt(_), Item::String(_)) => Ordering::Greater,
        }
    }
}

/// A segment of Items that auto-normalizes its contents. One segment looks something like "1.0.0",
/// "foo", "foo.bar", or "1.foo.bar". `last_segment` is whether we are the last segment. This is
/// needed for comparison purposes because Maven is weird.
#[derive(Debug, Hash, Clone)]
pub(super) struct Segment {
    items: Vec<Item>,
    last_segment: bool,
}

impl Segment {
    pub fn new(mut items: Vec<Item>) -> Self {
        // Strip trailing empty items
        for i in (0..items.len()).rev() {
            if items[i].is_null() {
                items.remove(i);
            } else {
                break;
            }
        }

        Segment {
            items,
            last_segment: false,
        }
    }

    pub fn is_null(&self) -> bool {
        self.items.is_empty() || self.items.iter().all(|i| i.is_null())
    }

    pub fn set_last_segment(&mut self) {
        self.last_segment = true;
    }

    /// Where this segment stands in comparison to no segment at all
    pub fn better_than_nothing(&self) -> Ordering {
        for i in &self.items {
            let better = i.better_than_nothing(false);
            if better != Ordering::Equal {
                return better;
            }
        }

        Ordering::Equal
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(
            &self
                .items
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join("."),
        )
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl Eq for Segment {}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut left = self.items.iter();
        let mut right = other.items.iter();

        loop {
            let l = left.next();
            let r = right.next();
            let order = match (l, r) {
                (Some(li), Some(ri)) => li.cmp(ri),
                (Some(li), None) => li.better_than_nothing(!other.last_segment),
                (None, Some(ri)) => ri.better_than_nothing(!self.last_segment).reverse(),
                (None, None) => break,
            };

            if order != Ordering::Equal {
                return order;
            }
        }

        Ordering::Equal
    }
}
