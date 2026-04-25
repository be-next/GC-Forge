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

```
gc-forge validate <LOG> --manifest <PATH> [--update-manifest] [--manifest-format yaml|json]
```

Re-checks a GC log against the manifest's `expected_invariants`. Each rule
is evaluated against the parsed log and reported as passed, failed, or
skipped (the latter for rules the validator does not yet recognise).

| Flag                      | Effect                                                                |
|---------------------------|-----------------------------------------------------------------------|
| `--manifest <PATH>`       | Run manifest carrying `expected_invariants`. Required.                |
| `--update-manifest`       | Persist the validation result back into the manifest's `validation` block. |
| `--manifest-format <FMT>` | Format on write (only used with `--update-manifest`). `yaml` (default) or `json`. |

**Recognised rule shapes** (iter 13):

| Metric                          | Comparators              | Notes                                        |
|---------------------------------|--------------------------|----------------------------------------------|
| `young_count`                   | `<`, `<=`, `>`, `>=`, `==` | Number of `Pause Young` events.            |
| `mixed_count`                   | same                     | Number of mixed-style pauses.                |
| `full_count`                    | same                     | Number of `Pause Full` events.               |
| `concurrent_cycle_count`        | same                     | Number of concurrent markers.                |
| `evacuation_failure_count`      | same                     | Number of evacuation-failure lines.          |
| `young_ratio`                   | `<`, `<=`, `>`, `>=`     | `young_count / total_count`.                 |
| `mean_pause_ms`                 | `<`, `<=`, `>`, `>=`     | Arithmetic mean of pause durations.          |
| `p50_pause_ms` … `p99_pause_ms` | `<`, `<=`, `>`, `>=`     | Percentiles of pause durations.              |
| `humongous_regions_in_log`      | (boolean)                | True when any humongous marker is observed.  |
| `no_evacuation_failure`         | (boolean)                | True when zero evacuation-failure lines.     |
| `oom_seen`                      | (boolean)                | True when an `OutOfMemoryError` was logged.  |

Unknown rules are skipped with a clear note; they do **not** fail the run.
This makes future invariant additions backward-compatible with older
manifests.

### Exit codes

| Code | Meaning                                                          |
|------|------------------------------------------------------------------|
| 0    | All recognised rules passed (or every rule was skipped).         |
| 1    | I/O or manifest parse error.                                     |
| 3    | At least one rule failed (matches SPEC §10.1's invariant code).  |

## `gc-forge batch`

_TODO iter 14._

## `gc-forge presets`, `selftest`, `variance-check`

_TODO iter 15._
