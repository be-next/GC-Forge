//! R6 — mixed-gc-pathological regime, Rust side.
//!
//! Reference: SPEC-FONCTIONNELLE §4.6.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "mixed-gc-pathological";
const KNOWN_KEYS: &[&str] = &[
    "old_gen_pressure",
    "fragmentation_factor",
    "survivor_age_target",
];

/// Typed view of `scenario.spec.regime.parameters` for [`MixedGcPathologicalRegime`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixedGcPathologicalParams {
    /// Fraction of the heap pre-filled with long-lived references. `(0, 1]`.
    pub old_gen_pressure: f64,
    /// Multiplier on free-list fragmentation. `[1.0, 3.0]`.
    pub fragmentation_factor: f64,
    /// Target tenuring age (≤ JVM hard cap of 15).
    pub survivor_age_target: u8,
}

impl Default for MixedGcPathologicalParams {
    fn default() -> Self {
        Self {
            old_gen_pressure: 0.7,
            fragmentation_factor: 2.0,
            survivor_age_target: 15,
        }
    }
}

impl MixedGcPathologicalParams {
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
        if let Some(v) = map.get(Value::String("old_gen_pressure".to_owned())) {
            params.old_gen_pressure = parse_pressure(v)?;
        }
        if let Some(v) = map.get(Value::String("fragmentation_factor".to_owned())) {
            params.fragmentation_factor = parse_fragmentation(v)?;
        }
        if let Some(v) = map.get(Value::String("survivor_age_target".to_owned())) {
            params.survivor_age_target = parse_age(v)?;
        }
        Ok(params)
    }
}

fn parse_pressure(v: &Value) -> Result<f64, RegimeError> {
    let n = parse_f64(v, "old_gen_pressure")?;
    if !(n > 0.0 && n <= 1.0) {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: "old_gen_pressure".to_owned(),
            value: n.to_string(),
            reason: "must be in (0, 1]",
        });
    }
    Ok(n)
}

fn parse_fragmentation(v: &Value) -> Result<f64, RegimeError> {
    let n = parse_f64(v, "fragmentation_factor")?;
    if !(1.0..=3.0).contains(&n) {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: "fragmentation_factor".to_owned(),
            value: n.to_string(),
            reason: "must be in [1.0, 3.0]",
        });
    }
    Ok(n)
}

fn parse_age(v: &Value) -> Result<u8, RegimeError> {
    let n: i64 = match v {
        Value::Number(n) => n.as_i64().ok_or_else(|| RegimeError::WrongType {
            regime: REGIME_ID,
            key: "survivor_age_target".to_owned(),
            expected: "integer in [1, 15]",
            got: format!("{v:?}"),
        })?,
        Value::String(s) => s.parse().map_err(|_| RegimeError::WrongType {
            regime: REGIME_ID,
            key: "survivor_age_target".to_owned(),
            expected: "integer in [1, 15]",
            got: s.clone(),
        })?,
        _ => {
            return Err(RegimeError::WrongType {
                regime: REGIME_ID,
                key: "survivor_age_target".to_owned(),
                expected: "integer in [1, 15]",
                got: yaml_type_name(v).to_owned(),
            });
        }
    };
    if !(1..=15).contains(&n) {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: "survivor_age_target".to_owned(),
            value: n.to_string(),
            reason: "must be in [1, 15] (JVM hard cap)",
        });
    }
    Ok(u8::try_from(n).expect("range-checked"))
}

fn parse_f64(v: &Value, key: &str) -> Result<f64, RegimeError> {
    let n = match v {
        Value::Number(n) => n.as_f64().ok_or_else(|| RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "finite number",
            got: format!("{v:?}"),
        })?,
        Value::String(s) => s.parse::<f64>().map_err(|_| RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "finite number",
            got: s.clone(),
        })?,
        _ => {
            return Err(RegimeError::WrongType {
                regime: REGIME_ID,
                key: key.to_owned(),
                expected: "finite number",
                got: yaml_type_name(v).to_owned(),
            });
        }
    };
    if !n.is_finite() {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: key.to_owned(),
            value: n.to_string(),
            reason: "must be finite",
        });
    }
    Ok(n)
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

