//! Evaluator that turns a manifest's `expected_invariants` into pass/fail
//! results against a parsed GC log.
//!
//! Recognised rule shapes are documented in the iteration 13 plan. Unknown
//! rules return [`Outcome::Skipped`], never `Failed`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use serde_yaml::Value;

use crate::parser::{GcEventKind, ParsedLog};

/// Result of evaluating one rule.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Passed { observed: Value },
    Failed { observed: Value, reason: String },
    Skipped { reason: String },
}

impl Outcome {
    /// Returns the observed value if any.
    #[must_use]
    pub fn observed(&self) -> Option<&Value> {
        match self {
            Self::Passed { observed } | Self::Failed { observed, .. } => Some(observed),
            Self::Skipped { .. } => None,
        }
    }
}

/// Comparison operator extracted from a rule string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
}

impl Op {
    fn parse(token: &str) -> Option<Self> {
        match token {
            "<" => Some(Self::Lt),
            "<=" => Some(Self::Le),
            ">" => Some(Self::Gt),
            ">=" => Some(Self::Ge),
            "==" => Some(Self::Eq),
            _ => None,
        }
    }

    fn check_int(self, observed: i64, threshold: i64) -> bool {
        match self {
            Self::Lt => observed < threshold,
            Self::Le => observed <= threshold,
            Self::Gt => observed > threshold,
            Self::Ge => observed >= threshold,
            Self::Eq => observed == threshold,
        }
    }

    fn check_float(self, observed: f64, threshold: f64) -> bool {
        match self {
            Self::Lt => observed < threshold,
            Self::Le => observed <= threshold,
            Self::Gt => observed > threshold,
            Self::Ge => observed >= threshold,
            Self::Eq => (observed - threshold).abs() < 1e-9,
        }
    }
}

/// Splits a rule into `(metric, op, threshold-from-rule)`. Threshold from the
/// rule string is informational only — the actual threshold comes from the
/// invariant's YAML payload.
fn split_rule(rule: &str) -> Option<(String, Op)> {
    let trimmed = rule.trim();
    // Try the longer ops first so that `<=` doesn't match `<`.
    for op_str in ["<=", ">=", "==", "<", ">"] {
        if let Some(idx) = trimmed.find(op_str) {
            let metric = trimmed[..idx].trim().to_owned();
            return Some((metric, Op::parse(op_str)?));
        }
    }
    None
}

