# CLI reference

This page documents every subcommand shipped in 0.1.0. Each
section lists flags, defaults, and exit codes. Detailed per-feature
behaviour is covered by the unit tests of the matching Rust crate.

## Synopsis

```
gc-forge [OPTIONS] <SUBCOMMAND>

Subcommands:
  lint              Validate a scenario file without launching a JVM.
  run               Run a scenario.
  validate          Re-check a log against a manifest.
  batch             Run a matrix of scenarios.
  presets           List, show, export shipped presets.
  selftest          Run all presets and report.
  variance-check    Measure inter-run variance on a preset.
```

## Global options

```
-v, -vv, -vvv         Increase verbosity (tracing levels: info, debug, trace).
    --quiet           Suppress all non-error output.
    --no-color        Disable ANSI colours; `NO_COLOR` is also honoured.
```

A machine-readable `--output json` mode is reserved for V1. In
0.1.0, the manifest written next to each log is the
machine-readable artefact: it is typed, hash-anchored, and
self-describing.

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
flag to the JVM (cf. SPEC-TECHNICAL §6.1).

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

```
gc-forge batch <MATRIX> [--out-dir DIR] [--image TAG] [--embedded-harness PATH]
                        [--harness-jar PATH] [--continue-on-error]
```

Expands a matrix YAML into its cells (cartesian product of axes × seeds,
minus filters), runs each cell sequentially through the same orchestrator
as `gc-forge run`, and writes an `<out-dir>/index.csv` listing every cell's
log, manifest, exit status, duration, and validation status.

**Matrix YAML schema** (`gc-forge/matrix.v1`). The authoritative
JSON Schema is committed at
[`schemas/matrix-v1.json`](../../schemas/matrix-v1.json) and is
generated from the typed Rust model.

```yaml
apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: presets/steady-g1-baseline.yaml
  axes:
    spec.gc.algorithm: [G1, ZGC, Parallel]
    spec.jvm.major: [17, 21]
  seeds: [1, 2, 3]
  filters:
    - { spec.gc.algorithm: ZGC, spec.regime.kind: humongous-pressure }
```

Per-cell output is `<out-dir>/cellN-<scenario-name>-<seed-hex>.{log,manifest.yaml}`
where `N` is the zero-padded cell index. Filenames stay distinct even when
several cells share the same scenario name and seed.

| Flag                       | Effect                                                 |
|----------------------------|--------------------------------------------------------|
| `--out-dir <DIR>`          | Where logs, manifests, and `index.csv` land. Default `out`. |
| `--image <TAG>`            | Override the Docker image for every cell.              |
| `--embedded-harness <PATH>`| Skip the host JAR mount when the image embeds it.      |
| `--harness-jar <PATH>`     | Host JAR used by every cell (when not embedded).       |
| `--continue-on-error`      | Keep going past failing cells. Default: stop on first failure. |

Exit codes: `0` if all cells succeed; `2` if any cell failed.

**Note:** `--parallel` is not yet wired in iter 14; cells run sequentially.
Concurrency lands in V1 once Docker concurrency on macOS has been
characterised.

## `gc-forge presets`

```
gc-forge presets list
gc-forge presets show <NAME>
gc-forge presets export <NAME>
```

Operates on the catalogue of presets embedded in the binary at compile
time (the contents of `presets/*.yaml` from the source tree). `list`
enumerates name + algorithm + regime where the body is self-contained;
presets that use `extends:` are listed by name with an `(extends another
preset)` annotation. `show` prints the YAML body. `export` is identical
to `show` and is meant for piping (`gc-forge presets export foo > foo.yaml`).

## `gc-forge selftest`

```
gc-forge selftest [--image TAG] [--embedded-harness PATH]
                  [--per-preset-duration DUR] [--out-dir DIR]
                  [--continue-on-error]
```

Runs every embedded preset through the orchestrator, validates each
log against the manifest's expected invariants, and reports a summary.

Defaults:

- `--per-preset-duration 12s`: keeps the full pass under ~3 minutes,
  which fits a nightly CI budget. Override for the spec-fidelity run.
- `--out-dir out/selftest`: per-preset logs and manifests land there.
- Stops on first failure unless `--continue-on-error` is set.

Exit codes: `0` if every recognised invariant passed (skipped invariants
don't fail the run); `3` if any preset's validator returned `Failed`.

## `gc-forge variance-check`

```
gc-forge variance-check <PRESET-OR-PATH> [--runs N]
                                         [--image TAG] [--embedded-harness PATH]
                                         [--duration DUR] [--out-dir DIR]
```

Repeats a scenario `N` times (default `5`) with the same seed, parses
each log, and reports the coefficient of variation (CV) on:

- `young_count`        — budget: ≤ 8 % CV.
- `mean_pause_ms`      — budget: ≤ 10 % CV.
- `p99_pause_ms`       — budget: ≤ 20 % CV (centiles vary more).

The first argument is either an embedded preset name (e.g.
`steady-g1-baseline`) or a path to a scenario YAML.

Exit codes: `0` when every metric stays within budget; `3` when any
metric exceeds it.
