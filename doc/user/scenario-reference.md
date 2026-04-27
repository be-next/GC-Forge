# Scenario reference (`gc-forge/scenario.v1`)

A GC-Forge scenario is a YAML document describing *what* to run
(which JVM, which GC algorithm, which workload regime) and *for
how long*. The authoritative JSON Schema is committed at
[`schemas/scenario-v1.json`](../../schemas/scenario-v1.json) and is
generated from the typed Rust model:

```sh
cargo run -p gc-forge-scenario --bin gen-schema
```

This page is the human-readable companion to that schema.

## Document structure

```yaml
apiVersion: gc-forge/scenario.v1
kind: Scenario

# Optional: inherit from another scenario file (relative to this one).
extends: ./baseline.yaml

metadata:
  name: <slug>                        # required
  version: <semver>                   # default "1.0.0"
  description: <string>
  tags: [<string>, ...]
  authors: [<string>, ...]

spec:
  jvm: { ... }                        # axis 1: JVM
  gc:  { ... }                        # axis 2: collector + heap
  regime: { ... }                     # axis 3: application workload

  duration: <duration>                # required
  warmup: <duration>                  # optional
  seed: <integer | "0xHEX">           # required

  output: { ... }                     # optional, where to write
  expected: { ... }                   # optional, ground-truth assertions
```

The fields are described below.

## `apiVersion` and `kind`

Always:

```yaml
apiVersion: gc-forge/scenario.v1
kind: Scenario
```

Any other value is rejected at load time. Future versions will be additive and
ship under `scenario.v2` rather than mutate this schema in place.

## `extends`

Optional path (relative to the current scenario file) to a parent scenario.
Loading a scenario follows the chain top-down and **deep-merges** parents and
children:

- Maps are merged key-by-key (children win on conflicting scalars).
- Sequences are replaced wholesale by the child.
- Cycles are detected and rejected.

There is no limit on chain depth besides cycle detection; the typical use is
two levels (`baseline → variant`).

## `metadata`

| Field        | Type                | Required | Notes |
|--------------|---------------------|----------|-------|
| `name`       | slug string         | yes      | Used in default output paths. |
| `version`    | string (SemVer)     | no       | Default `"1.0.0"`. |
| `description`| string              | no       | Free text. |
| `tags`       | array of string     | no       | Free taxonomy. |
| `authors`    | array of string     | no       | Informational only. |

## `spec.jvm`

| Field          | Type                                      | Required | Notes |
|----------------|-------------------------------------------|----------|-------|
| `vendor`       | `temurin` &#124; `corretto` &#124; `graalvm` &#124; `openj9` | yes | MVP supports `temurin`. Other vendors are accepted by the parser but rejected by the runner until V1+. |
| `major`        | integer                                   | yes      | MVP supports `17` and `21`. |
| `distribution` | `jdk` &#124; `jre`                        | no       | Default `jdk`. |
| `extra_flags`  | array of string                           | no       | Non-GC JVM flags appended verbatim. |

## `spec.gc`

| Field         | Type                                       | Required | Notes |
|---------------|--------------------------------------------|----------|-------|
| `algorithm`   | `G1` &#124; `ZGC` &#124; `Parallel` &#124; `Shenandoah` &#124; `Serial` &#124; `Epsilon` | yes | All six are present in Temurin 17 / 21. CMS (deprecated and removed after JDK 14) and OpenJ9 collectors land in V1+. |
| `options`     | object — see below                         | yes      | |
| `extra_flags` | array of string                            | no       | GC-specific flags appended verbatim. |
| `log_format`  | `unified` &#124; `legacy`                  | no       | Default `unified` (`-Xlog:gc*`). |

### `spec.gc.options`

| Field             | Type                  | Required | Notes |
|-------------------|-----------------------|----------|-------|
| `generational`    | bool                  | no       | ZGC only. The runner defaults to `true` on JDK 21+ when omitted; setting `false` selects the non-generational variant via `-XX:-ZGenerational`. |
| `heap`            | object — see below    | yes      | |
| `pause_target_ms` | integer               | no       | G1 (`-XX:MaxGCPauseMillis`); also honoured by Shenandoah. |
| `region_size_mb`  | integer               | no       | G1 (`-XX:G1HeapRegionSize`). |
| `ihop_percent`    | integer (0..100)      | no       | G1 (`-XX:InitiatingHeapOccupancyPercent`). |

