# API freeze — iteration 12 (regime-microservice, R7)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

R7 (`microservice-stop-and-go`) lands. Closes the MVP catalogue at 7/7
regimes.

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `MicroserviceStopGoRegime`            | struct | implements `Regime` for R7 |
| `MicroserviceStopGoParams`            | struct | typed view of R7 parameters |
| `MicroserviceStopGoParams::from_yaml` | fn     | parses with defaults, rejects unknown keys |
| `Cycles`                              | enum   | `Auto | Fixed(u32)` (mirrors R2's `BurstsCount`) |

`resolve(&RegimeSpec)` now also returns `MicroserviceStopGoRegime` for
`kind: "microservice-stop-and-go"`.

## Public Java surface added

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.regimes.MicroserviceStopGoRegime` | class | `id() = "microservice-stop-and-go"`, registered. |

## YAML schema changes

None at the typed level. Two new presets ship under `presets/`:

- `presets/microservice-g1-stop-go.yaml` — G1, 1 GiB heap, 5 min.
- `presets/microservice-zgc-stop-go.yaml` — ZGC, 1 GiB, extends G1.

## Invariants for Tester-unit

Rust:
- Defaults: `active_period_s=10`, `idle_period_s=20`, `active_rate_mb_s=100`,
  `cycles=auto`.
- Unknown keys → `RegimeError::UnknownParameter`.
- Zero `active_period_s`, zero `idle_period_s`, zero `active_rate_mb_s` rejected.
- `workload_args` emits the regime kind, ISO duration, hex seed, and four
  `key=value` parameters.
- `resolve` on `kind: "microservice-stop-and-go"` returns the regime.

Java:
- `MicroserviceStopGoRegime` runs to completion within a 5 s budget for
  short windows (≤ 2 s).
- The regime alternates: when wall-clock is in active phase, it allocates;
  when in idle phase, it sleeps (no allocations).

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — fill the R7 section. Note the catalogue is now
  complete.
- `CHANGELOG.md` — Unreleased: R7 regime, two presets, integration test.

## Approval

- Coder: A6 — frozen 2026-04-25
- Reviewer: A1 — `Approved: A1 2026-04-25` (read against SPEC-FONCTIONNELLE §4.7; alternation shape and `Cycles` enum mirror R2 cleanly).
