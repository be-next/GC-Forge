# Contributing to GC-Forge

Contributions are welcome under the project's MIT licence. This
document describes the conventions that govern contributed work.

## Scope and licensing

GC-Forge is distributed under the MIT licence. A contribution is
accepted on the understanding that it is offered under the same
terms; no contributor licence agreement is required.

## Documentation language

User-facing and contributor-facing documentation is written in
**English**. This includes the top-level `README.md`, the user
manuals under `doc/user/`, the architecture and concept documents
under `doc/architecture.md` and `doc/concepts/`, the changelog,
commit messages, code comments, and CLI help strings.

Internal product specifications and the original project brief
(under `doc/specs/` and `doc/brief-*.md`) are written in
**French**. They are framing documents authored by the team and
preserved in the language they were drafted in. Translating them
is not required and not encouraged: the user-facing surfaces above
are sufficient to use, integrate with, and contribute to GC-Forge.

Code identifiers (Rust and Java) are in English, following the
conventions of each language ecosystem.

## Repository layout

A short description of each top-level entry follows; refer to
[`doc/architecture.md`](doc/architecture.md) for the runtime view.

| Path                        | Purpose                                                   |
|-----------------------------|-----------------------------------------------------------|
| `Cargo.toml`, `crates/`     | Rust workspace (six crates).                              |
| `workload-harness/`         | Java workload harness (Maven, fat-jar).                   |
| `presets/`                  | Fourteen YAML scenarios shipped with the binary.          |
| `schemas/`                  | JSON Schemas generated from the Rust model.               |
| `Dockerfile`                | Runner image embedding the harness on Temurin 21.         |
| `Dockerfile.jdk17`          | Runner image variant on Temurin 17.                       |
| `scripts/`                  | Build helpers (`dod-gate.sh`, `build-corpus-g1.sh`).      |
| `Makefile`                  | Convenience targets (`build`, `docker-image`, `dod-gate`).|
| `doc/`                      | Documentation (see below).                                |
| `.github/workflows/`        | Continuous integration and release pipelines.             |

The `doc/` directory is organised as follows.

| Path                       | Audience      | Content                                            |
|----------------------------|---------------|----------------------------------------------------|
| `doc/architecture.md`      | Contributors  | Single-page system architecture.                   |
| `doc/concepts/`            | All           | Cross-cutting concepts (overview, traceability).   |
| `doc/user/`                | Users         | Tutorial, regime catalogue, reference manuals.     |
| `doc/process/`             | Contributors  | Development process and DoD gate.                  |
| `doc/specs/`               | Maintainers   | Internal specifications (FR).                      |
| `doc/brief-gcforge-*.md`   | Maintainers   | Original project brief (FR).                       |

## Development environment

The project targets recent UNIX-like systems (Linux, macOS) and
requires:

- Rust 1.94 or later (pinned by `rust-toolchain.toml`);
- Maven 3.9 or later;
- JDK 17 or later (a local JDK is sufficient to build the harness);
- Docker (Engine or Desktop), required to run JVMs through the
  Docker MVP runner.

A first-time setup is summarised by:

```sh
make build           # cargo build --release + Maven shade of the harness
make docker-image    # build the runner image (Temurin 21 + harness)
```

## Definition-of-Done gate

Every change is expected to pass the Definition-of-Done gate before
being merged to `main`. The gate is implemented by
`scripts/dod-gate.sh` and runs the following checks in order.

| Check                           | Tool                                  |
|---------------------------------|---------------------------------------|
| Formatting                      | `cargo fmt --all -- --check`          |
| Lints (warnings as errors)      | `cargo clippy --workspace -- -D warnings` |
| Rust tests                      | `cargo test --workspace`              |
| Dependency licences and CVEs    | `cargo deny check`                    |
| Java build and tests            | `mvn -f workload-harness/pom.xml verify` |
| End-to-end self-test            | `gc-forge selftest` (when available)  |
| Inter-run variance              | `gc-forge variance-check` (when available) |
| Open-bug census                 | No `high` or `critical` open in `BUGS.md` |

The same gate is invoked by `.github/workflows/ci.yml` on every
push and pull request. Local invocation is `make dod-gate`.

## Branching, commits, and pull requests

Iteration branches follow the convention `iter/<NN>-<short-title>`
(see [`doc/process/orchestration.md`](doc/process/orchestration.md)
for a description of the multi-role process used during the MVP
phase). Ad-hoc bugfix and feature branches use
`fix/<short-title>` or `feat/<short-title>`.

Commits are expected to:

- be signed off (the project does not require GPG signing yet);
- carry a subject of at most 72 characters and use the imperative
  mood (`add …`, `fix …`, `document …`);
- reference an issue or specification section when applicable;
- be self-contained: `cargo test --workspace` should pass at every
  commit on the branch.

Pull requests are merged in fast-forward mode after the DoD gate
turns green and at least one repository maintainer has approved the
change.

## Issue tracking

Bug reports and feature requests are tracked in GitHub Issues.
Bug reports should include the GC-Forge version (`gc-forge
--version`), the relevant section of the manifest produced by the
failing run, and the host's operating system and architecture. For
reproducibility issues, attaching the full manifest and a small
reproducer scenario is the most efficient form.

Severity is assigned by maintainers using the convention recorded
in `BUGS.md`: a `high` or `critical` open bug blocks the next
iteration's DoD gate.

## Cross-project consistency with GC-Insight

GC-Forge is one half of a pair; GC-Insight is the analyser
counterpart. Changes that affect the GC log model
(`gc-core` shape) require synchronisation with GC-Insight and are
covered by the `gc-core-roundtrip` test described in
[`SPEC-TECHNIQUE`](doc/specs/SPEC-TECHNIQUE.md) §7.4. The
[`traceability matrix`](doc/concepts/traceability.md) records the
contract surface between the two projects.

## Release process

The release process is documented in
[`doc/process/orchestration.md`](doc/process/orchestration.md)
under "Release procedure". Four steps are deliberately kept
out of automation and require maintainer action: `git push origin
main`, tag creation, manual approval of the `release` GitHub
environment for `publish-crates` and `publish-docker`, and
promotion of the GitHub Release from draft to public.
