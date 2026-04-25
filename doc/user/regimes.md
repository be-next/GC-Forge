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

**Available since:** iteration 10.

> A workload whose live-set grows linearly with time: each second adds
> `leak_rate_mb_s` MiB that never get evicted. Models a process with a
> classic memory leak. The after-GC footprint therefore climbs steadily
> until the heap can no longer accommodate it; mixed GCs become more
> frequent and full GC (or OOM) eventually fires.

### Parameters

| Key                   | Type           | Default | Notes |
|-----------------------|----------------|---------|-------|
| `leak_rate_mb_s`      | float (> 0)    | `0.5`   | MiB/s of unbounded growth, on top of the baseline live-set. |
| `live_set_initial_mb` | integer (≥ 0)  | `200`   | Resident MiB before the leak starts climbing. Pre-allocated at startup. |

### Expected signature

- After-GC live-set climbs with slope `≈ leak_rate_mb_s × T`.
- Mixed-GC frequency increases over time as old-gen tightens.
- Full GC or OOM at the end of the run.

### Phenomena exhibited

- `slow_leak`.
- Optionally `full_gc` or `oom` depending on `leak_rate × duration` vs heap size.

### Use cases

- Demonstrating leak detection (linear regression on after-GC footprint).
- Distinguishing the leak signature from a bounded cache (R5).

### Shipped baselines

- `presets/leak-g1-slow.yaml` — G1, 1 GiB heap, 10 min, default R4 params.
- `presets/leak-zgc-slow.yaml` — extends G1 preset, swaps to ZGC. Useful contrast: ZGC's concurrent reclamation produces shorter pauses while the leak progresses at the same wall-clock rate.

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

**Available since:** iteration 11.

> A workload that pre-fills old gen with a fragmented long-lived pool, then
> keeps the heap pinned to IHOP. G1 falls into a mixed-GC cycle where each
> mixed pause is longer than the last and reclaims an ever-smaller fraction
> of the heap — the canonical "mixed-GC pathological" signature.

### Parameters

| Key                    | Type           | Default | Notes |
|------------------------|----------------|---------|-------|
| `old_gen_pressure`     | float `(0, 1]` | `0.7`   | Fraction of the heap pre-filled with long-lived references. |
| `fragmentation_factor` | float `[1, 3]` | `2.0`   | Multiplier on the share of the largest size class in the long-lived pool — higher values produce rougher free-lists. |
| `survivor_age_target`  | integer `[1, 15]` | `15` | Target tenuring age. Forwarded to the harness for traceability; the JVM hard-caps at 15. |

### Expected signature

- Mixed-GC duration grows over the run.
- Reclaim per mixed-GC stays below 5 % of the heap.
- Effective IHOP descends (G1's ergonomic adjustment).
- Possible evacuation failure → full GC at the end of the run.

### Phenomena exhibited

- `mixed_gc_pathological`.

### Use cases

- Premium pathological showcase for GC-Insight: surfacing degrading
  mixed-GC efficiency is exactly the kind of root-cause analysis that
  saves operators hours of guesswork.

### Shipped baselines

- `presets/mixed-pathological-g1.yaml` — G1, 2 GiB heap, 5 min, default R6 params.

## R7 — `microservice-stop-and-go`

**Available since:** iteration 12.

> A workload that alternates between active periods (where it allocates at
> a configured rate) and idle periods (where it does nothing). Models the
> request-driven pattern of a microservice between bursts. On G1, the idle
> stretches are exactly when concurrent-mark cycles tend to fire; on ZGC,
> the contrast is gentler.

### Parameters

| Key                | Type             | Default | Notes |
|--------------------|------------------|---------|-------|
| `active_period_s`  | integer (> 0)    | `10`    | Length of each active (allocating) phase. |
| `idle_period_s`    | integer (> 0)    | `20`    | Length of each idle (sleeping) phase. |
| `active_rate_mb_s` | integer (> 0)    | `100`   | Allocation rate during active phases. |
| `cycles`           | integer or `auto`| `auto`  | Total number of (active+idle) cycles; `auto` derives from the run duration. |

### Expected signature

- Idle phases show very few young GCs (`≤ 1 / 10 s`).
- G1: a concurrent-cycle marker is visible during idle stretches.
- ZGC: log is much smoother — the regime is less of a behavioural showcase
  there but the contrast is itself pedagogical.

### Phenomena exhibited

- `concurrent_cycle_in_idle`.

### Use cases

- Modelling realistic microservice traffic patterns.
- Contrasting G1's stop-the-world cycle against ZGC's concurrent reclamation
  on the same workload.

### Shipped baselines

- `presets/microservice-g1-stop-go.yaml` — G1, 1 GiB heap, 5 min.
- `presets/microservice-zgc-stop-go.yaml` — extends G1, swaps to ZGC.
