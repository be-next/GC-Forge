# Changelog

All notable changes to GC-Forge are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Three new GC algorithms and one variant** in `gc-forge-scenario`'s
  `GcAlgorithm` enum:
    - `Shenandoah` — emits `-XX:+UseShenandoahGC`. Honours
      `pause_target_ms`. Available on Temurin 17+.
    - `Serial` — emits `-XX:+UseSerialGC`. Single-threaded
      stop-the-world.
    - `Epsilon` — emits
      `-XX:+UnlockExperimentalVMOptions -XX:+UseEpsilonGC`.
      Performs no collection.
- **ZGC non-generational variant**. Setting
  `spec.gc.options.generational: false` now emits the explicit
  `-XX:-ZGenerational` flag, selecting the non-generational ZGC
  variant on JDK 21. The default (omitted or `true`) remains
  generational ZGC.
- **Seven new presets** covering the new algorithms:
  `steady-shenandoah-baseline`, `leak-shenandoah-slow`,
  `steady-serial-baseline`, `cache-serial-churn`, `epsilon-baseline`
  (negative baseline; no collection), `epsilon-leak-pure`
  (deterministic OOM), `steady-zgc-nongen-baseline`. The shipped
  catalogue grows from 14 to 21 presets.
- **Negative phenomenon row** in `doc/concepts/traceability.md`:
  `no_collection`, exhibited by `epsilon-baseline`. Used as a
  false-positive guard for downstream analysers — detectors should
  produce no output on the corresponding logs.
- Six new unit tests in `gc-forge-runner::flags` covering the new
  algorithms, the explicit-false ZGC case, and the unlock-then-use
  ordering for Epsilon.
- `doc/architecture.md` — single-page system architecture
  documenting the Rust workspace, the Java workload harness, the
  Docker runner subsystem, the wire formats, and the quality
  gates.
- `CONTRIBUTING.md` — contributor conventions, including the
  documentation-language policy (English for user-facing material,
  French for internal specifications), the Definition-of-Done
  gate, and the branching model.
- `.github/dependabot.yml` — automated dependency-update
  surveillance for the four ecosystems present in the repository
  (Cargo workspace, Maven workload harness, GitHub Actions, Docker
  base image). Runs weekly and groups minor/patch updates per
  ecosystem to keep the review surface manageable; major bumps
  land as separate pull requests. A *Dependency management*
  section was added to `CONTRIBUTING.md` describing the workflow
  and the exception for `Dockerfile.jdk17` (not auto-tracked).

### Changed

- JSON Schemas (`schemas/scenario-v1.json`,
  `schemas/run-manifest-v1.json`) regenerated to reflect the
  extended `GcAlgorithm` enum.
- `doc/user/scenario-reference.md` documents each algorithm's
  emitted flags.
- `doc/user/regimes.md` enumerates the new presets per regime.
- `README.md` status table reflects the new collector and preset
  counts (6 collectors, 21 presets).
- Documentation restructured around a scientific-style writing
  policy. `doc/pitch.md` was renamed to
  `doc/concepts/overview.md`; `doc/traceability.md` was moved to
  `doc/concepts/traceability.md`. Both files were rewritten with a
  sober register and an explicit conventions section.
- `README.md` is now the canonical entry point for the project,
  with a status table, a quickstart, an architecture summary,
  audience-segmented documentation links, and a citation entry.
- `doc/user/getting-started.md`, `doc/user/regimes.md`,
  `doc/user/cli-reference.md`, `doc/user/scenario-reference.md`,
  and `doc/user/README.md` underwent a tonal harmonisation.
  Per-regime "Available since: iteration N" annotations were
  removed; informal expressions ("yak shave", "footgun") were
  replaced.
- `doc/process/orchestration.md` updated for the new path of the
  traceability matrix and minor wording.

## [0.1.0] — 2026-04-25

First public release of GC-Forge — the declarative generator of Java GC
logs that pairs with GC-Insight.

### Highlights

- **Seven application regimes** (steady-state-healthy,
  allocation-burst, humongous-pressure, slow-leak, cache-churn,
  mixed-gc-pathological, microservice-stop-and-go) implemented on
  both sides (Rust regime + Java harness regime).
- **Three GC collectors** on Temurin 17 and 21: G1, generational ZGC,
  Parallel.
