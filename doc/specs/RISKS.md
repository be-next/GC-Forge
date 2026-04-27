# GC-Forge — risks and open points

> **Reference**: brief of 25 April 2026, §8.5.
> **Status (2026-04-27)**: each risk carries an **outcome** line
> reporting how it played out during the 0.1.0 implementation.

## Conventions

- **Probability (P)**: 1 (rare) to 5 (very likely).
- **Impact (I)**: 1 (negligible) to 5 (project-blocking).
- **Severity**: `P × I`.
- **Mitigation**: preventive or risk-reducing action.
- **Triggers**: signals that should raise the alarm.
- **Outcome**: 0.1.0-era retrospective.

---

## 1. Technical risks

### R-T1 — Inter-host variance too high for invariant validation

- **P**: 3 **I**: 4 **Severity**: 12
- **Description**: Real GC timing depends on the OS, the CPU,
  and the system load. If the variance on aggregated metrics
  exceeds 5 % between runs or between hosts, the invariants no
  longer hold, and the "reproducible replay" promise breaks down.
- **Mitigation**:
  - Mandate Docker at MVP with explicit CPU/memory limits.
  - Measure variance from Phase 2 onwards (every shipped regime
    includes a `variance-check`).
  - In case of trouble: wider tolerance (10–15 % instead of 5 %)
    on the most sensitive metrics, invariants expressed as
    **ranges** rather than **thresholds**.
  - Fallback: CPU pinning on CI runners (`taskset` on Linux),
    turbo disabled.
- **Triggers**: variance > 10 % on aggregated metrics, systematic
  divergence between Linux and macOS.
- **Owner**: GC-Forge maintainer.
- **Outcome**: contained. The shipped `gc-forge variance-check`
  reports CV per metric with the documented budgets (≤ 8 % for
  counts, ≤ 10 % for means, ≤ 20 % for percentiles per
  SPEC-FUNCTIONAL §7.3). No regime was disqualified during
  development.

### R-T2 — GC log format changes between JDK versions

- **P**: 2 **I**: 3 **Severity**: 6
- **Description**: A JDK minor release can introduce a new tag
  or change a format. The `gc-core` parser must follow.
- **Mitigation**:
  - Per-JDK integration tests (CI matrix).
  - `gc-core` versioned in SemVer.
  - Watch OpenJDK release notes.
- **Triggers**: a new minor JDK breaks a parsing test.
- **Owner**: `gc-core` (shared between Forge and Insight).
- **Outcome**: not triggered during 0.1.0. The unified
  `-Xlog:gc*` format is stable across Temurin 17 and 21.

### R-T3 — Imperfect Java JAR reproducibility

- **P**: 3 **I**: 2 **Severity**: 6
- **Description**: Maven can produce JARs whose SHA-256 varies
  (embedded timestamps, file ordering).
- **Mitigation**:
  - `reproducible-build-maven-plugin` or `maven-jar-plugin`
    configured for fixed timestamps.
  - CI test: two consecutive builds yield the same hash.
- **Triggers**: hash test fails.
- **Owner**: harness maintainer.
- **Outcome**: handled by the Maven build configuration; no
  reproducibility issue surfaced in 0.1.0.

### R-T4 — OpenJ9 forces a parser overhaul

- **P**: 4 (if V1 includes OpenJ9) **I**: 3 **Severity**: 12
- **Description**: OpenJ9 has a log format distinct from
  HotSpot. Extending `gc-core` may reveal that the event
  abstraction is too HotSpot-centric.
- **Mitigation**:
  - Defer OpenJ9 to V1.0 (not MVP).
  - 2–3-day spike before planning V1 to estimate the actual
    effort.
  - Design `gc-core::EventKind` with OpenJ9 in mind.
- **Triggers**: OpenJ9 spike reveals an effort > 2 sprints.
- **Owner**: `gc-core`.
- **Outcome**: deferral confirmed. OpenJ9 stays in V1.0; no
  spike has been performed yet.

### R-T5 — Generational ZGC immature on JDK 17

- **P**: 5 **I**: 2 **Severity**: 10
- **Description**: Generational ZGC is GA only in JDK 21. On
  JDK 17, ZGC is non-generational, which changes the log
  signature.
- **Mitigation**:
  - Document clearly: `gc.options.generational: true` is only
    available on JDK 21+.
  - `gc-forge lint` rejects `generational: true` on JDK 17.
  - `*-zgc-*` presets target JDK 21 by default.
