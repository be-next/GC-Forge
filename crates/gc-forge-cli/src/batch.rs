//! `gc-forge batch <matrix.yaml>` — runs a matrix and writes an index CSV.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use clap::Args;
use thiserror::Error;
use uuid::Uuid;

use gc_forge_regimes::{resolve as resolve_regime, RegimeError};
use gc_forge_runner::{
    flags::build_jvm_command, DockerRunner, ExitStatus, RunOutcome, RunSpec, Runner, RunnerError,
};
use gc_forge_scenario::{
    sha256_hex, sha256_hex_bytes, ExitStatusRecord, ExpectedInvariantRecord, HostMeta, JvmRecord,
    JvmVendor, Matrix, OutputRecord, Override, ReproducibilityRecord, RunManifest, RunMeta,
    Scenario, ScenarioError, ScenarioRecord,
};

#[derive(Debug, Args)]
pub struct BatchArgs {
    /// Path to the matrix YAML.
    pub matrix: PathBuf,

    /// Output directory.
    #[arg(long = "out-dir", value_name = "DIR", default_value = "out")]
    pub out_dir: PathBuf,

    /// Override the Docker image tag for every cell.
    #[arg(long = "image", value_name = "TAG")]
    pub image: Option<String>,

    /// `--embedded-harness` for every cell.
    #[arg(long = "embedded-harness", value_name = "PATH_IN_IMAGE")]
    pub embedded_harness: Option<String>,

    /// `--harness-jar` (host path) for every cell.
    #[arg(long = "harness-jar", value_name = "PATH")]
    pub harness_jar: Option<PathBuf>,

    /// Stop on the first failing cell. Default: stop. Pass `--continue-on-error`
    /// to keep going (useful for the corpus regen pipeline).
    #[arg(long = "continue-on-error")]
    pub continue_on_error: bool,
}

#[derive(Debug, Error)]
pub enum BatchError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error(transparent)]
    Regime(#[from] RegimeError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
    #[error("scenario rejected an unsupported jvm vendor: {vendor:?}")]
    UnsupportedVendor { vendor: String },
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Entry point for `gc-forge batch`. Returns the number of cells that failed.
///
/// # Errors
/// Surfaces I/O and scenario-loading errors. Per-cell run failures are tallied
/// in the returned count rather than raised.
pub fn execute(args: &BatchArgs) -> Result<u32, BatchError> {
    let matrix = Matrix::from_path(&args.matrix)?;
    let base_path = matrix.resolved_base_path(&args.matrix);
    let cells = matrix.expand();
    let total = cells.len();
    println!("▶ batch: {total} cells (matrix={})", args.matrix.display());

    std::fs::create_dir_all(&args.out_dir).map_err(|e| BatchError::Io {
        path: args.out_dir.clone(),
        source: e,
    })?;
    let index_path = args.out_dir.join("index.csv");
    let mut index = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&index_path)
        .map_err(|e| BatchError::Io {
            path: index_path.clone(),
            source: e,
        })?;
    writeln!(
        index,
        "scenario,seed,log_path,manifest_path,exit_status,duration_actual_secs,validation_status"
    )
    .map_err(|e| BatchError::Io {
        path: index_path.clone(),
        source: e,
    })?;

    let runner = build_runner(args);
    let host_jar_default = PathBuf::from("workload-harness/target/workload-harness.jar");
    let host_jar = args.harness_jar.clone().unwrap_or(host_jar_default);

    let mut failures = 0u32;
    let cell_width = total.to_string().len();
    for (i, cell) in cells.iter().enumerate() {
        let label = format!("[{}/{}]", i + 1, total);
        let cell_tag = format!("cell{i:0cell_width$}");
        let scenario = match prepare_scenario(&base_path, &cell.overrides) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{label} ✗ scenario preparation failed: {e}");
                failures += 1;
                if !args.continue_on_error {
                    return Err(e.into());
                }
                continue;
            }
        };
        match run_cell(
            &scenario,
            &base_path,
            &args.out_dir,
            &host_jar,
            &runner,
            &cell_tag,
        ) {
            Ok((log_path, manifest_path, manifest)) => {
                let row = csv_row(&scenario, cell.seed, &log_path, &manifest_path, &manifest);
                writeln!(index, "{row}").map_err(|e| BatchError::Io {
                    path: index_path.clone(),
                    source: e,
                })?;
                println!(
                    "{label} ✓ {} → {}",
                    scenario.metadata.name,
                    log_path.display()
                );
            }
            Err(e) => {
                eprintln!("{label} ✗ {} failed: {e}", scenario.metadata.name);
                failures += 1;
                if !args.continue_on_error {
                    return Err(e);
                }
            }
        }
    }

    let total_u32 = u32::try_from(total).unwrap_or(u32::MAX);
    println!(
        "▶ batch done: {} succeeded, {} failed → {}",
        total_u32 - failures,
        failures,
        index_path.display()
    );
    Ok(failures)
}

