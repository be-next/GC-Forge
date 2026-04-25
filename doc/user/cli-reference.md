# CLI reference

> **Status:** skeleton — content lands alongside the corresponding feature
> iterations. Sections marked `_TODO iter N_` ship with that iteration.

## Synopsis

```
gc-forge [OPTIONS] <SUBCOMMAND>

Subcommands:
  lint              Validate a scenario file without launching a JVM.
  run               Run a scenario.                       _TODO iter 5_
  validate          Re-check a log against a manifest.    _TODO iter 13_
  batch             Run a matrix of scenarios.            _TODO iter 14_
  presets           List, show, export shipped presets.   _TODO iter 15_
  selftest          Run all presets and report.           _TODO iter 15_
  variance-check    Measure inter-run variance on a preset. _TODO iter 15_
```

## Global options

```
-v, -vv, -vvv         Increase verbosity (tracing levels: info, debug, trace).
    --quiet           Suppress all non-error output.
    --output <fmt>    Switch on machine output (`json`).            _TODO iter 5_
    --no-color        Disable ANSI colours; `NO_COLOR` is also honoured.
```

## Exit codes

| Code | Meaning                                          |
|------|--------------------------------------------------|
| 0    | Success.                                         |
| 1    | User error (bad YAML, missing JVM, …).           |
| 2    | Runtime error during JVM execution.              |
| 3    | Invariants violated (validation failed).         |
| 4    | Internal error (bug in GC-Forge).                |

## `gc-forge lint`

```
gc-forge lint <PATH> [--override KEY=VALUE]…
```

Validates a scenario without launching a JVM.

The check covers:

- syntactic YAML parse;
- `apiVersion` (`gc-forge/scenario.v1`) and `kind` (`Scenario`);
- `extends:` chain resolution and cycle detection;
- override applicability when `--override` is provided.

Exits `0` on success, `1` on failure with a typed error chain on stderr.

## `gc-forge run`

_TODO iter 5._

The run subcommand will surface the runner-related options that the
[`DockerRunner`](https://docs.rs/gc-forge-runner) already supports today
under the hood. Anticipated flags:

| Flag                       | Effect                                                                |
|----------------------------|-----------------------------------------------------------------------|
| `--out-dir <DIR>`          | Where the GC log and manifest are written.                            |
| `--image <TAG>`            | Override the Docker image tag.                                        |
| `--docker-cpus <N>`        | Pass `--cpus=<N>` to `docker run`.                                    |
| `--docker-memory <BYTES>`  | Pass `--memory=<BYTES>` to `docker run`.                              |
| `--rebase-timestamps`      | Post-process the log to start at `t=0` for reproducibility.           |
| `--manifest-format <FMT>`  | `yaml` (default) or `json`.                                           |

The runner currently defaults to:

- image `eclipse-temurin:<major>-jdk-jammy`, derived from `spec.jvm.major`;
- `--rm`, `--network=none`, `--entrypoint=java`;
- mounts `<out-dir>` at `/work` (rw) and the harness jar at `/work/harness.jar` (ro);
- when the image embeds the harness (e.g. `gc-forge-runner:*-jdk*` variants),
  the host jar mount is dropped automatically.

## `gc-forge validate`

_TODO iter 13._

## `gc-forge batch`

_TODO iter 14._

## `gc-forge presets`, `selftest`, `variance-check`

_TODO iter 15._
