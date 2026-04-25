# Changelog

All notable changes to GC-Forge are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
