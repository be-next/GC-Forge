# GC-Forge

A declarative generator of Java garbage-collection logs. From a
typed YAML scenario describing a `(JVM, GC algorithm, application
regime)` tuple, GC-Forge runs a parameterised workload on a real
Java Virtual Machine, captures the unified `-Xlog:gc*` output, and
emits a hash-anchored manifest sufficient to reproduce the run.

GC-Forge is the counterpart of [GC-Insight](#): where Insight
analyses GC logs, Forge produces them on demand with documented
ground truth.

## Status

Version `0.1.0` — first MVP release.

| Item                | Coverage                                                        |
|---------------------|-----------------------------------------------------------------|
| Application regimes | 7 (steady-state, burst, humongous, slow-leak, cache-churn, mixed-GC pathological, microservice stop-and-go) |
| GC collectors       | 6 (G1, ZGC generational and non-generational, Parallel, Shenandoah, Serial, Epsilon) |
| JVM distributions   | Eclipse Temurin 17 and 21                                       |
| Shipped presets     | 21 (embedded in the binary)                                     |
| CLI subcommands     | `lint`, `run`, `validate`, `batch`, `presets`, `selftest`, `variance-check` |
| Wire formats        | `gc-forge/scenario.v1`, `gc-forge/run-manifest.v1`, `gc-forge/matrix.v1` |
| Runner              | Docker (native runner planned for V1)                           |
| Licence             | [MIT](LICENSE)                                                  |

A summary of the rationale behind these choices is given in
[`doc/concepts/overview.md`](doc/concepts/overview.md).

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

A guided walk-through of the same path, with troubleshooting notes,
is provided in
[`doc/user/getting-started.md`](doc/user/getting-started.md).

## Documentation

The documentation is organised by audience.

### For users

- [`doc/user/getting-started.md`](doc/user/getting-started.md) —
  install, run a first preset, read the manifest.
- [`doc/user/regimes.md`](doc/user/regimes.md) — the seven MVP
  regimes, their parameters, expected signatures, and shipped
  presets.
- [`doc/user/scenario-reference.md`](doc/user/scenario-reference.md)
  — the YAML schema (`apiVersion`, `metadata`, `spec.gc`,
  `spec.jvm`, `spec.regime`, `extends`, override syntax).
- [`doc/user/cli-reference.md`](doc/user/cli-reference.md) —
  every subcommand, flags, defaults and exit codes.

### For contributors and integrators

- [`doc/architecture.md`](doc/architecture.md) — runtime and
  build-time architecture.
- [`doc/concepts/overview.md`](doc/concepts/overview.md) — design
  goals and non-goals.
- [`doc/concepts/traceability.md`](doc/concepts/traceability.md)
  — phenomenon × preset × analyser-detector matrix.
- [`doc/process/orchestration.md`](doc/process/orchestration.md)
  — development process, role rotation, Definition-of-Done gate.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — contribution conventions.
- [`CHANGELOG.md`](CHANGELOG.md) — release history.

### Internal product specifications (French)

The framing documents authored during the design phase are
preserved in their original language. Day-to-day use of GC-Forge
does not require reading them.

- [`doc/specs/SPEC-FONCTIONNELLE.md`](doc/specs/SPEC-FONCTIONNELLE.md)
- [`doc/specs/SPEC-TECHNIQUE.md`](doc/specs/SPEC-TECHNIQUE.md)
- [`doc/specs/ROADMAP.md`](doc/specs/ROADMAP.md)
- [`doc/specs/BACKLOG.md`](doc/specs/BACKLOG.md)
- [`doc/specs/RISQUES.md`](doc/specs/RISQUES.md)

## Architecture, in brief

GC-Forge consists of a Rust workspace that compiles to a single
`gc-forge` binary, and a Java workload harness packaged as a
shaded JAR. The two artefacts communicate exclusively through the
JVM command line: GC-Forge produces an `argv` for `java`, the JVM
emits a unified `-Xlog:gc*` log to a file, and GC-Forge reads the
file back. There is no embedded protocol.

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
> garbage-collection logs* (Version 0.1.0). MIT licence. Available
> at https://github.com/jerome-ramette/gc-forge.

A versioned BibTeX entry will be added once a DOI is assigned.

## Contributing

Contributions are welcome under the project's MIT licence; see
[`CONTRIBUTING.md`](CONTRIBUTING.md) for the conventions used by
the project, including the documentation language policy
(English for user-facing material, French for internal
specifications), the Definition-of-Done gate, and the branching
model.

Bug reports are tracked in GitHub Issues. A bug report is most
useful when accompanied by the relevant section of the manifest
produced by the failing run.

## Licence

[MIT](LICENSE).
