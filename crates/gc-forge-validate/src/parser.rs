//! Tiny GC log parser tuned for Temurin 21 unified logs.
//!
//! The parser is line-oriented and tolerant: lines that don't match a
//! recognised shape are skipped silently. The output is a [`ParsedLog`]
//! with the events the invariant evaluator needs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use std::fs;
use std::path::Path;

use thiserror::Error;

/// Coarse classification of a GC event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcEventKind {
    /// `Pause Young` (without the `Mixed` qualifier).
    Young,
    /// `Pause Young (Mixed)` or `Pause Young (Prepare Mixed)`.
    Mixed,
    /// `Pause Full`.
    Full,
    /// `Pause Remark`, `Pause Cleanup`, `Concurrent Mark Cycle`, etc.
    ConcurrentCycle,
}

/// One collection event extracted from the log.
#[derive(Debug, Clone, PartialEq)]
pub struct GcEvent {
    pub kind: GcEventKind,
    /// Wall-clock timestamp in milliseconds since the Unix epoch (best-effort:
    /// 0 if the timestamp couldn't be parsed).
    pub timestamp_ms: i64,
    /// Pause duration in milliseconds.
    pub pause_ms: f64,
    /// Heap occupancy before the event in bytes (0 if not extracted).
    pub heap_before_bytes: u64,
    /// Heap occupancy after the event in bytes (0 if not extracted).
    pub heap_after_bytes: u64,
}

/// Aggregate view of a parsed log.
#[derive(Debug, Clone, Default)]
pub struct ParsedLog {
    pub events: Vec<GcEvent>,
    /// `true` when at least one line matches the humongous markers
    /// (`humongous regions:`, `gc,humongous`, …).
    pub humongous_seen: bool,
    /// Number of evacuation-failure lines (Temurin: `Evacuation Failure`).
    pub evacuation_failures: u32,
    /// `true` when at least one `OutOfMemoryError` line was seen.
    pub oom_seen: bool,
    /// `true` when at least one concurrent-cycle marker was seen.
    pub concurrent_cycles: u32,
}

impl ParsedLog {
    /// Returns the number of events of a given kind.
    #[must_use]
    pub fn count(&self, kind: GcEventKind) -> usize {
        self.events.iter().filter(|e| e.kind == kind).count()
    }

    /// `young_count / total_count` (or 1.0 if there are no events).
    #[must_use]
    pub fn young_ratio(&self) -> f64 {
        if self.events.is_empty() {
            return 1.0;
        }
        let young = self.count(GcEventKind::Young) as f64;
        let total = self.events.len() as f64;
        young / total
    }

    /// Mean pause duration in milliseconds.
    #[must_use]
    pub fn mean_pause_ms(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.events.iter().map(|e| e.pause_ms).sum();
        sum / self.events.len() as f64
    }