fn build_runner(args: &BatchArgs) -> DockerRunner {
    let mut runner = DockerRunner::new();
    if let Some(image) = &args.image {
        runner = runner.with_image(image.clone());
    }
    if let Some(path) = &args.embedded_harness {
        runner = runner.with_embedded_harness(path.clone());
    }
    runner
}

fn prepare_scenario(
    base_path: &Path,
    overrides: &[(String, serde_yaml::Value)],
) -> Result<Scenario, ScenarioError> {
    let scenario = Scenario::resolve(base_path)?;
    if overrides.is_empty() {
        return Ok(scenario);
    }
    let parsed: Vec<Override> = overrides
        .iter()
        .map(|(k, v)| {
            let value = serde_yaml::to_string(v)
                .ok()
                .map(|s| s.trim().to_owned())
                .unwrap_or_default();
            Override::new(k.split('.'), value)
        })
        .collect();
    scenario.apply_overrides(&parsed)
}

fn run_cell(
    scenario: &Scenario,
    source_path: &Path,
    out_dir: &Path,
    host_jar: &Path,
    runner: &DockerRunner,
    cell_tag: &str,
) -> Result<(PathBuf, PathBuf, RunManifest), BatchError> {
    if !matches!(scenario.spec.jvm.vendor, JvmVendor::Temurin) {
        return Err(BatchError::UnsupportedVendor {
            vendor: format!("{:?}", scenario.spec.jvm.vendor).to_lowercase(),
        });
    }

    let regime = resolve_regime(&scenario.spec.regime)?;
    let workload_args = regime.workload_args(scenario)?;

    // Cell tag prefix avoids collisions when several cells share name+seed
    // (e.g. an axis on `gc.algorithm` doesn't change either).
    let stem = format!(
        "{}-{}-{:x}",
        cell_tag,
        scenario.metadata.name,
        scenario.spec.seed.as_u64()
    );
    let log_path = out_dir.join(format!("{stem}.log"));
    let manifest_path = out_dir.join(format!("{stem}.manifest.yaml"));

    let spec = RunSpec {
        scenario: scenario.clone(),
        log_path: log_path.clone(),
        harness_jar: host_jar.to_path_buf(),
        workload_args,
        cpu_limit: None,
        memory_limit: None,
    };

    let outcome = runner.execute(&spec)?;
    let manifest = build_manifest(
        source_path,
        scenario,
        host_jar,
        regime.expected_phenomena(),
        regime.expected_invariant_rules(),
        &outcome,
    )?;
    manifest.write_yaml(&manifest_path)?;

    Ok((log_path, manifest_path, manifest))
}

