//! High-level validator that turns a manifest's `expected_invariants` into a
//! populated `ValidationRecord`.

use chrono::Utc;
use serde_yaml::Value;

use gc_forge_scenario::{
    ExpectedInvariantRecord, RunManifest, ValidationRecord, ValidationResult, ValidationStatus,
};

use crate::invariant::{evaluate, Outcome};
use crate::parser::ParsedLog;

/// Returns a `ValidationRecord` summarising the rule outcomes against the log.
///
/// - `Passed` if every recognised rule passed (skipped rules don't penalise).
/// - `Failed` if at least one rule failed.
/// - `Skipped` if every rule was skipped (no recognised metric in the manifest).
#[must_use]
pub fn validate_invariants(
    invariants: &[ExpectedInvariantRecord],
    parsed: &ParsedLog,
    validator_version: String,
) -> ValidationRecord {
    let mut results: Vec<ValidationResult> = Vec::with_capacity(invariants.len());
    let mut any_failed = false;
    let mut any_passed = false;
    for inv in invariants {
        match evaluate(&inv.rule, &inv.threshold, parsed) {
            Outcome::Passed { observed } => {
                any_passed = true;
                results.push(ValidationResult {
                    rule: inv.rule.clone(),
                    observed,
                    threshold: inv.threshold.clone(),
                    passed: true,
                });
            }
            Outcome::Failed {
                observed,
                reason: _,
            } => {
                any_failed = true;
                results.push(ValidationResult {
                    rule: inv.rule.clone(),
                    observed,
                    threshold: inv.threshold.clone(),
                    passed: false,
                });
            }
            Outcome::Skipped { reason } => {
                results.push(ValidationResult {
                    rule: inv.rule.clone(),
                    observed: Value::Null,
                    threshold: inv.threshold.clone(),
                    passed: false,
                });
                let _ = reason;
            }
        }
    }

    let status = if any_failed {
        ValidationStatus::Failed
    } else if any_passed {
        ValidationStatus::Passed
    } else {
        ValidationStatus::Skipped
    };

    ValidationRecord {
        status,
        results,
        validated_at: Some(Utc::now()),
        validator_version,
    }
}

/// Convenience: validate the manifest's `expected_invariants` against a parsed
/// log and update its `validation` field in-place.
pub fn apply_validation(manifest: &mut RunManifest, parsed: &ParsedLog, validator_version: String) {
    let record = validate_invariants(&manifest.expected_invariants, parsed, validator_version);
    manifest.validation = record;
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc_forge_scenario::ExpectedInvariantRecord;

    fn empty_log() -> ParsedLog {
        ParsedLog::default()
    }

    fn inv(rule: &str, threshold: Value) -> ExpectedInvariantRecord {
        ExpectedInvariantRecord {
            rule: rule.to_owned(),
            threshold,
        }
    }

    #[test]
    fn passes_when_all_rules_pass() {
        let log = empty_log(); // no events ⇒ full_count == 0, young_ratio == 1
        let invs = vec![
            inv("full_count == 0", Value::Number(0.into())),
            inv("young_ratio >= 0.8", serde_yaml::to_value(0.8).unwrap()),
        ];
        let record = validate_invariants(&invs, &log, "test".to_owned());
        assert_eq!(record.status, ValidationStatus::Passed);
        assert!(record.results.iter().all(|r| r.passed));
    }

    #[test]
    fn fails_when_any_rule_fails() {
        let log = ParsedLog {
            evacuation_failures: 5,
            ..ParsedLog::default()
        };
        let invs = vec![
            inv("full_count == 0", Value::Number(0.into())),
            inv("no_evacuation_failure", Value::Bool(true)),
        ];
        let record = validate_invariants(&invs, &log, "test".to_owned());
        assert_eq!(record.status, ValidationStatus::Failed);
        let failed = record.results.iter().find(|r| !r.passed).unwrap();
        assert_eq!(failed.rule, "no_evacuation_failure");
    }

    #[test]
    fn all_unknown_rules_yield_skipped_status() {
        let log = empty_log();
        let invs = vec![
            inv("not_a_known_metric == 0", Value::Number(0.into())),
            inv("another_made_up_rule", Value::Bool(true)),
        ];
        let record = validate_invariants(&invs, &log, "test".to_owned());
        assert_eq!(record.status, ValidationStatus::Skipped);
        assert!(record.results.iter().all(|r| !r.passed));
    }

    #[test]
    fn empty_invariants_yield_skipped_status() {
        let log = empty_log();
        let record = validate_invariants(&[], &log, "test".to_owned());
        assert_eq!(record.status, ValidationStatus::Skipped);
        assert!(record.results.is_empty());
    }
}