/// R6 — mixed-gc-pathological.
#[derive(Debug, Clone, Copy, Default)]
pub struct MixedGcPathologicalRegime;

impl Regime for MixedGcPathologicalRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = MixedGcPathologicalParams::from_yaml(&scenario.spec.regime.parameters)?;
        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!("old_gen_pressure={}", params.old_gen_pressure));
        argv.push(format!(
            "fragmentation_factor={}",
            params.fragmentation_factor
        ));
        argv.push(format!(
            "survivor_age_target={}",
            params.survivor_age_target
        ));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["mixed_gc_pathological"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "mixed_gc_duration_increases_over_time",
            "mixed_gc_reclaim_pct < 5",
            "ihop_effective_decreases",
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
  name: mixed-patho-test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 2g, max: 2g }}
  regime:
    kind: mixed-gc-pathological
    parameters: {params_yaml}
  duration: 5m
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = MixedGcPathologicalParams::from_yaml(&Value::Null).unwrap();
        assert_eq!(p, MixedGcPathologicalParams::default());
    }

    #[test]
    fn parses_full_params() {
        let v: Value = serde_yaml::from_str(
            r"old_gen_pressure: 0.5
fragmentation_factor: 2.5
survivor_age_target: 10
",
        )
        .unwrap();
        let p = MixedGcPathologicalParams::from_yaml(&v).unwrap();
        assert!((p.old_gen_pressure - 0.5).abs() < 1e-9);
        assert!((p.fragmentation_factor - 2.5).abs() < 1e-9);
        assert_eq!(p.survivor_age_target, 10);
    }

    #[test]
    fn rejects_unknown_key() {
        let v: Value = serde_yaml::from_str("not_a_key: 1\n").unwrap();
        let err = MixedGcPathologicalParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_pressure_out_of_range() {
        for value in ["0", "1.1", "-0.5"] {
            let v: Value = serde_yaml::from_str(&format!("old_gen_pressure: {value}\n")).unwrap();
            let err = MixedGcPathologicalParams::from_yaml(&v).unwrap_err();
            assert!(
                matches!(err, RegimeError::OutOfRange { .. }),
                "value={value}"
            );
        }
    }

    #[test]
    fn rejects_fragmentation_out_of_range() {
        for value in ["0.5", "3.5", "0"] {
            let v: Value =
                serde_yaml::from_str(&format!("fragmentation_factor: {value}\n")).unwrap();
            let err = MixedGcPathologicalParams::from_yaml(&v).unwrap_err();
            assert!(
                matches!(err, RegimeError::OutOfRange { .. }),
                "value={value}"
            );
        }
    }

    #[test]
    fn rejects_age_out_of_range() {
        for value in ["0", "16", "-1"] {
            let v: Value =
                serde_yaml::from_str(&format!("survivor_age_target: {value}\n")).unwrap();
            let err = MixedGcPathologicalParams::from_yaml(&v).unwrap_err();
            assert!(
                matches!(
                    err,
                    RegimeError::OutOfRange { .. } | RegimeError::WrongType { .. }
                ),
                "value={value}"
            );
        }
    }

    #[test]
    fn workload_args_emits_positional_and_kv() {
        let s = scenario_with_params("{}");
        let argv = MixedGcPathologicalRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "mixed-gc-pathological");
        assert_eq!(argv[1], "PT5M");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "old_gen_pressure=0.7"));
        assert!(argv.iter().any(|a| a == "fragmentation_factor=2"));
        assert!(argv.iter().any(|a| a == "survivor_age_target=15"));
    }

    #[test]
    fn expected_phenomena_includes_mixed_gc_pathological() {
        assert_eq!(
            MixedGcPathologicalRegime.expected_phenomena(),
            vec!["mixed_gc_pathological"]
        );
    }
}
