# User documentation

This directory contains the user-facing documentation for GC-Forge
0.1.0. Four documents are provided, intended to be read in the order
below at first contact and used as reference afterwards.

## Reading order

1. [Getting started](getting-started.md) — installation, a first
   preset run, the structure of the manifest, troubleshooting of
   common environmental issues.
2. [Regimes](regimes.md) — the seven application regimes shipped
   with the MVP (steady-state, allocation-burst, humongous-pressure,
   slow-leak, cache-churn, mixed-GC pathological, microservice
   stop-and-go), with their parameters, expected signatures,
   exhibited phenomena, and shipped presets.
3. [Scenario reference](scenario-reference.md) — the YAML wire
   format consumed by `gc-forge run` (`apiVersion`, `metadata`,
   `spec.gc`, `spec.jvm`, `spec.regime`, `spec.duration`,
   `spec.warmup`, `extends`, override syntax).
4. [CLI reference](cli-reference.md) — exhaustive description of
   every subcommand (`lint`, `run`, `validate`, `batch`,
   `presets`, `selftest`, `variance-check`), flags, defaults, and
   exit codes.

## Cross-cutting documents

- [`../concepts/overview.md`](../concepts/overview.md) —
  motivation, audiences, scope of the 0.1.0 release.
- [`../concepts/traceability.md`](../concepts/traceability.md) —
  the phenomenon × preset × analyser-detector matrix that defines
  the contract surface with downstream analysers.
- [`../architecture.md`](../architecture.md) — system architecture,
  for users who need to integrate GC-Forge into a larger pipeline.
- [`../specs/`](../specs/) — internal product specifications (FR).
  Not required reading for day-to-day use of the CLI.
