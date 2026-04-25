//! R3 — humongous-pressure regime, Rust side.
//!
//! Reference: SPEC-FONCTIONNELLE §4.3.

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use gc_forge_scenario::Scenario;

use crate::regime::{Regime, RegimeError};

const REGIME_ID: &str = "humongous-pressure";
const KNOWN_KEYS: &[&str] = &[
    "humongous_ratio",
    "humongous_size_kb",
    "region_size_mb",
    "allocation_rate_mb_s",
];

/// Typed view of `scenario.spec.regime.parameters` for [`HumongousPressureRegime`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumongousPressureParams {
    /// Probability that a given allocation is humongous; in `(0.0, 1.0]`.
    pub humongous_ratio: f64,
    pub humongous_size_kb: HumongousSize,
    /// Informational; the JVM picks the actual region size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_size_mb: Option<u32>,
    pub allocation_rate_mb_s: u32,
}

impl Default for HumongousPressureParams {
    fn default() -> Self {
        Self {
            humongous_ratio: 0.5,
            humongous_size_kb: HumongousSize::Auto,
            region_size_mb: None,
            allocation_rate_mb_s: 80,
        }
    }
}

/// Either `auto` (harness-side default — 2 MiB) or a fixed integer KiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HumongousSize {
    #[serde(rename = "auto")]
    Auto,
    Fixed(u32),
}

impl HumongousSize {
    fn as_arg(self) -> String {
        match self {
            Self::Auto => "auto".to_owned(),
            Self::Fixed(n) => n.to_string(),
        }
    }
}

impl HumongousPressureParams {
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
        if let Some(v) = map.get(Value::String("humongous_ratio".to_owned())) {
            params.humongous_ratio = parse_ratio(v)?;
        }
        if let Some(v) = map.get(Value::String("humongous_size_kb".to_owned())) {
            params.humongous_size_kb = parse_size(v)?;
        }
        if let Some(v) = map.get(Value::String("region_size_mb".to_owned())) {
            params.region_size_mb = Some(positive_u32(v, "region_size_mb")?);
        }
        if let Some(v) = map.get(Value::String("allocation_rate_mb_s".to_owned())) {
            params.allocation_rate_mb_s = positive_u32(v, "allocation_rate_mb_s")?;
        }

        Ok(params)
    }
}

fn parse_ratio(v: &Value) -> Result<f64, RegimeError> {
    let n = match v {
        Value::Number(n) => n.as_f64().ok_or_else(|| RegimeError::WrongType {
            regime: REGIME_ID,
            key: "humongous_ratio".to_owned(),
            expected: "number in (0, 1]",
            got: format!("{v:?}"),
        })?,
        Value::String(s) => s.parse::<f64>().map_err(|_| RegimeError::WrongType {
            regime: REGIME_ID,
            key: "humongous_ratio".to_owned(),
            expected: "number in (0, 1]",
            got: s.clone(),
        })?,
        _ => {
            return Err(RegimeError::WrongType {
                regime: REGIME_ID,
                key: "humongous_ratio".to_owned(),
                expected: "number in (0, 1]",
                got: yaml_type_name(v).to_owned(),
            });
        }
    };
    if !(n > 0.0 && n <= 1.0) {
        return Err(RegimeError::OutOfRange {
            regime: REGIME_ID,
            key: "humongous_ratio".to_owned(),
            value: n.to_string(),
            reason: "must be in (0, 1]",
        });
    }
    Ok(n)
}

