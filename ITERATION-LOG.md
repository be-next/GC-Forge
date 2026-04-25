# Iteration log

Living log of GC-Forge implementation iterations. One entry per iteration.
Maintained by the Teamlead role. See `doc/process/orchestration.md` for the process.

---

## Iteration 3 — docker-runner-mvp

- **Started:** 2026-04-25
- **Status:** merged
- **Branch:** `iter/03-docker-runner-mvp` (merged into `main`)
- **Goal:** stand up the JVM orchestration layer — `Runner` trait, `DockerRunner` backend, and the JVM flag builder that turns a `Scenario` into a `java …` command line. Refs: SPEC-TECHNIQUE §4.5 and §6.1.

### Roles (this iteration)

| Role | Agent | Note |
|------|-------|------|
| Teamlead   | A2 | was Coder in iter 2 |
| Coder      | A3 | was Reviewer in iter 2 |
| Reviewer   | A4 | was Tester-unit in iter 2 |
| Tester-unit | A5 | was Tester-func in iter 2 |
| Tester-func | A6 | was Doc-writer in iter 2 |
| Doc-writer | A1 | was Teamlead in iter 2 |

Rotation rule satisfied.

### Plan

1. Define the `Runner` trait + `RunSpec` + `RunOutcome` + `RunnerError`.
2. Build the JVM flag translator (pure function `Scenario → Vec<String>`).
3. Implement `DockerRunner` (subprocess management, `docker run`, log capture, `java -version` probe).
4. Tests: heavy unit coverage on the flag builder; argv-construction tests for the runner; one integration test that actually launches Docker (gated to skip if Docker is absent).
5. Doc + CHANGELOG.

### Decisions log

