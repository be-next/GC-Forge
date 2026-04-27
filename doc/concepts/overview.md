# GC-Forge: an overview

## Abstract

GC-Forge is a declarative generator of Java garbage-collection (GC)
logs. From a YAML scenario describing a tuple `(JVM, GC algorithm,
application regime)`, the tool runs a parameterised workload on a
real Java Virtual Machine, captures the unified `-Xlog:gc*` output,
and emits a hash-anchored manifest that documents the run for later
reproduction. The MVP release (0.1.0) covers seven application
regimes, three GC collectors (G1, generational ZGC, Parallel) on
Eclipse Temurin 17 and 21, and ships fourteen presets embedded in
the binary. GC-Forge is distributed under the MIT licence.

## Motivation

Building a known-good corpus of GC logs is a recurring need across
several activities: validating GC-log analysers, demonstrating
analyser capabilities to prospective users, training detectors for
specific phenomena, and reproducing customer-side pathologies. In
practice the work is performed manually — engineers maintain a set
of JDK installations, write ad-hoc workloads, capture standard
output, and compare the produced log against an informal
expectation. The procedure is labour-intensive, hard to reproduce
between operators, and provides no machine-checkable ground truth.

GC-Forge addresses these limitations by making the production of a
labelled GC log a first-class operation: the inputs are a typed YAML
document, the execution path is a single command, and the output is
both the raw log and a manifest that records every parameter
sufficient to reproduce the run on another host.

## What the tool does

A single invocation suffices to generate a log:

```sh
gc-forge run presets/humongous-g1-classic.yaml
```

The command resolves the scenario (including any `extends:`
inheritance), instantiates the requested regime, dispatches the
configured runner (Docker MVP), captures the GC log emitted by the
JVM, and writes a manifest alongside. The manifest contains the
fully resolved scenario, the JVM build identifier, every flag passed
to the `java` invocation, the seed used by the workload, the
SHA-256 of both the captured log and the harness JAR, and an
`expected_invariants` block populated from the regime's
declarative ground truth. After `gc-forge validate <log> --manifest
<manifest.yaml>` runs, the manifest also carries a `validation`
block reporting per-rule status.

GC-Forge is not a synthetic generator. The log is the verbatim
output of a Temurin JVM executing a parameterised workload harness;
fidelity is therefore established by construction rather than
asserted by the tool itself.

## Audiences

The tool is designed for four user populations.

- **Engineers building GC-log analysers.** A reproducible corpus
  with machine-checkable ground truth replaces hand-curated
  fixtures and provides regression coverage.
- **Sales engineers and demonstrators.** The catalogue of fourteen
  presets covers the regimes that customers typically ask about,
  with timings short enough for live demonstrations.
- **Educators and content authors.** A cited GC log in an article
  or a training exercise can reference a published, reproducible
  source.
- **Practitioners building machine-learning datasets** of labelled
  GC traces. The matrix runner produces logs in bulk, with each row
  labelled by the regime and parameters that produced it.

## Scope of the 0.1.0 release

The first public release covers:

- seven regimes — `R1 steady-state-healthy`, `R2 allocation-burst`,
  `R3 humongous-pressure`, `R4 slow-leak`, `R5 cache-churn`,
  `R6 mixed-gc-pathological`, `R7 microservice-stop-and-go`;
- three collectors — G1, generational ZGC, Parallel;
- two JVM majors — Eclipse Temurin 17 and 21;
- fourteen presets, embedded in the binary, covering the matrix
  above with named scenarios suitable for direct use;
- seven CLI subcommands — `lint`, `run`, `validate`, `batch`,
  `presets`, `selftest`, `variance-check`;
- three frozen wire formats with published JSON Schemas —
  `gc-forge/scenario.v1`, `gc-forge/run-manifest.v1`,
  `gc-forge/matrix.v1`;
- a Docker runner, with a native runner planned for V1.

## Outlook

Subsequent releases broaden coverage along three axes. V1 extends
the JVM matrix (Corretto, OpenJ9, Shenandoah, Serial), introduces a
native runner that downloads JDK distributions on demand, and adds
a `mirror` subcommand that proposes a scenario from a customer's
production log. V2 introduces a synthetic-hybrid generator
calibrated on real traces — useful when machine-learning datasets
need to scale beyond what real JVMs can produce in reasonable
time — and pursues licensed JVMs (Zing, Prime). A detailed plan is
maintained in [`doc/specs/ROADMAP.md`](../specs/ROADMAP.md) (in
French; the project's internal specifications are written in the
language of the team that authored them).

## References

- [`doc/architecture.md`](../architecture.md) — system architecture.
- [`doc/concepts/traceability.md`](traceability.md) — the
  phenomenon × preset × analyser-detector matrix.
- [`doc/specs/SPEC-FONCTIONNELLE.md`](../specs/SPEC-FONCTIONNELLE.md)
  — functional specification (FR).
- [`doc/specs/SPEC-TECHNIQUE.md`](../specs/SPEC-TECHNIQUE.md) —
  technical specification (FR).
