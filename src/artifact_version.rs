use crate::ComparableVersion;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// This is an implementation of Maven's DefaultArtifactVersion.
///
/// ArtifactVersion parses a version into major, minor, incremental, build, and qualifier
/// components. The former three are separated by dots. The build or qualifier (there can only be
/// one of the two) will be set by whatever is after a dash, if present. Missing components will
/// be zero (in case of the numeric ones) or None (for the qualifier). In general, unknown version
/// formats will be ungracefully dumped into the qualifier section, leaving everything else as zero.
///
/// See [ComparableVersion] for an overview of how versions are parsed for the purposes of
/// comparison and equality. It's used here internally for comparison operations.
#[derive(Debug, Eq, Clone)]
pub struct ArtifactVersion {
    major: u32,
    minor: u32,
    incremental: u32,
    build: u32,
    qualifier: Option<String>,
    comparable: ComparableVersion,
}

#[cfg(feature = "serde")]
impl serde::Serialize for ArtifactVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.comparable.orig.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ArtifactVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(&<String as serde::Deserialize>::deserialize(deserializer)?))
    }
}

impl ArtifactVersion {
    /// Constructs an ArtifactVersion from the given string. This function cannot fail for any
    /// reason, and will always return a valid ComparableVersion. Make sure to strip whitespace
    /// before passing your input to this function. Otherwise, whitespace is treated identically to
    /// any other text. A empty string will parse as 0.0.0.0-, with an empty string in the qualifier
    /// section.
    pub fn new(s: &str) -> Self {
        // Which section we are in currently
        #[derive(PartialEq, Eq, Clone, Copy)]
        enum Section {
            Major,
            Minor,
            Incremental,
            BuildOrQualifier,
            DottedQualifier,
        }

        let fallback = || ArtifactVersion {
            major: 0,
            minor: 0,
            incremental: 0,
            build: 0,
            qualifier: Some(s.to_string()),
            comparable: ComparableVersion::new(s),
        };

        let mut major = 0;
        let mut minor = 0;
        let mut incremental = 0;
        let mut build = 0;
        let mut qualifier = None;

        let mut start_index = 0;
        let mut section = Section::Major;
        let last: u8 = 0;

        for (i, c) in s.bytes().enumerate() {
            // This is what maven does, don't question it
            if (i == 0 && c == b'.')
                || (c == b'-' && last == b'.')
                || (section != Section::BuildOrQualifier
                    && section != Section::DottedQualifier
                    && c == b'.'
                    && last == b'.')
            {
                return fallback();
            }

            match (section, c) {
                (Section::Major, b'.') => {
                    section = Section::Minor;

                    if s.bytes().nth(start_index).unwrap_or(b'_') == b'0' {
                        qualifier = Some(s[start_index..s.len()].to_string());
                        return ArtifactVersion {
                            major,
                            minor,
                            incremental,
                            build,
                            qualifier,
                            comparable: ComparableVersion::new(s),
                        };
                    }

                    if let Ok(i) = &s[start_index..i].parse::<i32>() {
                        major = *i as u32;
                    } else {
                        return fallback();
                    }

                    start_index = i + 1;
                }
                (Section::Minor, b'.') => {
                    section = Section::Incremental;

                    if let Ok(i) = &s[start_index..i].parse::<i32>() {
                        minor = *i as u32;
                    } else {
                        return fallback();
                    }

                    start_index = i + 1;
                }
                (Section::Incremental, b'.') => {
                    section = Section::DottedQualifier;

                    if let Ok(i) = &s[start_index..i].parse::<i32>() {
                        incremental = *i as u32;
                    } else {
                        return fallback();
                    }

                    start_index = i + 1;
                }
                (Section::Major, b'-') | (Section::Minor, b'-') | (Section::Incremental, b'-') => {
                    if section == Section::Major
                        && s.bytes().nth(start_index).unwrap_or(b'_') == b'0'
                    {
                        qualifier = Some(s[start_index..s.len()].to_string());
                        return ArtifactVersion {
                            major,
                            minor,
                            incremental,
                            build,
                            qualifier,
                            comparable: ComparableVersion::new(s),
                        };
                    }

                    if let Ok(i) = &s[start_index..i].parse::<i32>() {
                        match section {
                            Section::Major => major = *i as u32,
                            Section::Minor => minor = *i as u32,
                            Section::Incremental => incremental = *i as u32,
                            _ => unreachable!(),
                        }
                    } else {
                        return fallback();
                    }

                    section = Section::BuildOrQualifier;
                    start_index = i + 1;
                }
                (Section::Major, _) | (Section::Minor, _) | (Section::Incremental, _)
                    if c < b'0' && c > b'9' =>
                {
                    return fallback();
                }
                (Section::DottedQualifier, b'-') => return fallback(),
                (Section::BuildOrQualifier, _) | (Section::DottedQualifier, _) | _ => {}
            }
        }

        // Parse last section
        match section {
            Section::Major => {
                if s.bytes().nth(start_index).unwrap_or(b'_') == b'0' {
                    qualifier = Some(s[start_index..s.len()].to_string());
                    return ArtifactVersion {
                        major,
                        minor,
                        incremental,
                        build,
                        qualifier,
                        comparable: ComparableVersion::new(s),
                    };
                }

                if let Ok(i) = &s[start_index..s.len()].parse::<i32>() {
                    major = *i as u32;
                } else {
                    return fallback();
                }
            }
            Section::Minor => {
                if let Ok(i) = &s[start_index..s.len()].parse::<i32>() {
                    minor = *i as u32;
                } else {
                    return fallback();
                }
            }
            Section::Incremental => {
                if let Ok(i) = &s[start_index..s.len()].parse::<i32>() {
                    incremental = *i as u32;
                } else {
                    return fallback();
                }
            }
            Section::BuildOrQualifier => {
                let sec = &s[start_index..s.len()];

                if sec.bytes().nth(0).unwrap_or(b'_') == b'0' {
                    qualifier = Some(sec.to_string());
                } else {
                    if let Ok(i) = sec.parse::<i32>() {
                        build = i as u32;
                    } else {
                        qualifier = Some(sec.to_string());
                    }
                }
            }
            Section::DottedQualifier => {
                let sec = &s[start_index..s.len()];
                if sec.bytes().all(|b| b >= b'0' && b <= b'9') {
                    return fallback();
                } else {
                    qualifier = Some(sec.to_string());
                }
            }
        }

        ArtifactVersion {
            major,
            minor,
            incremental,
            build,
            qualifier,
            comparable: ComparableVersion::new(s),
        }
    }

