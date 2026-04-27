# GC-Forge — technical specification

> **Reference**: brief of 25 April 2026, §5, §6.
> **Status**: V1.0 — issued by Cowork. Translated to English and
> resynchronised with the 0.1.0 implementation on 2026-04-27.
> **Target reader**: a Rust developer who must be able to start
> the MVP implementation without major ambiguity.

## 1. Overview

GC-Forge consists of:

```
┌─────────────────────────────────────────────────────────────┐
│  gc-forge (Rust CLI)                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ scenario │  │ regimes  │  │ runner   │  │ validate │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│              ↓ readlock ↓                                   │
│           gc-core (crate shared with GC-Insight)            │
└─────────────────────────────────────────────────────────────┘
              │                                   │
              │ launches                          │ parses logs
              ▼                                   ▼
┌──────────────────────────┐          ┌──────────────────────┐
│  JVM (HotSpot/Temurin)   │   ───►   │  native GC log       │
│  + workload-harness.jar  │          │  manifest.yaml       │
│  (Docker or native)      │          └──────────────────────┘
└──────────────────────────┘
```

Three distributable components:
1. A **Rust CLI binary** (`gc-forge`).
2. A **Java harness** (`workload-harness.jar`) — fat-jar
   parameterised by CLI/JSON.
3. Optionally, light **Docker images** (Temurin + embedded
   harness).

## 2. Rust workspace and consistency with GC-Insight

### 2.1 Cargo workspace

GC-Forge is **not** a submodule of GC-Insight; the two projects
coexist in two separate repositories but consume a published
`gc-core` crate.

```
gc-insight/                          # separate repo, already exists
├── crates/
│   ├── gc-core/                     # ◄── source of truth
│   ├── gc-insight-cli/
│   └── gc-insight-lambda/

gc-forge/                            # this repo
├── Cargo.toml                       # workspace
├── crates/
│   ├── gc-forge-cli/                # CLI binary
│   ├── gc-forge-scenario/           # scenario YAML parsing/validation
│   ├── gc-forge-regimes/            # implementation of the seven regimes
│   ├── gc-forge-runner/             # JVM orchestration (Docker/native)
│   ├── gc-forge-validate/           # post-run validation
│   └── gc-forge-presets/            # embedded presets (build.rs)
├── workload-harness/                # Java/Maven project
│   ├── pom.xml
│   └── src/main/java/...
├── docker/
│   ├── jdk21/Dockerfile             # runner image on Temurin 21
│   └── jdk17/Dockerfile             # runner image on Temurin 17
├── schemas/                         # generated JSON Schemas (scenario, run-manifest, matrix)
├── presets/                         # YAML scenarios (21 presets)
└── doc/specs/                       # this directory
```

### 2.2 The shared `gc-core` crate

`gc-core` is versioned in SemVer, published to crates.io (or to a
private GitLab registry during the pre-1.0 phase).

**What `gc-core` contains (Forge ↔ Insight consistency)**:

```rust
// gc-core/src/model/mod.rs (indicative shape, iterated during implementation)
pub mod jvm {
    pub enum Vendor { Temurin, Corretto, GraalVM, OpenJ9, Other(String) }
    pub struct JvmInfo { pub vendor: Vendor, pub version: Version, pub flags: Vec<String> }
}

pub mod gc {
    pub enum Algorithm { G1, ZGC { generational: bool }, Parallel, Shenandoah, Serial, Epsilon, CMS }
    pub struct HeapConfig { pub min: ByteSize, pub max: ByteSize, pub new_size: Option<ByteSize> }
    pub enum EventKind { YoungGc, MixedGc, FullGc, ConcurrentMark, EvacuationFailure, HumongousAllocation, ... }
    pub struct GcEvent { pub kind: EventKind, pub timestamp: Duration, pub pause_ms: f64, pub heap_before: u64, pub heap_after: u64, ... }
}

pub mod regime {
    pub enum RegimeKind { SteadyStateHealthy, AllocationBurst, HumongousPressure, SlowLeak, CacheChurn, MixedGcPathological, MicroserviceStopAndGo }
    pub struct PhenomenonId(pub &'static str);
}

pub mod manifest {
    pub struct RunManifest { ... }     // schema in SPEC-FUNCTIONAL §6
}
```