fn parse_size(v: &Value) -> Result<HumongousSize, RegimeError> {
    match v {
        Value::String(s) if s == "auto" => Ok(HumongousSize::Auto),
        Value::Number(_) | Value::String(_) => {
            let n = positive_u32(v, "humongous_size_kb")?;
            if n == 0 {
                return Err(RegimeError::OutOfRange {
                    regime: REGIME_ID,
                    key: "humongous_size_kb".to_owned(),
                    value: "0".to_owned(),
                    reason: "must be > 0",
                });
            }
            Ok(HumongousSize::Fixed(n))
        }
        _ => Err(RegimeError::WrongType {
            regime: REGIME_ID,
            key: "humongous_size_kb".to_owned(),
            expected: "positive integer (KiB) or \"auto\"",
            got: yaml_type_name(v).to_owned(),
        }),
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

/// R3 — humongous-pressure.
#[derive(Debug, Clone, Copy, Default)]
pub struct HumongousPressureRegime;

impl Regime for HumongousPressureRegime {
    fn id(&self) -> &'static str {
        REGIME_ID
    }

    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError> {
        let params = HumongousPressureParams::from_yaml(&scenario.spec.regime.parameters)?;
        let mut argv = vec![
            REGIME_ID.to_owned(),
            iso_duration(&scenario.spec.duration),
            format!("0x{:X}", scenario.spec.seed.as_u64()),
        ];
        argv.push(format!("humongous_ratio={}", params.humongous_ratio));
        argv.push(format!(
            "humongous_size_kb={}",
            params.humongous_size_kb.as_arg()
        ));
        if let Some(rs) = params.region_size_mb {
            argv.push(format!("region_size_mb={rs}"));
        }
        argv.push(format!(
            "allocation_rate_mb_s={}",
            params.allocation_rate_mb_s
        ));
        Ok(argv)
    }

    fn expected_phenomena(&self) -> Vec<&'static str> {
        vec!["humongous_allocation"]
    }

    fn expected_invariant_rules(&self) -> Vec<&'static str> {
        vec![
            "humongous_alloc_per_sec >= 1",
            "humongous_regions_visible_in_log",
            "mixed_gc_triggered_below_ihop",
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
  name: humongous-test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: G1
    options:
      heap: {{ min: 2g, max: 2g }}
  regime:
    kind: humongous-pressure
    parameters: {params_yaml}
  duration: 2m
  seed: 0xC0FFEE
"
        );
        Scenario::from_yaml(&yaml, "<test>").unwrap()
    }

    #[test]
    fn defaults_when_parameters_null() {
        let p = HumongousPressureParams::from_yaml(&Value::Null).unwrap();
        assert_eq!(p, HumongousPressureParams::default());
    }

    #[test]
    fn parses_full_params() {
        let v: Value = serde_yaml::from_str(
            r"humongous_ratio: 0.7
humongous_size_kb: 1024
region_size_mb: 4
allocation_rate_mb_s: 100
",
        )
        .unwrap();
        let p = HumongousPressureParams::from_yaml(&v).unwrap();
        assert!((p.humongous_ratio - 0.7).abs() < 1e-9);
        assert_eq!(p.humongous_size_kb, HumongousSize::Fixed(1024));
        assert_eq!(p.region_size_mb, Some(4));
        assert_eq!(p.allocation_rate_mb_s, 100);
    }

    #[test]
    fn parses_humongous_size_auto() {
        let v: Value = serde_yaml::from_str("humongous_size_kb: auto\n").unwrap();
        let p = HumongousPressureParams::from_yaml(&v).unwrap();
        assert_eq!(p.humongous_size_kb, HumongousSize::Auto);
    }

    #[test]
    fn rejects_unknown_key() {
        let v: Value = serde_yaml::from_str("not_a_key: 1\n").unwrap();
        let err = HumongousPressureParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::UnknownParameter { .. }));
    }

    #[test]
    fn rejects_zero_ratio() {
        let v: Value = serde_yaml::from_str("humongous_ratio: 0.0\n").unwrap();
        let err = HumongousPressureParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_ratio_above_one() {
        let v: Value = serde_yaml::from_str("humongous_ratio: 1.5\n").unwrap();
        let err = HumongousPressureParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn rejects_negative_ratio() {
        let v: Value = serde_yaml::from_str("humongous_ratio: -0.1\n").unwrap();
        let err = HumongousPressureParams::from_yaml(&v).unwrap_err();
        assert!(matches!(err, RegimeError::OutOfRange { .. }));
    }

    #[test]
    fn workload_args_emits_positional_and_kv() {
        let s = scenario_with_params("{}");
        let argv = HumongousPressureRegime.workload_args(&s).unwrap();
        assert_eq!(argv[0], "humongous-pressure");
        assert_eq!(argv[1], "PT2M");
        assert_eq!(argv[2], "0xC0FFEE");
        assert!(argv.iter().any(|a| a == "humongous_ratio=0.5"));
        assert!(argv.iter().any(|a| a == "humongous_size_kb=auto"));
        assert!(argv.iter().any(|a| a == "allocation_rate_mb_s=80"));
        // region_size_mb is optional, should be absent at default
        assert!(!argv.iter().any(|a| a.starts_with("region_size_mb=")));
    }

    #[test]
    fn workload_args_includes_region_size_when_set() {
        let s = scenario_with_params("{ region_size_mb: 4 }");
        let argv = HumongousPressureRegime.workload_args(&s).unwrap();
        assert!(argv.iter().any(|a| a == "region_size_mb=4"));
    }

    #[test]
    fn expected_phenomena_includes_humongous_allocation() {
        assert_eq!(
            HumongousPressureRegime.expected_phenomena(),
            vec!["humongous_allocation"]
        );
    }

    #[test]
    fn expected_invariants_match_spec() {
        let inv = HumongousPressureRegime.expected_invariant_rules();
        assert!(inv.contains(&"humongous_alloc_per_sec >= 1"));
        assert!(inv.contains(&"humongous_regions_visible_in_log"));
    }
}