    /// The major version, or 0 if not specified.
    pub fn major(&self) -> u32 {
        self.major
    }
    /// The minor version, or 0 if not specified.
    pub fn minor(&self) -> u32 {
        self.minor
    }
    /// The incremental version, or 0 if not specified.
    pub fn incremental(&self) -> u32 {
        self.incremental
    }
    /// The build number, or 0 if not specified. If this is nonzero, qualifier will always be
    /// `None`.
    pub fn build(&self) -> u32 {
        self.build
    }
    /// The qualifier, or `None` if not specified. If this is `Some`, build will always be zero.
    pub fn qualifier(&self) -> &Option<String> {
        &self.qualifier
    }

    /// Returns the original string representation of the version, the same as the one passed as
    /// the argument to [`Self::new`]
    pub fn as_str(&self) -> &str {
        &self.comparable.orig
    }
}

impl AsRef<str> for ArtifactVersion {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for ArtifactVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.comparable.fmt(f)
    }
}

impl Hash for ArtifactVersion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.comparable.hash(state)
    }
}

impl PartialEq for ArtifactVersion {
    fn eq(&self, other: &Self) -> bool {
        self.comparable.eq(&other.comparable)
    }
}

impl PartialOrd for ArtifactVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArtifactVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparable.cmp(&other.comparable)
    }
}

impl From<&str> for ArtifactVersion {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl FromStr for ArtifactVersion {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}
