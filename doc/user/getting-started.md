# Getting started with GC-Forge

> **Status:** skeleton — content is filled iteration by iteration. Sections marked `_TODO iter N_` will be authored when the corresponding feature lands.

GC-Forge produces real Java GC logs from declarative scenarios. From a YAML description of a `(JVM, GC algorithm, application regime)`, it spins up a real JVM, runs a parameterised workload, and captures the native GC log along with a reproducible identity card.

## Overview

GC-Forge is meant for four primary audiences:

- engineers validating GC log analysers (notably GC-Insight) against ground truth;
- sales engineers and demonstrators looking for a catalogue of speaking GC logs;
- educators and content authors needing reproducible examples;
- ML practitioners building datasets of labelled GC traces.

It is _not_ a benchmarking tool: it does not measure application throughput or latency. It exhibits the GC behaviour of a workload, and only that.

## Prerequisites

- A POSIX shell (Linux or macOS).
- Docker Engine or Docker Desktop on the local machine. The MVP runs all JVMs inside Docker for reproducibility (see `doc/specs/SPEC-TECHNIQUE.md` §5).
- For local development of GC-Forge itself: Rust 1.83+, Maven 3.9+, JDK 17+.

A native runner that downloads JDKs on demand will land in V1 (see ROADMAP).

## Install

_TODO iter 17 — describe `cargo install gc-forge-cli`, the Homebrew tap (V1), and the `docker run …` one-liner._

## First scenario

The simplest path is to run the `steady-g1-baseline` preset shipped at
`presets/steady-g1-baseline.yaml`:

```sh
# Build the harness fat-jar and the runner Docker image once.
make build
make docker-image

# Validate the preset without launching the JVM.
gc-forge lint presets/steady-g1-baseline.yaml

# Run it. The 90-second G1 scenario writes a log + manifest under ./out.
gc-forge run presets/steady-g1-baseline.yaml \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

After ~90 seconds you should see two files:

```
out/
├── steady-g1-baseline-c0ffee.log              # raw GC log, like a prod app
└── steady-g1-baseline-c0ffee.manifest.yaml    # identity card
```

The CLI prints both paths on completion. The log is a verbatim
`-Xlog:gc*` capture from Temurin 21; tools that read JVM GC logs (such
as GC-Insight) consume it as-is.

## Reading the manifest

The manifest is a YAML document conforming to the schema at
[`schemas/run-manifest-v1.json`](../../schemas/run-manifest-v1.json).
Top-level fields:

| Field             | What it carries |
|-------------------|-----------------|
| `run`             | UUID v7 of the run, wall-clock window, exit status, host description (OS, arch, container). |
| `scenario`        | Source path, SHA-256 of the source file, and the **fully resolved** scenario after `extends:` and overrides. |
| `jvm`             | Vendor, the version string captured from `java -version`, and the exact list of JVM flags passed to the `java` invocation. |
| `reproducibility` | Seed (hex), SHA-256 of the harness JAR, GC-Forge version. |
| `output`          | Path of the GC log, its SHA-256 and size in bytes. |
| `expected_phenomena` / `expected_invariants` | Ground truth from the regime, used by `gc-forge validate` (lands in iter 13). |
| `validation`      | Status (`skipped` until validation runs), per-rule results, validator version. |

Because every flag, hash and host attribute is recorded, two operators
running the same preset on the same machine should land on the same
manifest modulo the run UUID and the start/end timestamps.

The validation block stays `skipped` until you run
`gc-forge validate <log> --manifest <manifest.yaml>` (iteration 13).

## Running a batch

_TODO iter 14 — describe `gc-forge batch matrix.yaml` and how the index is produced._

## Validating an existing log

_TODO iter 13 — describe `gc-forge validate <log> --manifest <manifest.yaml>` and what each invariant checks._

## Next steps

- The full list of regimes and their parameters: `doc/user/regimes.md` _(iter 16)_.
- The scenario YAML reference: `doc/user/scenario-reference.md` _(iter 16)_.
- The CLI reference: `doc/user/cli-reference.md` _(iter 16)_.

## Troubleshooting

_TODO iter 16 — collect FAQs and the most common errors as they emerge during development._
