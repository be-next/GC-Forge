# API freeze — iteration 7 (regime-burst, R2)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

R2 (`allocation-burst`) lands as the second concrete `Regime` on both sides
of the language boundary. The CLI orchestrator (iter 5) and runner (iter 3)
are unchanged; the new regime is wired in via the existing `RegimeRegistry`
(Java) and `resolve()` factory (Rust).

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `AllocationBurstRegime`              | struct | implements `Regime` for R2 |
| `AllocationBurstParams`              | struct | typed view of `regime.parameters` for R2 |
| `AllocationBurstParams::from_yaml`   | fn     | parses with defaults, rejects unknown keys |

`resolve(&RegimeSpec)` now also returns `AllocationBurstRegime` for
`kind: "allocation-burst"`.

## Public Java surface added

### Crate `workload-harness`

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.regimes.AllocationBurstRegime` | class | `id() = "allocation-burst"`, registered in `RegimeRegistry`. |

## YAML schema changes

None at the typed level. Two new preset YAMLs ship under `presets/`:

- `presets/burst-g1-30s.yaml` — G1, 2 GiB heap, 5 min duration, 30 s burst
  period, 5 s burst duration, base 30 MiB/s, burst 200 MiB/s.
- `presets/burst-parallel-30s.yaml` — same schedule, Parallel collector.

## Invariants for Tester-unit

Rust:
- `AllocationBurstParams::from_yaml(Value::Null)` returns the documented
  defaults (`base_rate_mb_s=30`, `burst_rate_mb_s=200`,
  `burst_duration_s=5`, `burst_period_s=30`).
- Unknown keys are rejected with `RegimeError::UnknownParameter`.
- Out-of-range values (`base_rate_mb_s` > `burst_rate_mb_s`,
  `burst_duration_s` > `burst_period_s`, zero period) fail with
  `RegimeError::OutOfRange`.
- `workload_args` emits the regime kind, ISO duration, hex seed, and the
  five `key=value` parameters in declaration order.
- `resolve` on `kind: "allocation-burst"` returns the regime.
- Both new presets pass `gc-forge lint`.

Java:
- `AllocationBurstRegime` runs to completion within the requested duration
  for short windows (< 5 s) without throwing.
- The two phases (`base` vs `burst`) are observable: a unit test instruments
  the regime with a fake clock and asserts that the rate switch happens at
  the expected boundary.
- Unknown parameter keys are rejected at run time.

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — fill the R2 section (parameters, defaults,
  signature attendue, phenomena), drop the `_TODO iter 7_` marker.
- `CHANGELOG.md` — Unreleased: R2 regime (Java + Rust), two presets,
  integration test extension.

## Approval

- Coder: A1 — frozen 2026-04-25
- Reviewer: A2 — `Approved: A2 2026-04-25` (read against SPEC-FONCTIONNELLE §4.2; phase-switch semantics OK; `bursts_count` deferral approved).