**What `gc-core` does not contain**:
- GC-Insight's analytical heuristics (stay in
  `gc-insight-core`).
- The regime implementations (stay in `gc-forge-regimes`).
- The unified log parser (can live in `gc-core` if deemed
  shareable, otherwise specific to Insight).

### 2.3 `gc-core` evolution strategy

- Any `gc-core` change that breaks Forge or Insight = major bump.
- Pre-1.0 period: `0.x.y` with minor bumps allowed to break,
  **provided** Forge and Insight are updated in the same
  coordinated release.
- From 1.0 onwards: strict SemVer.

**Consistency safeguard**: an integration test
`gc-core-roundtrip` parses a log produced by Forge with the
Insight parser and reconstructs the event sequence.

## 3. Real vs synthetic — decision

**Decision: real approach at MVP, hybrid in V2.**

### 3.1 Argument

For: real approach (real JVM execution + Java harness).
- **Absolute format fidelity**: a log produced by HotSpot with
  `-Xlog:gc*` is, by construction, identical to what the same
  JVM produces in production. No risk of hidden divergence.
- **Robustness to JVM evolution**: if Oracle changes the GC log
  format in JDK 25, nothing to patch on our side: launch the new
  JVM, the log changes as in production.
- **Product credibility**: in a domain where GC-Insight sells on
  precision, providing synthetic fixtures would be an original
  sin.
- **Engineering simplicity**: a Java harness of a few thousand
  lines is simpler to maintain than a Rust generator that has to
  model 6 GC algorithms × N JVM versions.

For: synthetic approach (generate the log directly in Rust).
- Faster (no JVM cold start).
- Trivial bit-for-bit reproducibility.
- No JVM dependency.

Against: synthetic approach.
- **Maintenance debt**: would require a model per
  vendor-version-algorithm, kept in sync with every JDK
  release.
- **Drift risk**: a synthetic format will inevitably diverge
  from the real format on cases nobody anticipated.
- **Statement of weakness**: "our fixtures are wrong, but they
  are good enough". Unacceptable for a product whose value is
  precision analysis.

### 3.2 Place of synthesis — V2

The synthetic approach retains a role in V2 for cases where real
execution is costly or impossible:
- **Massive volumes** for ML: generate 10,000 short logs for a
  dataset.
- **Extreme cases** hard to provoke for real (e.g. very large
  heap, very long run) — synthesis from a "dilated" real trace.
- **Deterministic GC-Insight bench**: a synthetic subset with
  bit-stable hashes for CI.

The `gc-forge-synth` module (V2) will produce logs **calibrated
on** real logs of the same regime, not from scratch. Architecture:
an autoencoder or a per-regime Markov model trained on the real
corpus.

### 3.3 Consequence on the MVP architecture

No synthesis module in MVP. The trajectory toward synthesis
imposes **no constraint** on the MVP architecture — the Rust API
that produces a log and its manifest is strictly the same whether
the log comes from a JVM or a generator.

## 4. Component architecture

### 4.1 `gc-forge-cli` — entry point

CLI built on `clap` (consistency with GC-Insight). Subcommands:
`run`, `batch`, `validate`, `lint`, `presets`, `variance-check`,
`selftest`.

Pseudo-code of the `run` flow:

