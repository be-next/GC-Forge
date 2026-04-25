//! R7 — microservice-stop-and-go regime, Rust side.
//!
//! Reference: SPEC-FONCTIONNELLE §4.7.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "microservice-stop-and-go";
const KNOWN_KEYS: &[&str] = &[
    "active_period_s",
    "idle_period_s",
    "active_rate_mb_s",
    "cycles",
];

/// Typed view of `scenario.spec.regime.parameters` for [`MicroserviceStopGoRegime`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroserviceStopGoParams {
    pub active_period_s: u32,
    pub idle_period_s: u32,
    pub active_rate_mb_s: u32,
    pub cycles: Cycles,
}

impl Default for MicroserviceStopGoParams {
    fn default() -> Self {
        Self {
            active_period_s: 10,
            idle_period_s: 20,
            active_rate_mb_s: 100,
            cycles: Cycles::Auto,
        }
    }
}

/// Number of stop-and-go cycles. `auto` is derived from the run duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cycles {
    #[serde(rename = "auto")]
    Auto,
    Fixed(u32),
}

impl Cycles {
    fn as_arg(self) -> String {
        match self {
            Self::Auto => "auto".to_owned(),
            Self::Fixed(n) => n.to_string(),
        }
    }
}

impl MicroserviceStopGoParams {
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
        if let Some(v) = map.get(Value::String("active_period_s".to_owned())) {
            params.active_period_s = positive_u32(v, "active_period_s")?;
        }
        if let Some(v) = map.get(Value::String("idle_period_s".to_owned())) {
            params.idle_period_s = positive_u32(v, "idle_period_s")?;
        }
        if let Some(v) = map.get(Value::String("active_rate_mb_s".to_owned())) {
            params.active_rate_mb_s = positive_u32(v, "active_rate_mb_s")?;
        }
        if let Some(v) = map.get(Value::String("cycles".to_owned())) {
            params.cycles = parse_cycles(v)?;
        }

        params.validate()?;
        Ok(params)
    }

    fn validate(&self) -> Result<(), RegimeError> {
        for (key, value) in [
            ("active_period_s", self.active_period_s),
            ("idle_period_s", self.idle_period_s),
            ("active_rate_mb_s", self.active_rate_mb_s),
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

fn parse_cycles(v: &Value) -> Result<Cycles, RegimeError> {
    match v {
        Value::String(s) if s == "auto" => Ok(Cycles::Auto),
        Value::Number(_) | Value::String(_) => {
            let n = positive_u32(v, "cycles")?;
            if n == 0 {
                return Err(RegimeError::OutOfRange {
                    regime: REGIME_ID,
                    key: "cycles".to_owned(),
                    value: "0".to_owned(),
                    reason: "must be > 0 or \"auto\"",
                });
            }
            Ok(Cycles::Fixed(n))
        }
        _ => Err(RegimeError::WrongType {
            regime: REGIME_ID,
            key: "cycles".to_owned(),
            expected: "positive integer or \"auto\"",
            got: yaml_type_name(v).to_owned(),
        }),
    }
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

/// R7 — microservice-stop-and-go.
#[derive(Debug, Clone, Copy, Default)]
pub struct MicroserviceStopGoRegime;

impl Regime for MicroserviceStopGoRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = MicroserviceStopGoParams::from_yaml(&scenario.spec.regime.parameters)?;
        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!("active_period_s={}", params.active_period_s));
        argv.push(format!("idle_period_s={}", params.idle_period_s));
        argv.push(format!("active_rate_mb_s={}", params.active_rate_mb_s));
        argv.push(format!("cycles={}", params.cycles.as_arg()));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["concurrent_cycle_in_idle"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "idle_phase_few_young_gc",
            "concurrent_cycle_during_idle_g1",
            "active_idle_phase_alternation_visible",
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
  name: micro-test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 1g, max: 1g }}
  regime:
    kind: microservice-stop-and-go
    parameters: {params_yaml}
  duration: 5m
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = MicroserviceStopGoParams::from_yaml(&Value::Null).unwrap();
        assert_eq!(p, MicroserviceStopGoParams::default());
    }

    #[test]
    fn parses_full_params() {
        let v: Value = serde_yaml::from_str(
            r"active_period_s: 5
idle_period_s: 10
active_rate_mb_s: 200
cycles: 8
",
        )
        .unwrap();
        let p = MicroserviceStopGoParams::from_yaml(&v).unwrap();
        assert_eq!(p.active_period_s, 5);
        assert_eq!(p.idle_period_s, 10);
        assert_eq!(p.active_rate_mb_s, 200);
        assert_eq!(p.cycles, Cycles::Fixed(8));
    }

    #[test]
    fn parses_auto_cycles() {
        let v: Value = serde_yaml::from_str("cycles: auto\n").unwrap();
        let p = MicroserviceStopGoParams::from_yaml(&v).unwrap();
        assert_eq!(p.cycles, Cycles::Auto);
    }

    #[test]
    fn rejects_unknown_key() {
        let v: Value = serde_yaml::from_str("not_a_key: 1\n").unwrap();
        let err = MicroserviceStopGoParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_zero_active_period() {
        let v: Value = serde_yaml::from_str("active_period_s: 0\n").unwrap();
        let err = MicroserviceStopGoParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_idle_period() {
        let v: Value = serde_yaml::from_str("idle_period_s: 0\n").unwrap();
        let err = MicroserviceStopGoParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_active_rate() {
        let v: Value = serde_yaml::from_str("active_rate_mb_s: 0\n").unwrap();
        let err = MicroserviceStopGoParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn workload_args_emits_positional_and_kv() {
        let s = scenario_with_params("{}");
        let argv = MicroserviceStopGoRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "microservice-stop-and-go");
        assert_eq!(argv[1], "PT5M");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "active_period_s=10"));
        assert!(argv.iter().any(|a| a == "idle_period_s=20"));
        assert!(argv.iter().any(|a| a == "active_rate_mb_s=100"));
        assert!(argv.iter().any(|a| a == "cycles=auto"));
    }

    #[test]
    fn expected_phenomena_includes_concurrent_cycle_in_idle() {
        assert_eq!(
            MicroserviceStopGoRegime.expected_phenomena(),
            vec!["concurrent_cycle_in_idle"]
        );
    }
}
