# Regimes

A *regime* is a parameterised JVM workload, characterised by its GC
fingerprint. GC-Forge ships seven regimes for the MVP. Each one exposes a
typed parameter block (the `spec.regime.parameters` map of the scenario
schema) and asserts a quantified signature that downstream validation can
check against the produced log.

This page documents the public contract of each regime. Implementation
details live in the spec (`doc/specs/SPEC-FONCTIONNELLE.md` §4).

## R1 — `steady-state-healthy`

**Available since:** iteration 4.

> Reference scenario: an application allocating at a steady mean rate, with a
> bounded live set, no leaks, and a short tail of long-lived allocations. Used
> as the baseline against which other regimes are contrasted.

### Parameters

| Key                          | Type            | Default | Notes |
|------------------------------|-----------------|---------|-------|
| `allocation_rate_mb_s`       | integer (≥ 0)   | `50`    | Mean allocation rate in MiB/s. |
| `live_set_mb`                | integer (≥ 0)   | `100`   | Cap on the bounded live set in MiB. |
| `object_size_distribution`   | enum            | `mixed` | `small` (16–256 B), `medium` (256–4096 B), `mixed` (80% small / 20% medium). |
| `lifetime_distribution`      | enum            | `mixed` | `short` (everything dies in young), `mixed` (10% promoted to a long-lived survivor pool capped at `live_set_mb / 10`). |

Unknown keys are rejected at scenario resolution time.

### Expected signature

- ≥ 80 % of collections are young (`young_ratio >= 0.8`).
- 0 full GC.
- p99 young pause < 50 ms at heap = 2 GiB and `allocation_rate_mb_s = 50`.
- Inter-run variance on total pause count ≤ 5 %.

### Phenomena exhibited

- `young_gc_steady`.

### Use cases

- Reference baseline for GC-Insight detectors.
- Sanity check that a JVM build is not pathologically broken.

### Shipped baselines

R1 ships with one preset per MVP collector, all extending the canonical YAML:

- `presets/steady-g1-baseline.yaml` — G1 (default).
- `presets/steady-zgc-baseline.yaml` — generational ZGC.
- `presets/steady-parallel-baseline.yaml` — Parallel.

The three differ only in `spec.gc.algorithm`; `extends:` keeps them in lock-step
when the regime parameters or invariants change.

## R2 — `allocation-burst`

_TODO iter 7._

## R3 — `humongous-pressure`

_TODO iter 8._

## R4 — `slow-leak`

_TODO iter 10._

## R5 — `cache-churn`

_TODO iter 9._

## R6 — `mixed-gc-pathological`

_TODO iter 11._

## R7 — `microservice-stop-and-go`

_TODO iter 12._
