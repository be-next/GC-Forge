# API freeze — iteration 9 (regime-cache-churn, R5)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

R5 (`cache-churn`) lands. Same template as previous regimes.

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `CacheChurnRegime`            | struct | implements `Regime` for R5 |
| `CacheChurnParams`            | struct | typed view of R5 parameters |
| `CacheChurnParams::from_yaml` | fn     | parses with defaults, rejects unknown keys |

`resolve(&RegimeSpec)` now also returns `CacheChurnRegime` for
`kind: "cache-churn"`.

## Public Java surface added

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.regimes.CacheChurnRegime` | class | `id() = "cache-churn"`, registered. |

## YAML schema changes

None at the typed level. Two new presets ship under `presets/`:

- `presets/cache-g1-churn.yaml` — G1, 4 GiB heap, 5 min, default R5 params.
- `presets/cache-parallel-churn.yaml` — extends G1 preset, swaps to Parallel.

## Invariants for Tester-unit

Rust:
- `CacheChurnParams::from_yaml(Value::Null)` returns the documented defaults.
- Unknown keys → `RegimeError::UnknownParameter`.
- Zero `cache_size_mb`, zero `eviction_rate_per_s`, zero `entry_lifetime_ms`,
  zero `entry_size_kb` all rejected with `RegimeError::OutOfRange`.
- `workload_args` emits the regime kind, ISO duration, hex seed, and the four
  `key=value` parameters.
- `resolve` on `kind: "cache-churn"` returns the regime.

Java:
- `CacheChurnRegime` runs to completion on the defaults within a 5 s budget
  for short windows (≤ 1 s).
- Unknown keys / non-positive integers are rejected at run time.

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — fill the R5 section.
- `CHANGELOG.md` — Unreleased: R5 regime, two presets, integration test
  extension.

## Approval

- Coder: A3 — frozen 2026-04-25
- Reviewer: A4 — `Approved: A4 2026-04-25` (read against SPEC-FONCTIONNELLE §4.5; survivor-pool-as-FIFO simplification noted; integration test marker is a proxy and acceptable.)