- **Triggers**: none — known constraint.
- **Owner**: SPEC + dev.
- **Outcome**: handled. The runner emits `-XX:+ZGenerational`
  by default on JDK 21+; an explicit `generational: false` on
  JDK 21 emits `-XX:-ZGenerational` and selects the
  non-generational variant. A dedicated preset
  (`steady-zgc-nongen-baseline`) was added during the
  consistency pass.

### R-T6 — Docker Desktop on macOS = heavy to install

- **P**: 3 **I**: 3 **Severity**: 9
- **Description**: Many macOS users do not have Docker, or have
  the paid commercial version.
- **Mitigation**:
  - Native runner (Phase 4 of MVP / V1) avoids Docker.
  - Document Colima / OrbStack as free alternatives.
  - No hard Docker requirement in the user documentation —
    Docker is one runner among others.
- **Triggers**: recurring user feedback.
- **Owner**: runner maintainer.
- **Outcome**: still active. The native runner is Phase 4
  (pending). Colima/OrbStack are not yet documented; they
  will be addressed before the public 0.1.0 announcement.

### R-T7 — JVM cold start amplifies short-run duration

- **P**: 4 **I**: 2 **Severity**: 8
- **Description**: Temurin cold start = 1–2 s. On a 90-s preset,
  that injects 2 % noise at the start.
- **Mitigation**:
  - Implement `warmup` in the harness: the measured duration
    starts after warmup.
  - Log the warmup but exclude it from invariants by default.
- **Triggers**: none, designed in from the start.
- **Owner**: harness maintainer.
- **Outcome**: handled. `spec.warmup` is part of the schema and
  defaults to 10 s in the runner.

### R-T8 — Full-corpus execution cost in CI

- **P**: 3 **I**: 2 **Severity**: 6
- **Description**: 21 presets × 2 JDKs × 3 seeds = 126 runs of
  30 s–5 min = several hours.
- **Mitigation**:
  - `gc-forge batch` parallelisation (V1).
  - CI reference corpus = subset (e.g. 1 seed, JDK 21 only).
  - Full corpus = nightly or weekly, not per PR.
- **Triggers**: blocking CI duration.
- **Owner**: CI.
- **Outcome**: handled. `gc-forge selftest` defaults to
  `--per-preset-duration 12s` (the full pass under ~3 min); a
  faithful selftest is reserved for nightly runs.

### R-T9 — Instability of `gc+humongous=trace` parsing

- **P**: 2 **I**: 2 **Severity**: 4
- **Description**: `gc+humongous` log lines have a less stable
  format than the main log.
- **Mitigation**:
  - Tolerant parser: unparseable line = warning, not error.
  - Regression tests on real captured logs.
- **Triggers**: parser false positives.
- **Owner**: `gc-core`.
- **Outcome**: not triggered. The validator's parser is
  line-by-line and tolerant by design.

---

## 2. Project risks

### R-P1 — Insufficient solo part-time capacity

- **P**: 3 **I**: 4 **Severity**: 12
- **Description**: 12 theoretical h/week can drop to 6 h under
  GC-Insight load. The 8–10-week target slips.
- **Mitigation**:
  - Decomposition into usable intermediate deliverables (cf.
    ROADMAP §3–7).
  - Phase 4 considered optional from the start.
  - If slippage > 2 weeks at end of Phase 2: reduce regime
    scope (e.g. drop R6 pathological to V1).
- **Triggers**: cumulative delay > 1 week at end of phase.
- **Owner**: maintainer.
- **Outcome**: not triggered. The 17 implementation iterations
  were all merged within the 8-week envelope.

### R-P2 — Contention with GC-Insight on `gc-core`

- **P**: 3 **I**: 3 **Severity**: 9
- **Description**: Forge and Insight share `gc-core`. A
  unilateral evolution can break the other.
- **Mitigation**:
  - Strict SemVer.
  - Cross-project tests (`gc-core-roundtrip`).
  - A PR on `gc-core` rebuilds both Forge and Insight in CI.
- **Triggers**: regular conflicts on shared structures.
- **Owner**: `gc-core` BDFL.
- **Outcome**: still active. The roundtrip test is documented
  in SPEC-TECHNICAL §7.4 but its enforcement is conditioned on
  GC-Insight being co-developed; the Forge-side counterpart is
  in place.

### R-P3 — Scope drift under "demo-friendly presets" pressure

- **P**: 4 **I**: 2 **Severity**: 8
- **Description**: Tendency to add "speaking" presets endlessly
  instead of finalising the trunk.
- **Mitigation**:
  - MVP quota = 14 presets (with 2 bonuses). Beyond: V1.
  - Clear backlog (cf. BACKLOG §EPIC 7).
- **Triggers**: frequent `presets/` commits without trunk
  progress.
