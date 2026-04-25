# Iteration log

Living log of GC-Forge implementation iterations. One entry per iteration.
Maintained by the Teamlead role. See `doc/process/orchestration.md` for the process.

---

## Iteration 6 — algos-zgc-parallel

- **Started:** 2026-04-25
- **Status:** merged
- **Branch:** `iter/06-algos-zgc-parallel` (merged into `main`)
- **Goal:** broaden algorithm coverage to ZGC (generational) and Parallel. The flag builder already supports them (iter 3); this iteration adds the two missing `steady-*-baseline` presets and exercises all three through Docker. Refs: SPEC-FONCTIONNELLE §3.2 (algo matrix), SPEC-TECHNIQUE §6.1 (`-Xlog`).

### Roles (this iteration)

| Role | Agent | Note |
|------|-------|------|
| Teamlead   | A5 | was Coder in iter 5 |
| Coder      | A6 | was Reviewer in iter 5 |
| Reviewer   | A1 | was Tester-unit in iter 5 |
| Tester-unit | A2 | was Tester-func in iter 5 |
| Tester-func | A3 | was Doc-writer in iter 5 |
| Doc-writer | A4 | was Teamlead in iter 5 |

Rotation rule satisfied.

### Plan

1. Author `presets/steady-zgc-baseline.yaml` and `presets/steady-parallel-baseline.yaml` using `extends: steady-g1-baseline.yaml` and overriding `spec.gc.algorithm` (and the regime's expected p99 threshold for ZGC, where the algorithm's natural p99 is sub-millisecond).
2. Lint both presets via `gc-forge lint`.
3. Add a Docker integration test that runs all three baselines (with a short duration override) and asserts the algorithm-specific marker in the GC log (`Using G1` / `Using The Z Garbage Collector` / `Using Parallel`).
4. Update the regime doc to mention the algorithm coverage and add a CHANGELOG entry.

### Decisions log

- **Use `extends:` for the new baselines** rather than full standalone YAMLs. The three preset files differ only in `spec.gc.algorithm` (and one threshold for ZGC); `extends` keeps them in lock-step when R1's invariants change. This also exercises the iter-2 `extends:` resolver against the iter-5 manifest pipeline end-to-end.
- **ZGC's `p99_pause_ms < 50` threshold stays** at 50 ms in the inherited expected_invariants, matching SPEC-FONCTIONNELLE §4.1's documented p99 ceiling for R1. ZGC will trivially clear it (typical p99 sub-millisecond), but tightening to e.g. 5 ms here would couple the regime threshold to the algorithm — out of scope for iter 6, and would require carving algorithm-specific invariants into the regime's `expected_invariant_rules`. Deferred to V1.
- **Integration test runs three short scenarios in sequence**, not in parallel. Docker-on-macOS contention with concurrent `docker run` invocations is not worth chasing at iter 6; the batch parallel runner lands in iter 14 with proper concurrency control.
- **Algorithm marker matching** uses substring assertions over exact phrases because Temurin 21's `gc,init` line is `Using ...` (with leading whitespace) — staying loose keeps the test resilient to JDK micro version drift.

### Metrics (at merge)

- Rust unit tests: 93/93 (an extra unit test in CLI for the embedded-harness path).
- Java unit tests: 18/18 (unchanged).
- Docker integration tests (CLI, gated): 3/3 — `g1_baseline_emits_using_g1`, `zgc_baseline_emits_using_zgc`, `parallel_baseline_emits_using_parallel`. Wall clock 14.2 s for the three (sequential).
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `mvn verify`: green.
- DoD-gate (phase 1): green.

### Decisions taken in flight

- **Bug found and fixed: `--embedded-harness` required a host JAR.** The first run of the integration test failed because `harness_jar_path` always asserted the host JAR existed, even when the runner image already embeds it. Fixed by scoping the existence check to the bind-mount mode; recorded as a fix in `CHANGELOG.md`. This also makes `cargo install gc-forge-cli`-based usage with the prebuilt runner image work without cloning the repo.
- **Test invocation uses `CARGO_BIN_EXE_gc-forge`**, the path to the freshly-built CLI binary that Cargo provides at compile time for tests in a binary crate. Avoids the brittle `target/debug/gc-forge` lookup and removes any reliance on `make build` ordering.
- **Tests run sequentially** via `-- --test-threads=1`. Three concurrent `docker run` invocations on macOS Docker Desktop spike CPU contention enough to push the inner JVM beyond the 3 s budget; sequential is fast enough (~14 s total) and stable.
- **`extends:` in the new presets** validates the iter-2 resolver against the iter-5 manifest pipeline — the resolved scenario in the manifest's `scenario.resolved` block contains the merged tree, with `extends:` stripped and the child's algorithm winning over the parent's.