```rust
fn cmd_run(args: RunArgs) -> Result<()> {
    let scenario = scenario::load(&args.path)?
        .merged_with(&args.overrides)?
        .resolved();
    scenario::lint(&scenario)?;

    let regime = regimes::resolve(&scenario.spec.regime)?;
    let runner = runner::pick(&args)?;            // Docker | Native
    let workload_args = regime.workload_args(&scenario);

    let outcome = runner.execute(RunSpec {
        jvm: scenario.spec.jvm.clone(),
        flags: scenario.spec.gc.flags(),
        log_target: args.out_dir.join("gc.log"),
        workload_args,
        duration: scenario.spec.duration,
    })?;

    let manifest = manifest::build(&scenario, &outcome)?;
    let validation = validate::run(&manifest, &outcome.log_path)?;
    manifest::write(&args.out_dir, manifest.with(validation))?;
    Ok(())
}
```

### 4.2 `gc-forge-scenario` — parsing and resolution

- Parsing: `serde_yaml` → typed `Scenario`.
- Validation: JSON Schema generated via `schemars` (published in
  `schemas/`), runtime validation via Rust typing + invariants.
- `extends:` resolved recursively with cycle detection.
- `--override` parsed as simple JSONPath, applied after the
  `extends` merge.

### 4.3 `gc-forge-regimes` — regime library

Each regime is a Rust module that implements:

```rust
pub trait Regime {
    fn id() -> &'static str;
    fn parameters_schema() -> JsonSchema;
    fn workload_args(&self, scenario: &Scenario) -> Vec<String>;     // → CLI args for the Java harness
    fn invariants(&self, scenario: &Scenario) -> Vec<Invariant>;
    fn expected_phenomena(&self, scenario: &Scenario) -> Vec<PhenomenonId>;
    fn validate(&self, parsed: &ParsedLog, scenario: &Scenario) -> ValidationReport;
}
```

The regime does **not generate** the log: it prepares the
arguments to pass to the Java harness and provides the post-run
validation rules.

### 4.4 `workload-harness` — parameterised Java harness

A single fat-jar that covers the seven regimes. Architecture:

```java
public class WorkloadHarness {
    public static void main(String[] args) {
        Args parsed = Args.parse(args);
        Regime regime = RegimeRegistry.lookup(parsed.regime());
        regime.run(parsed.params(), parsed.duration(), parsed.seed());
    }
}

public interface Regime {
    void run(Map<String,Object> params, Duration duration, long seed);
}

public class SteadyStateRegime implements Regime { ... }
public class AllocationBurstRegime implements Regime { ... }
public class HumongousPressureRegime implements Regime { ... }
// etc.
```

**Harness technical choices**:
- Java 17 (compile target) — guarantees compatibility with
  Temurin 17 and 21.
- No heavy framework: no Spring, no Quarkus. JDK + an optional
  JMH dependency if deterministic micro-benchmarks are needed.
- Allocation controlled via primitives `byte[]`, `Object[]`,
  `String` depending on the regime.
- Build: Maven or Gradle; **Maven** chosen for simplicity (`mvn
  package -DskipTests` produces the fat-jar).
- The SHA-256 of the JAR is recorded in the manifest
  (reproducibility).

**Why a single parameterised harness rather than N projects**:
- Maintenance: one build/release cycle.
- Reuse: shared allocation primitives (`AllocationEngine`,
  `LeakReservoir`, `BurstScheduler`).
- Testability: Java unit tests on the building blocks.
- Drawback: slightly more complexity inside the harness itself
  — accepted.

### 4.5 `gc-forge-runner` — JVM orchestration

Two backends implementing the same trait:

```rust
pub trait Runner {
    fn name(&self) -> &'static str;
    fn check_available(&self) -> Result<()>;
    fn ensure_jvm(&self, jvm: &JvmSpec) -> Result<JvmHandle>;
    fn execute(&self, spec: RunSpec) -> Result<RunOutcome>;
}

pub struct DockerRunner { ... }      // MVP default
pub struct NativeRunner { ... }      // V1
```

#### 4.5.1 `DockerRunner` (MVP)

- Base image: `eclipse-temurin:21-jdk-jammy` or `:17-jdk-jammy`.
- Variant with embedded harness: `gc-forge-runner:dev-jdk{17,21}`
  built from `docker/jdk{17,21}/Dockerfile` (published by CI on
  release).
