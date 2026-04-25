# API freeze — iteration 2 (scenario-parser)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

Iteration 2 lands the `gc-forge-scenario` crate's first real API: types modelling
the `gc-forge/scenario.v1` YAML schema, a loader that resolves `extends:`
chains and applies CLI overrides, JSON Schema generation, and the
`gc-forge lint` subcommand wiring.

## Public Rust surface added

### Crate `gc-forge-scenario`

| Item | Kind | Notes |
|------|------|-------|
| `Scenario`                                          | struct | top-level scenario, deserialised from YAML |
| `Metadata`                                          | struct | `name`, `version`, `description`, `tags`, `authors` |
| `Spec`                                              | struct | top-level spec block |
| `JvmSpec`                                           | struct | `vendor`, `major`, `distribution`, `extra_flags` |
| `JvmVendor`                                         | enum   | `Temurin` (MVP); other variants present but flagged unsupported |
| `Distribution`                                      | enum   | `Jdk`, `Jre` |
| `GcSpec`                                            | struct | `algorithm`, `options`, `extra_flags`, `log_format` |
| `GcAlgorithm`                                       | enum   | `G1`, `Zgc`, `Parallel` (MVP) |
| `GcOptions`                                         | struct | `generational`, `heap`, `pause_target_ms`, `region_size_mb`, `ihop_percent` |
| `HeapConfig`                                        | struct | `min`, `max`, `new_size` (all `ByteSize`) |
| `ByteSize`                                          | struct | parses `"2g"`, `"512m"`, `"1024k"`, raw bytes |
| `LogFormat`                                         | enum   | `Unified`, `Legacy` |
| `RegimeSpec`                                        | struct | `kind`, `parameters` (free-form `serde_yaml::Value`) |
| `OutputSpec`                                        | struct | `log_path`, `manifest_path`, `capture_jfr` |
| `ExpectedClause`                                    | struct | `phenomena: Vec<String>`, `invariants: Vec<InvariantRule>` |
| `InvariantRule`                                     | struct | `rule: String`, `threshold: serde_yaml::Value` |
| `Duration` (re-export of `humantime_serde`-flavoured) | type alias | parses `"90s"`, `"5m"` |
| `Scenario::from_path(&Path)`                        | fn     | load + apiVersion check (no extends/override yet) |
| `Scenario::resolve(base_dir: &Path)`                | fn     | follows `extends:` chains and merges |
| `Scenario::apply_overrides(&[Override])`            | fn     | applies dotted-path overrides on the resolved scenario |
| `Override::parse(&str)`                             | fn     | parses `"a.b.c=value"` |
| `ScenarioError`                                     | enum (thiserror) | `Io`, `Yaml`, `UnsupportedApiVersion`, `Cycle`, `MissingField`, `InvalidOverride`, … |
| `JSON_SCHEMA: &str`                                 | const  | the JSON Schema as embedded string (built from schemars) |
| `bin gen-schema`                                    | bin    | regenerates `schemas/scenario-v1.json` |

### Crate `gc-forge-cli`

| Item | Kind | Notes |
|------|------|-------|
| `lint` subcommand | clap subcommand | `gc-forge lint <path>` — load + extends, no execute |

## Java harness public surface

No change.

## YAML schema changes

This is where the schema lands. Reference: SPEC-FONCTIONNELLE §5.1. The
`apiVersion: gc-forge/scenario.v1` and `kind: Scenario` are required. Optional
top-level `extends: <relative-path>`.

## Invariants for Tester-unit

- A minimal valid scenario (apiVersion + kind + spec.jvm/gc/regime + duration + seed) parses.
- Missing `apiVersion` fails with a typed error.
- An unknown `gc.algorithm` fails at deserialisation (not at validation time).
- `extends:` resolves a single level, multi-level chain, and detects a cycle.
- Overrides on nested paths (e.g. `spec.gc.options.heap.max=4g`) succeed.
- An override with an unknown path fails with a typed error.
- The on-disk JSON schema matches a fresh regeneration (drift detection).
- `gc-forge lint <good.yaml>` exits 0, `gc-forge lint <bad.yaml>` exits non-zero with a clear message.

## Doc sections to author (Doc-writer)

- `doc/user/scenario-reference.md` — full YAML schema reference, one section per top-level field.
- `CHANGELOG.md` — `Unreleased` section: scenario types, loader, extends, overrides, JSON Schema, lint subcommand.
- Update `doc/user/getting-started.md` "First scenario" placeholder if a sensible step can land now (probably keep TODO until iter 5).

## Approval

- Coder: A2 — frozen 2026-04-25
- Reviewer: A3 — `Approved: A3 2026-04-25` (signed after a read-through of the public surface against SPEC-FONCTIONNELLE §5; no red flag).
