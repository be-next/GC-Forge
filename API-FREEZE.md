# API freeze — iteration 13 (validate-cmd)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

Stand up the GC log parser and invariant evaluator. Wires `gc-forge validate`.
This is the iteration where the manifest's `validation` block transitions
from "always Skipped" to actively-checked.

## Public Rust surface added

### Crate `gc-forge-validate`

| Item | Kind | Notes |
|------|------|-------|
| `ParsedLog`                    | struct | parsed view of a GC log |
| `GcEvent`                      | struct | one collection event (kind, timestamp, pause_ms, heap before/after) |
| `GcEventKind`                  | enum   | `Young`, `Mixed`, `Full`, `ConcurrentCycle` |
| `parse_log(path)`              | fn     | reads + parses a log file |
| `parse_log_text(&str)`         | fn     | parses a YAML-detached log string (for tests) |
| `Invariant`                    | struct | `rule: String`, `threshold: yaml::Value`, `evaluate(&ParsedLog) -> Outcome` |
| `Outcome`                      | enum   | `Passed`, `Failed { observed }`, `Skipped { reason }` |
| `validate_manifest(manifest, log) -> ValidationRecord` | fn | high-level entry: turns expected invariants into a populated ValidationRecord |
| `ParseError`                   | enum (thiserror) | I/O + line-shape errors |

### Crate `gc-forge-cli`

| Item | Kind | Notes |
|------|------|-------|
| `gc-forge validate` | clap subcommand | `<log>` `--manifest M` `[--update-manifest]` `[--manifest-format yaml|json]` |

## Public Java surface added

No change.

## YAML schema changes

None at the typed level. The manifest's `validation.results` field already
exists from iter 5; iter 13 starts populating it with real data.

## Invariants for Tester-unit

- `parse_log_text` extracts a `Pause Young` event from a Temurin-21 unified-log
  line and surfaces its pause duration in ms.
- `parse_log_text` distinguishes `Pause Young (Mixed)` from `Pause Young`.
- `parse_log_text` flags `humongous regions:` and `Evacuation failure` lines.
- `Invariant::evaluate` covers all the rule shapes listed in the iteration
  plan; unknown rules return `Skipped`.
- `validate_manifest` populates `ValidationRecord::status = Passed` when all
  recognised rules pass and `Failed` when any rule fails.
- The CLI integration test runs `gc-forge run ... && gc-forge validate ...` on
  the steady-g1-baseline preset and asserts the resulting manifest's
  `validation.status` is `Passed` or `Skipped` (never `Failed`).

## Doc sections to author (Doc-writer)

- `doc/user/cli-reference.md` — drop the `_TODO iter 13_` marker on the
  `validate` subcommand and document the rule grammar.
- `CHANGELOG.md` — Unreleased: log parser, invariant evaluator,
  `gc-forge validate` subcommand.

## Approval

- Coder: A1 — frozen 2026-04-25
- Reviewer: A2 — `Approved: A2 2026-04-25` (read against SPEC-FONCTIONNELLE §6.2 and §7.1; `Skipped`-by-default on unknown rules and threshold-from-YAML pattern noted as deliberate forward-compat).