- Launch: `docker run --rm --network=none --entrypoint=java
  -v <out-absolute>:/work -v <jar>:/work/harness.jar:ro
  <image> <flags> -jar /work/harness.jar <args>`.
- CPU/memory: by default, inherit the host's limits. Options
  `--docker-cpus`, `--docker-memory` exposed for reproducibility.
- Network disabled by default (`--network=none`) — the harness
  has no need for network.

#### 4.5.2 `NativeRunner` (V1)

- On-demand download of Adoptium distributions to
  `~/.gc-forge/jvms/temurin-{17,21}/`.
- SHA-256 verification against official Adoptium checksums.
- Local cache + lock file for concurrent executions.
- Direct launch of the extracted `java`, with the same flags as
  in Docker.

#### 4.5.3 Why Docker is the MVP priority

- **Reproducibility**: an image tag + a harness JAR = a frozen
  environment. Native requires managing a cache, downloads,
  distributions.
- **Multi-OS**: Docker works identically on Linux and macOS
  (with Rosetta on Apple Silicon for ARM64 — Temurin has
  multi-arch images anyway).
- **No host pollution**: the user does not have to install five
  JDKs.
- **Cost**: Docker Desktop on macOS = heavy. That argues for
  shipping a native runner in V1 to give users a choice.

### 4.6 `gc-forge-validate` — post-run validation

- Parses the log with a parser **shared** with GC-Insight
  (ideally in `gc-core`).
- Applies the `Invariant`s provided by the regime + the custom
  `expected_invariants` of the scenario.
- Produces a `ValidationReport` recorded in the manifest.

### 4.7 `gc-forge-presets` — embedded presets

- The preset YAML files are embedded in the binary via
  `include_str!` (generated by `build.rs`).
- `gc-forge presets export <name>` writes them to disk.
- Tests: every preset must `lint` cleanly and pass its
  `selftest`.

## 5. JVM provisioning — decision

**Decision: Docker at MVP, native via Adoptium download in V1,
BYO-JVM open via advanced flag in V1+.**

### 5.1 Strategy comparison

| Strategy | Reproducibility | Install cost | Multi-OS | Cold start | Verdict |
|----------|-----------------|--------------|----------|------------|---------|
| Official Docker images | ★★★★★ | ★★★ (Docker Desktop on Mac) | ★★★★★ | ★★★ (1–2 s) | **MVP** |
| Auto Adoptium download (SDKMAN-style) | ★★★★ | ★★★★ | ★★★★★ | ★★★★★ (instant after cache) | **V1** |
| BYO-JVM (user provides `JAVA_HOME`) | ★★ (host-dependent) | ★★★★★ | ★★★★ | ★★★★★ | V1, opt-in |
| SDKMAN as external dependency | ★★★ | ★★ (separate install) | ★★★★ | ★★★★ | rejected |

### 5.2 Why Docker is the priority

The main MVP use case is **GC-Insight CI + sales demos on
developer machines**. In both, Docker is already present or
trivial to install. Reproducibility outweighs cold start.

### 5.3 Why native is V1's priority

For users who want `gc-forge run scenario.yaml` instantly
without Docker (ops teams, trainers), a native mode with a local
cache is valuable. That is one or two weeks of effort in V1.

### 5.4 BYO-JVM (V1+, opt-in)

```bash
gc-forge run scenario.yaml --jvm-mode=byo --java-home=/opt/zulu21
```

Used to test on Zing/Prime without publishing an official image,
or on Oracle JDK that GC-Forge cannot redistribute for licensing
reasons.

## 6. GC log capture

### 6.1 Standard log flags

GC-Forge mandates the same `-Xlog` flags for every run
(consistency with the GC-Insight parser):

```
-Xlog:gc*=info,gc+heap=debug,gc+age=trace,gc+phases=debug,gc+humongous=trace:file=<path>:time,level,tags,pid,tid:filecount=0
```

