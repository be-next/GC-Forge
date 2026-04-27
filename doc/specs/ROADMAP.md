# GC-Forge — roadmap

> **Reference**: brief of 25 April 2026, §8.3.
> **MVP target as scoped**: extended MVP — 8 to 10 weeks solo
> part-time (~12 h / week of working time).
> **Status (2026-04-27)**: phases 0 through 3 are delivered; the
> 0.1.0 line is functionally complete on `main`. Phase 4 (native
> runner) is the next step. Sections 3–6 below are kept as a
> historical record of what was scoped versus what was delivered.

## 1. Guiding principle

Deliver an MVP that is **useful for the priority use case**
(testing and validating GC-Insight) with **all the essential
axes** (multiple collectors, two JDK versions, seven regimes,
rich manifest, batch mode). Defer to V1 anything that adds JVM
coverage (other JVMs, native runner, BYO-JVM) rather than core
value.

> **Scoping note**. The brief §9 targets 4–6 weeks for the MVP.
> This roadmap proposed 8–10 weeks for an **extended MVP**
> agreed at cadrage time (more collectors, two JDK versions,
> seven regimes, rich metadata from day one). The shorter 4–6
> week trajectory remains documented in §10 (*sensitivities*).
> The choice between the two trajectories is the maintainer's,
> trading urgency against initial coverage.

## 2. Phase overview

| Phase | Duration | Cumulative effort | Output | Status |
|-------|----------|-------------------|--------|--------|
| **Phase 0 — Setup** | 1 week | 1 wk | Cargo workspace, Maven harness, empty-but-green CI, first JVM-in-Docker `hello-world`. | **Delivered.** |
| **Phase 1 — Core trunk** | 2 weeks | 3 wk | `gc-forge run scenario.yaml` produces a log + manifest for one regime (steady-state). | **Delivered.** |
| **Phase 2 — Regimes and collectors** | 3 weeks | 6 wk | Seven regimes × initial three collectors (G1/ZGC/Parallel), shipped presets, invariant-based validation. | **Delivered.** Scope expanded during implementation to six collectors total (+Shenandoah, +Serial, +Epsilon). |
| **Phase 3 — Batch, quality, doc, release `0.1.0`** | 2 weeks | 8 wk | `gc-forge batch`, `selftest`, `variance-check`, user docs, `0.1.0` release. | **Delivered** (release pipeline ready locally; tag and publish are operator-gated). |
| **Phase 4 (optional)** | 2 weeks | 10 wk | Native runner, CLI polish, GC-Insight CI integration. | **Pending.** Decision is the maintainer's. |
| **V1** | T+3 months | — | Corretto, GraalVM, OpenJ9 ; mirror ; HTML variance-check ; Homebrew. | **Pending.** Shenandoah and Serial were promoted from V1 to MVP during implementation. |
| **V2** | T+6 months | — | Synthetic-hybrid generation, Zing/Prime, ML sweep, service mode. | **Pending.** |

## 3. Phase 0 — Setup (week 1)

**Objective**: a functional development environment and a green
CI on minimal code.

**Deliverables**:
- Cargo workspace with the crates from SPEC-TECHNICAL §2.1
  (skeletons).
- `workload-harness/pom.xml` building a "hello-world" fat-jar
  (trivial allocation loop).
- GitHub Actions: `lint`, `test`, `build`, green on `main`.
- `gc-forge --version` works.
- `Dockerfile` for `gc-forge-runner:dev-jdk21` (Temurin 21 +
  embedded harness).
- Developer README with `make bootstrap`, `make test`, `make
  demo`.

**Exit criteria**: a commit that passes CI and lets `make demo`
emit a GC log from a Docker container, however rudimentary.

**Risks at the time**: multi-arch Docker on Apple Silicon — 1–2
days of buffer.

**Outcome (2026-04-27)**: delivered as scoped.

## 4. Phase 1 — Core trunk (weeks 2–3)

**Objective**: deliver the full chain on **one** regime
(`steady-state-healthy`) with a single collector (G1 on Temurin
21).

**Scope**:
1. YAML scenario parser (`gc-forge-scenario`) with Rust-typed
   validation and a generated JSON Schema.
2. `extends` and `--override` operational.
3. `DockerRunner`: launches a Docker JVM, passes flags, captures
   the log.
4. Java harness `SteadyStateRegime` parameterised
   (`allocation_rate`, `live_set`, `lifetime`).
5. `gc-forge run scenario.yaml` produces a valid log and
   manifest.
6. Manifest contains the resolved scenario, JVM version, log
   SHA-256, and harness JAR SHA-256.
7. First preset: `steady-g1-baseline`.
8. `gc-forge lint` (validation without execution).

**Exit criteria**:
- `gc-forge run presets/steady-g1-baseline.yaml` finishes in
  < 2 min, producing a valid G1 log and a complete manifest.
- `gc-forge lint` rejects invalid YAML.
- CI integration test running the preset in docker-in-docker.

