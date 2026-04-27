//! R2 — allocation-burst regime, Rust side.
//!
//! Reference: SPEC-FUNCTIONAL §4.2.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "allocation-burst";
const KNOWN_KEYS: &[&str] = &[
    "base_rate_mb_s",
    "burst_rate_mb_s",
    "burst_duration_s",
    "burst_period_s",
    "bursts_count",
];

/// Typed view of `scenario.spec.regime.parameters` for [`AllocationBurstRegime`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationBurstParams {
    pub base_rate_mb_s: u32,
    pub burst_rate_mb_s: u32,
    pub burst_duration_s: u32,
    pub burst_period_s: u32,
    /// `auto` → derived from the run duration. The regime is deterministic on
    /// duration anyway, so callers rarely need to set this explicitly.
    pub bursts_count: BurstsCount,
}

impl Default for AllocationBurstParams {
    fn default() -> Self {
        Self {
            base_rate_mb_s: 30,
            burst_rate_mb_s: 200,
            burst_duration_s: 5,
            burst_period_s: 30,
            bursts_count: BurstsCount::Auto,
        }
    }
}

/// Either `auto` (default, derived from the run duration) or a fixed
/// integer count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BurstsCount {
    #[serde(rename = "auto")]
    Auto,
    Fixed(u32),
}

impl BurstsCount {
    fn as_arg(self) -> String {
        match self {
            Self::Auto => "auto".to_owned(),
            Self::Fixed(n) => n.to_string(),
        }
    }
}

impl AllocationBurstParams {
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
        if let Some(v) = map.get(Value::String("base_rate_mb_s".to_owned())) {
            params.base_rate_mb_s = positive_u32(v, "base_rate_mb_s")?;
        }
        if let Some(v) = map.get(Value::String("burst_rate_mb_s".to_owned())) {
            params.burst_rate_mb_s = positive_u32(v, "burst_rate_mb_s")?;
        }
        if let Some(v) = map.get(Value::String("burst_duration_s".to_owned())) {
            params.burst_duration_s = positive_u32(v, "burst_duration_s")?;
        }
        if let Some(v) = map.get(Value::String("burst_period_s".to_owned())) {
            params.burst_period_s = positive_u32(v, "burst_period_s")?;
        }
        if let Some(v) = map.get(Value::String("bursts_count".to_owned())) {
            params.bursts_count = parse_bursts_count(v)?;
        }

