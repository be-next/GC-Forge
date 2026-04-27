# Brief — GC-Forge

**Request:** Production of functional and technical specifications.
**Recipient:** Cowork.
**Author:** Jérôme.
**Date:** 25 April 2026.

> **Editorial note (2026-04-27).** This brief is the founding
> document of the GC-Forge project. It is preserved here as a
> historical record. The English translation below is faithful to
> the original French text; subsequent decisions (collector and
> preset scope, in particular) are recorded in the specifications
> under [`doc/specs/`](specs/) rather than amended here.

---

## 1. Context

I am developing **GC-Insight**, a SaaS platform for analysing Java
GC logs (initial target: G1 GC on JDK 17+, AWS serverless
architecture, Rust core with a shared `gc-core` crate, design
driven by YAML/regex pattern definitions).

To make GC-Insight live — testing it, demonstrating it, training
it, and illustrating its analytical capabilities — I need a
**rich, reproducible, and taxonomised corpus of GC logs**.
Producing such logs by hand is painful: it requires maintaining a
fleet of JVMs, writing representative workloads, and capturing
the output cleanly.

Hence **GC-Forge**, the generator counterpart to GC-Insight: where
Insight observes, Forge fabricates.

## 2. Vision and objectives

GC-Forge must allow, from a simple declarative description, the
**production of GC logs representative of chosen scenarios on
demand**, crossing three axes:

1. **JVM** (HotSpot/OpenJDK, GraalVM, Eclipse OpenJ9, Azul
   Zing/Prime, Amazon Corretto, etc.).
2. **GC algorithm** (G1, ZGC, Shenandoah, Parallel, Serial, and
   legacy CMS for historical cases).
3. **Application regime**: high-frequency transactional, batch-like
   compute, cache management (long-lived allocations), mixed
   workloads, microservice "stop-and-go", etc.

Each dimension must be combinable, and each combination must be
modulable to expose **different levels of GC pressure** (from
"all good" to "pathological GC", with the intermediate regimes
worth analysing in between).

## 3. Priority use cases

The specifications must size the project to cover at least the
following cases, by order of priority:

1. **Testing and validating GC-Insight**: produce reference logs
   (with known ground truth) to validate analyses, detect
   regressions, and benchmark the precision of heuristics.
2. **Sales demonstrations**: have a ready-to-use catalogue of
   "speaking" logs to illustrate every GC-Insight capability
   (humongous allocations, evacuation failures, pathological mixed
   GC, fragmentation, etc.).
3. **Pedagogy and content**: generate logs for blog articles,
   tutorials, training material — each log accompanied by its
   "identity card" (configuration, regime, expected phenomenon).
4. **Bug / pathology reproduction**: enable a GC-Insight user to
   mirror a GC behaviour observed in their environment, in order
   to diagnose it.
5. **Datasets for ML / heuristics**: eventually feed potential
   classification or anomaly-detection models.

## 4. Functional scope to specify

Cowork is expected to deliver on the following items:

- **Catalogue of supported JVMs at launch** (and extension
  strategy), with target versions.
- **Catalogue of supported GC algorithms**, and per-JVM
  compatibility.
- **Catalogue of application regimes**: for each, define the
  expected behavioural signature (allocation rate, object
  lifetimes, object sizes, temporal pattern, young/old ratio,
  etc.).
- **Scenario model**: what is the unit of description of a
  GC-Forge job? (Proposal to challenge: a declarative YAML file
  describing `{JVM, algorithm, heap config, regime, duration,
  variability parameters, seed}`.)
- **Per-scenario tunable parameters**: heap size, generational
  ratios, thresholds, target memory pressure, etc.
- **Output formats**: JVM-native raw log (top priority — identical
  to what a real production application would produce), plus
  structured metadata (scenario identity card in JSON/YAML), plus
  optionally exports to other formats (JFR, etc.).
- **Catalogue of pre-packaged scenarios** ("presets"): at least an
  initial set covering the most pedagogical cases (healthy
  steady-state, slow memory leak, allocation burst, humongous,
  old-gen fragmentation, etc.).
