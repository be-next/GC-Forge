# API freeze — iteration 11 (regime-mixed-pathological, R6)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

R6 (`mixed-gc-pathological`) lands. Same template as previous regimes.

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `MixedGcPathologicalRegime`            | struct | implements `Regime` for R6 |
| `MixedGcPathologicalParams`            | struct | typed view of R6 parameters |
| `MixedGcPathologicalParams::from_yaml` | fn     | parses with defaults, rejects unknown keys |

`resolve(&RegimeSpec)` now also returns `MixedGcPathologicalRegime` for
`kind: "mixed-gc-pathological"`.

## Public Java surface added

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.regimes.MixedGcPathologicalRegime` | class | `id() = "mixed-gc-pathological"`, registered. |

## YAML schema changes

None at the typed level. One new preset ships under `presets/`:

- `presets/mixed-pathological-g1.yaml` — G1, 2 GiB heap, 5 min.

## Invariants for Tester-unit

Rust:
- `MixedGcPathologicalParams::from_yaml(Value::Null)` returns the documented
  defaults (`old_gen_pressure=0.7`, `fragmentation_factor=2.0`,
  `survivor_age_target=15`).
- `old_gen_pressure` rejects values outside `(0, 1]`.
- `fragmentation_factor` rejects values outside `[1.0, 3.0]`.
- `survivor_age_target` rejects values > 15 (the JVM hard-cap).
- `workload_args` emits the regime kind, ISO duration, hex seed, and the three
  `key=value` parameters.
- `resolve` on `kind: "mixed-gc-pathological"` returns the regime.

Java:
- `MixedGcPathologicalRegime` runs to completion within a 5 s budget for short
  windows.
- Unknown keys / out-of-range values rejected at run time.

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — fill the R6 section.
- `CHANGELOG.md` — Unreleased: R6 regime, one preset, integration test.

## Approval

- Coder: A5 — frozen 2026-04-25
- Reviewer: A6 — `Approved: A6 2026-04-25` (read against SPEC-FONCTIONNELLE §4.6; heap-relative `old_gen_pressure` approximation via `Runtime.maxMemory` noted as best-effort).