        params.validate()?;
        Ok(params)
    }

    fn validate(&self) -> Result<(), RegimeError> {
        if self.burst_rate_mb_s < self.base_rate_mb_s {
            return Err(RegimeError::OutOfRange {
                regime: REGIME_ID,
                key: "burst_rate_mb_s".to_owned(),
                value: self.burst_rate_mb_s.to_string(),
                reason: "must be ≥ base_rate_mb_s",
            });
        }
        if self.burst_period_s == 0 {
            return Err(RegimeError::OutOfRange {
                regime: REGIME_ID,
                key: "burst_period_s".to_owned(),
                value: "0".to_owned(),
                reason: "must be > 0",
            });
        }
        if self.burst_duration_s == 0 || self.burst_duration_s > self.burst_period_s {
            return Err(RegimeError::OutOfRange {
                regime: REGIME_ID,
                key: "burst_duration_s".to_owned(),
                value: self.burst_duration_s.to_string(),
                reason: "must be in (0, burst_period_s]",
            });
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

fn parse_bursts_count(v: &Value) -> Result<BurstsCount, RegimeError> {
    match v {
        Value::String(s) if s == "auto" => Ok(BurstsCount::Auto),
        Value::Number(_) | Value::String(_) => {
            let n = positive_u32(v, "bursts_count")?;
            Ok(BurstsCount::Fixed(n))
        }
        _ => Err(RegimeError::WrongType {
            regime: REGIME_ID,
            key: "bursts_count".to_owned(),
            expected: "non-negative integer or \"auto\"",
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

/// R2 — allocation-burst.
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocationBurstRegime;

impl Regime for AllocationBurstRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = AllocationBurstParams::from_yaml(&scenario.spec.regime.parameters)?;
        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!("base_rate_mb_s={}", params.base_rate_mb_s));
        argv.push(format!("burst_rate_mb_s={}", params.burst_rate_mb_s));
        argv.push(format!("burst_duration_s={}", params.burst_duration_s));
        argv.push(format!("burst_period_s={}", params.burst_period_s));
        argv.push(format!("bursts_count={}", params.bursts_count.as_arg()));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["allocation_burst"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "young_gc_frequency_pulses",
            "no_evacuation_failure",
            "post_burst_recovery_within_2x_burst_duration",
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
  name: burst-test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 1g, max: 2g }}
  regime:
    kind: allocation-burst
    parameters: {params_yaml}
  duration: 5m
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = AllocationBurstParams::from_yaml(&Value::Null).unwrap();
        assert_eq!(p, AllocationBurstParams::default());
    }

    #[test]
    fn parses_full_params() {
        let v: Value = serde_yaml::from_str(
            r"base_rate_mb_s: 50
burst_rate_mb_s: 300
burst_duration_s: 10
burst_period_s: 60
bursts_count: 3
",
        )
        .unwrap();
        let p = AllocationBurstParams::from_yaml(&v).unwrap();
        assert_eq!(p.base_rate_mb_s, 50);
        assert_eq!(p.burst_rate_mb_s, 300);
        assert_eq!(p.burst_duration_s, 10);
        assert_eq!(p.burst_period_s, 60);
        assert_eq!(p.bursts_count, BurstsCount::Fixed(3));
    }

    #[test]
    fn parses_auto_bursts_count() {
        let v: Value = serde_yaml::from_str("bursts_count: auto\n").unwrap();
        let p = AllocationBurstParams::from_yaml(&v).unwrap();
        assert_eq!(p.bursts_count, BurstsCount::Auto);
    }

    #[test]
    fn rejects_unknown_key() {
        let v: Value = serde_yaml::from_str("not_a_key: 1\n").unwrap();
        let err = AllocationBurstParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_burst_below_base() {
        let v: Value = serde_yaml::from_str("base_rate_mb_s: 100\nburst_rate_mb_s: 50\n").unwrap();
        let err = AllocationBurstParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_period() {
        let v: Value = serde_yaml::from_str("burst_period_s: 0\n").unwrap();
        let err = AllocationBurstParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_burst_duration_above_period() {
        let v: Value = serde_yaml::from_str("burst_duration_s: 60\nburst_period_s: 30\n").unwrap();
        let err = AllocationBurstParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_zero_burst_duration() {
        let v: Value = serde_yaml::from_str("burst_duration_s: 0\n").unwrap();
        let err = AllocationBurstParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn workload_args_emits_positional_then_kv() {
        let s = scenario_with_params("{}");
        let argv = AllocationBurstRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "allocation-burst");
        assert_eq!(argv[1], "PT5M");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "base_rate_mb_s=30"));
        assert!(argv.iter().any(|a| a == "burst_rate_mb_s=200"));
        assert!(argv.iter().any(|a| a == "burst_duration_s=5"));
        assert!(argv.iter().any(|a| a == "burst_period_s=30"));
        assert!(argv.iter().any(|a| a == "bursts_count=auto"));
    }

    #[test]
    fn workload_args_uses_overrides() {
        let s =
            scenario_with_params("{ base_rate_mb_s: 50, burst_rate_mb_s: 400, bursts_count: 4 }");
        let argv = AllocationBurstRegime.workload_args(&s).unwrap();
        assert!(argv.iter().any(|a| a == "base_rate_mb_s=50"));
        assert!(argv.iter().any(|a| a == "burst_rate_mb_s=400"));
        assert!(argv.iter().any(|a| a == "bursts_count=4"));
    }

    #[test]
    fn expected_phenomena_includes_allocation_burst() {
        assert_eq!(
            AllocationBurstRegime.expected_phenomena(),
            vec!["allocation_burst"]
        );
    }

    #[test]
    fn expected_invariants_match_spec() {
        let inv = AllocationBurstRegime.expected_invariant_rules();
        assert!(inv.contains(&"young_gc_frequency_pulses"));
        assert!(inv.contains(&"no_evacuation_failure"));
    }
}