Algorithm-specific behaviour:

- **`G1`** — emits `-XX:+UseG1GC` followed by `MaxGCPauseMillis`,
  `G1HeapRegionSize`, and `InitiatingHeapOccupancyPercent` when the
  corresponding option is set.
- **`ZGC`** — emits `-XX:+UseZGC`. The generational mode is enabled
  by default (`-XX:+ZGenerational`); set `generational: false` to
  select the non-generational variant.
- **`Parallel`** — emits `-XX:+UseParallelGC`.
- **`Shenandoah`** — emits `-XX:+UseShenandoahGC`. Honours
  `pause_target_ms`.
- **`Serial`** — emits `-XX:+UseSerialGC`. Single-threaded
  stop-the-world; appropriate for small heaps and single-core
  environments.
- **`Epsilon`** — emits
  `-XX:+UnlockExperimentalVMOptions -XX:+UseEpsilonGC`. Performs
  no collection: every allocation extends the heap until either
  the run duration is reached or `OutOfMemoryError` fires.

### `spec.gc.options.heap`

| Field      | Type        | Required | Notes |
|------------|-------------|----------|-------|
| `min`      | byte size   | yes      | e.g. `"512m"`, `"2g"`. |
| `max`      | byte size   | yes      | e.g. `"2g"`. |
| `new_size` | byte size   | no       | Mostly relevant to Parallel. |

A **byte size** is parsed from the JVM-style suffix forms: `512`, `1024k`,
`512m`, `2g` (case-insensitive, with optional `b` suffix). Raw integers are
also accepted and treated as a byte count.

## `spec.regime`

| Field        | Type           | Required | Notes |
|--------------|----------------|----------|-------|
| `kind`       | string         | yes      | One of the seven MVP regimes; see [`regimes.md`](regimes.md). |
| `parameters` | free-form map  | no       | Schema is regime-specific; validated at run time, not at parse time. |

## `spec.duration`, `spec.warmup`

Durations accept several forms:

- Suffix: `"30s"`, `"5m"`, `"1h"`, `"2d"`, `"500ms"`.
- Minimal ISO-8601: `"PT30S"`, `"PT5M"`, `"PT1H"`.
- Bare integer: interpreted as seconds.

`duration` is required. `warmup` is optional and has no default at parse
time; the runner defaults to `10s` if omitted.

## `spec.seed`

Required. Either:

- A non-negative integer.
- A hex string prefixed with `0x` (e.g. `"0xC0FFEE"`).

The seed is passed to the harness for deterministic workload generation.

## `spec.output` (optional)

| Field           | Type   | Notes |
|-----------------|--------|-------|
| `log_path`      | path   | Default `<name>-<seed>.log` in the run output directory. |
| `manifest_path` | path   | Default `<name>-<seed>.manifest.yaml`. |
| `capture_jfr`   | bool   | V1+. |

## `spec.expected` (optional)

Ground truth used by `gc-forge validate` after the run.

| Field        | Type                                           |
|--------------|------------------------------------------------|
| `phenomena`  | array of phenomenon ID strings                 |
| `invariants` | array of `{ rule: <expr>, threshold: <value> }`|

Phenomenon IDs are listed in `SPEC-FUNCTIONAL.md` §6.3.

## CLI overrides

`gc-forge run --override KEY=VALUE` (and `gc-forge lint --override …`) accept
dotted paths into the resolved scenario tree, applied **after** the
`extends:` chain is merged:

```sh
gc-forge lint scenario.yaml \
  --override 'spec.gc.options.heap.max=4g' \
  --override 'spec.regime.parameters.humongous_ratio=0.7'
```

The right-hand side is parsed as YAML, so scalars (`"4g"`, `42`,
`true`), arrays (`"[a, b, c]"`), and maps are accepted. An override
that breaks the schema is rejected with the same error returned for
an invalid YAML file.

## Linting

```sh
gc-forge lint <path/to/scenario.yaml> [--override KEY=VALUE]...
```

Validates syntax, `apiVersion`/`kind`, the `extends:` chain, and override
applicability. Exits with code `0` on success, `1` on failure with a
diagnostic on stderr. No JVM is launched.