    /// Returns the percentile pause duration in milliseconds.
    /// `pct` is in `[0, 100]`.
    #[must_use]
    pub fn percentile_pause_ms(&self, pct: f64) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.events.iter().map(|e| e.pause_ms).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let pct = pct.clamp(0.0, 100.0);
        let rank = (pct / 100.0 * (sorted.len() as f64 - 1.0)).round() as usize;
        sorted[rank.min(sorted.len() - 1)]
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Parses a GC log file by reading it from disk.
///
/// # Errors
/// Fails on I/O errors. Bad lines are tolerated and skipped.
pub fn parse_log(path: &Path) -> Result<ParsedLog, ParseError> {
    let text = fs::read_to_string(path).map_err(|e| ParseError::Io {
        path: path.to_owned(),
        source: e,
    })?;
    Ok(parse_log_text(&text))
}

/// Parses a GC log from an in-memory string. Useful for tests.
#[must_use]
pub fn parse_log_text(text: &str) -> ParsedLog {
    let mut log = ParsedLog::default();
    for line in text.lines() {
        scan_line(line, &mut log);
    }
    log
}

fn scan_line(line: &str, log: &mut ParsedLog) {
    let lower = line.to_ascii_lowercase();
    if lower.contains("humongous regions:") || lower.contains("[gc,humongous") {
        log.humongous_seen = true;
    }
    if line.contains("Evacuation Failure") || line.contains("evacuation failure") {
        log.evacuation_failures += 1;
    }
    if line.contains("OutOfMemoryError") {
        log.oom_seen = true;
    }
    if line.contains("Concurrent Mark Cycle")
        || line.contains("Concurrent Cycle")
        || line.contains("Concurrent Undo Cycle")
        || line.contains("Pause Remark")
        || line.contains("Pause Cleanup")
    {
        log.concurrent_cycles += 1;
        // Many concurrent markers also carry a duration; capture the pause
        // value so it shows up in the percentile calculations.
        if let Some(pause) = extract_pause_ms(line) {
            log.events.push(GcEvent {
                kind: GcEventKind::ConcurrentCycle,
                timestamp_ms: extract_timestamp_ms(line),
                pause_ms: pause,
                heap_before_bytes: 0,
                heap_after_bytes: 0,
            });
        }
    }

    if let Some((kind, pause_ms, heap_before, heap_after)) = scan_pause_line(line) {
        log.events.push(GcEvent {
            kind,
            timestamp_ms: extract_timestamp_ms(line),
            pause_ms,
            heap_before_bytes: heap_before,
            heap_after_bytes: heap_after,
        });
    }
}

/// Returns `(kind, pause_ms, heap_before, heap_after)` for `Pause` lines.
fn scan_pause_line(line: &str) -> Option<(GcEventKind, f64, u64, u64)> {
    let kind = if line.contains("Pause Full") {
        GcEventKind::Full
    } else if line.contains("Pause Young (Mixed)") || line.contains("Pause Young (Prepare Mixed)") {
        GcEventKind::Mixed
    } else if line.contains("Pause Young") {
        GcEventKind::Young
    } else {
        return None;
    };

    let pause = extract_pause_ms(line)?;
    let (before, after) = extract_heap_window(line).unwrap_or((0, 0));
    Some((kind, pause, before, after))
}

/// Extracts the trailing `<n>.<m>ms` token from a pause line.
fn extract_pause_ms(line: &str) -> Option<f64> {
    // Pause lines end with `... <heap window> <duration>ms`. Walk
    // backwards to find an `Nms`-shaped suffix.
    let trimmed = line.trim_end();
    if !trimmed.ends_with("ms") {
        return None;
    }
    let body = &trimmed[..trimmed.len() - 2];
    let mut start = body.len();
    for (i, c) in body.char_indices().rev() {
        if c.is_ascii_digit() || c == '.' {
            start = i;
        } else {
            break;
        }
    }
    body[start..].parse::<f64>().ok()
}

/// Extracts a `<before>-><after>(<capacity>)` window in bytes from a heap
/// line like `84M->84M(190M)`.
fn extract_heap_window(line: &str) -> Option<(u64, u64)> {
    // Find `->` and the surrounding sizes.
    let arrow_idx = line.find("->")?;
    let prefix = &line[..arrow_idx];
    let suffix = &line[arrow_idx + 2..];

    // Walk the prefix backwards to capture `<n>(K|M|G)`.
    let before = parse_trailing_size(prefix)?;
    let after = parse_leading_size(suffix)?;
    Some((before, after))
}

fn parse_trailing_size(s: &str) -> Option<u64> {
    let mut chars: Vec<char> = s.chars().collect();
    // Strip trailing whitespace (between size and `->`).
    while chars.last().is_some_and(|c| c.is_whitespace()) {
        chars.pop();
    }
    let unit = chars.pop()?;
    let mult: u64 = match unit {
        'K' | 'k' => 1024,
        'M' | 'm' => 1024 * 1024,
        'G' | 'g' => 1024 * 1024 * 1024,
        c if c.is_ascii_digit() => {
            chars.push(c);
            1
        }
        _ => return None,
    };
    let mut digits = String::new();
    while let Some(c) = chars.last() {
        if c.is_ascii_digit() {
            digits.insert(0, *c);
            chars.pop();
        } else {
            break;
        }
    }
    digits.parse::<u64>().ok().map(|n| n * mult)
}

fn parse_leading_size(s: &str) -> Option<u64> {
    let mut digits = String::new();
    let mut chars = s.chars();
    let mut unit = None;
    for c in chars.by_ref() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            unit = Some(c);
            break;
        }
    }
    let mult: u64 = match unit {
        Some('K' | 'k') => 1024,
        Some('M' | 'm') => 1024 * 1024,
        Some('G' | 'g') => 1024 * 1024 * 1024,
        _ => 1,
    };
    digits.parse::<u64>().ok().map(|n| n * mult)
}

