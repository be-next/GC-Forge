//! Integration smoke for the three MVP algorithms.
//!
//! Runs `gc-forge run` against each `steady-*-baseline.yaml`, with the
//! duration overridden to a short value, and asserts the algorithm-specific
//! marker shows up in the GC log.
//!
//! Gated on the `docker-integration` feature so contributors without Docker
//! still get a green default `cargo test`. CI flips it on.

#![cfg(feature = "docker-integration")]

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

const RUNNER_IMAGE: &str = "gc-forge-runner:dev-jdk21";
const HARNESS_IN_IMAGE: &str = "/opt/gc-forge/harness.jar";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn cli_bin() -> PathBuf {
    // Cargo provides this env var for tests in a binary crate.
    PathBuf::from(env!("CARGO_BIN_EXE_gc-forge"))
}

fn run_baseline(preset: &str, marker: &str) {
    let root = workspace_root();
    let preset_path = root.join("presets").join(preset);
    assert!(
        preset_path.exists(),
        "preset missing: {}",
        preset_path.display()
    );

    let dir = tempdir().unwrap();
    let out_dir = dir.path();

    let status = Command::new(cli_bin())
        .args([
            "run",
            preset_path.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--image",
            RUNNER_IMAGE,
            "--embedded-harness",
            HARNESS_IN_IMAGE,
            "--override",
            "spec.duration=3s",
        ])
        .status()
        .expect("failed to invoke gc-forge");
    assert!(status.success(), "gc-forge run failed for {preset}");

    // Find the GC log emitted under out_dir.
    let log = locate_log(out_dir)
        .unwrap_or_else(|| panic!("no .log file produced under {}", out_dir.display()));
    let body = std::fs::read_to_string(&log).unwrap();
    assert!(
        body.contains(marker),
        "expected marker {marker:?} not found in GC log {} for preset {preset}",
        log.display()
    );
}

fn locate_log(dir: &Path) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("log") {
            return Some(path);
        }
    }
    None
}

#[test]
fn g1_baseline_emits_using_g1() {
    run_baseline("steady-g1-baseline.yaml", "Using G1");
}

#[test]
fn zgc_baseline_emits_using_zgc() {
    // Temurin 21 prints either "Using The Z Garbage Collector" (legacy line)
    // or "Using ZGC" (newer). Match the substring common to both.
    run_baseline("steady-zgc-baseline.yaml", "Using");
    // Tighter follow-up assertion: ZGC-specific "ZGC" or "Z Garbage" mention.
    let root = workspace_root();
    let dir = tempdir().unwrap();
    let status = Command::new(cli_bin())
        .args([
            "run",
            root.join("presets/steady-zgc-baseline.yaml")
                .to_str()
                .unwrap(),
            "--out-dir",
            dir.path().to_str().unwrap(),
            "--image",
            RUNNER_IMAGE,
            "--embedded-harness",
            HARNESS_IN_IMAGE,
            "--override",
            "spec.duration=3s",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let log = locate_log(dir.path()).expect("zgc log produced");
    let body = std::fs::read_to_string(&log).unwrap();
    assert!(
        body.contains("ZGC") || body.contains("Z Garbage"),
        "ZGC marker missing in {}",
        log.display()
    );
}

#[test]
fn parallel_baseline_emits_using_parallel() {
    run_baseline("steady-parallel-baseline.yaml", "Using Parallel");
}

#[test]
fn cache_g1_preset_runs_and_produces_log() {
    // R5 (cache-churn): we assert the regime is registered and the
    // pipeline produces a GC log starting with the G1 init banner. Behavioural
    // invariants (≥ 30 % promotion, etc.) are the validator's job (iter 13);
    // a 12-second integration run on a 4 GiB heap is too short to be
    // representative of the spec's signature.
    let root = workspace_root();
    let preset = root.join("presets/cache-g1-churn.yaml");
    let dir = tempdir().unwrap();
    let status = Command::new(cli_bin())
        .args([
            "run",
            preset.to_str().unwrap(),
            "--out-dir",
            dir.path().to_str().unwrap(),
            "--image",
            RUNNER_IMAGE,
            "--embedded-harness",
            HARNESS_IN_IMAGE,
            "--override",
            "spec.duration=12s",
        ])
        .status()
        .expect("failed to invoke gc-forge");
    assert!(status.success(), "gc-forge run failed for cache-g1-churn");
    let log = locate_log(dir.path()).expect("cache-g1 log produced");
    let body = std::fs::read_to_string(&log).unwrap();
    assert!(body.contains("Using G1"), "G1 marker missing");
    assert!(
        body.contains("Heap Region Size"),
        "Heap Region Size missing"
    );
}

#[test]
fn humongous_g1_preset_emits_humongous_marker() {
    let root = workspace_root();
    let preset = root.join("presets/humongous-g1-classic.yaml");
    let dir = tempdir().unwrap();
    let status = Command::new(cli_bin())
        .args([
            "run",
            preset.to_str().unwrap(),
            "--out-dir",
            dir.path().to_str().unwrap(),
            "--image",
            RUNNER_IMAGE,
            "--embedded-harness",
            HARNESS_IN_IMAGE,
            "--override",
            "spec.duration=12s",
        ])
        .status()
        .expect("failed to invoke gc-forge");
    assert!(
        status.success(),
        "gc-forge run failed for humongous-g1-classic"
    );
    let log = locate_log(dir.path()).expect("humongous log produced");
    let body = std::fs::read_to_string(&log).unwrap();
    // G1 logs humongous allocations on the `gc,humongous` tag (or via
    // "humongous regions" lines on `gc,heap`).
    assert!(
        body.contains("humongous"),
        "humongous marker missing in {}",
        log.display()
    );
}

#[test]
fn burst_g1_preset_runs_through_pipeline() {
    // R2 (allocation-burst) on G1. We override the duration down to a
    // value that still covers one full burst (5 s burst inside a 30 s
    // period — 12 s lets us see at least the burst onset and the start
    // of the recovery phase).
    let root = workspace_root();
    let preset = root.join("presets/burst-g1-30s.yaml");
    let dir = tempdir().unwrap();
    let status = Command::new(cli_bin())
        .args([
            "run",
            preset.to_str().unwrap(),
            "--out-dir",
            dir.path().to_str().unwrap(),
            "--image",
            RUNNER_IMAGE,
            "--embedded-harness",
            HARNESS_IN_IMAGE,
            "--override",
            "spec.duration=12s",
        ])
        .status()
        .expect("failed to invoke gc-forge");
    assert!(status.success(), "gc-forge run failed for burst-g1-30s");
    let log = locate_log(dir.path()).expect("burst-g1 log produced");
    let body = std::fs::read_to_string(&log).unwrap();
    assert!(
        body.contains("Using G1"),
        "G1 marker missing in {}",
        log.display()
    );
    // At least one young Pause entry — the burst should trigger one.
    assert!(
        body.contains("Pause Young"),
        "no Pause Young found in burst log {}",
        log.display()
    );
}
