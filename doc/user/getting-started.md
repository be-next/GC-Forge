# Getting started with GC-Forge

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
- For local development of GC-Forge itself: Rust 1.94+, Maven 3.9+, JDK 17+.

A native runner that downloads JDKs on demand will land in V1 (see ROADMAP).

## Install

For 0.1.0 the supported install path is from source:

```sh
git clone https://github.com/<org>/gc-forge.git
cd gc-forge
make build              # cargo build --release + Maven shade of the harness
make docker-image       # eclipse-temurin:21-jdk-jammy + harness embedded
```

`make build` produces `target/release/gc-forge`. Add it to `$PATH` or
invoke it directly. A `cargo install gc-forge-cli` path, prebuilt
Linux/macOS binaries and a Homebrew tap will be wired up at release
time (see `doc/specs/ROADMAP.md` Phase 3 / Phase 4).

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

The same flags work against the two sibling baselines —
`presets/steady-zgc-baseline.yaml` (generational ZGC) and
`presets/steady-parallel-baseline.yaml` (Parallel collector). All three share
their regime parameters via `extends: steady-g1-baseline.yaml`.

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

For exploratory work, write a small **matrix YAML** and let
`gc-forge batch` walk the cartesian product of axes × seeds. Example:

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

Each cell lands at `out/baseline-sweep/cellN-<name>-<seed>.{log,manifest.yaml}`.
A summary CSV is written to `out/baseline-sweep/index.csv` with the
exit status, duration and validation status of every cell. Filters and
the full schema are documented in `doc/user/cli-reference.md`.

## Validating an existing log

The `run` and `batch` commands write expected invariants into each
manifest (the regime's ground truth, copied from the scenario's
`spec.expected.invariants`). Re-check a log later with:

```sh
gc-forge validate out/steady-g1-baseline-c0ffee.log \
    --manifest out/steady-g1-baseline-c0ffee.manifest.yaml
```

The validator parses the GC log, evaluates each rule, and reports
`Passed`, `Failed` or `Skipped`. Unknown rules are skipped with a clear
note — they never fail the run, which keeps future invariants
backward-compatible. Add `--update-manifest` to persist the result
back into the manifest's `validation` block.

The full list of recognised rule shapes lives in
`doc/user/cli-reference.md#gc-forge-validate`.

## Next steps

- The full list of regimes and their parameters: [`regimes.md`](regimes.md).
- The scenario YAML reference: [`scenario-reference.md`](scenario-reference.md).
- The CLI reference: [`cli-reference.md`](cli-reference.md).
- The qualification commands (`gc-forge presets`, `selftest`,
  `variance-check`) are documented in the CLI reference.

## Troubleshooting

- **`Could not find or load main class java`** — your Docker image already
  has `java` as `ENTRYPOINT`. GC-Forge defends against this by always
  passing `--entrypoint=java` itself; if you build a custom image, do
  not override the entrypoint to something that re-prepends `java`.
- **Mount errors on macOS Docker Desktop with `/tmp` paths** — Docker
  Desktop on macOS does not auto-share `/tmp`. Use `--out-dir` under
  `$HOME` instead, or pass `--embedded-harness` so the host JAR mount
  is skipped entirely.
- **`Could not create the Java Virtual Machine`** with very small
  heaps — the harness keeps a small allocation budget but the JVM
  itself reserves overhead. Bump `spec.jvm.heap.max` to at least
  `256m` (the presets ship with sane minimums).
- **Stale image after editing the harness** — `make docker-image`
  does not currently track changes under `workload-harness/`. Run
  `make build && make docker-image` after touching Java sources.
- **`gc-forge selftest` fails on `humongous-g1-evac-fail`** in
  truncated CI runs — that preset's invariants assume a faithful
  duration; use `--per-preset-duration 60s` (or run it on its own).
