# System architecture

## Scope

This document describes the runtime and build-time architecture of
GC-Forge 0.1.0. It is intended for contributors and integrators who
need a single-page picture of how a `gc-forge run` invocation flows
through the system. Authoritative design rationale lives in the
internal technical specification ([`SPEC-TECHNIQUE.md`](specs/SPEC-TECHNIQUE.md),
in French).

## Component overview

GC-Forge is split into two top-level artefacts.

1. A **Rust workspace** of six crates that compiles to a single
   command-line binary, `gc-forge`. The workspace owns the user
   interface, the scenario model, the regime catalogue, the runner
   abstraction, the post-run validator, and the embedded preset
   catalogue.
2. A **Java workload harness**, packaged as a Maven shaded JAR
   (`workload-harness.jar`). The harness implements the seven
   regimes as a parameterised allocation programme; it is the
   process whose GC log is captured.

The two artefacts communicate exclusively through the JVM
command-line: GC-Forge produces an `argv` for `java`, the JVM
emits a unified `-Xlog:gc*` log to a file, GC-Forge reads the file
back. There is no embedded protocol, no socket, no file other than
the log itself.

```
                        ┌────────────────────┐
   scenario.yaml ──▶    │  gc-forge (Rust)   │  ─── docker run … java …
                        │  • parse           │
                        │  • resolve regime  │           ▼
                        │  • build JVM argv  │   ┌──────────────────────┐
                        │  • spawn runner    │   │  Temurin 17 / 21     │
                        │  • write manifest  │   │  workload-harness    │
                        └────────────────────┘   │  (Java fat-jar)      │
                                  ▲              └──────────────────────┘
                                  │                       │
                                  └─── reads back ────── -Xlog:gc* file
```

## Rust workspace

The workspace lives under `crates/`. Each crate has a single,
documented responsibility, and the dependency graph is acyclic.

| Crate                   | Responsibility                                                                                            |
|-------------------------|-----------------------------------------------------------------------------------------------------------|
| `gc-forge-scenario`     | Typed model of the YAML wire formats (`scenario.v1`, `run-manifest.v1`, `matrix.v1`); JSON Schema export. |
| `gc-forge-regimes`      | Implementation of the seven regimes and their parameter validation; emits the JVM workload arguments.     |
| `gc-forge-runner`       | Runner abstraction; concrete `DockerRunner` for the MVP. Builds the `docker run … java` argv.             |
| `gc-forge-validate`     | GC-log parser (Temurin unified format) and invariant evaluator.                                           |
| `gc-forge-presets`      | Compile-time embedding of `presets/*.yaml` into the binary (`build.rs`).                                  |
| `gc-forge-cli`          | Command-line interface (`clap`). Wires the previous five crates into the seven subcommands.               |

Internal dependency order, used as the topological sort for
`cargo publish`:

```
scenario  ←  regimes  ←  runner  ←  validate  ←  presets  ←  cli
```

The workspace targets Rust 1.94 (pinned by `rust-toolchain.toml`).
Workspace-level lints enable Clippy's `pedantic` group and forbid
`unsafe_code`. Public surfaces are not yet annotated with
`missing_docs`; that constraint is scheduled for a later phase.

## Java workload harness

