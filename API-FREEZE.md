# API freeze — iteration 10 (regime-slow-leak, R4)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

R4 (`slow-leak`) lands. Same template as previous regimes.

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `SlowLeakRegime`            | struct | implements `Regime` for R4 |
| `SlowLeakParams`            | struct | typed view of R4 parameters |
| `SlowLeakParams::from_yaml` | fn     | parses with defaults, rejects unknown keys |

`resolve(&RegimeSpec)` now also returns `SlowLeakRegime` for `kind: "slow-leak"`.

## Public Java surface added

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.regimes.SlowLeakRegime` | class | `id() = "slow-leak"`, registered. |

## YAML schema changes

None at the typed level. Two new presets ship under `presets/`:

- `presets/leak-g1-slow.yaml` — G1, 1 GiB heap, 10 min, default R4 params.
- `presets/leak-zgc-slow.yaml` — extends G1 preset, swaps to ZGC.

## Invariants for Tester-unit

Rust:
- `SlowLeakParams::from_yaml(Value::Null)` returns the documented defaults
  (`leak_rate_mb_s=0.5`, `live_set_initial_mb=200`).
- Unknown keys → `RegimeError::UnknownParameter`.
- `leak_rate_mb_s` rejects values ≤ 0; `live_set_initial_mb` rejects 0.
- Rust accepts `leak_rate_mb_s` as both float YAML scalar (`0.5`) and string
  (`"0.5"`).
- `workload_args` emits the regime kind, ISO duration, hex seed, and the two
  `key=value` parameters.
- `resolve` on `kind: "slow-leak"` returns the regime.

Java:
- `SlowLeakRegime` runs to completion within a 5 s budget for short windows.
- Unknown keys / non-positive values are rejected at run time.

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — fill the R4 section.
- `CHANGELOG.md` — Unreleased: R4 regime, two presets, integration test.

## Approval

- Coder: A4 — frozen 2026-04-25
- Reviewer: A5 — `Approved: A5 2026-04-25` (read against SPEC-FONCTIONNELLE §4.4; float `leak_rate_mb_s` and OOM-as-exit-status approach OK).