fn yaml_as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn yaml_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn yaml_as_bool(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" => Some(true),
            "false" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Evaluates a single rule against the parsed log.
#[must_use]
pub fn evaluate(rule: &str, threshold: &Value, parsed: &ParsedLog) -> Outcome {
    if let Some((metric, op)) = split_rule(rule) {
        return evaluate_comparison(rule, &metric, op, threshold, parsed);
    }
    // Boolean-shape rules: `humongous_regions_in_log`, `mixed_gc_regular`, …
    evaluate_boolean(rule, threshold, parsed)
}

fn evaluate_comparison(
    rule: &str,
    metric: &str,
    op: Op,
    threshold: &Value,
    parsed: &ParsedLog,
) -> Outcome {
    // Integer-valued metrics.
    let int_metric: Option<i64> = match metric {
        "young_count" => Some(parsed.count(GcEventKind::Young) as i64),
        "mixed_count" => Some(parsed.count(GcEventKind::Mixed) as i64),
        "full_count" => Some(parsed.count(GcEventKind::Full) as i64),
        "concurrent_cycle_count" => Some(parsed.concurrent_cycles as i64),
        "evacuation_failure_count" => Some(parsed.evacuation_failures as i64),
        _ => None,
    };

    // Float-valued metrics.
    let float_metric: Option<f64> = match metric {
        "young_ratio" => Some(parsed.young_ratio()),
        "mean_pause_ms" => Some(parsed.mean_pause_ms()),
        m if m.starts_with('p') && m.ends_with("_pause_ms") => {
            // `p99_pause_ms`, `p50_pause_ms`, …
            let pct_part = &m[1..m.len() - "_pause_ms".len()];
            pct_part
                .parse::<f64>()
                .ok()
                .map(|p| parsed.percentile_pause_ms(p))
        }
        _ => None,
    };

    if let Some(observed) = int_metric {
        let Some(t) = yaml_as_i64(threshold) else {
            return Outcome::Skipped {
                reason: format!("threshold for {rule:?} is not an integer"),
            };
        };
        let observed_yaml = Value::Number(serde_yaml::Number::from(observed));
        return if op.check_int(observed, t) {
            Outcome::Passed {
                observed: observed_yaml,
            }
        } else {
            Outcome::Failed {
                observed: observed_yaml,
                reason: format!("{observed} did not satisfy {rule}"),
            }
        };
    }
    if let Some(observed) = float_metric {
        let Some(t) = yaml_as_f64(threshold) else {
            return Outcome::Skipped {
                reason: format!("threshold for {rule:?} is not a number"),
            };
        };
        let observed_yaml = serde_yaml::to_value(observed).unwrap_or(Value::Null);
        return if op.check_float(observed, t) {
            Outcome::Passed {
                observed: observed_yaml,
            }
        } else {
            Outcome::Failed {
                observed: observed_yaml,
                reason: format!("{observed} did not satisfy {rule}"),
            }
        };
    }

    Outcome::Skipped {
        reason: format!("unknown metric {metric:?}"),
    }
}

fn evaluate_boolean(rule: &str, threshold: &Value, parsed: &ParsedLog) -> Outcome {
    let observed: Option<bool> = match rule.trim() {
        "humongous_regions_in_log" => Some(parsed.humongous_seen),
        "no_evacuation_failure" => Some(parsed.evacuation_failures == 0),
        "oom_seen" => Some(parsed.oom_seen),
        _ => None,
    };

    let Some(observed) = observed else {
        return Outcome::Skipped {
            reason: format!("rule {rule:?} not implemented in this version"),
        };
    };
    let expected = yaml_as_bool(threshold).unwrap_or(true);
    let observed_yaml = Value::Bool(observed);
    if observed == expected {
        Outcome::Passed {
            observed: observed_yaml,
        }
    } else {
        Outcome::Failed {
            observed: observed_yaml,
            reason: format!("expected {expected}, observed {observed}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{GcEvent, GcEventKind};

    fn log_with_events(events: Vec<GcEvent>) -> ParsedLog {
        ParsedLog {
            events,
            ..ParsedLog::default()
        }
    }

    fn ev(kind: GcEventKind, pause_ms: f64) -> GcEvent {
        GcEvent {
            kind,
            timestamp_ms: 0,
            pause_ms,
            heap_before_bytes: 0,
            heap_after_bytes: 0,
        }
    }

    #[test]
    fn full_count_zero_passes_when_no_full_events() {
        let log = log_with_events(vec![ev(GcEventKind::Young, 1.0)]);
        let outcome = evaluate("full_count == 0", &Value::Number(0.into()), &log);
        assert!(matches!(outcome, Outcome::Passed { .. }));
    }

    #[test]
    fn full_count_zero_fails_when_full_present() {
        let log = log_with_events(vec![ev(GcEventKind::Full, 100.0)]);
        let outcome = evaluate("full_count == 0", &Value::Number(0.into()), &log);
        assert!(matches!(outcome, Outcome::Failed { .. }));
    }

    #[test]
    fn young_ratio_passes_when_high_enough() {
        let log = log_with_events(vec![
            ev(GcEventKind::Young, 1.0),
            ev(GcEventKind::Young, 2.0),
            ev(GcEventKind::Young, 3.0),
            ev(GcEventKind::Young, 4.0),
            ev(GcEventKind::Mixed, 5.0),
        ]);
        let threshold = serde_yaml::to_value(0.8).unwrap();
        let outcome = evaluate("young_ratio >= 0.8", &threshold, &log);
        assert!(matches!(outcome, Outcome::Passed { .. }));
    }

    #[test]
    fn p99_pause_ms_strict_lt_passes() {
        let log = log_with_events(vec![
            ev(GcEventKind::Young, 5.0),
            ev(GcEventKind::Young, 6.0),
            ev(GcEventKind::Young, 7.0),
        ]);
        let threshold = Value::Number(50.into());
        let outcome = evaluate("p99_pause_ms < 50", &threshold, &log);
        assert!(matches!(outcome, Outcome::Passed { .. }));
    }

    #[test]
    fn unknown_metric_yields_skipped() {
        let log = ParsedLog::default();
        let outcome = evaluate("magic_number == 42", &Value::Number(42.into()), &log);
        assert!(matches!(outcome, Outcome::Skipped { .. }));
    }

    #[test]
    fn unknown_boolean_rule_yields_skipped() {
        let log = ParsedLog::default();
        let outcome = evaluate("nonsense_invariant", &Value::Bool(true), &log);
        assert!(matches!(outcome, Outcome::Skipped { .. }));
    }

    #[test]
    fn humongous_regions_passes_when_seen() {
        let log = ParsedLog {
            humongous_seen: true,
            ..ParsedLog::default()
        };
        let outcome = evaluate("humongous_regions_in_log", &Value::Bool(true), &log);
        assert!(matches!(outcome, Outcome::Passed { .. }));
    }

    #[test]
    fn no_evacuation_failure_passes_when_zero() {
        let log = ParsedLog::default();
        let outcome = evaluate("no_evacuation_failure", &Value::Bool(true), &log);
        assert!(matches!(outcome, Outcome::Passed { .. }));
    }

    #[test]
    fn no_evacuation_failure_fails_when_present() {
        let log = ParsedLog {
            evacuation_failures: 2,
            ..ParsedLog::default()
        };
        let outcome = evaluate("no_evacuation_failure", &Value::Bool(true), &log);
        assert!(matches!(outcome, Outcome::Failed { .. }));
    }

    #[test]
    fn integer_threshold_as_string_works() {
        let log = log_with_events(vec![ev(GcEventKind::Young, 1.0)]);
        let threshold = Value::String("1".to_owned());
        let outcome = evaluate("young_count >= 1", &threshold, &log);
        assert!(matches!(outcome, Outcome::Passed { .. }));
    }

    #[test]
    fn split_rule_handles_double_char_op() {
        let (metric, op) = split_rule("p99_pause_ms <= 50").unwrap();
        assert_eq!(metric, "p99_pause_ms");
        assert_eq!(op, Op::Le);
    }

    #[test]
    fn split_rule_returns_none_without_operator() {
        assert!(split_rule("just a rule").is_none());
    }
}
