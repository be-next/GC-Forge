# API freeze — iteration 8 (regime-humongous, R3)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

R3 (`humongous-pressure`) lands as the third concrete `Regime`. Same shape
as R1/R2 — Java implementation, Rust typed view, `resolve()` registration,
two presets.

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `HumongousPressureRegime`            | struct | implements `Regime` for R3 |
| `HumongousPressureParams`            | struct | typed view of R3 parameters |
| `HumongousPressureParams::from_yaml` | fn     | parses with defaults, rejects unknown keys |
| `HumongousSize`                      | enum   | `Auto` (default 2 MiB) or `Fixed(u32)` KiB |

`resolve(&RegimeSpec)` now also returns `HumongousPressureRegime` for
`kind: "humongous-pressure"`.

## Public Java surface added

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.regimes.HumongousPressureRegime` | class | `id() = "humongous-pressure"`, registered. |

## YAML schema changes

None at the typed level. Two new presets ship under `presets/`:

- `presets/humongous-g1-classic.yaml` — G1, 2 GiB heap, 2 min,
  `humongous_ratio: 0.5`, mixed-GC efficient.
- `presets/humongous-g1-evac-fail.yaml` — G1, 1 GiB heap, 2 min,
  `humongous_ratio: 0.7`, evacuation-failure expected.

## Invariants for Tester-unit

Rust:
- `HumongousPressureParams::from_yaml(Value::Null)` returns the defaults
  (`humongous_ratio=0.5`, `humongous_size_kb=Auto`, `allocation_rate_mb_s=80`).
- `humongous_ratio` rejects `0`, negative values, and values `> 1`.
- `humongous_size_kb: auto` parses to `HumongousSize::Auto`; integers parse to `Fixed(n)`.
- `workload_args` emits the regime kind, ISO duration, hex seed, and the four `key=value` parameters.
- `resolve` on `kind: "humongous-pressure"` returns the regime.

Java:
- `HumongousPressureRegime` runs to completion within the requested
  duration without throwing.
- Unknown parameter keys are rejected.
- `humongous_ratio` validation matches Rust (`(0.0, 1.0]`).

CLI integration:
- `humongous-g1-classic` preset, run for 12 s, produces a log containing
  the substring `humongous` (lower-case) — emitted by the G1 logger when
  humongous allocations occur.

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — fill the R3 section.
- `CHANGELOG.md` — Unreleased: R3 regime (Java + Rust), two presets.

## Approval

- Coder: A2 — frozen 2026-04-25
- Reviewer: A3 — `Approved: A3 2026-04-25` (read against SPEC-FONCTIONNELLE §4.3; `humongous_size_kb: auto = 2 MiB` simplification noted as deliberate; ratio clamping OK).
