# GC-Forge — pitch

## Problem

Java GC logs are produced by every JVM in production, yet **building a known-good corpus** for testing analysers, demonstrating capabilities, training detectors, or reproducing a customer's pathology is painfully manual: you maintain a parc of JDKs, write workloads, capture stdout, and pray the result matches what you wanted to exhibit. The effort kills experimentation, drowns documentation projects, and turns "show me a leak" into a half-day yak shave.

## What GC-Forge does

GC-Forge takes a **declarative YAML scenario** — `(JVM, GC algorithm, application regime)` — and produces, on demand, a **real GC log** indistinguishable from one a production application would emit, plus a hash-anchored manifest documenting how it was produced. The CLI is one command:

```sh
gc-forge run presets/humongous-g1-classic.yaml
```

…and a 2-minute log of G1 under humongous pressure lands on disk, alongside a manifest with the resolved scenario, the JVM version, every flag passed, the seed, and (after `gc-forge validate`) a populated invariants block proving the log exhibits the expected fingerprint.

GC-Forge is **not** a synthetic generator. The log comes from a real Temurin JVM running a parameterised workload harness — fidelity is non-negotiable, by construction.

## Who is this for

- **Engineers building GC-log analysers** (notably GC-Insight): a reproducible corpus with ground truth replaces hand-curated fixtures.
- **Sales engineers and demonstrators**: the catalogue of 14 presets covers every regime that a GC-savvy customer cares about.
- **Educators and content authors**: cited logs in articles or training material now have a published source-of-truth.
- **ML practitioners** building classification or anomaly-detection models: the matrix runner produces labelled datasets at the corpus scale.

## What ships in 0.1.0

- **7 application regimes** (R1 steady-state, R2 burst, R3 humongous, R4 slow-leak, R5 cache-churn, R6 mixed-pathological, R7 microservice) on the **3 MVP collectors** (G1, generational ZGC, Parallel) running on **Temurin 17 and 21**.
- **14 presets** shipped inside the binary; `cargo install gc-forge-cli` is self-sufficient.
- **`gc-forge run | validate | batch | lint | presets | selftest | variance-check`** — every documented surface from SPEC-FONCTIONNELLE §9.
- **JSON Schemas** for `gc-forge/scenario.v1` and `gc-forge/run-manifest.v1` published alongside the binary.
- **Apache-style permissive licensing** (MIT) and an open-source repo, enabling outside contributions and use in commercial pipelines.

## What's next

- **V1** broadens the JVM matrix (Corretto, OpenJ9, Shenandoah/Serial), ships a native runner that downloads Adoptium on demand, adds a `mirror` subcommand that proposes scenarios from a customer's prod log, and packages an HTML variance-check report.
- **V2** introduces synthetic-hybrid generation calibrated on real traces (for ML datasets at scale), and pursues Zing/Prime under licence.

The 0.1.0 line is the **fidelity-first MVP**: the corpus you can trust because nothing in the pipeline lies.
