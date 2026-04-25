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

_TODO iter 5 — once `gc-forge run` is wired up, walk the reader through the steady-state baseline preset and the resulting log + manifest._

## Reading the manifest

_TODO iter 5 — show a sample `gc-forge/run-manifest.v1` and explain each field._

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
