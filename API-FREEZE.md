# API freeze — iteration 6 (algos-zgc-parallel)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

Adds the two `steady-*-baseline` presets that were missing (ZGC, Parallel)
and a feature-gated integration test that runs all three baselines through
`gc-forge run` to validate the iter-3 flag builder + iter-5 orchestrator
against ZGC and Parallel in real Docker.

No public Rust or Java API change.

## Public Rust surface added

None.

## Public Java surface added

None.

## YAML schema changes

None. Two new preset YAMLs land under `presets/`:

- `presets/steady-zgc-baseline.yaml` — `extends: steady-g1-baseline.yaml`,
  overrides `spec.gc.algorithm` to `ZGC`, leaves `generational` to its
  algorithm-natural default (`true` on JDK 21+).
- `presets/steady-parallel-baseline.yaml` — `extends: steady-g1-baseline.yaml`,
  overrides `spec.gc.algorithm` to `Parallel`. Drops the G1-only knobs by
  inheritance (none are set in the parent anyway).

## Invariants for Tester-unit

- Both new presets pass `gc-forge lint`.
- `Scenario::resolve` on each new preset returns a scenario whose
  `spec.gc.algorithm` matches the file name.
- The Docker integration test, when feature-flagged on, runs each of the
  three baselines for ≤ 5 s and asserts:
  - `Using G1` appears in the G1 log;
  - `Using The Z Garbage Collector` (or `Using ZGC`) appears in the ZGC log;
  - `Using Parallel` appears in the Parallel log.

## Doc sections to author (Doc-writer)

- `doc/user/regimes.md` — add a sentence under R1 noting that all three
  MVP algorithms ship a `steady-*-baseline` preset.
- `doc/user/getting-started.md` — add an inline example showing how to
  swap baselines (`presets/steady-zgc-baseline.yaml`).
- `CHANGELOG.md` — Unreleased: two presets, three-algo integration test.

## Approval

- Coder: A6 — frozen 2026-04-25
- Reviewer: A1 — `Approved: A1 2026-04-25` (no functional change beyond presets and tests; ZGC threshold inheritance noted as deferred refinement, not a red flag).
