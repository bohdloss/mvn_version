// To demystify the "Vec<Segment>" a bit:
// Maven version strings consist of a linear sequence of version segments. Each of these looks
// something like "1.2.3", and they can also include strings e.g. "foo.bar.baz" or just "foo". These
// are separated in their canonical form by a dash, e.g. "1.2.3-foo", but they can also be delimited
// by the boundary between an integer and a string. For example, "1.2.3foo" parses identically to
// "1.2.3-foo". For each of these segments, we use a Vec of Items (hence the nested Vecs).

mod item;

use item::Item;
use item::Segment;
use itertools::join;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::{from_utf8_unchecked, FromStr};

/// This is an implementation of Maven's ComparableVersion.
///
/// ComparableVersions are made up of segments separated by dashes, or by boundary between character
/// and digit. Each of these segments is comprised of an arbitrary number of dot-separated numbers
/// or characters. They are fully case-insensitive and trailing zeroes are discarded. For some
/// examples:
///
/// - `1.0` parses as `[[1]]`
/// - `1.0-1` parses as `[[1], [1]]`
/// - `1.0-foo` parses as `[[1], [foo]]`
/// - `1.0foo` also parses as `[[1], [foo]]`
///
/// Some strings are also treated as special qualifiers. These have priority for purposes of
/// comparison in the order listed below:
///
/// - `alpha`, or `a` when followed by a number
/// - `beta`, or `b` when followed by a number
/// - `milestone`, or `m` when followed by a number
/// - `rc` or `cr`
/// - `snapshot`
/// - `sp` (this has lower precedence than the default)
///
/// Other qualifiers are ordered lexically.
///
/// To compare ComparableVersions, use the built-in comparison and equality operators.
#[derive(Debug, Hash)]
pub struct ComparableVersion {
    orig: String,
    segments: Vec<Segment>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for ComparableVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.orig.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ComparableVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(&<String as serde::Deserialize>::deserialize(deserializer)?))
    }
}

impl Display for ComparableVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.orig)
    }
}

// Max we can *safely* handle, else we might go above 4 million
static MAX_U32_LEN: usize = 9;

impl ComparableVersion {
    /// Constructs a ComparableVersion from the given string. This function cannot fail for any
    /// reason, and will always return a valid ComparableVersion. Make sure to strip whitespace
    /// before passing your input to this function. Otherwise, whitespace is treated identically to
    /// any other text.
    pub fn new(s: &str) -> Self {
        fn parse_item(s: &str, is_digit: bool, followed_by_digit: bool) -> Item {
            // Strip leading zeroes so we won't get a small BigInt
            let bytes = s.bytes().skip_while(|b| b == &b'0').collect::<Vec<u8>>();
            let s: &str = unsafe { from_utf8_unchecked(&bytes) };

            if is_digit && s.len() <= MAX_U32_LEN {
                // This can fail if we stripped everything off
                Item::Int(s.parse().unwrap_or(0))
            } else if is_digit {
                Item::BigInt(s.parse().unwrap())
            } else {
                Item::from_str(s, followed_by_digit)
            }
        }

        let mut segments = Vec::new();
        let mut cur_segment = Vec::new();
        let lower = s.to_lowercase();

        let mut is_digit = false;
        let mut start_index = 0;

        // We can go byte-by-byte here instead of char-by-char because we transparently copy over
        // strings anyways. This is not technically compliant with the Java implementation, because
        // Java will happily accept any UTF-16 "numbery" character as a number, but if anyone is
        // using २ instead of 2 they are Doing It Wrong.
        for (i, c) in lower.bytes().enumerate() {
            match c {
                b'.' | b'-' => {
                    if i == start_index {
                        cur_segment.push(Item::Int(0));
                    } else {
                        cur_segment.push(parse_item(&lower[start_index..i], is_digit, false));
                    }

                    start_index = i + 1;

                    if c == b'-' {
                        segments.push(Segment::new(cur_segment));
                        cur_segment = Vec::new();
                    }
                }
                _ => {
                    let will_be_digit = c >= b'0' && c <= b'9';

                    if (i > start_index)
                        && ((will_be_digit && !is_digit) || (!will_be_digit && is_digit))
                    {
                        cur_segment.push(parse_item(
                            &lower[start_index..i],
                            is_digit,
                            will_be_digit,
                        ));
                        start_index = i;

                        // Boundary between digit and non-digit
                        segments.push(Segment::new(cur_segment));
                        cur_segment = Vec::new();
                    }

                    is_digit = will_be_digit;
                }
            }
        }

        if lower.len() > start_index {
            cur_segment.push(parse_item(
                &lower[start_index..lower.len()],
                is_digit,
                false,
            ));
        }

        segments.push(Segment::new(cur_segment));

        // Strip trailing empty segments
        for i in (0..segments.len()).rev() {
            if segments[i].is_null() {
                segments.remove(i);
            } else {
                break;
            }
        }

        // Set the flag on the last segment
        if let Some(seg) = segments.last_mut() {
            seg.set_last_segment();
        }

        ComparableVersion {
            orig: s.to_string(),
            segments,
        }
    }

    /// Returns the canonical representation of this version string. The canonical representation is
    /// one in which all separators between segments are converted into dashes and all shortened or
    /// aliased qualifiers are expanded. In addition, the entire version string is lowercased and
    /// trailing zeroes are stripped. For example:
    ///
    /// ```
    /// # use mvn_version::ComparableVersion;
    /// let a = ComparableVersion::new("1.0A1");
    /// assert_eq!(a.canonical(), "1-alpha-1");
    /// ```
    pub fn canonical(&self) -> String {
        join(self.segments.iter().map(|s| s.to_string()), "-")
    }
}

impl PartialEq for ComparableVersion {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}

impl Eq for ComparableVersion {}

impl PartialOrd for ComparableVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ComparableVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut left = self.segments.iter();
        let mut right = other.segments.iter();

        loop {
            let l = left.next();
            let r = right.next();
            let order = match (l, r) {
                (Some(li), Some(ri)) => li.cmp(ri),
                (Some(li), None) => li.better_than_nothing(),
                (None, Some(ri)) => ri.better_than_nothing().reverse(),
                (None, None) => break,
            };

            if order != Ordering::Equal {
                return order;
            }
        }

        Ordering::Equal
    }
}

impl From<&str> for ComparableVersion {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl FromStr for ComparableVersion {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}