- **Fourteen presets** shipped in-tree and embedded in the binary:
  `steady-{g1,zgc,parallel}-baseline`, `burst-{g1,parallel}-30s`,
  `humongous-g1-{classic,evac-fail}`, `leak-{g1,zgc}-slow`,
  `cache-{g1,parallel}-churn`, `mixed-pathological-g1`,
  `microservice-{g1,zgc}-stop-go`.
- **Seven CLI subcommands**: `lint`, `run`, `validate`, `batch`,
  `presets list/show/export`, `selftest`, `variance-check`.
- **Three frozen wire formats** with JSON Schemas:
  `gc-forge/scenario.v1`, `gc-forge/run-manifest.v1`,
  `gc-forge/matrix.v1`.
- **Reproducible by construction**: every run writes a manifest
  carrying the resolved scenario, JVM flags, host fingerprint, seed,
  GC-log SHA-256 and harness JAR SHA-256.
- **Docker MVP runner** (eclipse-temurin:{17,21}-jdk-jammy) with
  `--rm --network=none --entrypoint=java` and an embedded-harness
  mode for the GC-Forge runner image.
- **Open-source MIT** dual licence + workspace metadata complete for
  crates.io publication.

### Added

- **Complete user documentation (iter 16).**
    - `doc/pitch.md` — one-page elevator pitch covering the problem,
      what GC-Forge does, target audiences, what ships in 0.1.0,
      and what comes next.
    - `doc/traceability.md` — phenomena × presets × GC-Insight detector
      matrix per SPEC-FONCTIONNELLE §11. The detector column carries
      `<TODO: detector-id>` placeholders until GC-Insight publishes
      stable detector identifiers; the GC-Forge-internal columns are
      frozen for 0.1.0.
    - `doc/user/README.md` — index that orders the four user-facing
      pages (`getting-started`, `regimes`, `scenario-reference`,
      `cli-reference`) for first-contact reading.
    - `doc/user/getting-started.md` — install stanza now reflects the
      from-source path; the batch and validate sections are filled in;
      a Troubleshooting section captures the recurring footguns
      encountered during MVP development (Docker entrypoint clash,
      macOS `/tmp` mount, stale runner image, faithful-duration vs.
      truncated CI runs).
    - `doc/user/cli-reference.md` and `doc/user/scenario-reference.md`:
      residual `_TODO iter N_` markers removed; surface frozen for
      0.1.0.
    - Top-level `README.md` upgraded with a Quickstart stanza pointing
      at the user docs.

- **Embedded preset catalogue.** `gc-forge-presets` now ships every `presets/*.yaml` body inside the binary via a `build.rs` script. `cargo install gc-forge-cli` installs a self-sufficient tool. `list_preset_names()`, `all_presets()`, `embedded_yaml(name)` are the public lookup helpers.
- `gc-forge presets list/show/export` subcommands. `list` shows name + algorithm + regime; presets using `extends:` are listed with an `(extends another preset)` annotation. `show` and `export` print the YAML body to stdout.
- `gc-forge selftest`: runs every embedded preset through the orchestrator, validates each log, exits `3` if any preset's validator failed. `--per-preset-duration` (default `12s`) keeps the full pass under ~3 minutes for nightly CI.
- `gc-forge variance-check <preset-or-path>`: repeats a scenario `N` times (default `5`) with the same seed and reports CV on `young_count`, `mean_pause_ms`, `p99_pause_ms`. Budgets: 8 % / 10 % / 20 % per SPEC §7.3. Exits `3` when any metric exceeds its budget.
- `doc/user/cli-reference.md`: dropped the iter-15 TODO marker; full documentation for presets, selftest, variance-check.
- `gc-forge-scenario::matrix` — typed model of `gc-forge/matrix.v1` (Matrix, MatrixSpec, MatrixCell). `Matrix::expand()` produces the cartesian product of axes and seeds, minus filter cells. Axis ordering is preserved via `IndexMap` so cell order is deterministic. 7 unit tests covering parsing, expansion (with/without seeds), filter exclusions, unknown api version.
- `gc-forge batch <matrix.yaml> [--out-dir DIR] [--image TAG] …` subcommand: runs each cell sequentially through the same orchestrator as `gc-forge run`, writes per-cell logs and manifests under `<out-dir>/cellN-<name>-<seed>.{log,manifest.yaml}`, and emits an `index.csv` row per cell. Supports `--continue-on-error` for the corpus regen pipeline; exits `2` when any cell failed.
- `doc/user/cli-reference.md` — `batch` section documents the matrix schema, axis/filter semantics, per-cell output filename convention, and exit codes.
- `gc-forge-validate`: GC log parser (`parse_log`, `ParsedLog`, `GcEvent`, `GcEventKind`) tuned for Temurin 21 unified logs. Tolerant line-by-line scan that surfaces young/mixed/full/concurrent events, pause durations, before/after heap sizes, plus humongous, evacuation-failure, OOM markers.
- `gc-forge-validate::evaluate` and `validate_invariants` — rule evaluator over the parsed log. Recognised rule shapes: count comparators on young/mixed/full/concurrent_cycle/evacuation_failure, ratio thresholds (`young_ratio`), pause percentile / mean thresholds, and boolean rules (`humongous_regions_in_log`, `no_evacuation_failure`, `oom_seen`). Unknown rules return `Skipped` for backward compatibility.
- `gc-forge validate <log> --manifest <path>` subcommand: prints a per-rule report, exits 3 on any failure (per SPEC §10.1), and optionally persists the result back into the manifest with `--update-manifest`.
- `doc/user/cli-reference.md` — `validate` section documented with the rule grammar.