Justifications:
- `gc*=info`: main stream (collections, pauses).
- `gc+heap=debug`: before/after sizes per generation.
- `gc+age=trace`: tenuring distribution (useful for cache-churn,
  mixed-pathological).
- `gc+phases=debug`: phase breakdown (root scan, copy, etc.).
- `gc+humongous=trace`: humongous allocation trace (G1).
- `time,level,tags`: canonical decorators.
- `filecount=0`: no rotation, single file.

### 6.2 Wall-clock timestamp suppression for reproducibility (V1)

To approach bit-for-bit reproducibility of the **manifest** (the
log itself remains non-bit-stable, cf. SPEC-FUNCTIONAL §4.7), two
options:
- Enable `uptime` instead of `time` in the decorators (timestamps
  in seconds since JVM start, reproducible up to clock drift).
- Post-process the log to rebase timestamps relative to start
  (option `--rebase-timestamps`).

Decision: by default, **`time` (ISO 8601)** like a production
log. The `--rebase-timestamps` option is exposed for CI.

### 6.3 Per-algorithm specifics

| Algorithm | Specifics to capture |
|-----------|----------------------|
| G1 | `humongous`, `ergonomics`, `marking`, `phases` |
| ZGC | `nmethod`, `phases`, `heap`, `start`, `marking`, `relocation` |
| Parallel | `phases`, `heap` |
| Shenandoah | `phases`, `heap`, `ergo`, `update-refs` |
| Serial | `phases`, `heap` |
| Epsilon | initialisation lines + shutdown summary only |

The exact per-algorithm flags are recorded in
`gc-forge-runner/src/flags.rs` and built from the scenario's
algorithm.

## 7. Test strategy and quality

### 7.1 Test pyramid

| Level | Target | Tool |
|-------|--------|------|
| Rust unit | Scenario parsing, invariant rules, `extends` resolution | `cargo test` |
| Java unit | Harness allocation building blocks | JUnit 5 |
| Short integration (90 s per test) | One preset per regime, invariant validation | `cargo test -- --ignored` (CI nightly) |
| Long integration | Every preset, inter-run variance | CI weekly |
| Cross-project | Forge log → parsed by Insight → invariants verified | `gc-core-roundtrip` |
| Performance | CLI cold start < 200 ms, MVP scenario complete in < 5 min | `criterion` |

### 7.2 `gc-forge selftest`

A single command that replays every shipped preset, checks
their invariants, measures variance, and emits a report.
Targeted at nightly CI.

### 7.3 Harness determinism

The Java harness must be **deterministic at fixed seed**:
- `java.util.Random` initialised by a unique seed.
- No `Math.random()` (global state).
- No `System.currentTimeMillis()` in business-logic decisions
  (only to measure total duration).
- `Thread.sleep` allowed but non-critical for GC-log determinism.

Test: `gc-forge variance-check <preset>` must show CV ≤ 5 % on
aggregated metrics.

### 7.4 Coverage

MVP target: ≥ 75 % on the Rust crates. No strict objective on
the Java harness (the Rust invariants validate the result).

## 8. Reproducibility and determinism

### 8.1 Levels of reproducibility

| Target | Achievable | Strategy |
|--------|------------|----------|
| Resolved manifest | Yes, bit-for-bit | Snapshot of the scenario after merge/override + harness hash + GC-Forge version |
| Workload harness | Yes, bit-for-bit | Reproducible Maven build (mvn 3.9+, Java 17, deps frozen in `pom.xml`); JAR SHA-256 recorded |
| GC behaviour | Semantically reproducible | Fixed seed, JVM identified by image tag, OS/arch recorded in the manifest |
| Bit-for-bit GC log | **Not achievable** on a real JVM | Documented limit; validation via invariants instead of hash |

### 8.2 Sources of non-determinism listed

