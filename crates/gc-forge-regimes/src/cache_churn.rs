//! R5 — cache-churn regime, Rust side.
//!
//! Reference: SPEC-FUNCTIONAL §4.5.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "cache-churn";
const KNOWN_KEYS: &[&str] = &[
    "cache_size_mb",
    "eviction_rate_per_s",
    "entry_lifetime_ms",
    "entry_size_kb",
];

/// Typed view of `scenario.spec.regime.parameters` for [`CacheChurnRegime`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheChurnParams {
    pub cache_size_mb: u32,
    pub eviction_rate_per_s: u32,
    pub entry_lifetime_ms: u32,
    pub entry_size_kb: u32,
}

impl Default for CacheChurnParams {
    fn default() -> Self {
        Self {
            cache_size_mb: 500,
            eviction_rate_per_s: 1000,
            entry_lifetime_ms: 2000,
            entry_size_kb: 8,
        }
    }
}

impl CacheChurnParams {
    /// Parses a free-form YAML parameter block.
    ///
    /// # Errors
    /// `UnknownParameter` for unknown keys, `WrongType` / `OutOfRange` for
    /// ill-typed or out-of-range values.
    pub fn from_yaml(parameters: &Value) -> Result<Self, RegimeError> {
        let map = match parameters {
            Value::Null => &Mapping::new(),
            Value::Mapping(m) => m,
            other => {
                return Err(RegimeError::WrongType {
                    regime: REGIME_ID,
                    key: "<parameters>".to_owned(),
                    expected: "mapping (object) or null",
                    got: yaml_type_name(other).to_owned(),
                });
            }
        };

        for key in map.keys() {
            let key_str = match key {
                Value::String(s) => s.as_str(),
                _ => continue,
            };
            if !KNOWN_KEYS.contains(&key_str) {
                return Err(RegimeError::UnknownParameter {
                    regime: REGIME_ID,
                    key: key_str.to_owned(),
                    known: KNOWN_KEYS.to_vec(),
                });
            }
        }

        let mut params = Self::default();
        if let Some(v) = map.get(Value::String("cache_size_mb".to_owned())) {
            params.cache_size_mb = positive_u32(v, "cache_size_mb")?;
        }
        if let Some(v) = map.get(Value::String("eviction_rate_per_s".to_owned())) {
            params.eviction_rate_per_s = positive_u32(v, "eviction_rate_per_s")?;
        }
        if let Some(v) = map.get(Value::String("entry_lifetime_ms".to_owned())) {
            params.entry_lifetime_ms = positive_u32(v, "entry_lifetime_ms")?;
        }
        if let Some(v) = map.get(Value::String("entry_size_kb".to_owned())) {
            params.entry_size_kb = positive_u32(v, "entry_size_kb")?;
        }

        params.validate()?;
        Ok(params)
    }

    fn validate(&self) -> Result<(), RegimeError> {
        for (key, value) in [
            ("cache_size_mb", self.cache_size_mb),
            ("eviction_rate_per_s", self.eviction_rate_per_s),
            ("entry_lifetime_ms", self.entry_lifetime_ms),
            ("entry_size_kb", self.entry_size_kb),
        ] {
            if value == 0 {
                return Err(RegimeError::OutOfRange {
                    regime: REGIME_ID,
                    key: key.to_owned(),
                    value: "0".to_owned(),
                    reason: "must be > 0",
                });
            }
        }
        Ok(())
    }
}

fn positive_u32(v: &Value, key: &str) -> Result<u32, RegimeError> {
    let n: i64 = match v {
        Value::Number(n) => n.as_i64().ok_or_else(|| RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "non-negative integer",
            got: format!("{v:?}"),
        })?,
        Value::String(s) => s.parse().map_err(|_| RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "non-negative integer",
            got: s.clone(),
        })?,
        _ => {
            return Err(RegimeError::WrongType {
                regime: REGIME_ID,
                key: key.to_owned(),
                expected: "non-negative integer",
                got: yaml_type_name(v).to_owned(),
            });
        }
    };
    if n < 0 {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: key.to_owned(),
            value: n.to_string(),
            reason: "must be ≥ 0",
        });
    }
    u32::try_from(n).map_err(|_| RegimeError::OutOfRange {
        regime: REGIME_ID,
        key: key.to_owned(),
        value: n.to_string(),
        reason: "must fit in u32",
    })
}

