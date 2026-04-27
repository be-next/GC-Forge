//! R4 — slow-leak regime, Rust side.
//!
//! Reference: SPEC-FUNCTIONAL §4.4.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "slow-leak";
const KNOWN_KEYS: &[&str] = &["leak_rate_mb_s", "live_set_initial_mb"];

/// Typed view of `scenario.spec.regime.parameters` for [`SlowLeakRegime`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlowLeakParams {
    /// MiB/s added to the live-set, never evicted. Float because the SPEC
    /// default is `0.5`.
    pub leak_rate_mb_s: f64,
    pub live_set_initial_mb: u32,
}

impl Default for SlowLeakParams {
    fn default() -> Self {
        Self {
            leak_rate_mb_s: 0.5,
            live_set_initial_mb: 200,
        }
    }
}

impl SlowLeakParams {
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
        if let Some(v) = map.get(Value::String("leak_rate_mb_s".to_owned())) {
            params.leak_rate_mb_s = positive_f64(v, "leak_rate_mb_s")?;
        }
        if let Some(v) = map.get(Value::String("live_set_initial_mb".to_owned())) {
            params.live_set_initial_mb = nonnegative_u32(v, "live_set_initial_mb")?;
        }
        Ok(params)
    }
}

fn positive_f64(v: &Value, key: &str) -> Result<f64, RegimeError> {
    let n = match v {
        Value::Number(n) => n.as_f64().ok_or_else(|| RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "positive number",
            got: format!("{v:?}"),
        })?,
        Value::String(s) => s.parse::<f64>().map_err(|_| RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "positive number",
            got: s.clone(),
        })?,
        _ => {
            return Err(RegimeError::WrongType {
                regime: REGIME_ID,
                key: key.to_owned(),
                expected: "positive number",
                got: yaml_type_name(v).to_owned(),
            });
        }
    };
    if !(n > 0.0 && n.is_finite()) {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: key.to_owned(),
            value: n.to_string(),
            reason: "must be > 0 and finite",
        });
    }
    Ok(n)
}

fn nonnegative_u32(v: &Value, key: &str) -> Result<u32, RegimeError> {
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

/// R4 — slow-leak.
#[derive(Debug, Clone, Copy, Default)]
pub struct SlowLeakRegime;

impl Regime for SlowLeakRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = SlowLeakParams::from_yaml(&scenario.spec.regime.parameters)?;
        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!("leak_rate_mb_s={}", params.leak_rate_mb_s));
        argv.push(format!(
            "live_set_initial_mb={}",
            params.live_set_initial_mb
        ));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["slow_leak"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "after_gc_live_set_grows_linearly",
            "mixed_gc_frequency_increases_over_time",
            "full_gc_or_oom_at_end",
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
  name: leak-test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 1g, max: 1g }}
  regime:
    kind: slow-leak
    parameters: {params_yaml}
  duration: 10m
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = SlowLeakParams::from_yaml(&Value::Null).unwrap();
        assert!((p.leak_rate_mb_s - 0.5).abs() < 1e-9);
        assert_eq!(p.live_set_initial_mb, 200);
    }

    #[test]
    fn parses_full_params() {
        let v: Value =
            serde_yaml::from_str("leak_rate_mb_s: 1.5\nlive_set_initial_mb: 100\n").unwrap();
        let p = SlowLeakParams::from_yaml(&v).unwrap();
        assert!((p.leak_rate_mb_s - 1.5).abs() < 1e-9);
        assert_eq!(p.live_set_initial_mb, 100);
    }

    #[test]
    fn rejects_unknown_key() {
        let v: Value = serde_yaml::from_str("not_a_key: 1\n").unwrap();
        let err = SlowLeakParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_zero_leak_rate() {
        let v: Value = serde_yaml::from_str("leak_rate_mb_s: 0\n").unwrap();
        let err = SlowLeakParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_negative_leak_rate() {
        let v: Value = serde_yaml::from_str("leak_rate_mb_s: -0.1\n").unwrap();
        let err = SlowLeakParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_non_finite_leak_rate() {
        let v: Value = serde_yaml::from_str("leak_rate_mb_s: .inf\n").unwrap();
        let err = SlowLeakParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn workload_args_emits_positional_and_kv() {
        let s = scenario_with_params("{}");
        let argv = SlowLeakRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "slow-leak");
        assert_eq!(argv[1], "PT10M");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "leak_rate_mb_s=0.5"));
        assert!(argv.iter().any(|a| a == "live_set_initial_mb=200"));
    }

    #[test]
    fn expected_phenomena_includes_slow_leak() {
        assert_eq!(SlowLeakRegime.expected_phenomena(), vec!["slow_leak"]);
    }
}
