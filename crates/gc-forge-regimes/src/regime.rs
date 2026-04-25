//! `Regime` trait and registry for GC-Forge.
//!
//! Reference: SPEC-TECHNIQUE §4.3.

use thiserror::Error;

use gc_forge_scenario::{RegimeSpec, Scenario};

use crate::allocation_burst::AllocationBurstRegime;
use crate::cache_churn::CacheChurnRegime;
use crate::humongous_pressure::HumongousPressureRegime;
use crate::steady_state::SteadyStateRegime;

/// One of the seven MVP regimes (R1..R7). The trait is intentionally
/// minimal: today it produces the harness CLI args and exposes the
/// expected phenomena/invariants. The full `validate(parsed_log, scenario)`
/// hook lands alongside the GC-log parser in iter 13.
pub trait Regime {
    /// Stable identifier (must match the value in `scenario.spec.regime.kind`
    /// and what the Java harness expects).
    fn id(&self) -> &'static str;

    /// Builds the workload-harness CLI arguments for this regime, given a
    /// resolved scenario. Returned slice is fed straight to
    /// `gc-forge-runner::RunSpec::workload_args`.
    ///
    /// # Errors
    /// Fails if the scenario's `regime.parameters` cannot be coerced into
    /// the regime's typed view.
    fn workload_args(&self, scenario: &Scenario) -> Result<Vec<String>, RegimeError>;

    /// Phenomenon ids the regime is designed to exhibit. Used to fill the
    /// manifest's `expected_phenomena` and to drive validation.
    fn expected_phenomena(&self) -> Vec<&'static str>;

    /// Quantified invariants attached to the regime. They surface in the
    /// manifest now and become enforceable in iter 13.
    fn expected_invariant_rules(&self) -> Vec<&'static str>;
}

/// Anything that can go wrong while resolving or parameterising a regime.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegimeError {
    #[error("unknown regime kind {0:?}")]
    UnknownKind(String),

    #[error("unknown parameter {key:?} for regime {regime:?} (known: {known:?})")]
    UnknownParameter {
        regime: &'static str,
        key: String,
        known: Vec<&'static str>,
    },

    #[error("parameter {key:?} for regime {regime:?} expected {expected}, got {got:?}")]
    WrongType {
        regime: &'static str,
        key: String,
        expected: &'static str,
        got: String,
    },

    #[error("parameter {key:?} = {value:?} is out of range for regime {regime:?}: {reason}")]
    OutOfRange {
        regime: &'static str,
        key: String,
        value: String,
        reason: &'static str,
    },
}

/// Factory: looks up the regime implementation registered for
/// `spec.kind`. The returned trait object is stateless w.r.t. the spec —
/// per-run parameters are passed on each call.
///
/// # Errors
/// Fails when the `kind` is not one of the registered MVP regimes.
pub fn resolve(spec: &RegimeSpec) -> Result<Box<dyn Regime>, RegimeError> {
    match spec.kind.as_str() {
        "steady-state-healthy" => Ok(Box::new(SteadyStateRegime)),
        "allocation-burst" => Ok(Box::new(AllocationBurstRegime)),
        "humongous-pressure" => Ok(Box::new(HumongousPressureRegime)),
        "cache-churn" => Ok(Box::new(CacheChurnRegime)),
        other => Err(RegimeError::UnknownKind(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc_forge_scenario::RegimeSpec;
    use serde_yaml::Value;

    fn spec(kind: &str) -> RegimeSpec {
        RegimeSpec {
            kind: kind.to_owned(),
            parameters: Value::Null,
        }
    }

    #[test]
    fn resolves_steady_state() {
        // Box<dyn Regime> is not Debug, so we can't use unwrap()/unwrap_err().
        match resolve(&spec("steady-state-healthy")) {
            Ok(r) => assert_eq!(r.id(), "steady-state-healthy"),
            Err(e) => panic!("expected Ok, got {e:?}"),
        }
    }

    #[test]
    fn rejects_unknown_kind() {
        match resolve(&spec("not-a-regime")) {
            Err(RegimeError::UnknownKind(_)) => {}
            Err(other) => panic!("expected UnknownKind, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }
}
