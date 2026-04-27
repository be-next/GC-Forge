# Traceability matrix: phenomena, presets, analyser detectors

## Purpose

Section 11 of the functional specification ([`SPEC-FONCTIONNELLE.md`](../specs/SPEC-FONCTIONNELLE.md))
mandates a traceability matrix linking each GC phenomenon to (i) the
presets that exhibit it under a faithful run and (ii) the GC-Insight
detectors expected to surface it. This document materialises the
matrix for the 0.1.0 release.

The phenomenon-to-preset columns are GC-Forge-internal and frozen
for 0.1.0. The detector column is filled with `<TODO: detector-id>`
placeholders pending publication of stable identifiers by the
GC-Insight project; the corpus listed in the second column is
already available and may be used to develop the corresponding
detectors.

## Conventions

A row of the phenomenon table asserts the following statement.

> When the listed presets are executed under a *faithful run*
> — that is, with their specification-defined duration and
> parameters, not the truncated values used in continuous
> integration — the named GC-Insight detector should fire.

Operational consequences of this convention:

- Compliance is evaluated at every minor release of either project.
- A row whose detector cell is `<TODO: detector-id>` records a
  contract that the GC-Forge corpus is in place; the GC-Insight
  detector is expected to be added in a future release.
- A regression in either project that breaks a row triggers a
  cross-project issue (see [Maintenance](#maintenance) below).

## Phenomena to presets

| Phenomenon                       | Presets                                                          | Expected GC-Insight detector |
|----------------------------------|------------------------------------------------------------------|------------------------------|
| `young_gc_steady`                | `steady-g1-baseline`, `steady-zgc-baseline`, `steady-zgc-nongen-baseline`, `steady-parallel-baseline`, `steady-shenandoah-baseline`, `steady-serial-baseline` | `<TODO: detector-id>` |
| `allocation_burst`               | `burst-g1-30s`, `burst-parallel-30s`                             | `<TODO: detector-id>` |
| `humongous_allocation`           | `humongous-g1-classic`, `humongous-g1-evac-fail`                 | `<TODO: detector-id>` |
| `evacuation_failure`             | `humongous-g1-evac-fail`                                         | `<TODO: detector-id>` |
| `mixed_gc_efficient`             | `humongous-g1-classic`                                           | `<TODO: detector-id>` |
| `slow_leak`                      | `leak-g1-slow`, `leak-zgc-slow`, `leak-shenandoah-slow`, `epsilon-leak-pure` | `<TODO: detector-id>` |
| `full_gc`                        | `leak-g1-slow`                                                   | `<TODO: detector-id>` |
| `oom`                            | `epsilon-leak-pure` (deterministic); any leak preset whose budget exceeds the heap | `<TODO: detector-id>` |
| `concurrent_cycle_in_idle`       | `microservice-g1-stop-go`, `microservice-zgc-stop-go`            | `<TODO: detector-id>` |
| `promotion_pressure`             | `cache-g1-churn`, `cache-parallel-churn`, `cache-serial-churn`   | `<TODO: detector-id>` |
| `mixed_gc_pathological`          | `mixed-pathological-g1`                                          | `<TODO: detector-id>` |
| `no_collection` (negative case)  | `epsilon-baseline`                                                | `<NONE: false-positive guard>` |

## Presets to expected phenomena

| Preset                            | Algorithm           | Heap   | Expected phenomena                                            |
|-----------------------------------|---------------------|--------|---------------------------------------------------------------|
| `steady-g1-baseline`              | G1                  | 2 GiB  | `young_gc_steady`                                             |
| `steady-zgc-baseline`             | ZGC (generational)  | 2 GiB  | `young_gc_steady`                                             |
| `steady-zgc-nongen-baseline`      | ZGC (non-gen)       | 2 GiB  | `young_gc_steady`                                             |
| `steady-parallel-baseline`        | Parallel            | 2 GiB  | `young_gc_steady`                                             |
| `steady-shenandoah-baseline`      | Shenandoah          | 2 GiB  | `young_gc_steady`                                             |
| `steady-serial-baseline`          | Serial              | 256 MiB| `young_gc_steady`                                             |
| `burst-g1-30s`                    | G1                  | 2 GiB  | `allocation_burst`                                            |
| `burst-parallel-30s`              | Parallel            | 2 GiB  | `allocation_burst`                                            |
| `humongous-g1-classic`            | G1                  | 2 GiB  | `humongous_allocation`, `mixed_gc_efficient`                  |
| `humongous-g1-evac-fail`          | G1                  | 1 GiB  | `humongous_allocation`, `evacuation_failure`                  |
| `leak-g1-slow`                    | G1                  | 1 GiB  | `slow_leak`, `full_gc` (and `oom` on long enough runs)        |
| `leak-zgc-slow`                   | ZGC (generational)  | 1 GiB  | `slow_leak` (less likely to reach `full_gc` than the G1 variant) |
| `leak-shenandoah-slow`            | Shenandoah          | 1 GiB  | `slow_leak`                                                   |
| `cache-g1-churn`                  | G1                  | 4 GiB  | `promotion_pressure`                                          |
| `cache-parallel-churn`            | Parallel            | 4 GiB  | `promotion_pressure`                                          |
| `cache-serial-churn`              | Serial              | 1 GiB  | `promotion_pressure`                                          |
| `mixed-pathological-g1`           | G1                  | 2 GiB  | `mixed_gc_pathological`                                       |
| `microservice-g1-stop-go`         | G1                  | 1 GiB  | `concurrent_cycle_in_idle`                                    |
| `microservice-zgc-stop-go`        | ZGC (generational)  | 1 GiB  | (smoother contrast; fewer concurrent-cycle markers)           |
| `epsilon-baseline`                | Epsilon (no-op)     | 2 GiB  | (negative case — no collection)                               |
| `epsilon-leak-pure`               | Epsilon (no-op)     | 256 MiB| `slow_leak`, `oom` (deterministic OOM time)                   |

## Use cases

**Developing a GC-Insight detector.** Identify the row whose
phenomenon the detector targets, run the corresponding presets
under faithful conditions, and use each manifest's
`expected_invariants` block as ground truth.

**Demonstrating an analyser capability.** Identify the entry whose
detector is to be exhibited, run the cited preset, and present the
log alongside the analyser output.

**Filing a regression.** If a preset stops exhibiting its declared
phenomenon between two GC-Forge releases, the issue is filed
against GC-Forge. If GC-Insight stops detecting a phenomenon that
GC-Forge still produces, the issue is filed against GC-Insight.
Cross-project regressions are tracked in coordinated pull requests.

## Maintenance

The matrix is reviewed at every minor release of either project.
Changes to GC-Forge that affect the first two columns require a
synchronised update to GC-Insight's expected detector behaviour;
changes to GC-Insight that affect the third column are merged with
the corresponding update to this document. The
`gc-core-roundtrip` integration test ([`SPEC-TECHNIQUE`
§7.4](../specs/SPEC-TECHNIQUE.md)) provides a coarse cross-project
check by parsing a GC-Forge log with the GC-Insight parser and
asserting structural agreement.