**Anti-goals**: no post-run validation (Phase 2), no batch
(Phase 3), no other collector.

**Outcome**: delivered as scoped.

## 5. Phase 2 — Regimes and collectors (weeks 4–6)

**Objective**: cover the seven regimes × three initial
collectors with invariant-based validation.

**Recommended sprints** (1 sprint = 1 week):

### Sprint 4 — Algorithms
- Generational ZGC (flags + capture).
- Parallel.
- Per-collector unit tests: a single `steady-state` must produce
  a conformant log on the three collectors.
- Baseline presets for ZGC and Parallel.

### Sprint 5 — Regimes 1/2
Implementation and presets:
- `allocation-burst` (R2).
- `humongous-pressure` (R3).
- `cache-churn` (R5).

For each regime: harness Java code, Rust invariants, YAML
preset, integration test.

### Sprint 6 — Regimes 2/2 + validation
Implementation:
- `slow-leak` (R4).
- `mixed-gc-pathological` (R6).
- `microservice-stop-and-go` (R7).
- `gc-forge validate <log> --manifest <m.yaml>` operational for
  every regime.
- Shared GC-log parser via `gc-core` (sufficient for the
  invariants; not an exhaustive parser).

**Phase 2 exit criteria**:
- 14 MVP presets shipped and passing.
- `gc-forge validate` returns 0 on the 14 presets when run with
  the nominal seed.
- Partial user documentation (one paragraph per regime +
  parameters).
- `gc-core-roundtrip` cross-project test: a log produced by
  GC-Forge is parsed by the proto-Insight parser and the
  reconstructed event sequence matches.

**Anti-goals**: no `variance-check` (Phase 3), no native runner
(Phase 4).

**Outcome**: delivered as scoped, plus three additional
collectors (Shenandoah, Serial, Epsilon) and seven additional
presets — promoted from V1 to MVP because they reuse the same
workload harness and the same parser. Final shipped catalogue:
21 presets across six collectors.

## 6. Phase 3 — Batch, quality, documentation, `0.1.0` release (weeks 7–8)

**Objective**: release packaging and quality.

**Deliverables**:
1. `gc-forge batch matrix.yaml` with controlled parallelism.
2. `gc-forge selftest`: runs every preset, checks invariants,
   summary report.
3. `gc-forge variance-check <preset> --runs N`: measures
   inter-run CV.
4. `gc-forge presets list/show/export`.
5. Complete user documentation:
   - `doc/user/getting-started.md`,
   - `doc/user/regimes.md` (one paragraph per regime + parameters
     + expected signature),
   - `doc/user/scenario-reference.md` (schema reference),
   - `doc/user/cli-reference.md` (subcommands).
6. Nightly CI running `selftest` and `variance-check`.
7. Release `0.1.0`:
   - GitHub Release with Linux x64/arm64 and macOS x64/arm64
     binaries.
   - Docker images `ghcr.io/<org>/gc-forge:0.1.0-jdk{17,21}` and
     `:latest`.
   - Crates published to crates.io.
   - CHANGELOG and pitch document.
8. **Written pitch**: one-page document under
   [`doc/concepts/overview.md`](../concepts/overview.md).

**Exit criteria**:
- A new user can install GC-Forge, run a preset, and read the
  manifest without support.
- `gc-forge selftest` is green in nightly CI.
- Inter-run variance measured and documented for the shipped
  presets.

**At this point brief §9 must be demonstrable**:
- A Rust developer understands the architecture →
  `SPEC-TECHNICAL.md` is sufficient.
- Structural choices are explicit → `SPEC-FUNCTIONAL.md` and
  `SPEC-TECHNICAL.md` together cover the points.
- GC-Insight consistency is demonstrated → `gc-core-roundtrip`
  test and the
  [traceability matrix](../concepts/traceability.md).
- Internal or external pitch → [`overview.md`](../concepts/overview.md).

**Outcome**: delivered as scoped, with the additional release
plumbing produced during the consistency pass (multi-arch
Docker images for both JDK 17 and JDK 21, Dependabot, etc.).
The four operator-gated steps (`git push`, tag creation,
`release` environment approvals, GitHub Release promotion) are
documented in
[`doc/process/orchestration.md`](../process/orchestration.md).

## 7. Phase 4 — Native runner and polish (weeks 9–10)

**Objective**: remove the Docker dependency for users who do not
have it.

**Deliverables**:
1. `NativeRunner`: Adoptium download, checksum verification,
   cache under `~/.gc-forge/jvms/`.
2. Auto Docker/native selector: uses Docker if available,
   otherwise native; configurable by flag.
3. Integration tests on both runners for the shipped presets.
4. CLI polish: error messages, suggestions, shell completion
   (bash/zsh/fish).
5. Homebrew tap (`brew install <tap>/gc-forge`).
6. GC-Insight CI integration: a `regenerate-corpus-reference`
   job operational and blocking.