### Fixed

- `gc-forge run` now copies the scenario's `spec.expected.invariants` into the manifest's `expected_invariants` (with their thresholds) instead of falling back to the regime's threshold-less rule list. Iter 13's validator uncovered this: previously every threshold landed as `null`, so every check was skipped.

- R7 `microservice-stop-and-go` regime on both sides — **closes the MVP regime catalogue at 7/7**:
    - `gc-forge-regimes::MicroserviceStopGoRegime` + `MicroserviceStopGoParams` with `active_period_s`, `idle_period_s`, `active_rate_mb_s`, `cycles` (`auto` or fixed). Registered in `resolve()`.
    - `dev.gcforge.harness.regimes.MicroserviceStopGoRegime` Java implementation: alternates active allocation phases with pure-sleep idle phases.
- Two new presets: `presets/microservice-g1-stop-go.yaml` (G1, 1 GiB) and `presets/microservice-zgc-stop-go.yaml` (ZGC, 1 GiB, extends G1).
- `doc/user/regimes.md` R7 section filled.
- R6 `mixed-gc-pathological` regime on both sides:
    - `gc-forge-regimes::MixedGcPathologicalRegime` + `MixedGcPathologicalParams` with `old_gen_pressure` (`(0, 1]`), `fragmentation_factor` (`[1.0, 3.0]`), `survivor_age_target` (`[1, 15]`). Registered in `resolve()`.
    - `dev.gcforge.harness.regimes.MixedGcPathologicalRegime` Java implementation: pre-fills old gen with chunks in interleaved size classes (small/medium/large), then sustains a steady allocation rate against the saturated heap. Registered in `RegimeRegistry`.
- One new preset: `presets/mixed-pathological-g1.yaml` (G1, 2 GiB).
- `doc/user/regimes.md` R6 section filled.
- R4 `slow-leak` regime on both sides:
    - `gc-forge-regimes::SlowLeakRegime` + `SlowLeakParams` with `leak_rate_mb_s` (`f64`, must be `> 0` and finite) and `live_set_initial_mb` (`u32`, ≥ 0). Registered in `resolve()`.
    - `dev.gcforge.harness.regimes.SlowLeakRegime` Java implementation. Pre-allocates the initial live-set, then bumps a never-evicted reference list at the configured leak rate. Registered in `RegimeRegistry`.
- Two new presets: `presets/leak-g1-slow.yaml` (G1, 1 GiB) and `presets/leak-zgc-slow.yaml` (ZGC, 1 GiB, extends G1).
- Integration test `leak_g1_preset_runs_through_pipeline` (gated on `docker-integration`) confirming the pipeline emits a G1 GC log for the leak preset on a short window.
- `doc/user/regimes.md` R4 section filled.
- R5 `cache-churn` regime on both sides:
    - `gc-forge-regimes::CacheChurnRegime` + `CacheChurnParams` with `cache_size_mb`, `eviction_rate_per_s`, `entry_lifetime_ms`, `entry_size_kb`. All four required and rejected at `0`. Registered in `resolve()`.
    - `dev.gcforge.harness.regimes.CacheChurnRegime` Java implementation: long-lived survivor pool with FIFO eviction at the configured rate. Registered in `RegimeRegistry`.