### Bilan

Three baselines, three GC algorithms, one orchestrator. Iter 6 closes the algo-coverage chapter of Phase 2 with surprisingly little code (two YAMLs + a 100-line integration test) — exactly the dividend the iter-3 flag builder and iter-5 orchestrator were designed to pay. The CLI integration test is the first contributor that will catch ZGC- or Parallel-specific runner regressions automatically.

The `--embedded-harness`-skips-host-JAR fix is the kind of correctness improvement that tends to surface only when running real Docker. It is now documented in CHANGELOG and exercised by the test set.

Soft items deferred:
1. **Algorithm-aware invariant tightening** (e.g. ZGC's p99 is sub-millisecond, Parallel's longer young pauses). The R1 invariants stay universal at this iteration; per-algo refinements land alongside the validator in iter 13 where they'll have observed-vs-threshold semantics anyway.
2. **Manifest enrichment for embedded-harness mode**: `workload_jar_sha256` is empty when the JAR is in the image. Replacing it with an "image digest" is a clean follow-up but requires Docker SDK or a `docker inspect` shellout — out of scope for iter 6, will land alongside iter 17's release pipeline.

Iteration 7 (`regime-burst`) can start: R2 allocation-burst lands as the next regime, with the same shape as R1 (Java `Regime` impl + Rust `Regime` impl + 2 presets + invariants + the CLI passes through unchanged).

---

## Iteration 5 — run-end-to-end

- **Started:** 2026-04-25
- **Status:** merged
- **Branch:** `iter/05-run-end-to-end` (merged into `main`)
- **Goal:** compose the four lower-level building blocks (`gc-forge-scenario`, `gc-forge-regimes`, `gc-forge-runner`, the harness) into a single `gc-forge run` subcommand, write the run manifest defined in SPEC-FONCTIONNELLE §6.2, and ship the first preset (`steady-g1-baseline`).

### Roles (this iteration)

| Role | Agent | Note |
|------|-------|------|
| Teamlead   | A4 | was Coder in iter 4 |
| Coder      | A5 | was Reviewer in iter 4 |
| Reviewer   | A6 | was Tester-unit in iter 4 |
| Tester-unit | A1 | was Tester-func in iter 4 |
| Tester-func | A2 | was Doc-writer in iter 4 |
| Doc-writer | A3 | was Teamlead in iter 4 |

Rotation rule satisfied.

### Plan

1. Add a `manifest` module to `gc-forge-scenario` modelling `gc-forge/run-manifest.v1`, with serde + schemars.
2. Implement `gc-forge run`: orchestrate scenario load + override, regime resolution, runner execution, manifest emission.
3. First preset: `presets/steady-g1-baseline.yaml`.
4. Tests: manifest round-trip + schema, CLI argv parsing, gated end-to-end smoke that runs the preset through Docker and checks the manifest hashes line up with the log file.
5. Fill the "First scenario" section of `doc/user/getting-started.md` with the new run command.

### Decisions log

- **Manifest types live in `gc-forge-scenario::manifest`** rather than the runner crate. Justification: the manifest persists a scenario's execution; the runner already depends on scenario, so putting manifest there avoids a dep cycle. The CLI maps `RunOutcome` (from runner) into `RunMeta` fields when assembling the manifest — the manifest module stays I/O-agnostic.
- **Manifest exit-status field is a string** (`success | failure | oom | timeout | signaled`) not a tagged enum, so downstream tooling that reads YAML/JSON without our typed library still gets a forward-compatible value. The Rust enum `ExitStatusRecord` deserialises via untagged.
- **`gc_forge_version`** captures the CLI binary's `CARGO_PKG_VERSION` at compile time. Git-SHA capture is deferred until iter 17 (release-pipeline) because it requires a build script and is non-trivial in `cargo install` flows.
- **Output path defaults**: when `--out-dir` is given, the log lands at `<out-dir>/<name>-<seed>.log` and the manifest at `<out-dir>/<name>-<seed>.manifest.yaml`. The naming matches SPEC-FONCTIONNELLE §6.2 and avoids overwriting prior runs of the same scenario at different seeds.
- **`--preset NAME`**: deferred to iter 15 (`selftest-variance`) where preset packaging lands. Iter 5 only takes a path. The CLI reference notes this.

### Metrics (at merge)

- Rust unit tests: 92/92 (46 scenario incl. 9 manifest, 15 regimes, 23 runner, 5 placeholders, 5 CLI run module).
- Java unit tests: 18/18 (unchanged).
- Manual end-to-end smoke: `gc-forge run` against `presets/steady-g1-baseline.yaml` (3 s variant) on `gc-forge-runner:dev-jdk21` produces a 19 KB G1 log + a 2.7 KB YAML manifest with `exit_status.kind: success`, populated SHA-256 of the source/log/jar, recorded JVM flags, and `validation.status: skipped`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `mvn verify`: green.
- DoD-gate (phase 1): green.

