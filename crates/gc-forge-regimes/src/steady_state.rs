//! R1 — steady-state-healthy regime, Rust side.
//!
//! Reference: SPEC-FUNCTIONAL §4.1.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "steady-state-healthy";
const KNOWN_KEYS: &[&str] = &[
    "allocation_rate_mb_s",
    "live_set_mb",
    "object_size_distribution",
    "lifetime_distribution",
];

/// Typed view of `scenario.spec.regime.parameters` for [`SteadyStateRegime`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteadyStateParams {
    pub allocation_rate_mb_s: u32,
    pub live_set_mb: u32,
    pub object_size_distribution: ObjectSizeDistribution,
    pub lifetime_distribution: LifetimeDistribution,
}

impl Default for SteadyStateParams {
    fn default() -> Self {
        Self {
            allocation_rate_mb_s: 50,
            live_set_mb: 100,
            object_size_distribution: ObjectSizeDistribution::default(),
            lifetime_distribution: LifetimeDistribution::default(),
        }
    }
}

/// Object size distribution (`small | medium | mixed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectSizeDistribution {
    Small,
    Medium,
    #[default]
    Mixed,
}

impl ObjectSizeDistribution {
    fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Mixed => "mixed",
        }
    }
}

/// Lifetime distribution (`short | mixed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifetimeDistribution {
    Short,
    #[default]
    Mixed,
}

impl LifetimeDistribution {
    fn as_str(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Mixed => "mixed",
        }
    }
}

impl SteadyStateParams {
    /// Parses a free-form YAML parameter block. `Value::Null` is treated as
    /// the empty map (returning the documented defaults).
    ///
    /// # Errors
    /// Returns [`RegimeError::UnknownParameter`] for unknown keys, and
    /// [`RegimeError::WrongType`] / [`RegimeError::OutOfRange`] for
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

        if let Some(v) = map.get(Value::String("allocation_rate_mb_s".to_owned())) {
            params.allocation_rate_mb_s = positive_u32(v, "allocation_rate_mb_s")?;
        }
        if let Some(v) = map.get(Value::String("live_set_mb".to_owned())) {
            params.live_set_mb = positive_u32(v, "live_set_mb")?;
        }
        if let Some(v) = map.get(Value::String("object_size_distribution".to_owned())) {
            params.object_size_distribution = parse_object_size(v)?;
        }
        if let Some(v) = map.get(Value::String("lifetime_distribution".to_owned())) {
            params.lifetime_distribution = parse_lifetime(v)?;
        }

        Ok(params)
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

fn parse_object_size(v: &Value) -> Result<ObjectSizeDistribution, RegimeError> {
    let s = string_value(v, "object_size_distribution")?;
    match s.as_str() {
        "small" => Ok(ObjectSizeDistribution::Small),
        "medium" => Ok(ObjectSizeDistribution::Medium),
        "mixed" => Ok(ObjectSizeDistribution::Mixed),
        _ => Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: "object_size_distribution".to_owned(),
            value: s,
            reason: "must be one of: small, medium, mixed",
        }),
    }
}

fn parse_lifetime(v: &Value) -> Result<LifetimeDistribution, RegimeError> {
    let s = string_value(v, "lifetime_distribution")?;
    match s.as_str() {
        "short" => Ok(LifetimeDistribution::Short),
        "mixed" => Ok(LifetimeDistribution::Mixed),
        _ => Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: "lifetime_distribution".to_owned(),
            value: s,
            reason: "must be one of: short, mixed",
        }),
    }
}

