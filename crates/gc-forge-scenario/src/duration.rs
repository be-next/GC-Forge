//! Duration parsing for scenarios (`"30s"`, `"5m"`, `"1h"`, `"PT30S"`).

use std::fmt;
use std::str::FromStr;
use std::time::Duration as StdDuration;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// A wall-clock duration, parsed from human-friendly suffix forms or
/// minimal ISO-8601.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, JsonSchema)]
#[schemars(with = "String")]
pub struct Duration(pub StdDuration);

impl Duration {
    /// Returns the underlying `std::time::Duration`.
    #[must_use]
    pub const fn as_std(self) -> StdDuration {
        self.0
    }

    /// Constructs from a number of seconds.
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(StdDuration::from_secs(secs))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseDurationError {
    #[error("empty duration")]
    Empty,
    #[error("invalid duration {0:?}: expected forms like \"30s\", \"5m\", \"1h\", or \"PT30S\"")]
    Invalid(String),
    #[error("duration {0:?} overflows")]
    Overflow(String),
}

impl FromStr for Duration {
    type Err = ParseDurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw.is_empty() {
            return Err(ParseDurationError::Empty);
        }

        // ISO-8601 PT… form: keep it minimal — single component up to hours.
        if let Some(rest) = raw.strip_prefix("PT").or_else(|| raw.strip_prefix("pt")) {
            return parse_iso(rest).ok_or_else(|| ParseDurationError::Invalid(s.to_owned()));
        }

        // Suffix form: `<digits><unit>` where unit is s/m/h/d (or ms).
        let (num_part, unit_part) = split_digits(raw);
        if num_part.is_empty() {
            return Err(ParseDurationError::Invalid(s.to_owned()));
        }
        let n: u64 = num_part
            .parse()
            .map_err(|_| ParseDurationError::Overflow(s.to_owned()))?;

        let secs: u64 = match unit_part.to_ascii_lowercase().as_str() {
            "ms" => return Ok(Self(StdDuration::from_millis(n))),
            "s" => Some(n),
            "m" | "min" => n.checked_mul(60),
            "h" => n.checked_mul(3600),
            "d" => n.checked_mul(86_400),
            _ => return Err(ParseDurationError::Invalid(s.to_owned())),
        }
        .ok_or_else(|| ParseDurationError::Overflow(s.to_owned()))?;

        Ok(Self(StdDuration::from_secs(secs)))
    }
}

fn split_digits(s: &str) -> (&str, &str) {
    let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    s.split_at(i)
}

fn parse_iso(rest: &str) -> Option<Duration> {
    // Minimal: a single H/M/S component. e.g. "30S", "5M", "1H".
    if rest.is_empty() {
        return None;
    }
    let last = rest.chars().last()?;
    let head = &rest[..rest.len() - last.len_utf8()];
    let n: u64 = head.parse().ok()?;
    let secs = match last.to_ascii_uppercase() {
        'S' => n,
        'M' => n.checked_mul(60)?,
        'H' => n.checked_mul(3600)?,
        _ => return None,
    };
    Some(Duration(StdDuration::from_secs(secs)))
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.0.as_secs();
        let nanos = self.0.subsec_nanos();
        if total == 0 && nanos == 0 {
            return f.write_str("0s");
        }
        if nanos != 0 {
            // Fall back to milliseconds for sub-second precision.
            let ms = self.0.as_millis();
            return write!(f, "{ms}ms");
        }
        if total.is_multiple_of(86_400) {
            write!(f, "{}d", total / 86_400)
        } else if total.is_multiple_of(3600) {
            write!(f, "{}h", total / 3600)
        } else if total.is_multiple_of(60) {
            write!(f, "{}m", total / 60)
        } else {
            write!(f, "{total}s")
        }
    }
}

impl Serialize for Duration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = Duration;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a duration string such as \"90s\", \"5m\", \"1h\"")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<Duration>().map_err(E::custom)
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Duration(StdDuration::from_secs(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                u64::try_from(v)
                    .map(StdDuration::from_secs)
                    .map(Duration)
                    .map_err(E::custom)
            }
        }
        deserializer.deserialize_any(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_suffix_forms() {
        assert_eq!("30s".parse::<Duration>().unwrap(), Duration::from_secs(30));
        assert_eq!("5m".parse::<Duration>().unwrap(), Duration::from_secs(300));
        assert_eq!("1h".parse::<Duration>().unwrap(), Duration::from_secs(3600));
        assert_eq!(
            "2d".parse::<Duration>().unwrap(),
            Duration::from_secs(2 * 86_400)
        );
        assert_eq!(
            "500ms".parse::<Duration>().unwrap(),
            Duration(StdDuration::from_millis(500))
        );
    }

    #[test]
    fn parses_iso_forms() {
        assert_eq!(
            "PT30S".parse::<Duration>().unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(
            "PT5M".parse::<Duration>().unwrap(),
            Duration::from_secs(300)
        );
        assert_eq!(
            "PT1H".parse::<Duration>().unwrap(),
            Duration::from_secs(3600)
        );
    }

    #[test]
    fn rejects_garbage() {
        for s in ["", "abc", "10x", "10 s", "PT", "PT5X"] {
            assert!(s.parse::<Duration>().is_err(), "{s}");
        }
    }

    #[test]
    fn round_trips_via_display() {
        for s in ["30s", "5m", "1h", "2d"] {
            let d: Duration = s.parse().unwrap();
            assert_eq!(d.to_string(), s, "{s}");
        }
    }

    #[test]
    fn serde_via_string() {
        let d = Duration::from_secs(90);
        let yaml = serde_yaml::to_string(&d).unwrap();
        assert!(yaml.contains("90s"));
        let back: Duration = serde_yaml::from_str("90s\n").unwrap();
        assert_eq!(back, d);
    }
}
