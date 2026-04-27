# GC-Forge — functional specification

> **Reference**: brief of 25 April 2026, §3, §4.
> **Status**: V1.0 — issued by Cowork. Translated to English and
> resynchronised with the 0.1.0 implementation on 2026-04-27.

## 1. Vision and positioning

GC-Forge is a **declarative** and **reproducible** tool that
produces, on demand, Java GC logs representative of chosen
scenarios, crossing three axes:

```
JVM (vendor, version)  ×  GC algorithm  ×  Application regime
```

The user describes a scenario in YAML, GC-Forge runs a real JVM
with a parameterised Java workload, and captures the collector's
native output. The output is a **GC log indistinguishable from a
production log**, accompanied by a **manifest** that records all
parameters needed for bit-for-bit traceability.

GC-Forge is **not** a simulator: it does not synthesise a fake
log from a statistical model. It is also not a replacement for an
application benchmark: it does not measure application
performance, it exhibits **GC behaviour**.

## 2. Detailed use cases

The five use cases of the brief, sized into concrete
requirements:

### 2.1 Testing and validating GC-Insight (P0)

**Personas**: Jérôme (Rust developer of GC-Insight), GC-Insight CI.

**Needs**:
- Produce a **reference corpus** (logs + ground truth) that is
  versionable and replayable.
- At each GC-Insight release, replay the corpus and compare
  produced analyses with the expected phenomena recorded in the
  manifest.
- Detect parsing regressions (real JVM logs, not synthetic).
- Measure heuristic precision on a stable dataset.

**Functional requirements**:
- F-VAL-1: `gc-forge batch --matrix corpus-reference.yaml`
  regenerates a complete corpus deterministically.
- F-VAL-2: Each generated log is accompanied by a manifest
  containing `expected_phenomena` (cf. §6).
- F-VAL-3: The SHA-256 of the log is stable across identical
  runs (modulo wall-clock timestamps — see §4 reproducibility).
- F-VAL-4: GC-Forge exposes a command `gc-forge validate <log>
  <manifest>` that re-verifies after the fact that the log meets
  the regime invariants (a safety net against the inherent
  non-reproducibility of real GC timing).

### 2.2 Sales demonstrations (P0)

**Personas**: GC-Insight sales/SE, prospect.

**Needs**:
- A **ready-to-use catalogue**: one log per analytical capability
  to demonstrate.
- Short logs (90 s to 5 min) but "speaking": the phenomenon to
  exhibit must be visible without scrolling 50 MB of log.
- Educational identity card shipped with each preset.

**Requirements**:
- F-DEMO-1: `gc-forge presets list` displays the catalogue with a
  one-line description.
- F-DEMO-2: `gc-forge presets show <name>` displays the
  educational identity card (regime, JVM, algorithm, expected
  phenomenon, what GC-Insight should reveal).
- F-DEMO-3: `gc-forge run --preset <name>` produces the log in
  under 5 minutes for every MVP preset.

### 2.3 Pedagogy and content (P1)

**Personas**: Jérôme writing a blog article, trainer.