fn string_value(v: &Value, key: &str) -> Result<String, RegimeError> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(RegimeError::WrongType {
            regime: REGIME_ID,
            key: key.to_owned(),
            expected: "string",
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

/// R1 — steady-state-healthy.
#[derive(Debug, Clone, Copy, Default)]
pub struct SteadyStateRegime;

impl Regime for SteadyStateRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = SteadyStateParams::from_yaml(&scenario.spec.regime.parameters)?;

        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!(
            "allocation_rate_mb_s={}",
            params.allocation_rate_mb_s
        ));
        argv.push(format!("live_set_mb={}", params.live_set_mb));
        argv.push(format!(
            "object_size_distribution={}",
            params.object_size_distribution.as_str()
        ));
        argv.push(format!(
            "lifetime_distribution={}",
            params.lifetime_distribution.as_str()
        ));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["young_gc_steady"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "young_ratio >= 0.8",
            "full_count == 0",
            "p99_pause_ms < 50",
            "variance_pause_count_pct < 5",
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
  name: test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 1g, max: 2g }}
  regime:
    kind: steady-state-healthy
    parameters: {params_yaml}
  duration: 30s
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = SteadyStateParams::from_yaml(&Value::Null).unwrap();
        assert_eq!(p, SteadyStateParams::default());
    }

    #[test]
    fn parses_full_params() {
        let v: Value = serde_yaml::from_str(
            r"allocation_rate_mb_s: 80
live_set_mb: 200
object_size_distribution: small
lifetime_distribution: short
",
        )
        .unwrap();
        let p = SteadyStateParams::from_yaml(&v).unwrap();
        assert_eq!(p.allocation_rate_mb_s, 80);
        assert_eq!(p.live_set_mb, 200);
        assert_eq!(p.object_size_distribution, ObjectSizeDistribution::Small);
        assert_eq!(p.lifetime_distribution, LifetimeDistribution::Short);
    }

    #[test]
    fn rejects_unknown_parameter() {
        let v: Value = serde_yaml::from_str("allocation_rate_mb_s: 50\nbogus_knob: 1\n").unwrap();
        let err = SteadyStateParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_negative_rate() {
        let v: Value = serde_yaml::from_str("allocation_rate_mb_s: -5\n").unwrap();
        let err = SteadyStateParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_string_for_numeric_field() {
        let v: Value = serde_yaml::from_str("allocation_rate_mb_s: fast\n").unwrap();
        let err = SteadyStateParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::WrongType { .. }));
    }

    #[test]
    fn rejects_unknown_object_size_distribution() {
        let v: Value = serde_yaml::from_str("object_size_distribution: huge\n").unwrap();
        let err = SteadyStateParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_non_mapping_parameters() {
        let v: Value = serde_yaml::from_str("- list\n- of items\n").unwrap();
        let err = SteadyStateParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::WrongType { .. }));
    }

    #[test]
    fn workload_args_emits_positional_then_kv() {
        let s = scenario_with_params("{ allocation_rate_mb_s: 80 }");
        let argv = SteadyStateRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "steady-state-healthy");
        assert_eq!(argv[1], "PT30S");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "allocation_rate_mb_s=80"));
        assert!(argv.iter().any(|a| a == "live_set_mb=100")); // default
        assert!(argv.iter().any(|a| a == "object_size_distribution=mixed"));
        assert!(argv.iter().any(|a| a == "lifetime_distribution=mixed"));
    }

    #[test]
    fn workload_args_uses_overrides() {
        let s =
            scenario_with_params("{ allocation_rate_mb_s: 200, object_size_distribution: small }");
        let argv = SteadyStateRegime.workload_args(&s).unwrap();
        assert!(argv.iter().any(|a| a == "allocation_rate_mb_s=200"));
        assert!(argv.iter().any(|a| a == "object_size_distribution=small"));
    }

    #[test]
    fn expected_phenomena_includes_young_gc_steady() {
        assert_eq!(
            SteadyStateRegime.expected_phenomena(),
            vec!["young_gc_steady"]
        );
    }

    #[test]
    fn expected_invariants_match_spec() {
        let inv = SteadyStateRegime.expected_invariant_rules();
        assert!(inv.contains(&"young_ratio >= 0.8"));
        assert!(inv.contains(&"full_count == 0"));
        assert!(inv.contains(&"p99_pause_ms < 50"));
    }

    #[test]
    fn iso_duration_handles_minutes_and_hours() {
        use gc_forge_scenario::Duration as Dur;
        assert_eq!(iso_duration(&Dur::from_secs(30)), "PT30S");
        assert_eq!(iso_duration(&Dur::from_secs(120)), "PT2M");
        assert_eq!(iso_duration(&Dur::from_secs(3600)), "PT1H");
        assert_eq!(iso_duration(&Dur::from_secs(0)), "PT0S");
    }
}