- OS scheduler timing (impact on concurrent-cycle triggering).
- Dynamic CPU frequency (P/E cores on Apple Silicon, Intel
  turbo).
- Concurrent system load.
- Memory addresses (ASLR) — may influence some JIT
  optimisations.
- Wall-clock in the log (mitigated by `--rebase-timestamps`).

### 8.3 Guarantees offered

- **Same scenario, same seed, same Docker image** ⇒ expected
  phenomena present with probability ≥ 99 % (measured by
  variance-check).
- **Same scenario, different hosts** ⇒ invariants held within
  ± 5 % on aggregated metrics, except p99 (± 15 %).

### 8.4 CI: corpus regeneration

`make corpus-reference` (Makefile target) triggers `gc-forge
batch` on the reference matrix and publishes the
`corpus-reference-<git_sha>.tar.zst` archive in GitHub artifacts.
The GC-Insight CI consumes it per version.

## 9. Open source vs proprietary — decision

**Decision: open source under MIT from the MVP onwards.**

### 9.1 Argument

For: open source.
- **Marketing leverage for GC-Insight**: if GC-Forge becomes a
  de facto standard for producing GC fixtures in the JVM
  community, every user crosses paths with GC-Insight as the
  authoring product.
- **Adoption by performance teams**: DBAs, ops, perf teams who
  want to reproduce a behaviour at home. These users are also
  prospects for Insight.
- **External contributions**: every user who adds a regime or a
  JVM brings value at no cost.
- **Technical credibility**: in an expert domain (GC tuning), a
  verifiable open-source tool inspires confidence.
- **No algorithmic secret**: a parameterised harness + an
  orchestrator is reproducible engineering. The moat is not
  there.
- **GC-Insight's moat lies elsewhere**: SaaS, analytical
  heuristics, product experience, dataset, support.

Against: open source.
- Risk of hostile fork by a competitor. **Rated low**: very few
  direct competitors on GC log analysis; those who exist already
  have their own internal tools.
- Cost of governance (issues, PRs, releases, security).
  **Acceptable**: MIT = few constraints, minimal governance
  possible (BDFL).

### 9.2 Modalities

- **Licence**: MIT (permissive, compatible with commercial use,
  maximum simplicity — no explicit patent clause, which matches
  the IP risk profile of GC-Forge: a parameterised harness
  without patentable algorithmic innovation). Already in place
  in `LICENSE`.
- **Repo**: public on GitHub (`<org>/gc-forge`).
- **CLA**: Developer Certificate of Origin (DCO), no heavy CLA
  at the start.
- **Governance**: BDFL (Jérôme); transition to open governance
  if adoption is significant (> 10 regular contributors).
- **Code of Conduct**: Contributor Covenant standard.
- **Releases**: SemVer. Tags `vX.Y.Z`. CHANGELOG kept up to
  date.

### 9.3 Implications on GC-Insight

No reverse technical dependency: GC-Insight does not depend on
GC-Forge at runtime (only on `gc-core`, which is itself
independent). GC-Forge can therefore evolve in open source
without constraining Insight.

`gc-core` must decide: open source (recommended, as a "lingua
franca" of the domain) or proprietary (acceptable but forces a
fork to be maintained in GC-Forge). **Recommendation**:
`gc-core` open-source MIT, **without** the GC-Insight analytical
heuristics (which stay in proprietary `gc-insight-core`).

## 10. Packaging and distribution

### 10.1 MVP targets

| Platform | Format | Build | Distribution |
|----------|--------|-------|--------------|
| Linux x86_64 | Static binary (`musl`) | `cargo build --release --target=x86_64-unknown-linux-musl` | GitHub Releases + `cargo install` |
| macOS x86_64 | Binary | `cargo build --release` | GitHub Releases + Homebrew tap (V1) |
| macOS aarch64 (Apple Silicon) | Binary | `cargo build --release --target=aarch64-apple-darwin` | GitHub Releases + Homebrew tap (V1) |
| Linux aarch64 | Binary | Cross via `cross` | GitHub Releases |
| Windows | — | — | V2 |