- **Owner**: maintainer.
- **Outcome**: drifted *upwards* but for principled reasons.
  The quota was raised to 21 presets when the new collectors
  (Shenandoah, Serial, Epsilon) were promoted from V1 to MVP;
  each additional preset reuses the existing harness and
  validator and adds no maintenance debt.

---

## 3. Product / ecosystem risks

### R-X1 — No user traction in open source

- **P**: 3 **I**: 2 **Severity**: 6
- **Description**: GC-Forge does not take off in open source.
  No contributors, no buzz.
- **Mitigation**:
  - The internal utility for validating GC-Insight already
    justifies the investment; external traction is a bonus.
  - Communicate from the `0.1.0` release: blog article,
    LinkedIn post, share in JVM communities.
  - JVM performance conferences (FOSDEM, Devoxx, JFokus).
- **Triggers**: 6 months post-release without external
  contributor or public mention.
- **Owner**: maintainer.
- **Outcome**: too early to evaluate (release pending).

### R-X2 — A competitor forks and positions

- **P**: 1 **I**: 3 **Severity**: 3
- **Description**: A competing vendor forks GC-Forge and uses
  it to market their own product.
- **Mitigation**:
  - MIT = legally fine, cannot be prevented.
  - Maintain the lead on Forge ↔ Insight consistency.
  - Brand recognition for GC-Forge in the community.
- **Triggers**: an active fork is detected.
- **Owner**: maintainer.
- **Outcome**: not triggered.

### R-X3 — "Generating fixtures = sign of weakness" perception

- **P**: 2 **I**: 2 **Severity**: 4
- **Description**: Possible adversarial argument: "if you have
  to generate your test logs, it means you have no real
  customers".
- **Mitigation**:
  - Communicate clearly: GC-Forge is for **validation** and
    **demonstration**, not for replacing real customer logs.
  - Real customer logs remain confidential — that is precisely
    why a public corpus is needed.
  - Note that serious vendors (HotSpot, ZGC) generate their own
    fixtures for their own tests.
- **Triggers**: objection raised by a prospect.
- **Owner**: pitch / sales.
- **Outcome**: not triggered.

---

## 4. Legal / licensing risks

### R-L1 — Oracle JDK licence

- **P**: 5 (if included) **I**: 4 **Severity**: 20
- **Description**: Oracle JDK is not freely redistributable. No
  Docker image possible.
- **Mitigation**:
  - **Do not include in GC-Forge Docker images.**
  - BYO-JVM (V1+) for users with their own Oracle licence.
- **Triggers**: none — known constraint.
- **Owner**: SPEC.
- **Outcome**: respected. Only Eclipse Temurin (Adoptium) is
  shipped.

### R-L2 — Azul Zing/Prime licence

- **P**: 5 **I**: 3 **Severity**: 15
- **Description**: Zing/Prime are proprietary, redistribution
  forbidden, log format not publicly documented.
- **Mitigation**:
  - Defer to V2 subject to agreement with Azul.
  - BYO-JVM lets an Azul customer use their own licence.
- **Owner**: to instruct in V2.
- **Outcome**: deferred.

### R-L3 — TCK and Java compatibility

- **P**: 1 **I**: 2 **Severity**: 2
- **Description**: Our Java harness does not touch the TCK (we
  consume a JVM, we do not implement one).
- **Mitigation**: none.
- **Outcome**: not applicable.

### R-L4 — Rust dependencies with incompatible licences

- **P**: 1 **I**: 3 **Severity**: 3
- **Description**: A transitive crate under GPL/AGPL pollutes
  the MIT licence.
- **Mitigation**:
  - `cargo deny check licenses` in CI with a permissive
    allowlist (MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS-2016,
    Zlib).
- **Triggers**: `cargo deny` red.
- **Owner**: CI.
- **Outcome**: handled. `deny.toml` is in place; the DoD-gate
  step is currently optional (`cargo-deny` not yet installed
  on every dev machine).

---

## 5. Security risks

### R-S1 — Unsandboxed harness execution

- **P**: 1 **I**: 4 **Severity**: 4
- **Description**: The harness ships with GC-Forge but runs on
  the host machine. A vulnerability in a Java dependency could
  be exploited.
- **Mitigation**:
  - No network (`--network=none`).
  - Read-only volumes except output.
  - Dependency scanning (`mvn dependency-check`).
  - No arbitrary user input in the harness — only typed
    parameters.
- **Owner**: harness + CI.
- **Outcome**: handled. The DockerRunner always passes
  `--network=none`; the harness reads typed parameters only.

### R-S2 — Supply chain (compromised Docker image)

