//! Integration smoke test that actually launches Docker.
//!
//! Gated behind the `docker-integration` feature so contributors without
//! Docker still get a green `cargo test`. CI flips it on (`make
//! docker-integration-tests` invokes it).

#![cfg(feature = "docker-integration")]

use std::path::PathBuf;
use std::time::Duration;

use gc_forge_runner::{DockerRunner, ExitStatus, RunSpec, Runner};
use gc_forge_scenario::Scenario;
use tempfile::tempdir;

fn scenario_yaml() -> &'static str {
    r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: docker-smoke
spec:
  jvm: { vendor: temurin, major: 21 }
  gc:
    algorithm: G1
    options:
      heap: { min: 256m, max: 256m }
  regime:
    kind: steady-state-healthy
  duration: 3s
  seed: 0xC0FFEE
"
}

fn harness_jar() -> PathBuf {
    // Built once by the workspace `make build` target; the integration
    // pipeline ensures it exists.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("workload-harness/target/workload-harness.jar")
}

#[test]
fn docker_runner_produces_a_gc_log() {
    // Use the runner image we bake locally (`gc-forge-runner:dev-jdk21`)
    // which embeds the harness — that sidesteps Docker Desktop on macOS
    // limitations around mounting nested temp paths read-only.
    let runner = DockerRunner::new()
        .with_image("gc-forge-runner:dev-jdk21")
        .with_embedded_harness("/opt/gc-forge/harness.jar");
    runner
        .check_available()
        .expect("Docker required for the docker-integration test set");

    let jar = harness_jar();
    assert!(
        jar.exists(),
        "build the harness jar first: `make build` (looked at {})",
        jar.display()
    );

    let dir = tempdir().unwrap();
    let log_path = dir.path().join("gc.log");

    let spec = RunSpec {
        scenario: Scenario::from_yaml(scenario_yaml(), "<test>").unwrap(),
        log_path: log_path.clone(),
        harness_jar: jar,
        // duration, allocation rate (mb/s), live-set (mb), seed
        workload_args: vec![
            "PT3S".to_owned(),
            "20".to_owned(),
            "20".to_owned(),
            "42".to_owned(),
        ],
        cpu_limit: Some(1.0),
        memory_limit: None,
    };

    let outcome = runner.execute(&spec).expect("docker run failed");

    assert_eq!(outcome.exit_status, ExitStatus::Success);
    assert!(outcome.log_path.exists(), "log file missing");
    let log = std::fs::read_to_string(&outcome.log_path).unwrap();
    assert!(
        log.contains("Using G1"),
        "did not see Using G1; log = {log}"
    );
    assert!(log.contains("[gc"), "no gc tags in log; log = {log}");

    // Sanity-check the wall-clock window.
    let elapsed = outcome
        .ended_at
        .duration_since(outcome.started_at)
        .unwrap_or_default();
    assert!(elapsed >= Duration::from_secs(1), "elapsed = {elapsed:?}");
    assert!(elapsed <= Duration::from_secs(60), "elapsed = {elapsed:?}");
}