#[allow(clippy::too_many_lines)]
fn build_manifest(
    source_path: &Path,
    scenario: &Scenario,
    harness_jar: &Path,
    phenomena: Vec<&'static str>,
    invariant_rules: Vec<&'static str>,
    outcome: &RunOutcome,
) -> Result<RunManifest, ScenarioError> {
    let source_sha = sha256_hex(source_path)?;
    let log_size = std::fs::metadata(&outcome.log_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let log_sha = if outcome.log_path.exists() {
        sha256_hex(&outcome.log_path)?
    } else {
        sha256_hex_bytes(b"")
    };
    let jar_sha = if harness_jar.exists() {
        sha256_hex(harness_jar)?
    } else {
        sha256_hex_bytes(b"")
    };

    let started_at: DateTime<Utc> = system_time_to_utc(outcome.started_at);
    let ended_at: DateTime<Utc> = system_time_to_utc(outcome.ended_at);
    let duration_actual = match outcome.ended_at.duration_since(outcome.started_at) {
        Ok(d) => gc_forge_scenario::Duration::from_secs(d.as_secs()),
        Err(_) => gc_forge_scenario::Duration::from_secs(0),
    };

    let exit_status = match outcome.exit_status {
        ExitStatus::Success => ExitStatusRecord::Success,
        ExitStatus::Failure(code) => ExitStatusRecord::Failure { code },
        ExitStatus::Oom => ExitStatusRecord::Oom,
        ExitStatus::Timeout => ExitStatusRecord::Timeout,
        ExitStatus::Signal(s) => ExitStatusRecord::Signaled { signal: s },
    };

    let host = HostMeta {
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        cpu_count: std::thread::available_parallelism()
            .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
            .unwrap_or(0),
        container: format!("docker:{}", outcome.jvm_locator),
    };

    let log_in_container = {
        let name = outcome.log_path.file_name().map_or_else(
            || std::ffi::OsString::from("gc.log"),
            std::ffi::OsStr::to_os_string,
        );
        let mut p = PathBuf::from("/work");
        p.push(name);
        p
    };
    let jvm_flags = build_jvm_command(scenario, &log_in_container);

    // Prefer the scenario's typed expected.invariants if present.
    let scenario_invariants: Vec<ExpectedInvariantRecord> = scenario
        .spec
        .expected
        .as_ref()
        .map(|e| {
            e.invariants
                .iter()
                .map(|inv| ExpectedInvariantRecord {
                    rule: inv.rule.clone(),
                    threshold: inv.threshold.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let invariants = if scenario_invariants.is_empty() {
        invariant_rules
            .into_iter()
            .map(|rule| ExpectedInvariantRecord {
                rule: rule.to_owned(),
                threshold: serde_yaml::Value::Null,
            })
            .collect()
    } else {
        scenario_invariants
    };

    Ok(RunManifest::new(
        RunMeta {
            id: Uuid::now_v7().to_string(),
            started_at,
            ended_at,
            duration_actual,
            exit_status,
            host,
        },
        ScenarioRecord {
            source_path: source_path.to_owned(),
            source_sha256: source_sha,
            resolved: scenario.clone(),
        },
        JvmRecord {
            vendor: format!("{:?}", scenario.spec.jvm.vendor).to_lowercase(),
            version: outcome.jvm_version.clone(),
            flags: jvm_flags,
        },
        ReproducibilityRecord {
            seed: format!("0x{:X}", scenario.spec.seed.as_u64()),
            workload_jar_sha256: jar_sha,
            gc_forge_version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        OutputRecord {
            log_path: outcome.log_path.clone(),
            log_sha256: log_sha,
            log_size_bytes: log_size,
        },
        phenomena.into_iter().map(str::to_owned).collect(),
        invariants,
        env!("CARGO_PKG_VERSION").to_owned(),
    ))
}

fn system_time_to_utc(t: SystemTime) -> DateTime<Utc> {
    DateTime::<Utc>::from(t)
}

fn csv_row(
    scenario: &Scenario,
    seed: Option<u64>,
    log_path: &Path,
    manifest_path: &Path,
    manifest: &RunManifest,
) -> String {
    let exit = match &manifest.run.exit_status {
        ExitStatusRecord::Success => "success".to_owned(),
        ExitStatusRecord::Failure { code } => format!("failure({code})"),
        ExitStatusRecord::Oom => "oom".to_owned(),
        ExitStatusRecord::Timeout => "timeout".to_owned(),
        ExitStatusRecord::Signaled { signal } => format!("signaled({signal})"),
    };
    let validation = format!("{:?}", manifest.validation.status).to_lowercase();
    let secs = manifest.run.duration_actual.as_std().as_secs();
    let seed_str = seed.map_or_else(
        || format!("0x{:X}", scenario.spec.seed.as_u64()),
        |s| s.to_string(),
    );
    format!(
        "{},{},{},{},{},{},{}",
        scenario.metadata.name,
        seed_str,
        log_path.display(),
        manifest_path.display(),
        exit,
        secs,
        validation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_runner_carries_image_and_embedded_harness() {
        let args = BatchArgs {
            matrix: PathBuf::from("matrix.yaml"),
            out_dir: PathBuf::from("out"),
            image: Some("ghcr.io/example/runner:1".to_owned()),
            embedded_harness: Some("/opt/harness.jar".to_owned()),
            harness_jar: None,
            continue_on_error: false,
        };
        let _runner = build_runner(&args);
    }

    #[test]
    fn csv_row_uses_scenario_name_and_seed() {
        use chrono::Utc;
        use gc_forge_scenario::{
            Duration, HostMeta, JvmRecord, OutputRecord, ReproducibilityRecord, RunManifest,
            RunMeta, ScenarioRecord, ValidationStatus,
        };
        let s = Scenario::from_yaml(
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: smoke
spec:
  jvm: { vendor: temurin, major: 21 }
  gc:
    algorithm: G1
    options: { heap: { min: 1g, max: 2g } }
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
",
            "<test>",
        )
        .unwrap();
        let manifest = RunManifest::new(
            RunMeta {
                id: "uuid".to_owned(),
                started_at: Utc::now(),
                ended_at: Utc::now(),
                duration_actual: Duration::from_secs(7),
                exit_status: ExitStatusRecord::Success,
                host: HostMeta {
                    os: "linux".to_owned(),
                    arch: "aarch64".to_owned(),
                    cpu_count: 4,
                    container: "docker:foo".to_owned(),
                },
            },
            ScenarioRecord {
                source_path: PathBuf::from("/x.yaml"),
                source_sha256: sha256_hex_bytes(b""),
                resolved: s.clone(),
            },
            JvmRecord {
                vendor: "temurin".to_owned(),
                version: None,
                flags: vec![],
            },
            ReproducibilityRecord {
                seed: "0x1".to_owned(),
                workload_jar_sha256: sha256_hex_bytes(b""),
                gc_forge_version: "0".to_owned(),
            },
            OutputRecord {
                log_path: PathBuf::from("out/smoke-c0ffee.log"),
                log_sha256: sha256_hex_bytes(b""),
                log_size_bytes: 0,
            },
            vec![],
            vec![],
            "test".to_owned(),
        );
        let row = csv_row(
            &s,
            Some(42),
            &PathBuf::from("out/smoke-c0ffee.log"),
            &PathBuf::from("out/smoke-c0ffee.manifest.yaml"),
            &manifest,
        );
        assert!(row.starts_with("smoke,42,"));
        assert!(row.contains(",success,7,skipped"));
        assert_eq!(manifest.validation.status, ValidationStatus::Skipped);
    }
}