- **P**: 1 **I**: 4 **Severity**: 4
- **Description**: The `eclipse-temurin` image or our own
  images get compromised.
- **Mitigation**:
  - Cosign / Sigstore signature on GHCR images.
  - SBOM published at every release.
- **Owner**: CI / release.
- **Outcome**: not yet implemented. Cosign signing is reserved
  for the post-release hardening pass; tracked under O-7
  below.

---

## 6. Open points to settle after the specifications

Decisions intentionally left open at spec time, to be addressed
during development:

### O-1 — `gc-core` open source or proprietary?

- **Status**: recommendation made (open source) in
  SPEC-TECHNICAL §9.3, to be confirmed with the GC-Insight
  project.
- **Deadline**: before Phase 1 (week 2).
- **Owner**: maintainer + GC-Insight.
- **Outcome**: GC-Forge ships under MIT. The cross-repository
  decision for `gc-core` is the same.

### O-2 — Third-party Homebrew tap or `homebrew-core`?

- **Status**: open. Third-party tap in V1, `homebrew-core`
  application when adoption justifies it (typically > 1k
  stars).
- **Deadline**: V1.0.

### O-3 — Custom `expected_invariants` syntax in scenarios

- **Status**: planned in the schema (cf. SPEC-FUNCTIONAL §5.1)
  but the expression syntax was not frozen at spec time.
  Options:
  - Custom DSL (`young_pause_p99 < 50ms`).
  - JSONLogic.
  - Embedded Lua/Rhai.
- **Recommendation**: minimal DSL in V1, embedded Lua if strong
  demand in V2.
- **Deadline**: V1.
- **Outcome**: a minimal rule language ships in 0.1.0
  (count comparators, ratio thresholds, percentile thresholds,
  boolean rules; cf. CLI reference). Unknown rules are skipped
  with a clear note for forward-compatibility.

### O-4 — Release strategy of `workload-harness.jar`

- **Status**: bundled in the Docker images, exposed as a
  GitHub Release asset. Should it be published to Maven Central?
- **Recommendation**: not in MVP (the harness is an
  implementation detail), yes in V2 if an embedding API is
  exposed.
- **Deadline**: V2.

### O-5 — Opt-in telemetry

- **Status**: not MVP, to be addressed in V2 depending on
  adoption.
- **Deadline**: V2.

### O-6 — Windows target

- **Status**: not MVP, not V1. Technically feasible (Rust
  supports it, the Java harness is platform-independent), a
  question of demand.
- **Deadline**: to be re-evaluated 6 months post-release.

### O-7 — Docker image signing

- **Status**: planned, tool not chosen (Cosign vs Notary v2).
- **Recommendation**: Cosign / Sigstore (CNCF standard).
- **Deadline**: post-release hardening pass.

### O-8 — Long-term governance

- **Status**: BDFL at start. Move to open governance
  (steering committee) if > 10 regular contributors.
- **Deadline**: 12 months post-release.

---

## 7. Risk review plan

- **Cadence**: review at each roadmap milestone (M1 to M6).
- **Retrospective**: at every release, this document is
  updated.
- **Escalation**: if a severity ≥ 12 risk turns red (trigger
  active), stop and replan rather than press on blindly.

## 8. Heat-map summary

```
              Impact →
            1    2    3    4    5
        ┌─────┬─────┬─────┬─────┬─────┐
   P  5 │     │     │ R-L2│ R-L1│     │
   r    ├─────┼─────┼─────┼─────┼─────┤
   o  4 │     │ R-T7│     │ R-T4│     │      severity 16-25 = critical
   b    ├─────┼─────┼─────┼─────┼─────┤      severity 8-15  = elevated
   a  3 │     │ R-T8│ R-P2│ R-T1│     │      severity 4-7   = moderate
   b    │     │ R-T3│ R-T6│ R-P1│     │      severity 1-3   = low
   i    │     │ R-X1│     │     │     │
   l    │     │     │     │     │     │
   i  2 │     │ R-T9│ R-T2│     │     │
   t    │     │ R-X3│     │     │     │
   y    ├─────┼─────┼─────┼─────┼─────┤
      1 │     │ R-L3│ R-X2│ R-S1│     │
        │     │     │ R-L4│ R-S2│     │
        └─────┴─────┴─────┴─────┴─────┘
```

**Priority watch risks**: R-T1 (variance), R-P1 (capacity),
R-T4 (OpenJ9), R-L1 (Oracle JDK).

**Status of priority watch risks at 0.1.0**: R-T1 contained
(no regime disqualified); R-P1 not triggered (delivered within
envelope); R-T4 deferred (OpenJ9 stays V1); R-L1 respected
(Temurin only).
