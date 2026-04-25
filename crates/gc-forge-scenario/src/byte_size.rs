//! Byte size parsing and serialisation.
//!
//! Accepts the JVM-style suffixes (`k`/`m`/`g`, case-insensitive, optional `b`)
//! and a raw byte count. Round-trips through the Display format.

use std::fmt;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// A size in bytes, parsed from JVM-style strings (`"2g"`, `"512m"`, `"1024"`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, JsonSchema)]
#[schemars(with = "String")]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub const KIB: u64 = 1024;
    pub const MIB: u64 = 1024 * Self::KIB;
    pub const GIB: u64 = 1024 * Self::MIB;

    /// Returns the size as a raw byte count.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseByteSizeError {
    #[error("empty byte size")]
    Empty,
    #[error(
        "invalid byte size {0:?}: expected digits optionally followed by k/m/g (with optional b)"
    )]
    Invalid(String),
    #[error("byte size {0:?} is too large to fit in u64")]
    Overflow(String),
}

impl FromStr for ByteSize {
    type Err = ParseByteSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ParseByteSizeError::Empty);
        }

        // Split into a leading run of digits and an optional unit.
        let split = trimmed
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(trimmed.len());
        let (num_part, unit_part) = trimmed.split_at(split);

        if num_part.is_empty() {
            return Err(ParseByteSizeError::Invalid(s.to_owned()));
        }

        let n: u64 = num_part
            .parse()
            .map_err(|_| ParseByteSizeError::Overflow(s.to_owned()))?;

        let multiplier: u64 = match unit_part.trim().to_ascii_lowercase().as_str() {
            "" | "b" => 1,
            "k" | "kb" | "kib" => Self::KIB,
            "m" | "mb" | "mib" => Self::MIB,
            "g" | "gb" | "gib" => Self::GIB,
            _ => return Err(ParseByteSizeError::Invalid(s.to_owned())),
        };

        n.checked_mul(multiplier)
            .map(ByteSize)
            .ok_or_else(|| ParseByteSizeError::Overflow(s.to_owned()))
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.0;
        if n == 0 {
            return f.write_str("0");
        }
        if n.is_multiple_of(Self::GIB) {
            write!(f, "{}g", n / Self::GIB)
        } else if n.is_multiple_of(Self::MIB) {
            write!(f, "{}m", n / Self::MIB)
        } else if n.is_multiple_of(Self::KIB) {
            write!(f, "{}k", n / Self::KIB)
        } else {
            write!(f, "{n}")
        }
    }
}

impl Serialize for ByteSize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ByteSize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = ByteSize;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a byte size string like \"2g\", \"512m\", or a raw byte count")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<ByteSize>().map_err(E::custom)
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(ByteSize(v))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                u64::try_from(v).map(ByteSize).map_err(E::custom)
            }
        }
        deserializer.deserialize_any(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_jvm_style_suffixes() {
        assert_eq!("0".parse::<ByteSize>().unwrap(), ByteSize(0));
        assert_eq!("1024".parse::<ByteSize>().unwrap(), ByteSize(1024));
        assert_eq!("1k".parse::<ByteSize>().unwrap(), ByteSize(ByteSize::KIB));
        assert_eq!("1K".parse::<ByteSize>().unwrap(), ByteSize(ByteSize::KIB));
        assert_eq!("1kb".parse::<ByteSize>().unwrap(), ByteSize(ByteSize::KIB));
        assert_eq!("1kib".parse::<ByteSize>().unwrap(), ByteSize(ByteSize::KIB));
        assert_eq!(
            "512m".parse::<ByteSize>().unwrap(),
            ByteSize(512 * ByteSize::MIB)
        );
        assert_eq!(
            "2g".parse::<ByteSize>().unwrap(),
            ByteSize(2 * ByteSize::GIB)
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(
            "".parse::<ByteSize>(),
            Err(ParseByteSizeError::Empty)
        ));
        assert!(matches!(
            "g".parse::<ByteSize>(),
            Err(ParseByteSizeError::Invalid(_))
        ));
        assert!(matches!(
            "2tb".parse::<ByteSize>(),
            Err(ParseByteSizeError::Invalid(_))
        ));
        assert!(matches!(
            "12345678901234567890g".parse::<ByteSize>(),
            Err(ParseByteSizeError::Overflow(_))
        ));
    }

    #[test]
    fn round_trips_via_display() {
        // Display uses the largest exact unit; raw byte counts that are
        // a power of 1024 canonicalise to the compact form.
        for s in ["0", "2g", "512m", "1k"] {
            let b: ByteSize = s.parse().unwrap();
            assert_eq!(b.to_string(), s, "{s}");
        }
        // Non-canonical inputs round-trip through their canonical form.
        let canon: ByteSize = "1024".parse().unwrap();
        assert_eq!(canon.to_string(), "1k");
        // Bytes that are not a clean multiple of any unit stay raw.
        let raw: ByteSize = "1023".parse().unwrap();
        assert_eq!(raw.to_string(), "1023");
    }

    #[test]
    fn serde_via_string() {
        let b = ByteSize(2 * ByteSize::GIB);
        let yaml = serde_yaml::to_string(&b).unwrap();
        assert!(yaml.trim().ends_with("2g"), "yaml = {yaml:?}");
        let back: ByteSize = serde_yaml::from_str("2g\n").unwrap();
        assert_eq!(back, b);
    }

    #[test]
    fn serde_accepts_integer_form() {
        let back: ByteSize = serde_yaml::from_str("4096\n").unwrap();
        assert_eq!(back, ByteSize(4096));
    }
}
