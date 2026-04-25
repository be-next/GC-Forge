# Changelog

All notable changes to GC-Forge are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