### 10.2 Docker images (MVP)

- `ghcr.io/<org>/gc-forge:<version>` and
  `ghcr.io/<org>/gc-forge:<version>-jdk21` —
  multi-architecture image with `gc-forge` +
  `workload-harness.jar` on Temurin 21.
- `ghcr.io/<org>/gc-forge:<version>-jdk17` — JDK-17 variant.
- `ghcr.io/<org>/gc-forge:latest` — alias of the JDK-21 image at
  the most recent release.

### 10.3 Cargo and publication

- Crates published to crates.io: `gc-forge-scenario`,
  `gc-forge-regimes`, `gc-forge-runner`, `gc-forge-validate`,
  `gc-forge-presets`. The binary `gc-forge-cli` is also
  published (for `cargo install gc-forge-cli`).
- `workload-harness.jar` is published as a GitHub Release asset
  (and via Maven Central in V1 if it makes sense).

### 10.4 User installation — target UX

```bash
# via cargo
cargo install gc-forge-cli

# via Homebrew (V1)
brew install <tap>/gc-forge

# via Docker
docker run --rm -v $PWD:/work ghcr.io/<org>/gc-forge run /work/scenario.yaml
```

## 11. CI integration

### 11.1 GC-Forge CI (this repo)

- GitHub Actions, jobs:
  - `lint`: `cargo fmt --check`, `cargo clippy -D warnings`,
    `cargo deny check`, `mvn validate` on the harness.
  - `test`: `cargo test --workspace` (unit), `mvn test` on the
    harness.
  - `selftest`: `gc-forge selftest` on every preset — triggered
    nightly (10–15 min).
  - `release`: on tag, build matrix, publication to Releases +
    crates.io + GHCR.

### 11.2 GC-Insight CI (neighbour repo)

A `regenerate-corpus-reference` job that:
1. Pulls the image
   `ghcr.io/<org>/gc-forge:<version>-jdk21`.
2. Runs `gc-forge batch corpus-reference.yaml --out-dir corpus/`.
3. Compares the result against a baseline (on invariants, not on
   hashes).
4. Fails if an invariant is violated or an unexpected new
   phenomenon appears.

### 11.3 Versions and compatibility

- A GC-Insight release declares a supported GC-Forge version
  range (e.g. `>=0.3, <0.4`).
- A compatibility matrix is published in the GC-Insight README.

## 12. Security, sandboxing, execution

- The Java harness is **non-privileged**, with no network access
  (`--network=none` in Docker).
- Docker volumes read-only except the output directory.
- No arbitrary user code execution in the harness — only typed
  parameters.
- Resource limits exposed: `--cpus`, `--memory` Docker, native
  `ulimit`.

## 13. Observability of GC-Forge itself

- Structured CLI logs (`tracing` + `tracing-subscriber`),
  adjustable level.
- Machine output: `--output json` (V1) returns the manifest +
  validation result as JSON on stdout.
- Telemetry: none by default. Opt-in `--telemetry` option in V2
  if adoption tracking is desired (anonymised, disableable).

## 14. Salient technical risks (cross-reference)

The structural technical risks are recorded in
[`RISKS.md`](./RISKS.md). Those most worth keeping in mind during
implementation:
- Inter-host variance too high for invariant validation
  (mitigation: Docker + CPU pinning).
- Cost of corpus regeneration in CI (mitigation: caching,
  parallelisation, short runs).
- GC log format evolution between JDKs (mitigation: per-version
  tests, versioned `gc-core` parser).
- Third-party JVM licensing (mitigation: Temurin only at MVP).

---

**Recommended reading to start the implementation**: §2
(workspace), §4 (components), §5 (runner), §6 (log capture),
then §8 (reproducibility). The Java harness can start in
parallel with the Rust CLI as soon as the argument contract
(§4.4) is frozen.