- Two new presets: `presets/cache-g1-churn.yaml` (G1, 4 GiB) and `presets/cache-parallel-churn.yaml` (Parallel, 4 GiB, extends G1).
- Integration test `cache_g1_preset_runs_and_produces_log` (gated on `docker-integration`) confirming the regime is registered and produces a G1 GC log.
- `doc/user/regimes.md` R5 section filled.
- R3 `humongous-pressure` regime on both sides:
    - `gc-forge-regimes::HumongousPressureRegime` + `HumongousPressureParams` with `humongous_ratio` (clamped to `(0, 1]`), `humongous_size_kb` (`auto` defaults to 2 MiB), optional `region_size_mb`, `allocation_rate_mb_s`. Registered in `resolve()`.
    - `dev.gcforge.harness.regimes.HumongousPressureRegime` Java implementation that allocates `humongous_size_kb`-KiB chunks at the configured ratio. Registered in `RegimeRegistry`.
- Two new presets: `presets/humongous-g1-classic.yaml` (G1, 2 GiB, ratio 0.5) and `presets/humongous-g1-evac-fail.yaml` (G1, 1 GiB, ratio 0.7, expects `evacuation_failure`).
- Integration test `humongous_g1_preset_emits_humongous_marker` (gated on `docker-integration`) validating the produced GC log contains the `humongous` marker.
- `doc/user/regimes.md` R3 section filled.
- R2 `allocation-burst` regime on both sides:
    - `gc-forge-regimes::AllocationBurstRegime` + `AllocationBurstParams` with `base_rate_mb_s`, `burst_rate_mb_s`, `burst_duration_s`, `burst_period_s`, `bursts_count` (`auto` or fixed). Validates `burst_rate ≥ base_rate` and `0 < burst_duration ≤ burst_period`. Registered in `resolve()`.
    - `dev.gcforge.harness.regimes.AllocationBurstRegime` Java implementation that alternates per-second allocation budgets on the burst schedule. Registered in `RegimeRegistry`. Public `rateAt` helper for testing the schedule without allocating.
- Two new presets: `presets/burst-g1-30s.yaml` (G1) and `presets/burst-parallel-30s.yaml` (Parallel, extends G1). Both 5-minute scenarios with a 5 s burst every 30 s, baseline 30 MiB/s, burst 200 MiB/s.
- `doc/user/regimes.md` R2 section filled (parameters, signature, phenomena, baselines).
- New CLI integration test `burst_g1_preset_runs_through_pipeline` (gated on `docker-integration`) running the burst preset for 12 s and asserting at least one Pause Young in the produced GC log.
- Two new presets: `presets/steady-zgc-baseline.yaml` (generational ZGC) and `presets/steady-parallel-baseline.yaml` (Parallel collector). Both `extends: steady-g1-baseline.yaml` and override only `spec.gc.algorithm`, so R1's regime parameters and expected invariants stay in lock-step across all three MVP algorithms.
- New CLI integration test (`crates/gc-forge-cli/tests/three_algos.rs`, gated on `docker-integration`) that runs all three baselines through `gc-forge run` and asserts the algorithm-specific marker shows up in each GC log (`Using G1` / `Using ZGC` / `Using Parallel`). Surface that the `--embedded-harness` flow does not require a host `harness.jar`.

### Fixed

- `gc-forge run` no longer requires the host harness JAR when `--embedded-harness` is set; the JAR check is now scoped to the bind-mount mode.

