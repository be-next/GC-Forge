# Changelog

All notable changes to GC-Forge are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Cargo workspace skeleton with six crates: `gc-forge-cli`, `gc-forge-scenario`, `gc-forge-regimes`, `gc-forge-runner`, `gc-forge-validate`, `gc-forge-presets`.
- Maven `workload-harness` skeleton (Java 17 fat-jar with a hello-world allocation loop).
- Dockerfile based on `eclipse-temurin:21-jdk` for the demo runner.
- Makefile with `bootstrap`, `test`, `demo`, `dod-gate`, `lint`, `clean` targets.
- CI workflow (`lint`, `test`, `build`) on GitHub Actions.
- Governance artifacts: `BUGS.md`, `ITERATION-LOG.md`, `API-FREEZE.md`, `CHANGELOG.md`.
- `scripts/dod-gate.sh` running the iteration-1 baseline checks.
- Documentation skeletons: `doc/process/orchestration.md`, `doc/user/getting-started.md`.
