# GC-Forge — internal specifications

Internal product specifications for GC-Forge, the declarative
generator of Java GC logs that pairs with GC-Insight.

**Status.** Originally produced on 2026-04-25 as a French set of
framing documents. Translated to English and resynchronised with
the implemented MVP on 2026-04-27. The specs are authoritative for
future evolution and are kept in sync with the code.

## Documents

| # | Document | Topic |
|---|----------|-------|
| 1 | [SPEC-FUNCTIONAL.md](./SPEC-FUNCTIONAL.md) | Use cases, YAML scenario model, catalogue of regimes / JVMs / collectors, input and output formats, shipped presets. |
| 2 | [SPEC-TECHNICAL.md](./SPEC-TECHNICAL.md) | Architecture, technology choices, Rust workspace, Java harness, JVM management, packaging, test strategy, reproducibility, CI. |
| 3 | [ROADMAP.md](./ROADMAP.md) | MVP → V1 → V2 trajectory, milestones, exit criteria, dependencies. |
| 4 | [BACKLOG.md](./BACKLOG.md) | Top-level epics and user stories, MVP prioritisation. |
| 5 | [RISKS.md](./RISKS.md) | Risk register (technical, project, legal, product) and open points. |

## Synthesis of structural decisions

The structural choices fixed during the specification phase
(see brief §7), all delivered in the 0.1.0 MVP:

| # | Question | Decision | Reference |
|---|----------|----------|-----------|
| 1 | Real workload vs synthetic generation | **Real JVMs at MVP** (parameterised workload harness on a real JVM). Synthetic-hybrid generation deferred to V2. | TECH §3 |
| 2 | Scenario model | **Declarative YAML** (`gc-forge/scenario.v1`), validated against a JSON Schema generated from the typed model. | FUNC §5 |
| 3 | MVP regimes | **Seven regimes**: `steady-state-healthy`, `allocation-burst`, `humongous-pressure`, `slow-leak`, `cache-churn`, `mixed-gc-pathological`, `microservice-stop-and-go`. | FUNC §4 |
| 4 | MVP JVMs and collectors | **Eclipse Temurin** JDK 17 and 21 only. **Six MVP collectors**: G1, ZGC (generational and non-generational), Parallel, Shenandoah, Serial, Epsilon. Corretto, GraalVM, and OpenJ9 deferred to V1. | FUNC §3 |
| 5 | JVM provisioning | **Docker at MVP** (`eclipse-temurin:{17,21}-jdk-jammy`), **native runner** via Adoptium download in V1. BYO-JVM deferred to V1+. | TECH §5 |
| 6 | Licence | **MIT** from MVP onwards. | TECH §9 |
| 7 | Manifest format | **YAML** by default (`gc-forge/run-manifest.v1`), JSON optional. | FUNC §6 |
| 8 | Quality gate | **Rules-as-code** per regime (quantified invariants), **`gc-forge selftest`** rerun at every release, **inter-run variance** ≤ 5 % targeted. | FUNC §7, TECH §7 |

## Note on collector scope

The original cadrage (2026-04-25) listed three MVP collectors —
G1, generational ZGC, Parallel — and deferred Shenandoah and
Serial to V1.1. The implementation as delivered in 0.1.0 promotes
Shenandoah, Serial, and the Epsilon (no-op) collector to the MVP
because they are all available in Temurin without parser
extension. The 0.1.0 MVP therefore covers six collectors instead
of three; OpenJ9 (which would require extending the GC-log
parser) remains a V1 item. ROADMAP §V1 has been updated
accordingly.

## Consistency with GC-Insight

GC-Forge reuses and extends the GC-Insight ecosystem:

- **Shared `gc-core` crate**: common types (GC model, normalised
  events, identifiers) — see TECHNICAL §2.
- **Same declarative YAML philosophy** as the GC-Insight pattern
  definitions.
- **Consistent CLI experience**: shared subcommand conventions,
  output formats, verbosity levels.

The contract surface is documented in
[`doc/concepts/traceability.md`](../concepts/traceability.md).

## MVP scope

**Target: extended MVP, 8–10 weeks solo part-time** (see ROADMAP §2).

Scope as delivered in 0.1.0: `gc-forge run <scenario.yaml>`
produces a real GC log on a Temurin 17 or 21 JVM under one of six
collectors, accompanied by a hash-anchored manifest, executed via
Docker, with twenty-one shipped presets and a matrix batch mode.

**Note on scope drift versus brief §9.** The original brief
targeted 4–6 weeks for an MVP. Cowork proposed 8–10 weeks (with
Phase 4 *native runner* optional) for an **extended scope**: 3
collectors (vs 1–2), 2 JDK versions (vs 1), 7 regimes (vs ~5),
14 presets, rich metadata from day one. The actual delivery
expanded that further during implementation: 6 collectors and 21
presets, with the additional collectors and presets adding
marginal effort because they reuse the same workload harness and
the same parser. The short 4–6-week trajectory remains
documented in ROADMAP §10 (*sensitivities*) for future
re-applicability.

## Spec acceptance criteria

Reference: brief §9. Self-checked after delivery:

- [x] A Rust developer understands the target architecture and
  starts implementing without major ambiguity. (Backed by the
  delivery of all 17 implementation iterations on the original
  decomposition.)
- [x] All points from brief §7 are settled and argued.
- [x] Consistency with GC-Insight is established (`gc-core`
  shared crate, common conventions, traceability matrix).
- [x] The proposed MVP is delivered in the 8–10-week solo
  part-time envelope.
- [x] The document supports a written pitch and an internal
  presentation (see [`doc/concepts/overview.md`](../concepts/overview.md)).
