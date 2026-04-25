# API freeze — iteration 5 (run-end-to-end)

Gate 1 artifact. Overwritten at every iteration before parallel work starts.

## Scope

The `gc-forge run` subcommand lands. It composes the parser (iter 2), the
regime translator (iter 4), and the Docker runner (iter 3), and emits the
run manifest defined in SPEC-FONCTIONNELLE §6.2. Iteration 5 also ships
the first preset YAML.

## Public Rust surface added

### Crate `gc-forge-scenario` — new `manifest` module

| Item | Kind | Notes |
|------|------|-------|
| `RunManifest`                  | struct | top-level `gc-forge/run-manifest.v1` document |
| `RunMeta`                      | struct | `id`, `started_at`, `ended_at`, `duration_actual`, `exit_status`, `host` |
| `HostMeta`                     | struct | `os`, `arch`, `cpu_count`, `container` |
| `ScenarioRecord`               | struct | `source_path`, `source_sha256`, `resolved` (full Scenario) |
| `JvmRecord`                    | struct | `vendor`, `version`, `flags` |
| `ReproducibilityRecord`        | struct | `seed` (hex string), `workload_jar_sha256`, `gc_forge_version` |
| `OutputRecord`                 | struct | `log_path`, `log_sha256`, `log_size_bytes` |
| `ExpectedInvariantRecord`      | struct | `rule`, `threshold` (free-form YAML) |
| `ValidationRecord`             | struct | `status`, `results`, `validated_at`, `validator_version` |
| `ExitStatusRecord`             | enum   | `Success`, `Failure(i32)`, `Oom`, `Timeout`, `Signaled(i32)` — `serde(tag="kind")` |
| `RunManifest::write_yaml(&Path)`| fn    | atomic write |
| `RunManifest::write_json(&Path)`| fn    | atomic write |
| `manifest::sha256_hex(&Path)`   | fn    | helper (used internally + reusable by validators) |

### Crate `gc-forge-cli`

| Item | Kind | Notes |
|------|------|-------|
| `gc-forge run <scenario>` | clap subcommand | the orchestrator |
| flags: `--out-dir`, `--override`, `--image`, `--docker-cpus`, `--docker-memory`, `--harness-jar`, `--manifest-format` | clap | all wired |

## Java harness public surface

No change.

## YAML schema changes

New schema `gc-forge/run-manifest.v1` rendered into
`schemas/run-manifest-v1.json` via the same `gen-schema` binary
(extended to write both files).

## Invariants for Tester-unit

- `RunManifest` round-trips through both YAML and JSON.
- `RunManifest::write_yaml` produces a file that re-loads to the same
  struct (sha-stable on the resolved scenario subtree).
- `manifest::sha256_hex` matches `sha256sum`'s output for a few canned
  byte strings.
- `gc-forge run` parses and routes flags correctly (no JVM launched in
  the unit test — only argv assertions).
- The end-to-end smoke (docker-integration feature) writes a manifest
  whose `output.log_sha256` matches the actual log file.

## Doc sections to author (Doc-writer)

- `doc/user/getting-started.md` — fill the "First scenario" and
  "Reading the manifest" sections.
- `doc/user/cli-reference.md` — drop the `_TODO iter 5_` markers on
  the `run` subcommand.
- `CHANGELOG.md` — Unreleased: manifest module, run subcommand,
  steady-g1-baseline preset, run-manifest schema.

## Approval

- Coder: A5 — frozen 2026-04-25
- Reviewer: A6 — `Approved: A6 2026-04-25` (read against SPEC-FONCTIONNELLE §6.2 and SPEC-TECHNIQUE §4.1; manifest module placement and exit-status string serialisation OK; --preset deferral noted in CLI doc).