- The Runner trait stays minimal at iter 3 (`check_available` + `execute`); `ensure_jvm` mentioned in SPEC §4.5 lands in iter 18+ for the native runner — Docker images don't need it.
- The integration test is **runtime-gated** rather than `#[ignore]` so contributors without Docker still get a green `cargo test`. The DoD gate script invokes a separate `make` target for Docker integration tests.
- Per SPEC §6.1, the unified `-Xlog` shape is hard-coded for now (`gc*=info,gc+heap=debug,gc+age=trace,gc+phases=debug,gc+humongous=trace:file=<path>:time,level,tags,pid,tid:filecount=0`). Algo-specific tags (e.g. ZGC's `relocation`, `marking`) land alongside their algorithm presets in iter 6.
- Image tag policy: `eclipse-temurin:<major>-jdk-jammy` is the default; the `--image` runner option lets callers override (used by the corpus regen pipeline of GC-Insight to pin a known SHA).

### Metrics (at merge)

- Rust unit tests: 64/64 passed across the workspace (37 scenario + 23 runner + 4 placeholder version + 1 doctest, 1 CLI smoke).
- Java unit tests: 2/2 passed (unchanged).
- Docker integration test: 1/1 passed locally against `gc-forge-runner:dev-jdk21` (3.3 s wall-clock for a 3 s G1 scenario).
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `mvn verify`: green.
- DoD-gate (phase 1): green.

### Decisions taken in flight

- **`--entrypoint=java` is forced** by `DockerRunner` regardless of image. Discovered when running against `gc-forge-runner:dev-jdk21` (whose Dockerfile has `ENTRYPOINT ["java"]`): the assembled `java <flags> …` argument list landed *after* the entrypoint, producing `java java <flags>` and a `Could not find or load main class java` error. Forcing `--entrypoint=java` makes the runner image-agnostic and the produced command line equally readable in either image variant.
- **Embedded-harness mode** (`with_embedded_harness`): added as a first-class option. Production runner images (`gc-forge-runner:*-jdk*`) bake the harness at `/opt/gc-forge/harness.jar`; in that mode the host jar mount is dropped. This also sidesteps Docker Desktop on macOS struggling with overlapping bind mounts of host paths in different filesystems (encountered while writing the integration test with a `tempdir()`-mounted log directory).
- **Integration test gating**: feature-flag (`docker-integration`) over `#[ignore]`. Reason: contributors without Docker still get a green default `cargo test`, and `make docker-integration-tests` builds the image and harness jar before invoking the gated tests — so the dependency is explicit, not "works on my machine".
- **Probe before run**: a separate `docker run … java -version` invocation captures the JVM's full version string (e.g. `openjdk version "21.0.10" 2026-…`) for the manifest. The probe is best-effort: if it fails, `RunOutcome::jvm_version` is `None` and the actual run still proceeds.
- **Exit code 137 maps to `ExitStatus::Oom`**: Docker's OOM killer signals SIGKILL which translates to 128+9=137 on Linux. Treating this as `Oom` rather than `Failure(137)` makes downstream policy clearer.

### Bilan

The runner stack is the spine of every subsequent iteration: starting iter 4, every regime will pipe through `RunSpec → DockerRunner::execute → RunOutcome → log file on disk`. Today's implementation gives that spine a clean separation:

- the **flag builder** is a pure function (no I/O, 8 dedicated unit tests) so changing JVM defaults later is safe;
- the **Docker argv builder** is also pure (10 unit tests) so we can add corpus-pinned images, sandbox policies, or non-Docker backends without rewriting the run loop;
- the **Runner trait** is only two methods, leaving room for a `NativeRunner` in iter 18+ without surface churn.

The `--entrypoint` and embedded-harness fixes were both surfaced by *running real Docker*, which validates the choice of feature-gated integration tests over more-elaborate mocking. Both fixes are documented and tested.

Two soft items deferred:
1. **Wall-clock timeout** is declared in `RunnerError::Timeout` but not yet enforced — `Command::status()` blocks indefinitely. A timeout wrapper lands in iter 5 (`run-end-to-end`) where the CLI flag exists.
2. **JFR capture** (`output.capture_jfr`) is parsed but ignored by the runner. The flag will be wired in V1 per SPEC-FONCTIONNELLE §6.

Iteration 4 (`harness-steady-state`) can start: the runner takes a scenario today and produces a 3 s G1 GC log against Temurin 21 with 18+ events. Wiring the typed steady-state regime is the next link.

---

## Iteration 2 — scenario-parser

- **Started:** 2026-04-25
- **Status:** merged
- **Branch:** `iter/02-scenario-parser` (merged into `main`)
- **Goal:** parse the `gc-forge/scenario.v1` YAML, resolve `extends:` chains, apply CLI `--override`, generate the JSON Schema, and expose `gc-forge lint`. Refs: SPEC-FONCTIONNELLE §5, §9.4.

### Roles (this iteration)

| Role | Agent | Note |
|------|-------|------|
| Teamlead   | A1 | was Coder in iter 1 |
| Coder      | A2 | was Reviewer |
| Reviewer   | A3 | was Tester-unit |
| Tester-unit | A4 | was Tester-func |
| Tester-func | A5 | was Doc-writer |
| Doc-writer | A6 | was Teamlead |

Rotation rule satisfied: nobody holds the same role two iterations in a row.

### Plan

Backbone work, no scope creep:

1. Define Rust types for `Scenario` (metadata, spec, jvm, gc, regime, expected). Use `serde` + `schemars` + `thiserror`. Reject unknown vendor/algo at the type level.
2. Implement the loader: `Scenario::from_path(&Path)`, with `apiVersion`/`kind` check and structured error reporting.
3. Resolve `extends:` recursively (deep map merge, scalar override, cycle detection, relative-path resolution).
4. Apply `--override KEY=VALUE` (dotted path, scalar coercion).
5. Generate `schemas/scenario-v1.json` from `schemars`. Hash-check the schema in tests to detect silent drift.
6. Wire `gc-forge lint <path>` (non-zero exit on failure, clear error output).
7. Tests: unit for types, integration with sample YAML fixtures (good + bad).
8. Documentation: `doc/user/scenario-reference.md`, `CHANGELOG` entry.

### Decisions log

- Override syntax: dotted path (`spec.gc.options.heap.max=4g`), no JSONPath dialect at this stage. Quoted strings via shell quoting only — scalar values are coerced to the target field's type via `serde_yaml`.
- `apiVersion: gc-forge/scenario.v1` is rejected in any other form (no fuzzy matching).
- The schema file is committed and regenerated from `schemars` via `cargo run --bin gen-schema` (binary in `gc-forge-scenario`). CI compares the on-disk schema against a fresh regeneration.

### Metrics (at merge)

- Rust unit tests: 41/41 passed across the workspace (37 in `gc-forge-scenario`, 5 placeholder `version()` tests, 1 doctest, 1 CLI smoke).
- Java unit tests: 2/2 passed (unchanged — no harness changes this iteration).
- `cargo fmt --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo deny`: skipped (binary not installed locally).
- Coverage floor: 0 % (phase 1).
- `mvn verify`: green.
- `gc-forge lint`: smoke-tested manually on a good and bad scenario; exit codes 0 / 1 respectively.
- DoD-gate (phase 1): green.

### Decisions taken in flight

- **`generational` field default**: dropped the eager `default_generational` function (clippy `unnecessary_wraps`) — the field is `Option<bool>`, naturally `None` when absent. The runner will resolve the algorithm-specific default (`true` for ZGC on JDK 21+) when it lands in iter 6.
- **Workspace lint `missing_docs`**: temporarily lowered to `allow`. 102 warnings would have buried real issues; we'll progressively re-enable it per-crate as APIs stabilise (target end of Phase 2 for the shared surface).
- **Workspace lint `clippy::nursery`**: removed (kept `pedantic`). Nursery flags fight with bootstrapping APIs; revisit after the surface stabilises.
- **DoD-gate script bug fix**: `cmd && ok "..."` chains masked failures because `set -e` does not exit on the failing left-hand side of an `&&` short-circuit. Rewrote the checks as explicit `if ... then ok else fail fi` blocks. `cargo fmt` failures now properly fail the gate.
- **Override syntax**: dotted path on the resolved scenario tree (no JSONPath dialect). Right-hand side parsed as YAML so scalars, arrays, and maps are all uniformly accepted; schema-breaking overrides are rejected on re-deserialisation.
- **Schema drift detection**: an in-tree drift test reads `schemas/scenario-v1.json` and compares it to a freshly-rendered schema. CI surfaces drift early; the dedicated `gen-schema` binary regenerates the file.

### Bilan

The scenario crate landed clean: 37 focused unit tests covering byte sizes, durations, types, the loader, the `extends` chain (single + three-level + cycle), overrides (parse + apply + bad path + schema-breaking), and the JSON Schema drift contract. `gc-forge lint <good>` exits 0; `gc-forge lint <bad-apiVersion>` exits 1 with a typed error. The end-to-end CLI surface is now `gc-forge --version`, `gc-forge lint <path> [--override KEY=VALUE]…`.

The DoD-gate bug discovered during this iteration is exactly the kind of silent failure the gate was meant to prevent — it was masking fmt drift. Both the bug and the fix are documented above so the script's contract is no longer misleading.

Two soft items deferred to later iterations:
1. The runner-side default for `generational` (iter 3 or 6 depending on when ZGC lands).
2. Re-enabling `missing_docs` per crate once each crate has a stable public surface.

Iteration 3 (`docker-runner-mvp`) can start on a clean baseline: a typed scenario, validated and overrideable, ready to be turned into JVM flags.

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