/// Parses an ISO-8601 timestamp from the leading `[2026-04-25T13:13:47.389+0000]`
/// decorator. Returns 0 on failure.
fn extract_timestamp_ms(line: &str) -> i64 {
    let Some(start) = line.find('[') else {
        return 0;
    };
    let Some(end) = line[start + 1..].find(']') else {
        return 0;
    };
    let candidate = &line[start + 1..start + 1 + end];
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(candidate) {
        return dt.timestamp_millis();
    }
    // Temurin 21 uses `+0000` instead of `+00:00`; rewrite and retry.
    if let Some((date_part, tz_part)) = candidate.rsplit_once('+') {
        if tz_part.len() == 4 {
            let normalised = format!("{}+{}:{}", date_part, &tz_part[..2], &tz_part[2..]);
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&normalised) {
                return dt.timestamp_millis();
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_young_pause_line() {
        let line = "[2026-04-25T13:13:47.389+0000][1][7][info ][gc          ] GC(18) Pause Young (Normal) (G1 Evacuation Pause) 84M->84M(190M) 5.455ms";
        let log = parse_log_text(line);
        assert_eq!(log.events.len(), 1);
        let event = &log.events[0];
        assert_eq!(event.kind, GcEventKind::Young);
        assert!((event.pause_ms - 5.455).abs() < 1e-6);
        assert_eq!(event.heap_before_bytes, 84 * 1024 * 1024);
        assert_eq!(event.heap_after_bytes, 84 * 1024 * 1024);
    }

    #[test]
    fn parses_mixed_pause_line() {
        let line =
            "[info][gc] GC(20) Pause Young (Mixed) (G1 Evacuation Pause) 100M->90M(200M) 12.345ms";
        let log = parse_log_text(line);
        assert_eq!(log.events.len(), 1);
        assert_eq!(log.events[0].kind, GcEventKind::Mixed);
    }

    #[test]
    fn parses_prepare_mixed_as_mixed() {
        let line = "[info][gc] GC(18) Pause Young (Prepare Mixed) (G1 Evacuation Pause) 84M->84M(190M) 5.455ms";
        let log = parse_log_text(line);
        assert_eq!(log.events[0].kind, GcEventKind::Mixed);
    }

    #[test]
    fn parses_full_pause_line() {
        let line = "[info][gc] GC(50) Pause Full (System.gc()) 200M->50M(2G) 120.5ms";
        let log = parse_log_text(line);
        assert_eq!(log.events[0].kind, GcEventKind::Full);
        assert_eq!(log.events[0].heap_after_bytes, 50 * 1024 * 1024);
    }

    #[test]
    fn detects_humongous_marker() {
        let line = "[info][gc,heap] GC(7) Humongous regions: 4->2";
        let log = parse_log_text(line);
        assert!(log.humongous_seen);
    }

    #[test]
    fn detects_evacuation_failure() {
        let line = "[info][gc] GC(42) Evacuation Failure (Allocation Failure)";
        let log = parse_log_text(line);
        assert_eq!(log.evacuation_failures, 1);
    }

    #[test]
    fn detects_oom() {
        let line = "Exception in thread \"main\" java.lang.OutOfMemoryError: Java heap space";
        let log = parse_log_text(line);
        assert!(log.oom_seen);
    }

    #[test]
    fn aggregate_methods_handle_empty_log() {
        let log = ParsedLog::default();
        assert_eq!(log.count(GcEventKind::Young), 0);
        assert!((log.young_ratio() - 1.0).abs() < 1e-9);
        assert!(log.mean_pause_ms().abs() < 1e-9);
        assert!(log.percentile_pause_ms(99.0).abs() < 1e-9);
    }

    #[test]
    fn aggregates_match_hand_calculation() {
        let lines = "\
[info][gc] GC(0) Pause Young (Normal) 1M->1M(2M) 2.0ms
[info][gc] GC(1) Pause Young (Normal) 1M->1M(2M) 4.0ms
[info][gc] GC(2) Pause Young (Mixed) 1M->1M(2M) 10.0ms
[info][gc] GC(3) Pause Full 1M->1M(2M) 100.0ms
";
        let log = parse_log_text(lines);
        assert_eq!(log.events.len(), 4);
        assert_eq!(log.count(GcEventKind::Young), 2);
        assert_eq!(log.count(GcEventKind::Mixed), 1);
        assert_eq!(log.count(GcEventKind::Full), 1);
        assert!((log.young_ratio() - 0.5).abs() < 1e-9);
        assert!((log.mean_pause_ms() - 29.0).abs() < 1e-9);
        let p99 = log.percentile_pause_ms(99.0);
        assert!(p99 >= 100.0 - 0.01);
    }

    #[test]
    fn extracts_iso_timestamp_with_compact_offset() {
        let line = "[2026-04-25T13:13:47.389+0000][info][gc] GC(0) Pause Young 1M->1M(2M) 2.0ms";
        let log = parse_log_text(line);
        assert!(
            log.events[0].timestamp_ms > 0,
            "got {}",
            log.events[0].timestamp_ms
        );
    }

    #[test]
    fn skips_unrelated_lines() {
        let lines = "Hello world\nNothing to see here";
        let log = parse_log_text(lines);
        assert!(log.events.is_empty());
        assert!(!log.humongous_seen);
        assert_eq!(log.evacuation_failures, 0);
    }
}
