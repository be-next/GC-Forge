# API freeze — iteration 3 (docker-runner-mvp)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

The `gc-forge-runner` crate gains its first real API: a `Runner` trait, the
`DockerRunner` backend, and the function that translates a resolved
`Scenario` into a JVM command line. No CLI subcommand wiring this iteration —
that lands with `gc-forge run` in iter 5.

## Public Rust surface added

### Crate `gc-forge-runner`

| Item | Kind | Notes |
|------|------|-------|
| `Runner`                        | trait  | `name`, `check_available`, `execute` |
| `RunSpec`                       | struct | resolved scenario + log path + harness jar + workload args + (optional) cpu/memory limits |
| `RunOutcome`                    | struct | log path, exit status, started_at, ended_at, jvm version, image used |
| `ExitStatus`                    | enum   | `Success`, `Failure(i32)`, `Oom`, `Timeout`, `Signal(i32)` |
| `RunnerError`                   | enum (thiserror) | `NotAvailable`, `Spawn`, `LogCapture`, `Wait`, `Timeout`, `Other` |
| `DockerRunner`                  | struct | the iter-3 backend |
| `DockerRunner::new`             | fn     | builder with sensible defaults |
| `DockerRunner::with_image`      | fn     | override the image tag |
| `DockerRunner::with_cpus(f64)`  | fn     | optional `--cpus` |
| `DockerRunner::with_memory(ByteSize)` | fn | optional `--memory` |
| `flags::build_jvm_command(&Scenario, &Path)` | fn | pure function → `Vec<String>` |
| `flags::log_decorators()`       | fn (const) | the standardised `-Xlog` decorator string |

### Crate `gc-forge-cli`

No change this iteration.

## Java harness public surface

No change.

## YAML schema changes

None.

## Invariants for Tester-unit

- `flags::build_jvm_command` produces `-XX:+UseG1GC` for `algorithm: G1`,
  `-XX:+UseZGC -XX:+ZGenerational` for `ZGC` with `generational != Some(false)`,
  `-XX:+UseParallelGC` for `Parallel`.
- The `-Xlog` flag is always emitted with the canonical decorator set.
- `-Xms` and `-Xmx` are always emitted, in that order, in JVM-style suffix
  form (matching `ByteSize::Display`).
- G1-specific knobs (`MaxGCPauseMillis`, `G1HeapRegionSize`, `IHOP`) are emitted
  only when the algorithm is G1 *and* the field is `Some`.
- `extra_flags` (jvm and gc) are appended after the standard flags, in
  declaration order, without de-duplication.
- `DockerRunner::check_available` returns `Ok(())` when `docker version` exits 0.
- The runner returns `RunnerError::NotAvailable` (not `Spawn`) when Docker
  itself is missing.
- Argv construction includes `--rm`, `--network=none`, and the mounts for
  `<out_dir>:/work` and `<harness_jar>:/work/harness.jar:ro`.

## Doc sections to author (Doc-writer)

- `doc/user/cli-reference.md` — initial skeleton with the future `gc-forge run`
  options that surface today's runner flags (`--image`, `--docker-cpus`,
  `--docker-memory`). Marked `_TODO iter 5_` for the run subcommand body.
- `CHANGELOG.md` — `Unreleased`: Runner trait, DockerRunner, flag builder.

## Approval

- Coder: A3 — frozen 2026-04-25
- Reviewer: A4 — `Approved: A4 2026-04-25` (read against SPEC-TECHNIQUE §4.5 and §6.1; runtime-gated integration test approved over `#[ignore]`).