**Needs**:
- Output logs and commented excerpts.
- Ability to vary parameters around a preset ("what if we doubled
  the heap?").
- Captures of phenomenology at multiple resolutions (truncated
  log for an example, full log for download).

**Requirements**:
- F-PED-1: `gc-forge run --preset <name> --override
  'gc.heap.max=4g'` enables ad-hoc tweaks.
- F-PED-2: a `--snippet <event_kind>` option that produces a log
  excerpt centred on an event type (e.g. `--snippet
  humongous_allocation`).

### 2.4 User-side pathology reproduction (P1)

**Personas**: GC-Insight user who has observed a pathological
behaviour in production.

**Needs**:
- Describe their context (JVM, flags, heap, observed behaviour)
  and obtain a mirror scenario to run.
- Compare their production log and the GC-Forge log on the same
  axes.

**Requirements (V1, out of MVP)**:
- F-REPRO-1: `gc-forge mirror <prod.log>` proposes a minimal YAML
  scenario consistent with the detected flags and observed
  phenomenology. Simple heuristic in V1 (flags → JVM + algorithm
  mapping, coarse regime classification).

### 2.5 Datasets for ML (P2)

**Personas**: Jérôme training a regime classifier or an anomaly
detector.

**Needs**:
- Volume: thousands of logs of a few minutes each.
- Automatic labelling (label = regime, sub-label = key
  parameters).
- Controlled variability (different seeds, parameters drawn from
  a range).

**Requirements (V2)**:
- F-ML-1: `gc-forge sweep <template.yaml> --count 1000
  --seed-range 1..1000` generates a parametric dataset.
- F-ML-2: Index export (CSV / Parquet) listing all files and
  their labels.

## 3. Catalogue of JVMs and GC algorithms

### 3.1 Supported JVMs

| Vendor | Distribution | MVP versions | V1 versions | V2 versions |
|--------|--------------|--------------|-------------|-------------|
| Eclipse Adoptium | Temurin | 17, 21 | 11 (legacy), 23 | — |
| Amazon | Corretto | — | 17, 21 | 11, 23 |
| Oracle | GraalVM CE | — | 21 (Native Image out of scope) | 23 |
| Eclipse | OpenJ9 | — | 17, 21 | — |
| Azul | Zing / Prime | — | — | 21 (subject to licence) |
| Oracle | HotSpot Oracle JDK | — | — | 21 (subject to licence — TCK) |

**MVP**: Temurin only, JDK 17 and 21. Justification:
- Temurin is free, redistributable, documented, and offers
  up-to-date official Docker images
  `eclipse-temurin:{17,21}-jdk-jammy`.
- 95 % of production GC-log use cases today are on HotSpot — cover
  the majority target first.
- 17 and 21 are the current LTS releases; 17 is still widely
  installed, 21 is gaining ground.
- OpenJ9 and Zing have **different GC log formats** from HotSpot:
  adding them in V1/V2 will require dedicated parsing/validation
  work on the GC-Insight side, hence to be coordinated.

### 3.2 GC algorithms

Compatibility per JVM (HotSpot/Temurin only for MVP):

| Algorithm | JVM flag | JDK 17 | JDK 21 | MVP | Notes |
|-----------|----------|--------|--------|-----|-------|
| G1 | `-XX:+UseG1GC` (default ≥9) | ✓ | ✓ | **✓** | Primary target, the best documented in terms of pathologies. |
| ZGC | `-XX:+UseZGC` | ✓ (non-generational) | ✓ + **`-XX:+ZGenerational`** | **✓** (both modes) | Generational ZGC GA in JDK 21 — exposed at MVP. The non-generational variant is selectable on JDK 21 via `generational: false`. |
| Parallel | `-XX:+UseParallelGC` | ✓ | ✓ | **✓** | Throughput-oriented, relevant for batch. |
| Shenandoah | `-XX:+UseShenandoahGC` | ✓ (Temurin) | ✓ | **✓** | Originally V1; promoted to MVP. Concurrent low-latency like ZGC, distinct behaviour. |
| Serial | `-XX:+UseSerialGC` | ✓ | ✓ | **✓** | Originally V1; promoted to MVP. Niche (tiny containers) but useful for pedagogy. |
| Epsilon | `-XX:+UnlockExperimentalVMOptions -XX:+UseEpsilonGC` | ✓ | ✓ | **✓** | No-op collector. Negative-baseline use case (false-positive guard for analysers) and deterministic OOM scenarios. |
| CMS | `-XX:+UseConcMarkSweepGC` | ✗ (removed) | ✗ | — | **V2 on JDK 8** only, for historical cases. |

**Justification of the MVP six-collector scope**:
- G1 = default collector, the most encountered in production, the
  richest in pathologies to exhibit.
- Generational ZGC = upcoming default, low-latency, hot topic
  2025–2026.
- Parallel = throughput, useful contrast with G1/ZGC on
  batch/compute regimes.
- Shenandoah = Red Hat / OpenJDK ecosystem; same parser as the
  others (unified `-Xlog:gc*`).
- Serial = single-threaded baseline; pedagogical contrast and
  small-container support.
- Epsilon = no-op; provides a *negative* baseline (no detection
  expected) and a *deterministic* OOM time on leak scenarios for
  detector calibration.

OpenJ9, Corretto, GraalVM remain V1: not a technical blocker,
just marginal effort to delay until the architecture is stable.

## 4. Catalogue of application regimes

A **regime** is the behavioural signature of a workload from the
GC point of view: allocation rate, lifetime distribution,
temporal pattern, object size.

For each regime we define:
- **Parameters**: what the user can tune.
- **Expected signature**: what the log must exhibit (quantified
  invariants — see §7).
- **GC-Insight use case**: what the analytical capability must
  reveal.

### Summary of MVP regimes

| # | Regime | Key parameters | Expected GC signature (G1) |
|---|--------|----------------|----------------------------|
| R1 | `steady-state-healthy` | `allocation_rate_mb_s`, `live_set_mb` | Regular young GCs, p99 pause < 50 ms, 0 pathological mixed, 0 full. |
| R2 | `allocation-burst` | `base_rate_mb_s`, `burst_rate_mb_s`, `burst_duration_s`, `burst_period_s` | Young frequency pulses with the burst rhythm, no evacuation failure if well-dimensioned. |
| R3 | `humongous-pressure` | `humongous_ratio`, `humongous_size_kb`, `region_size_mb` | Humongous regions visible, mixed GC triggered early, pressure on old gen. |
| R4 | `slow-leak` | `leak_rate_mb_s`, `live_set_initial_mb` | After-GC footprint grows linearly, mixed becomes more frequent, full GC or OOM at end of run. |
| R5 | `cache-churn` | `cache_size_mb`, `eviction_rate_per_s`, `entry_lifetime_ms` | High promotion rate to old, frequent mixed GCs, fluctuating old-gen size. |
| R6 | `mixed-gc-pathological` | `old_gen_pressure`, `fragmentation_factor` | Inefficient mixed GC (low reclaim), mixed duration grows, IHOP descends. |
| R7 | `microservice-stop-and-go` | `active_period_s`, `idle_period_s`, `active_rate_mb_s` | Alternation of young bursts / silence, possible concurrent cycles in idle. |

### 4.1 R1 — `steady-state-healthy` (reference)

**Parameters**:
- `allocation_rate_mb_s` (default: 50) — young-generation
  allocation throughput.
- `live_set_mb` (default: 100) — stable size of the live set.
- `object_size_distribution`: `small` (16–256 B) / `medium`
  (256–4 KB) / `mixed` (default).
- `lifetime_distribution`: `short` (dies in young), `mixed` (10 %
  promoted).

**Expected GC signature**:
- ≥ 80 % of collections are young.
- 0 full GC, 0 pathological mixed GC.
- p99 (young pause) < 50 ms at heap = 2 GB,
  `allocation_rate = 50 MB/s`.
- Inter-run variance on the total pause count ≤ 5 %.

**GC-Insight use case**: "all good" reference log — serves as a
baseline for anomaly detection.

### 4.2 R2 — `allocation-burst`

**Parameters**:
- `base_rate_mb_s` (default: 30).
- `burst_rate_mb_s` (default: 200).
- `burst_duration_s` (default: 5).
- `burst_period_s` (default: 30).
- `bursts_count` (default: `auto` = `duration / burst_period`).

**Signature**:
- Young-GC frequency pulses with the burst rhythm (visible on a
  trace).
- After a burst exits, frequency returns to baseline within
  ≤ 2× `burst_duration_s`.
- No evacuation failure if `burst_rate_mb_s` ≤
  `young_capacity / pause_young`.

**GC-Insight use case**: demonstrate burst detection by
GC-Insight, and the correlation with pauses.

### 4.3 R3 — `humongous-pressure`

**Parameters**:
- `humongous_ratio` (0.0–1.0, default: 0.5) — fraction of
  allocations that are humongous.
- `humongous_size_kb` (default: auto = 2 MiB, comfortably above
  G1's region size for any heap up to 32 GiB).
- `region_size_mb` (default: auto, depends on heap).
- `allocation_rate_mb_s` (default: 80).

**Signature**:
- ≥ 1 humongous allocation per second in the log.
- Humongous regions visible in the G1 log (`humongous regions: N`).
- Mixed GC triggered even though `IHOP` is not reached (humongous
  forces marking).
- If `humongous_ratio > 0.7`: possibility of evacuation failure.

**GC-Insight use case**: demonstrate GC-Insight's ability to
isolate humongous allocations as a root cause.

### 4.4 R4 — `slow-leak`

**Parameters**:
- `leak_rate_mb_s` (default: 0.5) — live-set growth.
- `live_set_initial_mb` (default: 200).
- `duration` (default: `auto` until OOM or 30 min).

**Signature**:
- After-GC live set grows with slope ≈ `leak_rate_mb_s × T`.
- Mixed-GC frequency grows with time.
- Full GC (or OOM if fail-fast) at end of run.
- Mean pause grows monotonically (Pearson correlation > 0.7
  with t).

**GC-Insight use case**: demonstrate memory-leak detection by
GC-Insight (linear regression on after-GC footprint).

### 4.5 R5 — `cache-churn`

**Parameters**:
- `cache_size_mb` (default: 500).
- `eviction_rate_per_s` (default: 1000) — entries evicted per
  second.
- `entry_lifetime_ms` (default: 2000) — typical entry lifetime.
- `entry_size_kb` (default: 8).

**Signature**:
- Promotion rate (young → old) ≥ 30 % of young throughput.
- Old gen oscillates between `cache_size × 0.8` and
  `cache_size × 1.2`.
- Mixed GC frequent but regular (not pathological).

**GC-Insight use case**: demonstrate the "cache" vs "leak"
signature — distinguish bounded growth from unbounded growth.

### 4.6 R6 — `mixed-gc-pathological`

**Parameters**:
- `old_gen_pressure` (0.0–1.0, default: 0.7) — fraction of the
  heap occupied by old.
- `fragmentation_factor` (1.0–3.0, default: 2.0) — multiplier on
  the simulated fragmentation.
- `survivor_age_target` (default: 15) — resistant survivors.

**Signature**:
- Mixed GC duration grows over time.
- Reclaim per mixed GC < 5 % of the heap.
- Effective IHOP descends (G1 ergonomic adjustment).
- Possible evacuation failure → full GC at end of run.

**GC-Insight use case**: a notable pathological case where the
root cause is not obvious — demonstrates GC-Insight's diagnostic
value.

### 4.7 R7 — `microservice-stop-and-go`

**Parameters**:
- `active_period_s` (default: 10).
- `idle_period_s` (default: 20).
- `active_rate_mb_s` (default: 100).
- `cycles` (default: `auto`).

**Signature**:
- Idle periods with ≤ 1 young GC / 10 s.
- Concurrent mark cycles triggered in idle (G1).
- With ZGC: smoother, less visible signature — pedagogical in
  itself.

**GC-Insight use case**: show the contrast in behaviour between
collectors on a "modern" microservice workload.

### 4.8 Exploratory regime (V1) — `compute-batch`

To be documented in V1: a "batch compute" regime with massive
temporary allocations, interesting to contrast Parallel vs G1.

## 5. Scenario model (YAML schema)

### 5.1 `gc-forge/scenario.v1` schema

```yaml
apiVersion: gc-forge/scenario.v1
kind: Scenario

metadata:
  name: <string, slug, required>
  version: <semver, default "1.0.0">
  description: <string, optional>
  tags: [<string>, ...]
  authors: [<string>, ...]

spec:
  # Axis 1: JVM
  jvm:
    vendor: temurin | corretto | graalvm | openj9   # MVP: temurin
    major: 17 | 21                                  # MVP
    distribution: jdk | jre                         # default: jdk
    extra_flags: [<string>, ...]                    # non-GC flags

  # Axis 2: GC algorithm + heap
  gc:
    algorithm: G1 | ZGC | Parallel | Shenandoah | Serial | Epsilon
    options:
      generational: true | false                    # ZGC only; default true
      heap:
        min: <size>                                 # e.g. "2g", "512m"
        max: <size>                                 # e.g. "2g"
        new_size: <size, optional>                  # G1: rarely useful, Parallel: relevant
      pause_target_ms: <int, optional>              # G1, Shenandoah: -XX:MaxGCPauseMillis
      region_size_mb: <int, optional>               # G1: -XX:G1HeapRegionSize
      ihop_percent: <int, optional>                 # G1
    extra_flags: [<string>, ...]                    # custom GC flags
    log_format: unified | legacy                    # MVP: unified (-Xlog:gc*)

  # Axis 3: application regime
  regime:
    kind: <string>                                  # see §4
    parameters: { ... }                             # see §4 per regime

  # Execution control
  duration: <duration>                              # e.g. "90s", "5m"
  warmup: <duration>                                # default: "10s"
  seed: <hex|int>                                   # required for reproducibility

  # Output
  output:
    log_path: <path, optional>                      # default: <name>-<seed>.log
    manifest_path: <path, optional>                 # default: <name>-<seed>.manifest.yaml
    capture_jfr: <bool>                             # default: false (V1+)

  # Post-run validation
  expected:
    phenomena: [<phenomenon-id>, ...]               # used by `validate`
    invariants: [<invariant-rule>, ...]             # additional custom rules
```

### 5.2 Validation

- The schema is expressed in **JSON Schema** (generated from Rust
  via `schemars`) and published at `schemas/scenario-v1.json`.
- `gc-forge lint <scenario.yaml>` validates the scenario without
  executing it.
- Explicit validation errors: `gc.algorithm: "Z" not in {G1, ZGC,
  Parallel, Shenandoah, Serial, Epsilon}; available algorithms
  for jvm.major=17: G1, ZGC, Parallel, Shenandoah, Serial,
  Epsilon`.

### 5.3 Composition and inheritance

To limit duplication between similar scenarios:

```yaml
extends: presets/g1-baseline.yaml
spec:
  regime:
    kind: humongous-pressure
    parameters:
      humongous_ratio: 0.7
```

Inheritance is resolved at load time (recursive map merge, scalar
override).

### 5.4 CLI override

`--override` accepts JSONPath-style paths:

```bash
gc-forge run scenario.yaml \
  --override 'spec.gc.options.heap.max=4g' \
  --override 'spec.regime.parameters.humongous_ratio=0.8'
```

## 6. Output formats

### 6.1 Raw log (top priority)

The GC log is **strictly** the collector's native output, captured
without transformation:
- HotSpot JDK 9+: Unified Logging
  (`-Xlog:gc*=info,gc+heap=debug,gc+age=trace:file=<path>:time,level,tags`).
- The tag format is fixed by GC-Forge for consistency with
  GC-Insight (cf. TECHNICAL §6).
- A log produced by GC-Forge must be **indistinguishable** from a
  log produced by a real Java application with the same flags.
  This is the non-negotiable invariant.

### 6.2 Manifest — `gc-forge/run-manifest.v1` schema

Emitted alongside each log. YAML format by default, JSON via
`--manifest-format=json`.

```yaml
apiVersion: gc-forge/run-manifest.v1
kind: RunManifest

# Run identity
run:
  id: <uuid v7>
  started_at: <ISO 8601>
  ended_at: <ISO 8601>
  duration_actual: <duration>
  exit_status: success | failure | oom | timeout
  host:
    os: <linux|macos>
    arch: <x86_64|aarch64>
    cpu_count: <int>
    container: docker:<image-tag> | native

# Link with the scenario
scenario:
  source_path: <path>
  source_sha256: <hex>
  resolved: { ... }              # post-merge/override snapshot

# Real JVM configuration
jvm:
  vendor: temurin
  version: <full version, e.g. "21.0.2+13">
  flags: [<string>, ...]         # flags effectively passed to the JVM

# Reproducibility
reproducibility:
  seed: <hex>
  workload_jar_sha256: <hex>     # hash of the harness JAR used
  gc_forge_version: <semver+git_sha>

# Output
output:
  log_path: <path>
  log_sha256: <hex>
  log_size_bytes: <int>

# Ground truth
expected_phenomena: [<phenomenon-id>, ...]
expected_invariants: [{ rule: <string>, threshold: <value> }, ...]

# Post-run validation (filled by `validate`)
validation:
  status: passed | failed | skipped
  results: [{ rule, observed, threshold, passed: bool }, ...]
  validated_at: <ISO 8601>
  validator_version: <semver>
```

### 6.3 `phenomenon-id` catalogue

Controlled, versioned list, aligned with what GC-Insight is
expected to detect:

| ID | Description |
|----|-------------|
| `young_gc_steady` | Stable young frequency. |
| `allocation_burst` | Detectable allocation pulses. |
| `humongous_allocation` | Humongous allocations present. |
| `evacuation_failure` | At least one evacuation failure. |
| `mixed_gc_efficient` | Mixed GC reclaiming > 30 % of the targeted heap. |
| `mixed_gc_pathological` | Mixed GC reclaiming < 5 % of the heap. |
| `slow_leak` | After-GC live set grows linearly. |
| `full_gc` | At least one full GC. |
| `oom` | OutOfMemoryError. |
| `concurrent_cycle_in_idle` | Concurrent cycle triggered during an idle period. |
| `promotion_pressure` | Young → old promotion rate above threshold. |
| `no_collection` | Negative case: no GC pause, no collection event. Exhibited only by Epsilon presets; serves as a false-positive guard for downstream analysers. |

### 6.4 Batch index

`gc-forge batch` additionally produces an `index.csv` file (and
`index.parquet` in V1) listing all runs with their labels —
targeting the ML use case.

## 7. Quality criteria for a generated log

A log is **valid** if it meets the quantified invariants of the
regime that produced it. Three mechanisms:

### 7.1 Per-regime invariants (rules-as-code)

Every regime exposes a function `validate(log, params) ->
Result<ValidationReport>` on the Rust side. Indicative example
for `R1 steady-state-healthy`:

```rust
// gc-forge-regimes/src/steady_state.rs (indicative shape)
fn validate(parsed: &ParsedLog, params: &SteadyStateParams) -> ValidationReport {
    rule!("young_ratio >= 0.8", parsed.young_count as f64 / parsed.total_count as f64);
    rule!("full_count == 0", parsed.full_count);
    rule!("p99_pause_ms < 50", parsed.young_pause_p99_ms());
    rule!("variance_pause_count_pct < 5", parsed.cv_pause_count() * 100.0);
}
```

The manifest's `expected_invariants` are populated from these
rules.

### 7.2 Validation suite replayed at every release

`gc-forge selftest` runs every shipped preset and verifies that
each one passes its invariants. Blocks CI on failure. Target:
100 % of presets pass on every commit to `main`.

### 7.3 Inter-run variance

A scenario replayed N times (same seed, same JVM, same host)
must produce logs with **close** aggregated metrics. Two
tolerance levels per metric category:

| Metric category | Target | Acceptable tolerance | Blocking ceiling |
|-----------------|--------|----------------------|------------------|
| Counts (n young, n mixed) | CV ≤ 5 % | CV ≤ 8 % | CV > 10 % ⇒ regime not qualified |
| Sums (total duration, bytes allocated) | CV ≤ 5 % | CV ≤ 8 % | CV > 10 % ⇒ regime not qualified |
| Means (after-GC heap, mean pause) | CV ≤ 5 % | CV ≤ 10 % | CV > 12 % |
| Extreme percentiles (p99, max) | CV ≤ 15 % | CV ≤ 20 % | CV > 25 % |

The asymmetry between aggregated metrics and extreme percentiles
reflects the inherent non-determinism of real GC timing — a
single outlier can move a p99 by 30 % without changing the
overall regime signature.

**Fallback strategy** (cf. RISKS R-T1): if the 5 % target is not
attainable on a given regime after 2 stabilisation iterations,
the ≤ 8 % tolerance is accepted, the gap is documented, and the
related invariants are loosened. If even the tolerance is not
met, the regime is marked "experimental" and excluded from
`selftest`.

`gc-forge variance-check <scenario.yaml> --runs 10` automates
this measurement to qualify a new regime.

### 7.4 Bit-for-bit reproducibility — limits

Bit-for-bit log reproducibility is not attainable on a real JVM
(GC timing depends on the OS scheduler, CPU frequency, system
load). What is guaranteed instead:
- Reproducibility of the **resolved manifest** (configuration +
  workload + seed).
- **Semantic** reproducibility of the log: same phenomena, same
  orders of magnitude, invariants respected.
- Stable SHA-256 of the **Java harness** and of the resolved
  scenario (but not of the log).

## 8. Shipped MVP presets

The MVP ships **21 presets** (originally scoped at 14, expanded
to cover Shenandoah, Serial, Epsilon, and the non-generational
ZGC variant when those collectors were promoted from V1 to MVP).
All presets target JDK 21 by default; the `--jvm.major=17`
override switches to JDK 17.

| ID | Regime | Algorithm | Heap | Duration | Phenomenon |
|----|--------|-----------|------|----------|------------|
| `steady-g1-baseline` | R1 | G1 | 2 GB | 90 s | `young_gc_steady` |
| `steady-zgc-baseline` | R1 | ZGC (gen) | 2 GB | 90 s | `young_gc_steady` |
| `steady-zgc-nongen-baseline` | R1 | ZGC (non-gen) | 2 GB | 90 s | `young_gc_steady` |
| `steady-parallel-baseline` | R1 | Parallel | 2 GB | 90 s | `young_gc_steady` |
| `steady-shenandoah-baseline` | R1 | Shenandoah | 2 GB | 90 s | `young_gc_steady` |
| `steady-serial-baseline` | R1 | Serial | 256 MB | 90 s | `young_gc_steady` |
| `burst-g1-30s` | R2 | G1 | 2 GB | 5 min | `allocation_burst` |
| `burst-parallel-30s` | R2 | Parallel | 2 GB | 5 min | `allocation_burst` |
| `humongous-g1-classic` | R3 | G1 | 2 GB | 2 min | `humongous_allocation`, `mixed_gc_efficient` |
| `humongous-g1-evac-fail` | R3 | G1 | 1 GB | 2 min | `humongous_allocation`, `evacuation_failure` |
| `leak-g1-slow` | R4 | G1 | 1 GB | 10 min | `slow_leak`, `full_gc` |
| `leak-zgc-slow` | R4 | ZGC (gen) | 1 GB | 10 min | `slow_leak` |
| `leak-shenandoah-slow` | R4 | Shenandoah | 1 GB | 10 min | `slow_leak` |
| `cache-g1-churn` | R5 | G1 | 4 GB | 5 min | `promotion_pressure` |
| `cache-parallel-churn` | R5 | Parallel | 4 GB | 5 min | `promotion_pressure` |
| `cache-serial-churn` | R5 | Serial | 1 GB | 5 min | `promotion_pressure` |
| `mixed-pathological-g1` | R6 | G1 | 2 GB | 5 min | `mixed_gc_pathological` |
| `microservice-g1-stop-go` | R7 | G1 | 1 GB | 5 min | `concurrent_cycle_in_idle` |
| `microservice-zgc-stop-go` | R7 | ZGC (gen) | 1 GB | 5 min | (contrast) |
| `epsilon-baseline` | R1 | Epsilon | 2 GB | 60 s | `no_collection` (negative case) |
| `epsilon-leak-pure` | R4 | Epsilon | 256 MB | 5 min | `slow_leak`, `oom` (deterministic) |

**Educational bonus matrices** (out of the 21-preset count):
- `compare-young-pause-g1-vs-zgc`: a matrix that produces both
  baselines side-by-side.
- `humongous-region-size-sweep`: 4 runs with `region_size` ∈
  {1, 2, 4, 8} MB, exhibiting the parameter's effect.

## 9. Execution modes

### 9.1 Single mode

```bash
gc-forge run scenario.yaml [--override KEY=VAL]... [--out-dir DIR]
gc-forge run --preset humongous-g1-classic
```

### 9.2 Batch mode

```bash
gc-forge batch matrix.yaml [--parallel N] [--out-dir DIR]
```

`matrix.yaml` example:

```yaml
apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: presets/g1-baseline.yaml
  axes:
    jvm.major: [17, 21]
    gc.algorithm: [G1, ZGC, Parallel]
    regime.kind: [steady-state-healthy, allocation-burst, humongous-pressure]
  seeds: [1, 2, 3]
  filters:                       # exclude illegal combinations
    - { gc.algorithm: ZGC, regime.kind: humongous-pressure }   # ZGC has no humongous specifics
```

Produces: `out/index.csv` + one log/manifest file per cell.

### 9.3 Validate mode

```bash
gc-forge validate <log> --manifest <manifest.yaml>
```

Re-verifies the expected invariants after the fact. Non-zero
exit code on failure.

### 9.4 Lint mode

```bash
gc-forge lint scenario.yaml
```

Syntactic and semantic validation without execution (checks
JVM/algorithm compatibility, regime/algorithm coherence,
parameter ranges).

### 9.5 Presets mode

```bash
gc-forge presets list [--regime R3] [--algo G1]
gc-forge presets show humongous-g1-classic
gc-forge presets export <name> > my-scenario.yaml   # extract to tweak
```

### 9.6 Variance-check mode

```bash
gc-forge variance-check scenario.yaml --runs 10 [--report.html]
```

The `--report.html` option is V1.

## 10. CLI interface — global conventions

- Machine output via `--output json` on every subcommand
  (reserved for V1).
- Return codes: 0 success, 1 user error (invalid YAML, missing
  JVM), 2 JVM execution error, 3 invariant violation, 4 internal
  error.
- Verbosity: `-v`, `-vv`, `-vvv`; `--quiet` for silence except on
  errors.
- Auto colours, `NO_COLOR` honoured.
- Conventions consistent with the GC-Insight CLI (same
  cross-cutting flags).

## 11. Explicitly out of scope

- No **synthetic** log generation (reserved for V2 for extreme
  cases).
- No application performance measurement (business latency, RPS
  throughput) — GC-Forge is not a benchmark.
- No multi-JVM simulation or distributed workloads — one
  scenario = one JVM process.
- No support for non-Java JVM languages (Kotlin, Scala) — no
  interest for GC-log production.
- No graphical UI — CLI only at least until V1.

---

**Consistency with GC-Insight**: every `phenomenon-id` produced
by GC-Forge must be detectable by GC-Insight, and conversely,
GC-Insight's analytical capabilities must be demonstrable by at
least one GC-Forge preset. This traceability is recorded in a
`phenomenon × preset × insight-capability` matrix maintained at
[`doc/concepts/traceability.md`](../concepts/traceability.md).
