use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

pub type IdError = String;

pub fn validate_segment(value: &str, label: &str) -> Result<(), IdError> {
    if value.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if value.len() > 128 {
        return Err(format!("{label} must be at most 128 characters"));
    }
    let allowed = value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !allowed {
        return Err(format!(
            "{label} '{value}' may only contain [A-Za-z0-9._-]"
        ));
    }
    Ok(())
}

pub fn validate_dotted(value: &str, label: &str) -> Result<(), IdError> {
    if value.len() > 192 {
        return Err(format!("{label} must be at most 192 characters"));
    }
    let mut parts = value.split('.');
    let head = parts.next().unwrap_or("");
    let tail = parts.next();
    match tail {
        Some(tail) => {
            validate_segment(head, label)?;
            validate_segment(tail, label)?;
            if parts.next().is_some() {
                return Err(format!(
                    "{label} '{value}' must have at most two dot-separated segments"
                ));
            }
            Ok(())
        }
        None => validate_segment(head, label),
    }
}

macro_rules! define_id {
    ($( $(#[$meta:meta])* $name:ident => $label:expr => $check:path ),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
            #[serde(try_from = "String", into = "String")]
            pub struct $name(String);

            impl $name {
                pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                    let value = value.into();
                    $check(&value, $label)?;
                    Ok(Self(value))
                }

                pub fn as_str(&self) -> &str {
                    &self.0
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(&self.0)
                }
            }

            impl fmt::Debug for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}({:?})", stringify!($name), self.0)
                }
            }

            impl Deref for $name {
                type Target = str;
                fn deref(&self) -> &str {
                    &self.0
                }
            }

            impl AsRef<str> for $name {
                fn as_ref(&self) -> &str {
                    &self.0
                }
            }

            impl Borrow<str> for $name {
                fn borrow(&self) -> &str {
                    &self.0
                }
            }

            impl TryFrom<String> for $name {
                type Error = IdError;
                fn try_from(value: String) -> Result<Self, IdError> {
                    Self::new(value)
                }
            }

            impl TryFrom<&str> for $name {
                type Error = IdError;
                fn try_from(value: &str) -> Result<Self, IdError> {
                    Self::new(value)
                }
            }

            impl From<$name> for String {
                fn from(value: $name) -> String {
                    value.0
                }
            }

            impl std::str::FromStr for $name {
                type Err = IdError;
                fn from_str(value: &str) -> Result<Self, IdError> {
                    Self::new(value)
                }
            }
        )*
    };
}

define_id! {
    PluginId => "plugin id" => validate_segment,
    ServiceId => "service id" => validate_dotted,
    ContributionPointId => "contribution point id" => validate_dotted,
    CommandId => "command id" => validate_dotted,
    Capability => "capability" => validate_dotted,
    EventType => "event type" => validate_dotted,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn major(&self) -> u32 {
        self.major
    }
    pub const fn minor(&self) -> u32 {
        self.minor
    }
    pub const fn patch(&self) -> u32 {
        self.patch
    }

    pub fn is_compatible_with(&self, host: &Version) -> bool {
        // self.major == host.major && self <= host
        self <= host
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

impl TryFrom<String> for Version {
    type Error = IdError;
    fn try_from(value: String) -> Result<Self, IdError> {
        value.parse()
    }
}

impl std::str::FromStr for Version {
    type Err = IdError;
    fn from_str(value: &str) -> Result<Self, IdError> {
        let mut parts = value.split('.');
        let parse = |p: Option<&str>, field: &str| -> Result<u32, IdError> {
            p.ok_or_else(|| format!("version '{value}' is missing {field}"))?
                .parse::<u32>()
                .map_err(|_| format!("version '{value}' has non-numeric {field}"))
        };
        let major = parse(parts.next(), "major")?;
        let minor = parse(parts.next(), "minor")?;
        let patch = parse(parts.next(), "patch")?;
        if parts.next().is_some() {
            return Err(format!("version '{value}' must have exactly 3 segments"));
        }
        Ok(Self::new(major, minor, patch))
    }
}

impl From<Version> for String {
    fn from(value: Version) -> String {
        value.to_string()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}