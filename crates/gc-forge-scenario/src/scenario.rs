//! Scenario data model — the typed representation of `gc-forge/scenario.v1`.
//!
//! Reference: SPEC-FUNCTIONAL §5.1.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::byte_size::ByteSize;
use crate::duration::Duration;

/// Required `apiVersion` of a scenario document.
pub const API_VERSION: &str = "gc-forge/scenario.v1";
/// Required `kind` of a scenario document.
pub const KIND: &str = "Scenario";

/// Top-level scenario document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<PathBuf>,

    pub metadata: Metadata,
    pub spec: Spec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub name: String,
    #[serde(default = "default_metadata_version")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
}

fn default_metadata_version() -> String {
    "1.0.0".to_owned()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub jvm: JvmSpec,
    pub gc: GcSpec,
    pub regime: RegimeSpec,

    pub duration: Duration,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warmup: Option<Duration>,

    pub seed: Seed,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputSpec>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<ExpectedClause>,
}

/// Seed, accepted as either a non-negative integer or a hex string `"0xC0FFEE"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
#[schemars(with = "String")]
pub struct Seed(pub u64);

impl Seed {
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl Serialize for Seed {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("0x{:X}", self.0))
    }
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = Seed;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a non-negative integer or a hex string like \"0xC0FFEE\"")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let trimmed = v.trim();
                let parsed = if let Some(hex) = trimmed
                    .strip_prefix("0x")
                    .or_else(|| trimmed.strip_prefix("0X"))
                {
                    u64::from_str_radix(hex, 16)
                } else {
                    trimmed.parse::<u64>()
                };
                parsed.map(Seed).map_err(E::custom)
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Seed(v))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                u64::try_from(v).map(Seed).map_err(E::custom)
            }
        }
        deserializer.deserialize_any(V)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JvmSpec {
    pub vendor: JvmVendor,
    pub major: u8,
    #[serde(default)]
    pub distribution: Distribution,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum JvmVendor {
    Temurin,
    Corretto,
    Graalvm,
    Openj9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Distribution {
    #[default]
    Jdk,
    Jre,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GcSpec {
    pub algorithm: GcAlgorithm,
    #[serde(default)]
    pub options: GcOptions,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_flags: Vec<String>,
    #[serde(default)]
    pub log_format: LogFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum GcAlgorithm {
    G1,
    #[serde(rename = "ZGC", alias = "Zgc", alias = "zgc")]
    Zgc,
    Parallel,
    Shenandoah,
    Serial,
    Epsilon,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GcOptions {
    /// Generational mode for ZGC. Defaults to `None` at parse time; the runner
    /// applies the algorithm-specific default (`true` for ZGC on JDK 21+).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generational: Option<bool>,

    pub heap: HeapConfig,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_target_ms: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_size_mb: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ihop_percent: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeapConfig {
    pub min: ByteSize,
    pub max: ByteSize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_size: Option<ByteSize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Unified,
    Legacy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegimeSpec {
    pub kind: String,
    /// Free-form parameters; schema describes a generic JSON object.
    #[serde(default)]
    #[schemars(with = "serde_json::Value")]
    pub parameters: serde_yaml::Value,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_jfr: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectedClause {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phenomena: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<InvariantRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvariantRule {
    pub rule: String,
    /// Free-form threshold (number/string/array depending on the rule).
    #[serde(default)]
    #[schemars(with = "serde_json::Value")]
    pub threshold: serde_yaml::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_yaml() -> &'static str {
        r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: smoke
spec:
  jvm:
    vendor: temurin
    major: 21
  gc:
    algorithm: G1
    options:
      heap:
        min: 1g
        max: 2g
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
"
    }

    #[test]
    fn parses_minimal_scenario() {
        let s: Scenario = serde_yaml::from_str(minimal_yaml()).unwrap();
        assert_eq!(s.api_version, API_VERSION);
        assert_eq!(s.kind, KIND);
        assert_eq!(s.metadata.name, "smoke");
        assert_eq!(s.metadata.version, "1.0.0");
        assert_eq!(s.spec.jvm.vendor, JvmVendor::Temurin);
        assert_eq!(s.spec.jvm.major, 21);
        assert_eq!(s.spec.gc.algorithm, GcAlgorithm::G1);
        assert_eq!(s.spec.gc.options.heap.max, ByteSize(2 * ByteSize::GIB));
        assert_eq!(s.spec.regime.kind, "steady-state-healthy");
        assert_eq!(s.spec.duration, Duration::from_secs(30));
        assert_eq!(s.spec.seed.0, 0x00C0_FFEE);
    }

    #[test]
    fn rejects_unknown_field() {
        let bad = format!("{}\nbogus: 42\n", minimal_yaml().trim_end());
        assert!(serde_yaml::from_str::<Scenario>(&bad).is_err());
    }

    #[test]
    fn rejects_unknown_algorithm() {
        let bad = minimal_yaml().replace("G1", "BogusGC");
        assert!(serde_yaml::from_str::<Scenario>(&bad).is_err());
    }

    #[test]
    fn round_trips_through_yaml() {
        let s: Scenario = serde_yaml::from_str(minimal_yaml()).unwrap();
        let dump = serde_yaml::to_string(&s).unwrap();
        let again: Scenario = serde_yaml::from_str(&dump).unwrap();
        assert_eq!(s, again);
    }
}
