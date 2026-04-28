# GC-Forge

*A declarative, reproducible generator of Java garbage-collection logs.*

[![CI](https://github.com/be-next/GC-Forge/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/be-next/GC-Forge/actions/workflows/ci.yml)
[![Release](https://github.com/be-next/GC-Forge/actions/workflows/release.yml/badge.svg)](https://github.com/be-next/GC-Forge/actions/workflows/release.yml)
[![Latest tag](https://img.shields.io/github/v/tag/be-next/GC-Forge?label=release&sort=semver&color=blue)](https://github.com/be-next/GC-Forge/releases)
[![Licence: MIT](https://img.shields.io/github/license/be-next/GC-Forge?color=informational)](LICENSE)
[![Rust 1.94+](https://img.shields.io/badge/rust-1.94%2B-orange?logo=rust)](rust-toolchain.toml)
[![Temurin 17 / 21](https://img.shields.io/badge/Temurin-17%20%7C%2021-success?logo=eclipseadoptium)](https://adoptium.net/)
[![Docker multi-arch](https://img.shields.io/badge/docker-amd64%20%7C%20arm64-blue?logo=docker)](docker/jdk21/Dockerfile)
[![Documentation](https://img.shields.io/badge/docs-getting--started-success)](doc/user/getting-started.md)

## What it does

From a typed YAML scenario describing a `(JVM, GC algorithm,
application regime)` tuple, GC-Forge runs a parameterised
workload on a real Java Virtual Machine, captures the unified
`-Xlog:gc*` output, and emits a hash-anchored manifest sufficient
to reproduce the run on another host.

GC-Forge is the counterpart of *GC-Insight*: where Insight
analyses GC logs, Forge produces them on demand with documented
ground truth. The contract surface between the two projects is
documented in
[`doc/concepts/traceability.md`](doc/concepts/traceability.md).

GC-Forge is **not** a benchmarking tool. It does not measure
application throughput or latency; it exhibits the GC behaviour
of a workload, and only that. Fidelity is established by
construction (real Temurin JVM, parameterised harness), not by a
statistical model.

## Status

Version `0.2.1` — extended MVP release.

| Item                | Coverage                                                                                                       |
|---------------------|----------------------------------------------------------------------------------------------------------------|
| Application regimes | 7 (steady-state, burst, humongous, slow-leak, cache-churn, mixed-GC pathological, microservice stop-and-go)    |
| GC collectors       | 6 (G1, ZGC generational and non-generational, Parallel, Shenandoah, Serial, Epsilon)                           |
| JVM distributions   | Eclipse Temurin 17 and 21                                                                                      |
| Shipped presets     | 21 (embedded in the binary)                                                                                    |
| CLI subcommands     | `lint`, `run`, `validate`, `batch`, `presets`, `selftest`, `variance-check`                                    |
| Wire formats        | `gc-forge/scenario.v1`, `gc-forge/run-manifest.v1`, `gc-forge/matrix.v1` (JSON Schema draft 2020-12)            |
| Runner              | Docker (native runner planned for V1)                                                                          |
| Licence             | [MIT](LICENSE)                                                                                                 |

A summary of the rationale behind these choices is given in
[`doc/concepts/overview.md`](doc/concepts/overview.md).

## What you get

A single `gc-forge run` invocation produces two files: the raw
GC log (verbatim Temurin output) and a manifest that documents
how it was produced.

```sh
gc-forge run presets/steady-shenandoah-baseline.yaml \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

**The log** — a verbatim `-Xlog:gc*` capture, byte-identical to
what the same JVM would write in production:

```
[2026-04-27T06:59:21.901+0000][1][7][info][gc] Using Shenandoah
[2026-04-27T06:59:21.901+0000][1][7][info][gc,init] Heap Min Capacity: 2G
[2026-04-27T06:59:21.901+0000][1][7][info][gc,init] Heap Initial Capacity: 2G
[2026-04-27T06:59:21.901+0000][1][7][info][gc,init] Heap Max Capacity: 2G
[2026-04-27T06:59:23.184+0000][1][9][info][gc      ] GC(0) Concurrent reset
[2026-04-27T06:59:23.187+0000][1][9][info][gc      ] GC(0) Pause Init Mark 0.512ms
[2026-04-27T06:59:23.193+0000][1][9][info][gc      ] GC(0) Concurrent marking
[2026-04-27T06:59:23.231+0000][1][9][info][gc      ] GC(0) Pause Final Mark 0.398ms
…
```

**The manifest** — a YAML identity card sufficient to reproduce
the run elsewhere:

```yaml
apiVersion: gc-forge/run-manifest.v1
kind: RunManifest
run:
  id: 019dc617-5cd3-7272-9ddc-48b2c9b96e31
  started_at: 2026-04-27T06:59:21.842Z
  ended_at:   2026-04-27T07:00:51.108Z
  duration_actual: 90s
  exit_status: { kind: success }
  host: { os: linux, arch: aarch64, cpu_count: 16, container: docker:gc-forge-runner:dev-jdk21 }
scenario:
  source_path:   presets/steady-shenandoah-baseline.yaml
  source_sha256: bf39b4e9272cb0bde6e390b28e472f4270d3071b7c3c007b94d133150e6f4476
  resolved:      { … fully merged scenario after extends/overrides … }
jvm:
  vendor:  temurin
  version: 21.0.8+9
  flags:   [-Xms2g, -Xmx2g, -XX:+UseShenandoahGC, …]
reproducibility:
  seed:                 0xC0FFEE
  workload_jar_sha256:  6e9c3cdf4f9c…
  gc_forge_version:     0.2.1+93f707e
output:
  log_path:        out/steady-shenandoah-baseline-c0ffee.log
  log_sha256:      a2dca41f7c5e…
  log_size_bytes:  220487
expected_phenomena:  [young_gc_steady]
expected_invariants: [{ rule: "full_count == 0", threshold: 0 }, …]
validation:
  status: passed
  results: [ … per-rule report … ]
```

Every flag, hash, and host attribute is recorded; two operators
running the same preset with the same Docker image obtain
manifests that differ only by run UUID and wall-clock
timestamps.

## Quickstart

```sh
# Build the CLI and the runner image once.
make build
make docker-image

# Lint, then run a shipped preset.
gc-forge lint presets/steady-g1-baseline.yaml
gc-forge run  presets/steady-g1-baseline.yaml \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar

# Re-check the produced log against its manifest.
gc-forge validate out/steady-g1-baseline-c0ffee.log \
    --manifest  out/steady-g1-baseline-c0ffee.manifest.yaml
```

A guided walk-through of the same path, with troubleshooting
notes, is in
[`doc/user/getting-started.md`](doc/user/getting-started.md).

## Use cases

GC-Forge is designed for four populations.

| You are…                                  | You get…                                                                                            |
|-------------------------------------------|-----------------------------------------------------------------------------------------------------|
| Building a GC-log analyser                | A reproducible corpus with machine-checkable ground truth replacing hand-curated fixtures.          |
| Demonstrating GC behaviour to a customer  | A 21-preset catalogue covering every regime that comes up in a sales conversation.                  |
| Writing an article or training material   | A cited, reproducible source for every log embedded in your content.                                |
| Building an ML dataset of labelled traces | The `gc-forge batch` matrix runner produces logs in bulk, each labelled by the regime that produced it. |

## Examples

A few commands worth knowing, beyond the quickstart:

**Run all 21 presets and check their invariants** (full pass
under three minutes by default):

```sh
gc-forge selftest \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

**Sweep a preset across collectors and seeds** (cartesian
product, with a CSV index of every cell):

```yaml
# matrix.yaml
apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: presets/steady-g1-baseline.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC, Parallel, Shenandoah]
  seeds: [1, 2, 3]
```

```sh
gc-forge batch matrix.yaml --out-dir out/sweep \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

**Quantify inter-run variance** (fails non-zero if budgets are
exceeded):

```sh
gc-forge variance-check steady-g1-baseline --runs 5 \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

**Tweak a preset on the fly without copying the YAML**:

```sh
gc-forge run presets/humongous-g1-classic.yaml \
    --override 'spec.gc.options.heap.max=4g' \
    --override 'spec.regime.parameters.humongous_ratio=0.7' \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar
```

## Installation

For 0.2.1 the supported install path is from source. Prebuilt
Linux/macOS binaries (x86_64 and aarch64) and a Homebrew tap are
scheduled for a later release.

```sh
git clone https://github.com/be-next/GC-Forge.git
cd GC-Forge
make build              # cargo build --release + Maven shade of the harness
make docker-image       # eclipse-temurin:21-jdk-jammy + harness embedded
```

`make build` produces `target/release/gc-forge`. Add it to your
`$PATH` or invoke it directly. To target Temurin 17 instead of
21, build the alternate image with `make docker-image
JDK_MAJOR=17` and pass `--image gc-forge-runner:dev-jdk17`.

Prerequisites: a POSIX shell, Docker Engine or Docker Desktop,
Rust 1.94+, Maven 3.9+, JDK 17+.

## Documentation

The documentation is organised by audience.

### For users

| Document                                                                              | Purpose                                                            |
|---------------------------------------------------------------------------------------|--------------------------------------------------------------------|
| [`getting-started.md`](doc/user/getting-started.md)                                   | Install, run a first preset, read the manifest, troubleshoot.      |
| [`regimes.md`](doc/user/regimes.md)                                                   | Seven regimes, parameters, expected signatures, shipped presets.   |
| [`scenario-reference.md`](doc/user/scenario-reference.md)                             | YAML schema, `extends`, `--override` syntax.                       |
| [`cli-reference.md`](doc/user/cli-reference.md)                                       | Every subcommand, flags, defaults, exit codes.                     |

### For contributors and integrators

| Document                                                            | Purpose                                                |
|---------------------------------------------------------------------|--------------------------------------------------------|
| [`doc/architecture.md`](doc/architecture.md)                        | Single-page system architecture.                       |
| [`doc/concepts/overview.md`](doc/concepts/overview.md)              | Design goals and non-goals.                            |
| [`doc/concepts/traceability.md`](doc/concepts/traceability.md)      | Phenomenon × preset × analyser-detector matrix.        |
| [`doc/process/orchestration.md`](doc/process/orchestration.md)      | Development process, role rotation, DoD gate.          |
| [`CONTRIBUTING.md`](CONTRIBUTING.md)                                | Contribution conventions.                              |
| [`CHANGELOG.md`](CHANGELOG.md)                                      | Release history.                                       |

### Internal product specifications

These framing documents capture the structural decisions and
remain authoritative for future evolution. Day-to-day use of
GC-Forge does not require reading them.

- [`doc/specs/SPEC-FUNCTIONAL.md`](doc/specs/SPEC-FUNCTIONAL.md) — functional specification.
- [`doc/specs/SPEC-TECHNICAL.md`](doc/specs/SPEC-TECHNICAL.md) — technical specification.
- [`doc/specs/ROADMAP.md`](doc/specs/ROADMAP.md) — phased delivery plan.
- [`doc/specs/BACKLOG.md`](doc/specs/BACKLOG.md) — epics and user stories.
- [`doc/specs/RISKS.md`](doc/specs/RISKS.md) — risk register and open points.

## Architecture, in brief

GC-Forge consists of a Rust workspace that compiles to a single
`gc-forge` binary, and a Java workload harness packaged as a
shaded JAR. The two artefacts communicate exclusively through
the JVM command line: GC-Forge produces an `argv` for `java`,
the JVM emits a unified `-Xlog:gc*` log to a file, and GC-Forge
reads the file back. There is no embedded protocol.

The Rust workspace is split into six crates with an acyclic
dependency graph:

```
scenario  ←  regimes  ←  runner  ←  validate  ←  presets  ←  cli
```

A full description, including the runner subsystem and the wire
formats, is given in [`doc/architecture.md`](doc/architecture.md).

## Citing GC-Forge

If GC-Forge contributes to a published work, please cite it as:

> Ramette, J. (2026). *GC-Forge: a declarative generator of Java
> garbage-collection logs* (Version 0.2.1). MIT licence.
> Available at https://github.com/be-next/GC-Forge.

A versioned BibTeX entry will be added once a DOI is assigned.

## Contributing

Contributions are welcome under the project's MIT licence; see
[`CONTRIBUTING.md`](CONTRIBUTING.md) for the conventions used
by the project, including the documentation language policy
(English throughout), the Definition-of-Done gate, and the
branching model.

Bug reports are tracked in GitHub Issues. A bug report is most
useful when accompanied by the relevant section of the manifest
produced by the failing run.

## Licence

[MIT](LICENSE).
