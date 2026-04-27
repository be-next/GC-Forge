//! Translation from a resolved [`Scenario`] to a JVM command line.
//!
//! Reference: SPEC-TECHNICAL §6.1 (unified `-Xlog`) and §4.5 (algorithm flags).
//! Pure function — no I/O, no environment lookup, no OS-specific logic.

use std::path::Path;

use gc_forge_scenario::{GcAlgorithm, Scenario};

/// The `-Xlog` decorator suffix. Documents what each tag is captured for
/// and is intentionally identical across MVP runs so the GC-Insight parser
/// has a single shape to handle.
#[must_use]
pub const fn log_decorators() -> &'static str {
    "time,level,tags,pid,tid"
}

/// Builds the full `java …` argv (excluding the leading `java` itself) for a
/// resolved scenario. The `log_path` is the path the JVM will see — typically
/// a path *inside* the runner's container (e.g. `/work/gc.log`).
///
/// The output is layered:
///   1. JVM-level `extra_flags` (verbatim, in declaration order);
///   2. heap (`-Xms`, `-Xmx`, optionally `-Xmn`);
///   3. GC algorithm + algorithm-specific tunables;
///   4. GC-level `extra_flags` (verbatim);
///   5. the unified `-Xlog:gc*` directive aimed at `log_path`.
#[must_use]
pub fn build_jvm_command(scenario: &Scenario, log_path: &Path) -> Vec<String> {
    let mut argv: Vec<String> = Vec::new();
    let spec = &scenario.spec;

    // Layer 1: caller's JVM-level extras.
    argv.extend(spec.jvm.extra_flags.iter().cloned());

    // Layer 2: heap.
    argv.push(format!("-Xms{}", spec.gc.options.heap.min));
    argv.push(format!("-Xmx{}", spec.gc.options.heap.max));
    if let Some(new_size) = spec.gc.options.heap.new_size {
        argv.push(format!("-Xmn{new_size}"));
    }

    // Layer 3: GC algorithm + algorithm tunables.
    match spec.gc.algorithm {
        GcAlgorithm::G1 => {
            argv.push("-XX:+UseG1GC".to_owned());
            if let Some(ms) = spec.gc.options.pause_target_ms {
                argv.push(format!("-XX:MaxGCPauseMillis={ms}"));
            }
            if let Some(mb) = spec.gc.options.region_size_mb {
                argv.push(format!("-XX:G1HeapRegionSize={mb}m"));
            }
            if let Some(p) = spec.gc.options.ihop_percent {
                argv.push(format!("-XX:InitiatingHeapOccupancyPercent={p}"));
            }
        }
        GcAlgorithm::Zgc => {
            argv.push("-XX:+UseZGC".to_owned());
            // Generational ZGC is the default on JDK 21+. `None` means
            // "use the algorithm's natural default" (= true). An explicit
            // `false` disables generational mode for the non-generational
            // ZGC variant on JDK 21.
            match spec.gc.options.generational {
                None | Some(true) => argv.push("-XX:+ZGenerational".to_owned()),
                Some(false) => argv.push("-XX:-ZGenerational".to_owned()),
            }
        }
        GcAlgorithm::Parallel => {
            argv.push("-XX:+UseParallelGC".to_owned());
        }
        GcAlgorithm::Shenandoah => {
            argv.push("-XX:+UseShenandoahGC".to_owned());
            if let Some(ms) = spec.gc.options.pause_target_ms {
                argv.push(format!("-XX:MaxGCPauseMillis={ms}"));
            }
        }
        GcAlgorithm::Serial => {
            argv.push("-XX:+UseSerialGC".to_owned());
        }
        GcAlgorithm::Epsilon => {
            // Epsilon is an experimental no-op GC. It must be unlocked
            // before -XX:+UseEpsilonGC is accepted by the JVM.
            argv.push("-XX:+UnlockExperimentalVMOptions".to_owned());
            argv.push("-XX:+UseEpsilonGC".to_owned());
        }
    }

    // Layer 4: caller's GC-level extras.
    argv.extend(spec.gc.extra_flags.iter().cloned());

    // Layer 5: unified `-Xlog`.
    argv.push(format!(
        "-Xlog:gc*=info,gc+heap=debug,gc+age=trace,gc+phases=debug,gc+humongous=trace:file={}:{}:filecount=0",
        log_path.display(),
        log_decorators(),
    ));

    argv
}

#[cfg(test)]
mod tests {
    use super::*;

    use gc_forge_scenario::Scenario;

    fn scenario_yaml(algorithm: &str, extra: &str) -> String {
        format!(
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: test
spec:
  jvm: {{ vendor: temurin, major: 21 }}
  gc:
    algorithm: {algorithm}
    options:
      heap: {{ min: 1g, max: 2g }}
{extra}
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
"
        )
    }

    fn s(yaml: &str) -> Scenario {
        Scenario::from_yaml(yaml, "<test>").unwrap()
    }

