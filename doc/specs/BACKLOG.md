# GC-Forge — initial backlog

> **Reference**: brief of 25 April 2026, §8.4.
> Backlog structured as **epics** and top-level **user stories**,
> prioritised MVP / V1 / V2.
> **Status (2026-04-27)**: every MVP (P0) story below is
> **delivered** unless explicitly noted. The status column on the
> right reports the disposition.

## Conventions

- **Priorities**: `P0` (MVP, blocking), `P1` (V1, important), `P2`
  (V2 or opportunistic).
- **Estimation**: XS (≤ 0.5 d), S (1 d), M (2–3 d), L (4–7 d),
  XL (> 1 wk).
- **Story format**: *As a `<persona>`, I want `<action>`, in
  order to `<benefit>`.*
- **Acceptance criteria**: conditions that can be verified in
  QA/CI.
- **Status**: ✅ delivered, 🟡 in progress, ⏳ pending, 🚫 dropped.

---

## EPIC 1 — Infrastructure and core trunk

**Objective**: lay down the technical skeleton on which everything
else stacks.

### US-1.1 — Cargo workspace and initial CI

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a GC-Forge developer, I want a multi-crate Cargo
  workspace and a green CI on a minimal commit, in order to
  iterate without environment debt.
- **Acceptance**:
  - `cargo build --workspace` passes on Linux and macOS.
  - `cargo fmt --check`, `cargo clippy -D warnings`, `cargo deny
    check` pass in CI.
  - Workspace contains the six crates of SPEC-TECHNICAL §2.1
    (empty skeletons accepted).

### US-1.2 — Java harness Maven project

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a GC-Forge developer, I want a Maven project
  producing a `workload-harness.jar` fat-jar, in order to embed
  it in JVM executions.
- **Acceptance**:
  - `mvn package` produces `target/workload-harness-<version>.jar`.
  - Reproducible build: stable SHA-256 across identical runs.
  - Java CI green on Maven 3.9 + JDK 17.

### US-1.3 — Docker runner image

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a GC-Forge developer, I want a Docker image
  `gc-forge-runner` that contains Temurin and the harness, in
  order to have a frozen execution environment.
- **Acceptance**:
  - `jdk17` and `jdk21` variants under `docker/jdk{17,21}/`.
  - Multi-architecture `linux/amd64` + `linux/arm64`.
  - Automatic publication to GHCR on tag.

---

## EPIC 2 — Scenario model

**Objective**: expose the declarative YAML model promised by
brief §4.

### US-2.1 — Parser and validation of `gc-forge/scenario.v1`

- **Priority**: P0 — **Estimation**: M — **Status**: ✅
- **Story**: As a user, I want to write a YAML scenario
  conforming to the `scenario.v1` schema and receive explicit
  errors when something is wrong.
- **Acceptance**:
  - All fields typed in Rust with `serde`.
  - JSON Schema generated via `schemars` and published at
    `schemas/scenario-v1.json`.
  - Errors carry line/column information and a suggestion (e.g.
    typo on `kind`).

### US-2.2 — `extends` mechanism and resolution

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a user, I want a scenario to inherit from another
  via `extends:`, in order to reuse bases without duplicating.
- **Acceptance**:
  - Recursive map merge, scalar override.
  - Cycle detection on `extends`.
  - Path resolved relative to the extender file.

### US-2.3 — CLI override `--override`

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a user, I want to pass
  `--override 'spec.gc.options.heap.max=4g'` to tweak a scenario
  without copying it.
- **Acceptance**:
  - Simple JSONPath-style syntax (dotted notation).
  - Override applied after `extends`.
  - Type-checked against the schema.

### US-2.4 — `gc-forge lint` subcommand

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a user, I want to validate a scenario without
  executing it.
- **Acceptance**:
  - Exit 0 if valid, exit 1 otherwise.
  - Checks compatibility (jvm/algo, algo/regime).
  - JSON output reserved for V1 (`--output json`).

---

## EPIC 3 — Application regimes

**Objective**: produce the expected GC behaviour for each of the
seven regimes.

### US-3.1 — `Regime` trait and registry

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: As a GC-Forge developer, I want a Rust `Regime`
  trait and a registry, so each regime follows the same contract.
- **Acceptance**:
  - Trait defined with methods: `id`, `parameters_schema`,
    `workload_args`, `invariants`, `expected_phenomena`,
    `validate`.
  - Typed registry `RegimeRegistry::lookup(&str) ->
    Option<Box<dyn Regime>>`.