### Decisions taken in flight

- **Iter-1 image required a rebuild.** The runner integration test from iter 3 happened to work because it was running against the freshly-built dev image at the time. By iter 5 the image still contained the iter-1 monolithic harness, so the harness rejected the new positional args (`Duration.parse(args[1])` blew up). Discovery → fix: rebuild via `make docker-image` whenever the harness signature changes. Made a note in `doc/user/getting-started.md` to run `make build && make docker-image` before the first `gc-forge run`.
- **`HostMeta::os` / `arch` from `std::env::consts`** rather than build-time `CARGO_CFG_TARGET_OS`. The latter is a `build.rs`-only env var and panicked at compile time when used in regular code. The runtime constants are simpler and more honest (they describe the host the binary runs on, not the host it was compiled for — which is what the manifest should record).
- **Filename seed in lowercase hex, no `0x`** prefix. Avoids edge cases on case-insensitive filesystems and keeps shell completion predictable. The seed in the manifest body still uses the canonical `0xC0FFEE` form.
- **Manifest module placement**: ended up in `gc-forge-scenario` (not `gc-forge-runner`) as the API-FREEZE planned. Mapping `RunOutcome` → `ExitStatusRecord` happens in the CLI `run.rs`, keeping the `manifest` module I/O- and runner-agnostic.
- **`validation.status: skipped`** by construction at iter 5. The `ValidationRecord::validator_version` already records the GC-Forge version that emitted the manifest, so iter 13's `gc-forge validate` will be able to detect manifests it should re-validate vs ones already validated.

### Bilan

The MVP "happy path" exists end to end: `gc-forge run presets/steady-g1-baseline.yaml` produces a real G1 GC log on Temurin 21 plus a typed, hash-anchored manifest documenting how it was produced. From iteration 5 forward, every regime added in iters 7–12 plugs into this same orchestrator without touching the runner or the manifest writer.

The single most-load-bearing line of code in this iteration is `regime.workload_args(&scenario)?` in `cli/src/run.rs`: it is the seam between the typed scenario world and the harness CLI world. The Rust regimes own that translation; the Java harness only has to honour the agreed argv layout. The same seam will absorb six more regimes without growing.

The end-to-end smoke also surfaced two minor production-quality items:

1. **Image rebuild discipline.** The Makefile already has `docker-image: $(HARNESS_JAR) …` so changing the harness invalidates the image automatically — but a stale image lurks in CI and on dev machines. Documenting it in `getting-started.md` is the iter-5 mitigation; the proper fix is iter 17 (`release-pipeline`) which will produce content-addressable image tags.
2. **Wall-clock timeout** still not enforced. `RunnerError::Timeout` has been declared since iter 3; the wiring lands in iter 14 (`batch`) where running 100+ scenarios in sequence makes a hung process catastrophic.

Iteration 6 (`algos-zgc-parallel`) can start: ZGC and Parallel land alongside their two `*-baseline` presets, exercising the same `gc-forge run` path with different `-XX:+UseZGC`/`-XX:+UseParallelGC` flags.

---

## Iteration 4 — harness-steady-state

- **Started:** 2026-04-25
- **Status:** merged
- **Branch:** `iter/04-harness-steady-state` (merged into `main`)
- **Goal:** introduce the `Regime` abstraction on both sides of the language boundary. Java side: `Regime` interface + `RegimeRegistry` + `SteadyStateRegime` implementing R1. Rust side: `Regime` trait in `gc-forge-regimes` + typed `SteadyStateParams` + `workload_args` translator. Refs: SPEC-FONCTIONNELLE §4.1, SPEC-TECHNIQUE §4.3, §4.4.

### Roles (this iteration)

| Role | Agent | Note |
|------|-------|------|
| Teamlead   | A3 | was Coder in iter 3 |
| Coder      | A4 | was Reviewer in iter 3 |
| Reviewer   | A5 | was Tester-unit in iter 3 |
| Tester-unit | A6 | was Tester-func in iter 3 |
| Tester-func | A1 | was Doc-writer in iter 3 |
| Doc-writer | A2 | was Teamlead in iter 3 |

Rotation rule satisfied.

### Plan

