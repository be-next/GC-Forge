# Traceability matrix — phenomena × presets × GC-Insight capabilities

> **Scope**: SPEC-FONCTIONNELLE §11 mandates a matrix tracing each
> phenomenon to the presets that exhibit it and the GC-Insight detectors
> that should surface it. The third column is filled with placeholders
> (`<TODO>`) until GC-Insight publishes its stable detector identifiers;
> the first two columns are GC-Forge-internal and frozen for 0.1.0.

## Conventions

- A row asserts: when a preset exhibits its phenomenon under a faithful run, GC-Insight's named detector should fire.
- "Faithful run" = the spec-defined duration and parameters (not the truncated CI defaults). Iter 17's release pipeline will mark each row's "verified" status against the actual long-form runs.
- A `<TODO: detector-id>` cell means GC-Insight is expected to grow a detector for this phenomenon in V1; the corpus is already in place.

## Phenomena → presets

| Phenomenon                       | Presets                                                          | Expected GC-Insight detector            |
|----------------------------------|------------------------------------------------------------------|------------------------------------------|
| `young_gc_steady`                | `steady-g1-baseline`, `steady-zgc-baseline`, `steady-parallel-baseline` | `<TODO: detector-id>`                    |
| `allocation_burst`               | `burst-g1-30s`, `burst-parallel-30s`                             | `<TODO: detector-id>`                    |
| `humongous_allocation`           | `humongous-g1-classic`, `humongous-g1-evac-fail`                 | `<TODO: detector-id>`                    |
| `evacuation_failure`             | `humongous-g1-evac-fail`                                         | `<TODO: detector-id>`                    |
| `mixed_gc_efficient`             | `humongous-g1-classic`                                           | `<TODO: detector-id>`                    |
| `slow_leak`                      | `leak-g1-slow`, `leak-zgc-slow`                                  | `<TODO: detector-id>`                    |
| `full_gc`                        | `leak-g1-slow`                                                   | `<TODO: detector-id>`                    |
| `oom`                            | (any leak preset whose budget exceeds the heap)                  | `<TODO: detector-id>`                    |
| `concurrent_cycle_in_idle`       | `microservice-g1-stop-go`, `microservice-zgc-stop-go`            | `<TODO: detector-id>`                    |
| `promotion_pressure`             | `cache-g1-churn`, `cache-parallel-churn`                         | `<TODO: detector-id>`                    |
| `mixed_gc_pathological`          | `mixed-pathological-g1`                                          | `<TODO: detector-id>`                    |

## Presets → expected phenomena

| Preset                            | Algo     | Heap  | Expected phenomena                                            |
|-----------------------------------|----------|-------|---------------------------------------------------------------|
| `steady-g1-baseline`              | G1       | 2 GiB | `young_gc_steady`                                             |
| `steady-zgc-baseline`             | ZGC      | 2 GiB | `young_gc_steady`                                             |
| `steady-parallel-baseline`        | Parallel | 2 GiB | `young_gc_steady`                                             |
| `burst-g1-30s`                    | G1       | 2 GiB | `allocation_burst`                                            |
| `burst-parallel-30s`              | Parallel | 2 GiB | `allocation_burst`                                            |
| `humongous-g1-classic`            | G1       | 2 GiB | `humongous_allocation`, `mixed_gc_efficient`                  |
| `humongous-g1-evac-fail`          | G1       | 1 GiB | `humongous_allocation`, `evacuation_failure`                  |
| `leak-g1-slow`                    | G1       | 1 GiB | `slow_leak`, `full_gc` (and `oom` on long enough runs)        |
| `leak-zgc-slow`                   | ZGC      | 1 GiB | `slow_leak` (less likely to reach `full_gc` than the G1 variant) |
| `cache-g1-churn`                  | G1       | 4 GiB | `promotion_pressure`                                          |
| `cache-parallel-churn`            | Parallel | 4 GiB | `promotion_pressure`                                          |
| `mixed-pathological-g1`           | G1       | 2 GiB | `mixed_gc_pathological`                                       |
| `microservice-g1-stop-go`         | G1       | 1 GiB | `concurrent_cycle_in_idle`                                    |
| `microservice-zgc-stop-go`        | ZGC      | 1 GiB | (smoother contrast, fewer concurrent-cycle markers)           |

## How to use this matrix

- **Building a GC-Insight detector:** pick the row whose phenomenon the detector targets, run the corresponding preset(s) under faithful conditions, and use the manifest's `expected_invariants` as ground truth.
- **Demonstrating a GC-Insight capability:** pick the column entry for the detector and run the cited preset.
- **Filing a regression:** if a preset stops exhibiting its declared phenomenon between runs, file the issue against GC-Forge; if GC-Insight stops detecting a phenomenon GC-Forge still produces, file against GC-Insight.

The matrix is reviewed at every minor release of either project; updates land in coordinated pull requests.