### US-3.2 — `steady-state-healthy` regime

- **Priority**: P0 — **Estimation**: M — **Status**: ✅
- **Story**: Implement R1 on the Rust side and the Java harness
  side.
- **Acceptance**:
  - Invariants defined (cf. SPEC-FUNCTIONAL §4.1):
    `young_ratio`, `full_count`, `p99_pause_ms`.
  - Integration test: preset `steady-g1-baseline` produces a
    valid log.
  - Variance CV ≤ 5 % over 5 runs.

### US-3.3 to US-3.8 — Other regimes

- **Priority**: P0 (R2, R3, R4, R5, R6, R7) — **Estimation**:
  M each (R6 and R4 = L) — **Status**: ✅
- **Story**: Same as US-3.2 for R2, R3, R4, R5, R6, R7.
- **Acceptance**: invariants defined, presets passing, variance
  acceptable.

### US-3.9 — Reusable Java allocation building blocks

- **Priority**: P0 — **Estimation**: M — **Status**: ✅
- **Story**: As a harness developer, I want utility classes
  (`AllocationEngine`, `BurstScheduler`, `LeakReservoir`,
  `LruCacheChurn`) reusable across regimes.
- **Acceptance**:
  - Java unit tests on each block.
  - Reproducibility for a given seed.

---

## EPIC 4 — Execution and orchestration

**Objective**: launch a JVM, capture a log, manage robustness.

### US-4.1 — `DockerRunner` MVP

- **Priority**: P0 — **Estimation**: L — **Status**: ✅
- **Story**: Launch a JVM in a Docker container with the right
  flags and capture the log.
- **Acceptance**:
  - Temurin 17 and 21 images supported.
  - Output volume mounted (absolute host path).
  - `--network=none` by default.
  - Return codes translated into `RunOutcome`.
  - Configurable timeout, clean kill.

### US-4.2 — Unified GC log capture

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Build the `-Xlog:...` string per algorithm and
  capture the log file.
- **Acceptance**:
  - Per-algorithm flags conform to SPEC-TECHNICAL §6.1.
  - Log written without transformation, SHA-256 computed.

### US-4.3 — JVM error handling

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Distinguish OOM, timeout, launch failure, runtime
  error.
- **Acceptance**:
  - `RunOutcome::status` ∈ {Success, Oom, Timeout, JvmError,
    Internal}.
  - JVM stderr captured for diagnostics.

### US-4.4 — `NativeRunner` (V1)

- **Priority**: P1 — **Estimation**: L — **Status**: ⏳
- **Story**: Adoptium download, local cache, native execution
  without Docker.
- **Acceptance**:
  - Cache `~/.gc-forge/jvms/`.
  - SHA-256 verification against Adoptium checksums.
  - Lock file for concurrent executions.

### US-4.5 — Auto Docker/native selection

- **Priority**: P1 — **Estimation**: S — **Status**: ⏳
- **Story**: Automatically choose the available runner.

### US-4.6 — BYO-JVM

- **Priority**: P1 — **Estimation**: S — **Status**: ⏳
- **Story**: Allow the user to provide their own `JAVA_HOME`.

---

## EPIC 5 — Manifest and reproducibility

**Objective**: produce the identity card of every run and
guarantee semantic reproducibility.

### US-5.1 — `gc-forge/run-manifest.v1` schema

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Define the manifest structure (cf. SPEC-FUNCTIONAL §6.2).
- **Acceptance**:
  - Rust type + generated JSON Schema.
  - YAML serialisation by default, JSON via flag.

### US-5.2 — Post-run manifest emission

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Build a manifest from the resolved scenario and the
  `RunOutcome`.
- **Acceptance**:
  - Log hash + jar hash + GC-Forge version present.
  - `expected_phenomena` populated from the regime.

### US-5.3 — Reproducible harness hash

- **Priority**: P0 — **Estimation**: XS — **Status**: ✅
- **Story**: The SHA-256 of `workload-harness.jar` must be
  stable.
- **Acceptance**:
  - Maven build with frozen timestamps.
  - Deterministic file ordering in the jar.

---

## EPIC 6 — Post-run validation

**Objective**: verify that a generated log matches the announced
invariants.

### US-6.1 — Shared GC log parser

