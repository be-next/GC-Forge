# Iteration log

Living log of GC-Forge implementation iterations. One entry per iteration.
Maintained by the Teamlead role. See `doc/process/orchestration.md` for the process.

---

## Iteration 1 — bootstrap

- **Started:** 2026-04-25
- **Status:** merged
- **Branch:** `iter/01-bootstrap` (merged into `main`)
- **Goal:** scaffold Cargo workspace, Maven harness, CI, Docker, governance artifacts. Per plan §"Itération 1 (bootstrap)".

### Roles (this iteration)

| Role | Agent |
|------|-------|
| Teamlead   | A6 |
| Coder      | A1 |
| Reviewer   | A2 |
| Tester-unit | A3 |
| Tester-func | A4 |
| Doc-writer | A5 |

Bootstrap iteration: same agent (Claude) plays all roles sequentially in a single session, since the rotation rules apply from iteration 2 onward (no prior history).

### Decisions log

- Cargo workspace flat under `crates/`, members listed in root `Cargo.toml`. Aligned with SPEC-TECHNIQUE §2.1.
- Maven for the Java harness (`workload-harness/`), single fat-jar via `maven-shade-plugin`. Aligned with SPEC-TECHNIQUE §4.4.
- Rust toolchain pinned via `rust-toolchain.toml` to keep CI and dev in sync.
- `cargo deny` with permissive license allowlist (MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS-2016, Zlib) — see `RISQUES.md` R-L4.
- Docker base image: `eclipse-temurin:21-jdk` for the demo runner; `21-jdk-jammy` flavor for predictable glibc.
- DoD-gate plancher pour cette itération : `cargo fmt`, `cargo clippy`, `cargo test`, `cargo deny`, `mvn verify`. `selftest`, `variance-check`, coverage gates : reportés à partir de l'itération 5 (quand `gc-forge run` aura un sens à exécuter).

### Metrics (at merge)

- Rust workspace builds clean: yes (6 crates, 60 transitive deps).
- Rust unit tests: 5/5 passed (one `version()` test per library crate).
- Java unit tests: 2/2 passed (`WorkloadHarnessTest`).
- `cargo fmt --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo deny`: skipped (binary not installed locally; CI exercises it).
- Coverage floor: not enforced this phase (iter 1, floor 0 %); raises to 70 % from iter 6 onward.
- `mvn verify` (workload-harness): green, JaCoCo agent attached.
- `make demo`: produces a 327-line G1 GC log on Temurin 21.0.10+7-LTS, 18+ GC events including a Prepare-Mixed pause.
- Bugs opened: 0. Bugs closed: 0.
- DoD-gate (phase 1): green.
- Duration (calendar): same day, single working session.

### Decisions taken in flight

- **Rust toolchain bumped from 1.83 to 1.94.1.** Rationale: clap 4.6 (latest in the 4.x range) requires Rust 1.85 (edition 2024). Pinning the floor lower would have required restrictive transitive version pins, fragile against routine deps updates. 1.94.1 is the current stable as of 2026-04-25 and was already on the dev machine.
- **Clippy `nursery` group removed from workspace lints.** Pedantic stays. Nursery flags (e.g. `missing_const_for_fn`, `unnecessary_wraps`) fight against speculative API design at this stage of the project — they will be re-enabled once each crate has a stable surface (target: end of Phase 2).
- **Maven installed locally via Homebrew (3.9.15).** Required to run the harness build and the demo. `cargo-deny` and `cargo-llvm-cov` left uninstalled for now; `dod-gate.sh` skips them gracefully and CI installs them. To be installed locally on first iteration that needs the corresponding floor (iter 6 for coverage, sooner if a deny advisory shows up).
- **Branch policy**: iter/01-bootstrap is fast-forward merged onto main locally. No push to remote — that requires human approval per the autonomy boundary policy.

### Bilan

Bootstrap iteration is the smallest validation that the whole stack hangs together. It does. The end-to-end chain — Cargo workspace → Java fat-jar → Docker runner → GC log on disk — produces a real, parseable Temurin 21 G1 log with `make demo`. The DoD gate runs in ~12 s and is green. Iteration 2 (`scenario-parser`) can start from a known-good baseline.

Two latent risks to keep an eye on in subsequent iterations:
1. The Rust 1.94 floor pulls in modern crate versions; pin transitive deps in `Cargo.lock` and lean on `cargo deny` once installed.
2. The harness JAR is currently rebuilt from local `target/` for the Docker image — fine for iter 1, but the Dockerfile should switch to a multi-stage build with `mvn` inside the container by iter 5 to make the demo reproducible from a clean checkout.