**Exit criteria**: MVP delivered.

**Phase 4 = optional** in the sense that `0.1.0` (end of Phase
3) is usable and useful. Phase 4 advances to `0.2.0` with the
native runner and finalises the GC-Insight integration.

**Status**: not started. Triggering it is the maintainer's
decision.

## 8. V1 — Coverage and ergonomics (T+3 months after MVP)

### V1.0 — JVM coverage broadening
- **Corretto** 17 and 21 (effectively Temurin under another
  label; small effort).
- **OpenJ9** 17 and 21 — **different log format**, requires
  extending the `gc-core` parser. Significant effort. The single
  V1 item that needs more than trivial work.
- **GraalVM CE 21** (HotSpot variant — small effort).

### V1.1 — Pathology mirroring
- `gc-forge mirror <prod.log>`: heuristic that proposes a mirror
  scenario from a customer's production log (use case F-REPRO of
  the brief).
- Onboarding: "paste your log, I propose a mirror scenario".

### V1.2 — Ergonomics
- JFR export (`capture_jfr: true` in the scenario).
- Self-contained HTML output for `variance-check`
  (`--report.html`).
- Snippet shortcuts (`--snippet humongous`).

> **Note on the original V1.1 scope.** The earlier roadmap also
> placed Shenandoah and Serial in V1.1. They were brought
> forward to the MVP during implementation (they are available
> in Temurin and require no parser change), so the V1 focus is
> now JVM coverage rather than collector coverage.

## 9. V2 — Strategic (T+6 months)

**Tracks**:
- **Synthetic-hybrid generation**: Rust generator calibrated on
  real traces, for ML datasets.
- **`sweep` mode**: parametric sweep across N seeds.
- **Zing / Prime / Oracle JDK** subject to licensing.
- **Service mode**: HTTP daemon producing logs on demand
  (internal SaaS).
- **Legacy CMS** on JDK 8 for historical cases (specific
  effort).

## 10. Estimation and cadence

**Capacity assumption**: 12 h / week solo (compatible with the
brief's "part-time"). With a 20 % margin, realistic capacity =
**9–10 h useful / week**.

| Phase | Calendar duration | Useful effort | Buffer |
|-------|-------------------|---------------|--------|
| 0 | 1 wk | 8 h | 2 h |
| 1 | 2 wk | 18 h | 4 h |
| 2 | 3 wk | 27 h | 6 h |
| 3 | 2 wk | 18 h | 4 h |
| 4 | 2 wk | 18 h | 4 h |
| **Total** | **10 wk** | **89 h** | **20 h** |

**Sensitivities**:
- +30 % if OpenJ9 enters the MVP (different log format).
- +20 % if inter-host variance becomes problematic and demands
  stabilisation work (CPU pinning, etc.).
- −10 to −20 % if Phase 4 is dropped (native runner deferred to
  V1.0).
- **−40 % toward a "brief-original MVP" trajectory (4–6 weeks)**:
  one collector (G1 only), one JDK (21 only), five regimes
  (drop R5/R6/R7), no batch mode, no Phase 3 polish, no Phase 4.
  Scope: `gc-forge run scenario.yaml` with post-run validation,
  five presets. This is a demonstrator, not yet a CI-grade
  validation tool.

**Recommendation**: aim for end of Phase 3 (release `0.1.0`) at
8 weeks; keep Phase 4 optional if capacity allows. If urgent or
capacity-constrained, switch to the short trajectory above.

## 11. Milestones and decision points

| Milestone | Wk | Decision / Validation |
|-----------|----|-----------------------|
| M1 — Bootstrap CI green | 1 | None. If not reached in 1 wk, simplify the CI matrix. |
| M2 — First log produced | 3 | Validate the captured log format (Insight consistency). |
| M3 — Three collectors × steady-state | 4 | Verify that the three log parsers are aligned. |
| M4 — Seven regimes implemented | 6 | **Go/no-go for release**: if invariant quality is not there, replan. |
| M5 — `0.1.0` release | 8 | Public announcement, GC-Insight visibility. |
| M6 — Native runner | 10 | MVP delivered or packaging decision V1. |

## 12. Critical external dependencies

- **`gc-core` availability**: if `gc-core` is immature on the
  GC-Insight side at the start of Phase 1, add 2 weeks to
  stabilise it or temporarily clone the structures into
  GC-Forge.
- **Eclipse Temurin Docker images**: very stable, no risk.
- **Adoptium API** (V1): stable since 2021, low-risk.
- **No AWS dependency**, per brief §6.

## 13. Post-MVP success metrics

To measure 4 weeks after the `0.1.0` release:

- ≥ 1 external contributor (issue or PR).
- GC-Insight reference corpus regenerated 100 % from CI.
- ≥ 3 blog articles or training assets using GC-Forge logs.
- Search-index presence (`site:github.com gc-forge`).
- Invariant bug rate: zero silent regression on the shipped
  presets.