- **Priority**: P0 — **Estimation**: L — **Status**: ✅
- **Story**: Have a unified GC log parser in `gc-core` (or a
  prototype in `gc-forge-validate` to be promoted to `gc-core`).
- **Acceptance**:
  - Covers G1, ZGC, Parallel on JDK 17 and 21.
  - Sufficient event extraction for every invariant defined in
    the regimes.
  - Not an exhaustive parser — focused on what is needed.

### US-6.2 — Invariant engine

- **Priority**: P0 — **Estimation**: M — **Status**: ✅
- **Story**: Evaluate a set of `Invariant`s against a `ParsedLog`.
- **Acceptance**:
  - Type `Invariant { rule: String, threshold: Threshold, ... }`.
  - `ValidationReport` recorded in the manifest.
  - Return codes distinguishing success / violation / parse
    error.

### US-6.3 — `gc-forge validate` subcommand

- **Priority**: P0 — **Estimation**: XS — **Status**: ✅
- **Story**: Re-check a log + manifest after the fact.
- **Acceptance**: exit 0 if every invariant passes, exit 3
  otherwise.

---

## EPIC 7 — Presets

### US-7.1 — MVP presets (originally 14)

- **Priority**: P0 — **Estimation**: M (one per regime + variants)
  — **Status**: ✅
- **Story**: Ship the MVP presets defined in SPEC-FUNCTIONAL §8.
- **Acceptance**:
  - Embedded via `include_str!` in the binary.
  - All pass `selftest`.
  - **Outcome**: 21 presets shipped (the additional ones cover
    Shenandoah, Serial, Epsilon, and the non-generational ZGC
    variant).

### US-7.2 — `gc-forge presets list/show/export`

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Browse the catalogue from the CLI.
- **Acceptance**:
  - `list` filterable by regime/algorithm.
  - `show` displays the educational identity card.
  - `export` writes the raw YAML to disk for tweaking.

---

## EPIC 8 — Batch mode

### US-8.1 — `matrix.v1` schema

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Define the `gc-forge/matrix.v1` schema (cf.
  SPEC-FUNCTIONAL §9.2). JSON Schema published at
  `schemas/matrix-v1.json`.

### US-8.2 — Cartesian product generation and filters

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Compute the cells to execute from axes and
  exclusion filters.

### US-8.3 — Bounded parallel execution

- **Priority**: P0 — **Estimation**: M — **Status**: 🟡 partial
- **Story**: Launch N runs in parallel, aggregate results,
  handle partial failures.
- **Acceptance**:
  - `--parallel` configurable.
  - A failing cell does not block the others.
  - Consolidated final report.
- **Outcome**: sequential execution shipped in 0.1.0;
  `--parallel` deferred to V1 once Docker concurrency on macOS
  is characterised.

### US-8.4 — CSV index

- **Priority**: P0 — **Estimation**: XS — **Status**: ✅
- **Story**: Emit `<out-dir>/index.csv` listing runs and labels.

### US-8.5 — Parquet index (V1)

- **Priority**: P1 — **Estimation**: S — **Status**: ⏳
- **Story**: Parquet variant for ML datasets.

---

## EPIC 9 — Quality and CI

### US-9.1 — `gc-forge selftest`

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: Run every preset, check invariants, summary report.
- **Acceptance**: exit code and JSON report.

### US-9.2 — Nightly CI job

- **Priority**: P0 — **Estimation**: S — **Status**: ⏳
- **Story**: `selftest` runs each night in CI.
- **Outcome**: not yet wired in `.github/workflows/`. Tracked as
  part of the post-release hardening pass.

### US-9.3 — `gc-forge variance-check`

- **Priority**: P0 — **Estimation**: M — **Status**: ✅
- **Story**: Measure inter-run CV over N runs.
- **Acceptance**:
  - Aggregated metrics computed: count, sum, p50, p99.
  - CV computed and reported.
  - HTML report (V1) — `--report.html` option.
- **Outcome**: shipped as documented in CLI reference; HTML
  report deferred to V1.3.

### US-9.4 — `gc-core-roundtrip` cross-project test

- **Priority**: P0 — **Estimation**: M — **Status**: 🟡 partial
- **Story**: A log produced by GC-Forge is parsed by GC-Insight
  and the event sequence is reconstructed.
- **Outcome**: Forge-side counterpart in place; full
  cross-project enforcement is conditioned on co-development with
  GC-Insight.

---

## EPIC 10 — Documentation and release

