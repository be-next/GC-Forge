# Getting started with GC-Forge

GC-Forge produces real Java garbage-collection (GC) logs from
declarative scenarios. From a YAML description of a `(JVM, GC
algorithm, application regime)` tuple, the tool runs a real JVM,
executes a parameterised workload, and captures the native GC log
together with a reproducible manifest.

This document walks through the first run, the structure of the
manifest, the validation step, batch execution, and the most
common environmental issues. The complete reference for each
subcommand is in [`cli-reference.md`](cli-reference.md).

## Audiences

GC-Forge is designed for four user populations:

- engineers validating GC-log analysers (notably GC-Insight) against
  ground truth;
- sales engineers and demonstrators looking for a catalogue of
  speaking GC logs;
- educators and content authors needing reproducible examples;
- machine-learning practitioners building datasets of labelled GC
  traces.

GC-Forge is *not* a benchmarking tool: it does not measure
application throughput or latency. It exhibits the GC behaviour of a
workload, and only that.

## Prerequisites

- A POSIX shell on Linux or macOS.
- Docker Engine or Docker Desktop on the local machine. The MVP
  runs all JVMs inside Docker for reproducibility (see
  [`SPEC-TECHNICAL`](../specs/SPEC-TECHNICAL.md) §5).
- For local development of GC-Forge itself: Rust 1.94 or later,
  Maven 3.9 or later, JDK 17 or later.

A native runner that downloads JDK distributions on demand is
planned for V1 and is described in
[`ROADMAP`](../specs/ROADMAP.md).

## Installation

The supported installation path for 0.1.0 is from source:

```sh
git clone https://github.com/<org>/gc-forge.git
cd gc-forge
make build              # cargo build --release + Maven shade of the harness
make docker-image       # eclipse-temurin:21-jdk-jammy + harness embedded
```

`make build` produces `target/release/gc-forge`. The binary may be
added to `$PATH` or invoked directly. Prebuilt binaries for Linux
and macOS, and a Homebrew tap, are scheduled for a later release
(see [`ROADMAP`](../specs/ROADMAP.md), Phases 3 and 4).

## A first scenario

The simplest path is to run the `steady-g1-baseline` preset shipped
at `presets/steady-g1-baseline.yaml`:

```sh
# Validate the preset without launching the JVM.
gc-forge lint presets/steady-g1-baseline.yaml

# Run it. The 90-second G1 scenario writes a log and a manifest under ./out.
gc-forge run presets/steady-g1-baseline.yaml \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

The same flags work against the two sibling baselines —
`presets/steady-zgc-baseline.yaml` (generational ZGC) and
`presets/steady-parallel-baseline.yaml` (Parallel collector). All
three share their regime parameters via
`extends: steady-g1-baseline.yaml`.

After approximately 90 seconds the output directory contains two
files:

```
out/
├── steady-g1-baseline-c0ffee.log              # raw GC log
└── steady-g1-baseline-c0ffee.manifest.yaml    # identity card
```

The CLI prints both paths on completion. The log is a verbatim
`-Xlog:gc*` capture from Temurin 21; tools that consume JVM GC
logs (such as GC-Insight) read it as-is.

## Reading the manifest

The manifest is a YAML document conforming to the schema at
[`schemas/run-manifest-v1.json`](../../schemas/run-manifest-v1.json).
Its top-level fields are summarised below.

| Field             | Content                                                                                   |
|-------------------|-------------------------------------------------------------------------------------------|
| `run`             | UUID v7 of the run, wall-clock window, exit status, host description (OS, architecture, container indication). |
| `scenario`        | Source path, SHA-256 of the source file, and the fully resolved scenario after `extends:` and overrides. |
| `jvm`             | Vendor, the version string captured from `java -version`, and the exact list of JVM flags passed. |
| `reproducibility` | Seed (in hexadecimal), SHA-256 of the harness JAR, GC-Forge version. |
| `output`          | Path of the GC log, its SHA-256 and size in bytes.                                        |
| `expected_phenomena` / `expected_invariants` | Ground truth from the regime, used by `gc-forge validate`. |
| `validation`      | Status (`skipped` until validation runs), per-rule results, validator version.            |

Because every flag, hash, and host attribute is recorded, two
operators running the same preset on the same machine should land on
the same manifest modulo the run UUID and the start/end timestamps.

The validation block remains `skipped` until
`gc-forge validate <log> --manifest <manifest.yaml>` is run.

## Validating an existing log

The `run` and `batch` subcommands write the regime's expected
invariants into each manifest, copied from the scenario's
`spec.expected.invariants` block. A log can be re-checked at any
time:

```sh
gc-forge validate out/steady-g1-baseline-c0ffee.log \
    --manifest out/steady-g1-baseline-c0ffee.manifest.yaml
```

The validator parses the GC log, evaluates each rule, and reports
`Passed`, `Failed`, or `Skipped`. Unknown rules are skipped with a
clear note; they never fail the run, which keeps future invariants
backward-compatible. Adding `--update-manifest` persists the result
back into the manifest's `validation` block.

The full list of recognised rule shapes is given in
[`cli-reference.md`](cli-reference.md#gc-forge-validate).

## Running a batch

For exploratory work, a small **matrix** YAML and `gc-forge batch`
walk the cartesian product of axes and seeds. The matrix document
follows the `gc-forge/matrix.v1` schema, committed at
[`schemas/matrix-v1.json`](../../schemas/matrix-v1.json). For
example:

```yaml
apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: presets/steady-g1-baseline.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC, Parallel]
  seeds: [1, 2, 3]
```

```sh
gc-forge batch matrix.yaml --out-dir out/baseline-sweep \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

Each cell is written to
`out/baseline-sweep/cellN-<name>-<seed>.{log,manifest.yaml}`. A
summary is recorded in `out/baseline-sweep/index.csv` listing the
exit status, duration, and validation status of every cell. Filters
and the full schema are documented in
[`cli-reference.md`](cli-reference.md).

## Next steps

- The full list of regimes and their parameters:
  [`regimes.md`](regimes.md).
- The scenario YAML reference:
  [`scenario-reference.md`](scenario-reference.md).
- The CLI reference: [`cli-reference.md`](cli-reference.md).
- The qualification commands (`gc-forge presets`, `selftest`,
  `variance-check`) are described in the CLI reference.

## Troubleshooting

The following issues recur often enough to be listed explicitly.

- **`Could not find or load main class java`** — the Docker image
  used as a runner already has `java` configured as `ENTRYPOINT`.
  GC-Forge guards against this by passing `--entrypoint=java`
  itself; if a custom image is built, do not override the entry
  point with a wrapper that prepends `java`.
- **Mount errors on macOS Docker Desktop with `/tmp` paths** —
  Docker Desktop on macOS does not auto-share `/tmp`. Use
  `--out-dir` under `$HOME` instead, or pass `--embedded-harness`
  so that the host JAR mount is skipped entirely.
- **`Could not create the Java Virtual Machine`** with very small
  heaps — the harness keeps a small allocation budget, but the
  JVM itself reserves overhead. Setting `spec.jvm.heap.max` to at
  least `256m` resolves the case (the shipped presets ship with
  sensible minimums).
- **Stale image after editing the harness** — `make docker-image`
  does not currently track changes under `workload-harness/`. After
  a Java source change, run `make build && make docker-image`.
- **`gc-forge selftest` failure on `humongous-g1-evac-fail`** in
  truncated continuous-integration runs — the preset's invariants
  assume a faithful duration. Set `--per-preset-duration 60s` or
  run the preset on its own to obtain a meaningful result.