The harness is a standalone Maven project under `workload-harness/`.
A single `RegimeRegistry` dispatches on the regime name (the first
positional argument passed by GC-Forge) and instantiates the
matching `Regime` implementation. Each regime consumes its
parameters from the remaining `KEY=VALUE` arguments. The output is
the JVM's own GC log, written to the path supplied via `-Xlog:…
file=…`.

The harness is built with Maven and packaged as a fat JAR via the
`maven-shade-plugin`. The path of this JAR — either on the host or
embedded inside the runner container image — is recorded in the
manifest (with its SHA-256) for reproducibility.

## Runner subsystem

The runner is the boundary between the Rust orchestrator and the
external JVM. The MVP ships a single implementation, `DockerRunner`,
that:

- always passes `--rm`, `--network=none`, and `--entrypoint=java`,
  for hermeticity and to isolate the JVM's stdout/stderr from any
  default container `ENTRYPOINT`;
- mounts the host output directory at `/work` (read-write) using
  an absolute path, so the JVM can write the log file directly;
- mounts the harness JAR at `/work/harness.jar` read-only, unless
  the runner image already embeds the harness — in which case the
  mount is skipped and `--embedded-harness <PATH>` is passed
  through to `java -jar`;
- builds the standardised `-Xlog:gc*=info,gc+heap=debug,gc+age=trace,gc+phases=debug,gc+humongous=trace`
  flag mandated by [`SPEC-TECHNIQUE`](specs/SPEC-TECHNIQUE.md) §6.1;
- selects the JVM image from the scenario's `spec.jvm.major`
  (`eclipse-temurin:{17,21}-jdk-jammy` by default; overridable via
  `--image`).

A `NativeRunner` that downloads JDKs from Adoptium on demand is
planned for the V1 release; selecting between Docker and native
will then become a matter of CLI flag plus availability detection.

## Wire formats

Three YAML schemas are versioned independently, each with a
`gc-forge/<name>.v1` `apiVersion`:

- **`scenario.v1`** — the input. Describes the `(JVM, GC, regime,
  duration, seed)` tuple, supports `extends:` inheritance and
  override application.
- **`run-manifest.v1`** — the output identity card. Records the
  resolved scenario, JVM build, JVM flags, host fingerprint, seed,
  the SHA-256 of both the captured log and the harness JAR, the
  expected invariants, and (after `gc-forge validate`) the
  validation status.
- **`matrix.v1`** — a batch description. Cartesian product of axes
  and seeds, minus filter cells; consumed by `gc-forge batch`.

JSON Schemas for the three formats are generated from the typed
Rust model (`schemars`) and shipped under `schemas/`. A drift test
at compile time fails if the in-tree schemas diverge from the
typed model.

## Reproducibility and observability

Each run records, in the manifest:

- a UUID v7 for the run, encoding wall-clock ordering;
- the resolved scenario after `extends:` and `--override`
  application, byte-for-byte;
- the JVM `-version` first line, captured from a probe run;
- every flag passed to `java`, in argv order;
- the seed, in hexadecimal;
- SHA-256 hashes for the captured log and the harness JAR;
- the host fingerprint (OS, architecture, container indication).

This information is sufficient, in principle, to reproduce the run
on a different host modulo the inherent non-determinism of the
JVM. Inter-run variance budgets are documented in
[`SPEC-FONCTIONNELLE`](specs/SPEC-FONCTIONNELLE.md) §7.3 and
verified at release time by `gc-forge variance-check`.

## Quality gates

The project enforces a per-iteration *Definition-of-Done gate*
(`scripts/dod-gate.sh`) that runs `cargo fmt --check`, `cargo
clippy --workspace -- -D warnings`, `cargo test --workspace`,
`cargo deny check`, `mvn verify`, and `gc-forge selftest`. The
gate is a precondition for merging an iteration branch and is
re-evaluated by GitHub Actions on every push to `main` and on
every pull request.

The release pipeline (`.github/workflows/release.yml`) adds a
matrix build of the CLI for four targets (Linux x86_64, Linux
aarch64, macOS x86_64, macOS aarch64) plus two human-gated jobs
that publish the six crates to crates.io and a multi-architecture
Docker image to GHCR.

## References

- [`doc/specs/SPEC-TECHNIQUE.md`](specs/SPEC-TECHNIQUE.md) —
  authoritative technical specification (FR), with rationale.
- [`doc/specs/SPEC-FONCTIONNELLE.md`](specs/SPEC-FONCTIONNELLE.md)
  — authoritative functional specification (FR).
- [`doc/process/orchestration.md`](process/orchestration.md) —
  development process and DoD gate.
- [`doc/concepts/traceability.md`](concepts/traceability.md) —
  cross-project contract surface with GC-Insight.
