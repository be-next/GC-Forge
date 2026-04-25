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

**Available since:** iteration 7.

> A workload that alternates between a baseline allocation rate and a higher
> burst rate on a fixed cadence. Used to demonstrate young-GC frequency that
> pulses in step with traffic without falling into evacuation failures.

### Parameters

| Key                | Type           | Default | Notes |
|--------------------|----------------|---------|-------|
| `base_rate_mb_s`   | integer (≥ 0)  | `30`    | Allocation rate outside burst windows. |
| `burst_rate_mb_s`  | integer (≥ 0)  | `200`   | Allocation rate during a burst. Must be ≥ `base_rate_mb_s`. |
| `burst_duration_s` | integer (> 0)  | `5`     | Length of each burst window. Must be ≤ `burst_period_s`. |
| `burst_period_s`   | integer (> 0)  | `30`    | Cadence of the burst schedule (one window per period). |
| `bursts_count`     | integer or `auto` | `auto` | Reserved; today the burst count is derived from the run duration. |

### Expected signature

- Young-GC frequency pulses with the burst cadence (visible on a frequency-vs-time tracé).
- After a burst exits, frequency falls back to baseline within ~ 2× `burst_duration_s`.
- No evacuation failure on the canonical preset (G1, 2 GiB heap, 200 MiB/s burst).

### Phenomena exhibited

- `allocation_burst`.

### Use cases

- Demonstrating burst-detection in GC-Insight, with a clear ground-truth schedule.
- Stress-testing a heap dimension against a known burst envelope.

### Shipped baselines

- `presets/burst-g1-30s.yaml` — G1, 2 GiB heap, 5 min duration, 30 s burst period.
- `presets/burst-parallel-30s.yaml` — Parallel collector, same schedule (extends the G1 preset).

## R3 — `humongous-pressure`

**Available since:** iteration 8.

> A workload where a configurable fraction of allocations are large enough to
> hit the JVM's "humongous" path (G1 region size or more). Used to demonstrate
> humongous-allocation detection, mixed-GC triggering ahead of IHOP, and — at
> high ratios — evacuation failure.

### Parameters

| Key                    | Type             | Default | Notes |
|------------------------|------------------|---------|-------|
| `humongous_ratio`      | float in `(0, 1]`| `0.5`   | Probability that a given allocation is humongous. |
| `humongous_size_kb`    | integer or `auto`| `auto`  | Humongous chunk size. `auto` = 2 MiB, comfortably above G1's region size for any heap up to 32 GiB. |
| `region_size_mb`       | integer (≥ 0)    | unset   | Informational; the JVM picks the actual region size. Forwarded to the harness for traceability. |
| `allocation_rate_mb_s` | integer (≥ 0)    | `80`    | Mean allocation rate in MiB/s. |

### Expected signature

- ≥ 1 humongous allocation per second visible in the GC log.
- G1 emits humongous-region accounting (`humongous regions: N`).
- Mixed GC triggered ahead of IHOP because humongous allocations force the
  marking cycle.
- At `humongous_ratio > 0.7` on a tight heap, an evacuation failure becomes
  expected.

### Phenomena exhibited

- `humongous_allocation`.
- Optionally `evacuation_failure` (catalogued in the `evac-fail` preset).

### Use cases

- Demonstrating humongous detection, including the "small object batch
  allocated in a row that becomes humongous when escape-analysed" pattern.
- Stressing G1's region accounting under controlled pressure.

### Shipped baselines

- `presets/humongous-g1-classic.yaml` — G1, 2 GiB heap, ratio 0.5, mixed-GC
  efficient. Reference scenario.
- `presets/humongous-g1-evac-fail.yaml` — G1, 1 GiB heap, ratio 0.7,
  evacuation failure expected.

## R4 — `slow-leak`

_TODO iter 10._

## R5 — `cache-churn`

**Available since:** iteration 9.

> A workload that maintains a bounded in-memory cache: each entry lives for
> a fixed lifetime and is then evicted, FIFO-style. The continuous mid-life
> tenuring drives a steady promotion rate from young to old, exercising
> mixed GCs. Used to distinguish a bounded cache pattern from a slow leak
> (R4): both grow old gen, only the leak grows it unboundedly.

### Parameters

| Key                   | Type           | Default | Notes |
|-----------------------|----------------|---------|-------|
| `cache_size_mb`       | integer (> 0)  | `500`   | Nominal cache capacity. The harness FIFO-evicts to keep it under this size. |
| `eviction_rate_per_s` | integer (> 0)  | `1000`  | Eviction (and insertion) rate. Drives the steady-state allocation rate. |
| `entry_lifetime_ms`   | integer (> 0)  | `2000`  | Time-to-live of an entry. The combination of `lifetime_ms` and `eviction_rate_per_s` determines the live-set; if the entries don't all fit, FIFO eviction holds the size below `cache_size_mb`. |
| `entry_size_kb`       | integer (> 0)  | `8`     | Size of each cache entry. |

### Expected signature

- Promotion rate (young → old) ≥ 30 % of young throughput.
- Old gen oscillates between `cache_size × 0.8` and `cache_size × 1.2`.
- Mixed GCs are frequent but regular (not pathological).

### Phenomena exhibited

- `promotion_pressure`.

### Use cases

- Differentiating a bounded cache footprint from a slow leak (R4).
- Stressing G1's mixed-GC pacing under sustained promotion.

### Shipped baselines

- `presets/cache-g1-churn.yaml` — G1, 4 GiB heap, 5 min, default R5 params.
- `presets/cache-parallel-churn.yaml` — extends the G1 preset, swaps to Parallel.

## R6 — `mixed-gc-pathological`

_TODO iter 11._

## R7 — `microservice-stop-and-go`

_TODO iter 12._
