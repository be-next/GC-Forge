# User documentation

This directory holds the user-facing documentation for GC-Forge 0.1.0.
The four pages below are designed to be read in order on a first
contact and used as reference afterwards.

## Reading order

1. [**Getting started**](getting-started.md) — install, run a first
   preset, read the manifest, troubleshoot the most common issues.
2. [**Regimes**](regimes.md) — the seven MVP regimes (steady-state,
   allocation-burst, humongous-pressure, slow-leak, cache-churn,
   mixed-gc-pathological, microservice-stop-and-go), their parameters,
   expected signatures, phenomena and shipped presets.
3. [**Scenario reference**](scenario-reference.md) — the YAML schema:
   `apiVersion`, `metadata`, `spec.gc`, `spec.jvm`, `spec.regime`,
   `spec.duration`, `spec.warmup`, `extends`, override syntax.
4. [**CLI reference**](cli-reference.md) — every subcommand
   (`lint`, `run`, `validate`, `batch`, `presets`, `selftest`,
   `variance-check`), flags, defaults and exit codes.

## Cross-cutting documents

- [`../pitch.md`](../pitch.md) — one-page elevator pitch for GC-Forge.
- [`../traceability.md`](../traceability.md) — phenomena × presets ×
  GC-Insight detector matrix (the contract surface for downstream
  analysers).
- [`../specs/`](../specs/) — internal product specifications. Useful
  when extending GC-Forge itself or when arbitration is needed; not
  required reading for users of the CLI.
