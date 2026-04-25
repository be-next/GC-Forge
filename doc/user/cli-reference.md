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

```
gc-forge run <PATH> [OPTIONS]
```

Loads the scenario at `<PATH>`, resolves its `extends:` chain, applies
any `--override` substitutions, picks the correct regime implementation,
runs the JVM through the runner, and writes the GC log + run manifest to
the output directory.

| Flag                              | Default                                              | Effect                                                                                          |
|-----------------------------------|------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| `--out-dir <DIR>`                 | `./out`                                              | Output directory; the log and manifest land at `<DIR>/<name>-<seed>.{log,manifest.yaml}`.       |
| `--override KEY=VALUE`            | none                                                 | Repeatable. Applies dotted-path overrides on the resolved scenario.                             |
| `--image <TAG>`                   | `eclipse-temurin:<major>-jdk-jammy`                  | Pin the Docker image (digest pinning recommended for reproducibility).                          |
| `--embedded-harness <PATH>`       | none                                                 | When set, treat the image as a `gc-forge-runner:*-jdk*` variant; skips mounting the host jar.   |
| `--docker-cpus <N>`               | host limits                                          | `docker run --cpus=<N>`.                                                                        |
| `--docker-memory <BYTES>`         | host limits                                          | `docker run --memory=<BYTES>`.                                                                  |
| `--harness-jar <PATH>`            | `workload-harness/target/workload-harness.jar`       | Host path to the workload fat-jar (ignored when `--embedded-harness` is set).                   |
| `--manifest-format <yaml\|json>`  | `yaml`                                               | Manifest serialisation; the log is always raw `-Xlog` output regardless.                        |

The runner always passes `--rm`, `--network=none`, and `--entrypoint=java`,
mounts the output directory at `/work`, and emits the unified
`-Xlog:gc*=info,gc+heap=debug,gc+age=trace,gc+phases=debug,gc+humongous=trace:file=/work/<name>-<seed>.log:time,level,tags,pid,tid:filecount=0`
flag to the JVM (cf. SPEC-TECHNIQUE §6.1).

When the scenario's `metadata.name` is `foo` and `spec.seed` is `0xC0FFEE`, the
output filenames become `foo-c0ffee.log` and `foo-c0ffee.manifest.yaml`. The
seed is rendered in lowercase hex without the `0x` prefix to keep filenames
stable across operating systems.

`--preset NAME` is reserved for iteration 15 (preset packaging). For now,
pass the path to the YAML in `presets/`.

## `gc-forge validate`

_TODO iter 13._

## `gc-forge batch`

_TODO iter 14._

## `gc-forge presets`, `selftest`, `variance-check`

_TODO iter 15._