fn yaml_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Sequence(_) => "sequence",
        Value::Mapping(_) => "mapping",
        Value::Tagged(_) => "tagged",
    }
}

/// R5 — cache-churn.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheChurnRegime;

impl Regime for CacheChurnRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = CacheChurnParams::from_yaml(&scenario.spec.regime.parameters)?;
        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!("cache_size_mb={}", params.cache_size_mb));
        argv.push(format!(
            "eviction_rate_per_s={}",
            params.eviction_rate_per_s
        ));
        argv.push(format!("entry_lifetime_ms={}", params.entry_lifetime_ms));
        argv.push(format!("entry_size_kb={}", params.entry_size_kb));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["promotion_pressure"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "promotion_rate_pct >= 30",
            "old_gen_oscillates_within_cache_band",
            "mixed_gc_regular",
        ]
    }
}

fn iso_duration(d: &gc_forge_scenario::Duration) -> String {
    let secs = d.as_std().as_secs();
    if secs == 0 {
        "PT0S".to_owned()
    } else if secs.is_multiple_of(3600) {
        format!("PT{}H", secs / 3600)
    } else if secs.is_multiple_of(60) {
        format!("PT{}M", secs / 60)
    } else {
        format!("PT{secs}S")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc_forge_scenario::Scenario;

    fn scenario_with_params(params_yaml: &str) -> Scenario {
        let yaml = format!(
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: cache-test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 4g, max: 4g }}
  regime:
    kind: cache-churn
    parameters: {params_yaml}
  duration: 5m
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = CacheChurnParams::from_yaml(&Value::Null).unwrap();
        assert_eq!(p, CacheChurnParams::default());
    }

    #[test]
    fn parses_full_params() {
        let v: Value = serde_yaml::from_str(
            r"cache_size_mb: 1000
eviction_rate_per_s: 2000
entry_lifetime_ms: 3000
entry_size_kb: 16
",
        )
        .unwrap();
        let p = CacheChurnParams::from_yaml(&v).unwrap();
        assert_eq!(p.cache_size_mb, 1000);
        assert_eq!(p.eviction_rate_per_s, 2000);
        assert_eq!(p.entry_lifetime_ms, 3000);
        assert_eq!(p.entry_size_kb, 16);
    }

    #[test]
    fn rejects_unknown_key() {
        let v: Value = serde_yaml::from_str("not_a_key: 1\n").unwrap();
        let err = CacheChurnParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_zero_cache_size() {
        let v: Value = serde_yaml::from_str("cache_size_mb: 0\n").unwrap();
        let err = CacheChurnParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_eviction_rate() {
        let v: Value = serde_yaml::from_str("eviction_rate_per_s: 0\n").unwrap();
        let err = CacheChurnParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_lifetime() {
        let v: Value = serde_yaml::from_str("entry_lifetime_ms: 0\n").unwrap();
        let err = CacheChurnParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_entry_size() {
        let v: Value = serde_yaml::from_str("entry_size_kb: 0\n").unwrap();
        let err = CacheChurnParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn workload_args_emits_positional_and_kv() {
        let s = scenario_with_params("{}");
        let argv = CacheChurnRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "cache-churn");
        assert_eq!(argv[1], "PT5M");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "cache_size_mb=500"));
        assert!(argv.iter().any(|a| a == "eviction_rate_per_s=1000"));
        assert!(argv.iter().any(|a| a == "entry_lifetime_ms=2000"));
        assert!(argv.iter().any(|a| a == "entry_size_kb=8"));
    }

    #[test]
    fn expected_phenomena_includes_promotion_pressure() {
        assert_eq!(
            CacheChurnRegime.expected_phenomena(),
            vec!["promotion_pressure"]
        );
    }
}
