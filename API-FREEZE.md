# API freeze — iteration 4 (harness-steady-state)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

Introduces the `Regime` abstraction on both sides of the language boundary:

- Java: `Regime` interface + `RegimeRegistry` + `SteadyStateRegime` implementing R1.
- Rust: `Regime` trait in `gc-forge-regimes` + typed `SteadyStateParams` + `workload_args` translator.

The full scenario → regime → runner → log pipeline lands next iteration.

## Public Rust surface added

### Crate `gc-forge-regimes`

| Item | Kind | Notes |
|------|------|-------|
| `Regime`                                       | trait  | `id`, `workload_args(scenario)`, `expected_phenomena`, `expected_invariant_rules` |
| `RegimeError`                                  | enum (thiserror) | `UnknownKind`, `UnknownParameter`, `WrongType`, `OutOfRange` |
| `SteadyStateRegime`                            | struct | implements `Regime` for R1 |
| `SteadyStateParams`                            | struct | typed view of `regime.parameters` for R1 |
| `SteadyStateParams::from_yaml(&Value)`         | fn     | parses with defaults, rejects unknown keys |
| `ObjectSizeDistribution`                       | enum   | `Small`, `Medium`, `Mixed` (default) |
| `LifetimeDistribution`                         | enum   | `Short`, `Mixed` (default) |
| `resolve(&RegimeSpec)`                         | fn     | factory: kind string → `Box<dyn Regime>` |

### Crate `gc-forge-cli`

No change.

### Crate `gc-forge-runner`

No change.

## Java harness public surface

| Item | Kind | Notes |
|------|------|-------|
| `dev.gcforge.harness.Regime`                   | interface | `void run(Map<String,String> params, Duration duration, long seed)` |
| `dev.gcforge.harness.RegimeRegistry`           | class | static `lookup(String kind)` returns the regime or throws |
| `dev.gcforge.harness.regimes.SteadyStateRegime`| class | implements `Regime` for R1 |
| `dev.gcforge.harness.WorkloadHarness.main`     | unchanged signature | new behaviour: parses `[kind] [duration] [seed] [k=v]…`; falls back to `steady-state-healthy PT10S 0xC0FFEE` when called with zero args. |
| `dev.gcforge.harness.alloc.ChunkSizer`         | class | parameterised chunk-size sampler (Small/Medium/Mixed) |

## YAML schema changes

None at the typed level. The free-form `regime.parameters` is now validated
when the regime is `steady-state-healthy`: the four documented keys are
accepted, anything else rejected with a precise error path.

## Invariants for Tester-unit

Rust:
- `SteadyStateParams::from_yaml(&Value::Null)` returns the documented defaults.
- Each parameter accepts integer and string forms where the spec allows.
- An unknown key fails with `RegimeError::UnknownParameter`.
- `workload_args` always emits at least the four positional args (`steady-state-healthy`, duration ISO, seed hex, …`k=v`).
- `expected_phenomena` returns `["young_gc_steady"]`.
- `resolve(&RegimeSpec { kind: "steady-state-healthy", … })` returns a `SteadyStateRegime`.
- `resolve(&RegimeSpec { kind: "unknown-kind", … })` returns `RegimeError::UnknownKind`.

Java:
- `WorkloadHarness.main(new String[]{})` runs the default fallback to completion (≤ 12 s).
- `WorkloadHarness.main` rejects unknown regime kinds with a non-zero exit.
- `SteadyStateRegime` consumes the documented parameters and rejects unknown keys.
- Two runs of `SteadyStateRegime` with the same seed produce the same chunk-size sequence (probed via `ChunkSizer.sample` directly).

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — new file with R1 section filled (parameters, defaults, signature attendue, expected phenomena). R2–R7 are listed with `_TODO iter N_` cross-references.
- `doc/user/cli-reference.md` — note the no-args fallback for the harness.
- `CHANGELOG.md` — Unreleased: Regime trait/registry, SteadyStateRegime (Java + Rust), SteadyStateParams, ObjectSizeDistribution, LifetimeDistribution.

## Approval

- Coder: A4 — frozen 2026-04-25
- Reviewer: A5 — `Approved: A5 2026-04-25` (read against SPEC-FONCTIONNELLE §4.1 and SPEC-TECHNIQUE §4.3 / §4.4; the symmetric Java/Rust split is consistent with the spec's "regime ne génère pas le log" principle).