    fn log_target() -> &'static Path {
        Path::new("/work/gc.log")
    }

    #[test]
    fn g1_emits_use_g1gc() {
        let argv = build_jvm_command(&s(&scenario_yaml("G1", "")), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseG1GC"));
        assert!(!argv.iter().any(|a| a.starts_with("-XX:+UseZGC")));
    }

    #[test]
    fn zgc_default_includes_generational() {
        let argv = build_jvm_command(&s(&scenario_yaml("ZGC", "")), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseZGC"));
        assert!(argv.iter().any(|a| a == "-XX:+ZGenerational"));
    }

    #[test]
    fn zgc_explicit_false_emits_negated_generational() {
        let yaml = scenario_yaml("ZGC", "      generational: false\n");
        let argv = build_jvm_command(&s(&yaml), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseZGC"));
        assert!(argv.iter().any(|a| a == "-XX:-ZGenerational"));
        assert!(!argv.iter().any(|a| a == "-XX:+ZGenerational"));
    }

    #[test]
    fn parallel_emits_use_parallelgc() {
        let argv = build_jvm_command(&s(&scenario_yaml("Parallel", "")), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseParallelGC"));
    }

    #[test]
    fn shenandoah_emits_use_shenandoahgc() {
        let argv = build_jvm_command(&s(&scenario_yaml("Shenandoah", "")), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseShenandoahGC"));
        assert!(!argv.iter().any(|a| a.contains("UseG1GC")));
    }

    #[test]
    fn shenandoah_honours_pause_target() {
        let yaml = scenario_yaml("Shenandoah", "      pause_target_ms: 50\n");
        let argv = build_jvm_command(&s(&yaml), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseShenandoahGC"));
        assert!(argv.iter().any(|a| a == "-XX:MaxGCPauseMillis=50"));
    }

    #[test]
    fn serial_emits_use_serialgc() {
        let argv = build_jvm_command(&s(&scenario_yaml("Serial", "")), log_target());
        assert!(argv.iter().any(|a| a == "-XX:+UseSerialGC"));
    }

    #[test]
    fn epsilon_unlocks_experimental_then_uses_epsilongc() {
        let argv = build_jvm_command(&s(&scenario_yaml("Epsilon", "")), log_target());
        let unlock = argv
            .iter()
            .position(|a| a == "-XX:+UnlockExperimentalVMOptions");
        let epsilon = argv.iter().position(|a| a == "-XX:+UseEpsilonGC");
        assert!(unlock.is_some(), "missing unlock flag: {argv:?}");
        assert!(epsilon.is_some(), "missing epsilon flag: {argv:?}");
        assert!(
            unlock.unwrap() < epsilon.unwrap(),
            "unlock must precede UseEpsilonGC: {argv:?}"
        );
    }

    #[test]
    fn heap_flags_are_present_in_order() {
        let argv = build_jvm_command(&s(&scenario_yaml("G1", "")), log_target());
        let xms = argv.iter().position(|a| a == "-Xms1g").unwrap();
        let xmx = argv.iter().position(|a| a == "-Xmx2g").unwrap();
        assert!(xms < xmx, "argv = {argv:?}");
    }

    #[test]
    fn heap_new_size_emits_xmn() {
        let yaml = scenario_yaml("Parallel", "").replace(
            "heap: { min: 1g, max: 2g }",
            "heap: { min: 1g, max: 2g, new_size: 256m }",
        );
        let argv = build_jvm_command(&s(&yaml), log_target());
        assert!(argv.iter().any(|a| a == "-Xmn256m"));
    }

    #[test]
    fn g1_knobs_only_when_set() {
        let plain = build_jvm_command(&s(&scenario_yaml("G1", "")), log_target());
        assert!(!plain.iter().any(|a| a.starts_with("-XX:MaxGCPauseMillis")));
        assert!(!plain.iter().any(|a| a.starts_with("-XX:G1HeapRegionSize")));
        assert!(!plain
            .iter()
            .any(|a| a.starts_with("-XX:InitiatingHeapOccupancyPercent")));

        let with_knobs = scenario_yaml(
            "G1",
            "      pause_target_ms: 50\n      region_size_mb: 4\n      ihop_percent: 35\n",
        );
        let argv = build_jvm_command(&s(&with_knobs), log_target());
        assert!(argv.iter().any(|a| a == "-XX:MaxGCPauseMillis=50"));
        assert!(argv.iter().any(|a| a == "-XX:G1HeapRegionSize=4m"));
        assert!(argv
            .iter()
            .any(|a| a == "-XX:InitiatingHeapOccupancyPercent=35"));
    }

    #[test]
    fn extra_flags_are_appended_in_order() {
        let yaml = r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: test
spec:
  jvm:
    vendor: temurin
    major: 21
    extra_flags: ['-XX:+AlwaysPreTouch', '-XX:+UseStringDeduplication']
  gc:
    algorithm: G1
    options:
      heap: { min: 1g, max: 2g }
    extra_flags: ['-XX:G1NewSizePercent=20']
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
";
        let argv = build_jvm_command(&s(yaml), log_target());

        let pre_touch = argv
            .iter()
            .position(|a| a == "-XX:+AlwaysPreTouch")
            .unwrap();
        let dedup = argv
            .iter()
            .position(|a| a == "-XX:+UseStringDeduplication")
            .unwrap();
        let new_pct = argv
            .iter()
            .position(|a| a == "-XX:G1NewSizePercent=20")
            .unwrap();
        let g1 = argv.iter().position(|a| a == "-XX:+UseG1GC").unwrap();

        // Order: jvm extras first, then heap, then algo (G1), then gc extras.
        assert!(pre_touch < dedup, "argv = {argv:?}");
        assert!(dedup < g1, "argv = {argv:?}");
        assert!(g1 < new_pct, "argv = {argv:?}");
    }

    #[test]
    fn xlog_flag_targets_the_provided_path() {
        let argv = build_jvm_command(&s(&scenario_yaml("G1", "")), log_target());
        let xlog = argv.iter().find(|a| a.starts_with("-Xlog:")).unwrap();
        assert!(xlog.contains("file=/work/gc.log"), "{xlog}");
        assert!(xlog.contains("time,level,tags,pid,tid"), "{xlog}");
        assert!(xlog.contains("filecount=0"), "{xlog}");
    }
}