- `gc-forge run <scenario.yaml>`: end-to-end orchestrator. Loads the scenario (with `extends:` resolution + `--override` substitution), resolves the regime, executes through the Docker runner, and writes the GC log plus a `gc-forge/run-manifest.v1` document to `<out-dir>/<name>-<seed>.{log,manifest.yaml}`.
- `gc-forge-scenario::manifest`: typed model for the run manifest — `RunManifest`, `RunMeta`, `HostMeta`, `ScenarioRecord`, `JvmRecord`, `ReproducibilityRecord`, `OutputRecord`, `ExpectedInvariantRecord`, `ValidationRecord`, `ExitStatusRecord` (tagged enum). YAML and JSON output via `write_yaml` / `write_json`. SHA-256 helpers `sha256_hex` / `sha256_hex_bytes`.
- JSON Schema for the manifest at `schemas/run-manifest-v1.json`, generated alongside the scenario schema. Both are drift-checked at every test run.
- First preset shipped: `presets/steady-g1-baseline.yaml` — G1 / Temurin 21 / 2g heap / 90s steady-state-healthy, with the documented `expected_phenomena` and `expected_invariants`.
- `gc-forge-regimes`: `Regime` trait (`id`, `workload_args`, `expected_phenomena`, `expected_invariant_rules`) and `resolve(&RegimeSpec)` factory. `SteadyStateRegime` (R1) lands as the first concrete implementation, with `SteadyStateParams` typed view, `ObjectSizeDistribution` (small/medium/mixed) and `LifetimeDistribution` (short/mixed) enums, and a strict parameter parser that rejects unknown keys.
- Java workload harness: split into `Regime` interface, `RegimeRegistry`, and `dev.gcforge.harness.regimes.SteadyStateRegime`. `WorkloadHarness#main` now parses `[kind] [duration] [seed] [k=v]…` and falls back to `steady-state-healthy PT10S 0xC0FFEE` when called without arguments (preserves `make demo`). Allocation primitives in `dev.gcforge.harness.alloc` (`ChunkSizer` with deterministic seeded sampling).
- `doc/user/regimes.md`: regime catalog skeleton with R1 documented.
- `gc-forge-runner`: `Runner` trait with `check_available` + `execute`, `RunSpec`/`RunOutcome`/`ExitStatus`/`RunnerError` types.
- `gc-forge-runner::DockerRunner`: Docker-backed runner. Supports image override, embedded-harness mode (skips the host jar mount when the image bakes the harness), `--cpus` and `--memory` limits. Always passes `--rm --network=none --entrypoint=java` for reproducibility and security.
- `gc-forge-runner::flags::build_jvm_command`: pure translation of a resolved `Scenario` into a JVM argv. Implements the unified `-Xlog` shape mandated by SPEC-TECHNIQUE §6.1, plus the algorithm flag (`G1`/`ZGC[+ZGenerational]`/`Parallel`) and the per-algorithm tunables.
- 23 unit tests covering the flag builder and Docker argv construction; one integration test (gated on the `docker-integration` feature) that actually launches Docker against a 256 MB G1 scenario and asserts a parseable GC log.
- `doc/user/cli-reference.md`: CLI reference skeleton with the `lint` subcommand documented and TODO markers for the upcoming subcommands.
- `gc-forge-scenario`: typed model of `gc-forge/scenario.v1` (`Scenario`, `Spec`, `JvmSpec`, `GcSpec`, `HeapConfig`, `RegimeSpec`, …) with `serde`, `schemars`, and `thiserror`. `ByteSize` and `Duration` accept the JVM-style suffix forms.
- `Scenario::from_path` / `Scenario::resolve` — load a scenario file, with `extends:` chain resolution (deep merge, sequence replacement, cycle detection).
- `Override::parse` and `Scenario::apply_overrides` — `KEY=VALUE` overrides on dotted paths, applied after `extends:` resolution. Schema-breaking overrides are rejected.
- `gc-forge lint <scenario.yaml> [--override KEY=VALUE]…` — validates syntax, `apiVersion`/`kind`, the extends chain, and overrides without launching a JVM.
- `cargo run -p gc-forge-scenario --bin gen-schema` — generates `schemas/scenario-v1.json` from the typed model. A drift test fails the build if the on-disk schema is out of sync.
- Cargo workspace skeleton with six crates: `gc-forge-cli`, `gc-forge-scenario`, `gc-forge-regimes`, `gc-forge-runner`, `gc-forge-validate`, `gc-forge-presets`.
- Maven `workload-harness` skeleton (Java 17 fat-jar with a hello-world allocation loop).
- Dockerfile based on `eclipse-temurin:21-jdk` for the demo runner.
- Makefile with `bootstrap`, `test`, `demo`, `dod-gate`, `lint`, `clean` targets.
- CI workflow (`lint`, `test`, `build`) on GitHub Actions.
- Governance artifacts: `BUGS.md`, `ITERATION-LOG.md`, `API-FREEZE.md`, `CHANGELOG.md`.
- `scripts/dod-gate.sh` running the iteration-baseline checks.
- Documentation: `doc/process/orchestration.md`, `doc/user/getting-started.md`, `doc/user/scenario-reference.md`.