1. Java: split monolithic `WorkloadHarness` into `Regime` interface, `RegimeRegistry` and `SteadyStateRegime`. Keep a no-args fallback so `make demo` does not regress.
2. Java: implement R1 with the four documented parameters and a `Random`-driven, seed-deterministic chunk picker.
3. Rust: `Regime` trait in `gc-forge-regimes` (`id`, `workload_args`, `expected_phenomena`, `expected_invariant_rules`). `SteadyStateParams` is the typed view; `from_yaml(parameters)` accepts defaults and rejects unknown keys.
4. Test both sides in isolation. The full pipeline (scenario → regime → runner → log validation) lands in iter 5.

### Decisions log

- **Java parameter encoding**: positional `[kind] [duration] [seed]` followed by `key=value` pairs. Rationale: avoids pulling Jackson or javax.json into the harness, keeps the wire format inspectable in plain `docker run` invocations, lets future regimes register their own parser without contaminating `RegimeRegistry`.
- **Rust regime trait stays small** (no `validate` yet). Validation against parsed logs lands in iter 13 alongside the parser; `expected_invariant_rules` returns the rules-as-strings now so the manifest in iter 5 can carry them.
- **Defaults policy**: missing parameter ⇒ regime's documented default (per SPEC-FONCTIONNELLE §4.1 for R1). Unknown parameter ⇒ typed error. Wrong type ⇒ typed error. Validation happens in Rust; Java trusts Rust to have already rejected garbage.
- **Backward-compatibility for `make demo`**: when invoked without arguments, the harness defaults to `steady-state-healthy PT10S 0xC0FFEE`. Doc-writer notes this in `cli-reference.md`.

### Metrics (at merge)

- Rust unit tests: 78/78 passed (15 new in `gc-forge-regimes` + 63 from earlier crates).
- Java unit tests: 18/18 passed (9 new `SteadyStateRegimeTest` + 9 expanded `WorkloadHarnessTest`).
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `mvn verify`: green (Maven build also produces JaCoCo report).
- DoD-gate (phase 1): green.

### Decisions taken in flight

- **Default impl Default for `SteadyStateParams`** rather than `#[derive(Default)]` because the documented defaults (50 MiB/s, 100 MiB live set) are not the type's natural zero — they are spec-mandated values that benefit from being centralised in one place.
- **`iso_duration` collapses days into hours**: `PT24H` rather than `P1D`. JVM's `Duration.parse` accepts both, and this keeps the formatter monotone in seconds (no `T`-vs-no-`T` branch). Originally the function had a separate day branch but it was dead-equivalent to the hour branch; clippy caught it via `if_same_then_else`.
- **Java parameter encoding stayed positional + key=value** (no Jackson). The harness's only third-party deps remain JUnit (test-scope) and the JaCoCo agent. The deterministic-seed test (`chunk_sizer_is_seed_deterministic`) confirms the design holds.
- **`Box<dyn Regime>` is not `Debug`**: the test for `resolve(unknown)` therefore can't `unwrap_err()`. Switched to a manual `match` that's just as terse and avoids forcing `Debug` on the trait. If we ever need `Debug`, we can add it as a super-trait later.
- **Backward-compat fallback in `WorkloadHarness.main`**: zero-args runs `steady-state-healthy PT10S 0xC0FFEE`. This keeps `make demo` working without changing its target. The fallback is exercised by a unit test (`no_args_falls_back_to_default_steady_state`).

### Bilan

The Regime abstraction cleanly bridges the Rust and Java sides. On the Rust side, a `Scenario` becomes a `Vec<String>` of harness CLI args via `Regime::workload_args` — a pure transformation that's already covered by 11 dedicated unit tests for R1. On the Java side, the harness reads those args through a typed registry and dispatches to the regime implementation; another 18 tests verify each layer in isolation.

The R1 implementation is also the template for the next six regimes (R2–R7 in iters 7–12). Each will follow the same shape: a typed `Params` struct in Rust with `from_yaml`, a `Regime` impl that builds workload args, and a Java class registered in `RegimeRegistry`. The `make demo` and Docker runner integration test still pass because the no-args fallback preserves iter 1's behaviour.

Two soft items deferred:
1. **Allocation rate self-clamping**: the rate-limiting loop in `SteadyStateRegime` is naïve (busy-allocate then sleep to fill the second). Real workloads will need finer control. Deferred to iter 7 or beyond once we have variance data.
2. **Survivor pool memory accounting**: the survivor cap is `liveSetCap / 10` rather than configurable. The `lifetime_distribution: mixed` semantics in the spec leave room for tweaking — keeping it simple now, will revisit once R5 (`cache-churn`) lands and surfaces overlapping requirements.

Iteration 5 (`run-end-to-end`) can start: the runner has its scenario→argv translator (this iteration), its JVM launcher (iter 3), its scenario parser (iter 2), and its bootstrap harness (iter 1). Wiring `gc-forge run` into a single command is now a composition exercise, not new design work.

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