- **Batch mode**: produce a complete corpus (matrix JVM × algo ×
  regime) in a single run.
- **Reproducibility**: mandatory seed, same configuration ⇒ same
  log byte-for-byte.

## 5. Technical scope to specify

Structural decisions Cowork must investigate and settle (with a
reasoned recommendation):

- **Fundamental approach**: (a) real JVM execution with generated
  Java workloads vs (b) pure log synthesis from a statistical
  model, vs (c) hybrid approach. My intuition strongly leans
  towards (a) at least for the MVP — fidelity is non-negotiable —
  but is open to challenge.
- **Proposed architecture**, consistent with the GC-Insight
  ecosystem:
  - Rust workspace extending the shared `gc-core` crate.
  - Java component for the workloads (one parameterised harness
    rather than N distinct projects).
  - Orchestration: a Rust CLI that drives JVM launches and
    captures the logs.
- **JVM fleet management**: on-demand download? Pre-built Docker
  containers? SDKMAN-like?
- **Packaging and distribution**: cross-platform CLI binary
  (Linux/macOS prioritised), Docker images, possibly a service
  mode.
- **Bit-for-bit reproducibility**: strategy to manage sources of
  non-determinism (real GC timing, JVM seeds, system load).
- **Scenario versioning**: a scenario must produce an identical
  log across versions, or explicitly signal the change.
- **CI integration**: enable regenerating the GC-Insight reference
  corpus from CI.

## 6. Cross-cutting constraints and requirements

- **Consistency with GC-Insight**: same core language (Rust),
  declarative YAML philosophy, ability to share structures via
  `gc-core`.
- **Multi-OS**: Linux for CI/server, macOS for local development
  (my workstation).
- **Open core or proprietary?** to investigate — GC-Forge could be
  open-source as a fixtures-generation tool (marketing leverage
  for GC-Insight), or remain an internal asset. Recommendation
  expected.
- **Controlled execution cost**: generating a complete corpus must
  not require a cluster.
- **No strong dependency on AWS**: unlike GC-Insight, GC-Forge
  must run locally and in standard CI.

## 7. Key questions to settle in the specifications

Cowork must explicitly rule (with rationale) on:

1. Real vs synthetic approach — MVP choice and trajectory.
2. Scenario description model (proposed YAML schema).
3. Target list of regimes for the MVP (propose 5 to 8 priority
   regimes, justify).
4. Target list of JVMs/algorithms for the MVP (recommend a
   realistic subset vs the long-term target).
5. JVM fleet management strategy (Docker vs native vs hybrid).
6. Open-source vs proprietary model.
7. Metadata format accompanying each log (schema).
8. Quality criteria for a generated log (how to validate that a
   "high-frequency transactional" scenario actually produces what
   is expected?).

## 8. Expected deliverables

1. **Functional specification document** (detailed use cases,
   scenario model, target catalogue of regimes / JVMs /
   algorithms, input and output formats, MVP pre-packaged
   presets).
2. **Technical specification document** (architecture, reasoned
   technology choices, workspace structure, integration with
   GC-Insight, packaging strategy, test strategy).
3. **MVP → V1 roadmap**: what is delivered first? What is the
   progression?
4. **Initial structured backlog** (top-level epics and user
   stories).
5. **List of risks and open points** remaining to be settled
   after the specifications.

## 9. Specification success criteria

The specifications delivered by Cowork will be judged successful if:

- A Rust developer, by reading them alone, can understand the
  target architecture and start implementing the MVP without
  major ambiguity.
- The structural choices are explicit and argued (no "to be
  decided later" on the points listed in §7).
- Consistency with GC-Insight is demonstrated and quantified
  (reuse of `gc-core`, common conventions).
- The proposed MVP is deliverable with reasonable effort
  (target: 4 to 6 weeks of solo part-time development — to be
  confirmed by Cowork).
- The document supports building an internal pitch or one for
  third parties (clear value positioning).

---

**Note for Cowork**: do not hesitate to challenge the assumptions
of this brief — in particular on the real-vs-synthetic approach,
the MVP scope, and the open-source opportunity. A specification
that contradicts the brief with a better argument is preferable
to one that follows the brief literally.