### US-10.1 — Getting started

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: `doc/user/getting-started.md` walks a new user from
  zero to a produced log in 10 min.

### US-10.2 — Regime reference

- **Priority**: P0 — **Estimation**: M — **Status**: ✅
- **Story**: `doc/user/regimes.md` detailing each regime, its
  parameters, its signature.

### US-10.3 — Scenario schema reference

- **Priority**: P0 — **Estimation**: S — **Status**: ✅

### US-10.4 — CLI reference

- **Priority**: P0 — **Estimation**: S — **Status**: ✅

### US-10.5 — Product pitch

- **Priority**: P0 — **Estimation**: S — **Status**: ✅
- **Story**: `doc/concepts/overview.md` (formerly `doc/pitch.md`)
  — one-page summary of why GC-Forge.

### US-10.6 — `0.1.0` release

- **Priority**: P0 — **Estimation**: M — **Status**: 🟡 release
  pipeline in place; tag and publish are operator-gated.
- **Story**: Build matrix, GitHub Release, crates.io, GHCR.

### US-10.7 — Homebrew tap

- **Priority**: P1 — **Estimation**: S — **Status**: ⏳

---

## EPIC 11 — Reproduction and ergonomics (V1)

### US-11.1 — `gc-forge mirror <prod.log>` heuristic

- **Priority**: P1 — **Estimation**: L — **Status**: ⏳
- **Story**: Brief use case F-REPRO — propose a mirror scenario
  from a production log.

### US-11.2 — Snippets

- **Priority**: P1 — **Estimation**: S — **Status**: ⏳

### US-11.3 — JFR export

- **Priority**: P1 — **Estimation**: M — **Status**: ⏳

---

## EPIC 12 — JVM and collector coverage broadening (V1)

### US-12.1 — Corretto and GraalVM CE

- **Priority**: P1 — **Estimation**: S (Corretto), M (GraalVM)
  — **Status**: ⏳

### US-12.2 — OpenJ9

- **Priority**: P1 — **Estimation**: XL — **Status**: ⏳
- **Story**: OpenJ9 has a distinct GC log format — parser
  extension and dedicated tests.

### US-12.3 — Shenandoah

- **Priority**: P1 → **promoted to MVP** — **Status**: ✅
- **Outcome**: shipped in 0.1.0 (presets
  `steady-shenandoah-baseline`, `leak-shenandoah-slow`).

### US-12.4 — Serial GC

- **Priority**: P1 → **promoted to MVP** — **Status**: ✅
- **Outcome**: shipped in 0.1.0 (presets
  `steady-serial-baseline`, `cache-serial-churn`).

### US-12.5 — `compute-batch` regime

- **Priority**: P1 — **Estimation**: M — **Status**: ⏳

---

## EPIC 13 — V2 strategic

### US-13.1 — Synthetic-hybrid generation for ML

- **Priority**: P2 — **Estimation**: XL — **Status**: ⏳

### US-13.2 — `gc-forge sweep` for parametric datasets

- **Priority**: P2 — **Estimation**: L — **Status**: ⏳

### US-13.3 — HTTP service mode

- **Priority**: P2 — **Estimation**: XL — **Status**: ⏳

### US-13.4 — Zing / Prime / Oracle JDK

- **Priority**: P2 — **Estimation**: XL (legal included) —
  **Status**: ⏳

### US-13.5 — Legacy CMS on JDK 8

- **Priority**: P2 — **Estimation**: L — **Status**: ⏳

---

## Prioritisation summary

| Tier | Epics | US count | Cumulative estimation (person-day equivalents) | Delivery |
|------|-------|----------|------------------------------------------------|----------|
| **MVP (P0)** | 1–10 | ~35 stories | ~50–60 d (consistent with 8–10 wk × 5 d × ~12 h ≈ ~50 d-equivalent) | All delivered or in late-stage state. |
| **V1 (P1)** | 11–12 + remainder | ~10 stories | ~25 d | Pending. Shenandoah and Serial were promoted to MVP and removed from V1. |
| **V2 (P2)** | 13 | 5 stories | ~40 d | Pending. |

## First sprint (week 1) — historical

The originally suggested first sprint stories (now delivered):
1. US-1.1 — Cargo workspace.
2. US-1.2 — Maven harness project.
3. US-1.3 — Docker runner image (could start in parallel).
4. US-2.1 — Scenario parser (typed skeleton).
5. US-3.1 — `Regime` trait.
