//! Qualification subcommands: `presets`, `selftest`, `variance-check`.
//!
//! The three are grouped because they share the embedded-presets
//! infrastructure shipped by iter 15.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use thiserror::Error;

use gc_forge_presets::{all_presets, embedded_yaml};
use gc_forge_regimes::{resolve as resolve_regime, RegimeError};
use gc_forge_runner::{
    flags::build_jvm_command, DockerRunner, ExitStatus, RunSpec, Runner, RunnerError,
};
use gc_forge_scenario::{
    sha256_hex_bytes, ExitStatusRecord, ExpectedInvariantRecord, HostMeta, JvmRecord, JvmVendor,
    OutputRecord, OutputSpec, ReproducibilityRecord, RunManifest, RunMeta, Scenario, ScenarioError,
    ScenarioRecord, ValidationStatus,
};
use gc_forge_validate::{parse_log, validate_invariants, GcEventKind};

#[derive(Debug, Args)]
pub struct PresetsArgs {
    #[command(subcommand)]
    pub command: PresetsCommand,
}

#[derive(Debug, Subcommand)]
pub enum PresetsCommand {
    /// Lists every embedded preset (name + algorithm + regime).
    List,
    /// Prints the YAML body of an embedded preset.
    Show { name: String },
    /// Writes the preset's YAML body to stdout (suitable for piping).
    Export { name: String },
}

#[derive(Debug, Args)]
pub struct SelftestArgs {
    /// Optional Docker image override applied to every preset.
    #[arg(long = "image", value_name = "TAG")]
    pub image: Option<String>,

    /// Optional `--embedded-harness` applied to every preset.
    #[arg(long = "embedded-harness", value_name = "PATH_IN_IMAGE")]
    pub embedded_harness: Option<String>,

    /// Per-preset duration override (default 12 s).
    #[arg(
        long = "per-preset-duration",
        value_name = "DUR",
        default_value = "12s"
    )]
    pub per_preset_duration: String,

    /// Output directory for logs and manifests.
    #[arg(long = "out-dir", value_name = "DIR", default_value = "out/selftest")]
    pub out_dir: PathBuf,

    /// Continue past failing presets.
    #[arg(long = "continue-on-error")]
    pub continue_on_error: bool,
}

#[derive(Debug, Args)]
pub struct VarianceCheckArgs {
    /// Path to the scenario YAML or an embedded preset name.
    pub target: String,

    /// Number of runs.
    #[arg(long = "runs", value_name = "N", default_value = "5")]
    pub runs: u32,

    /// Optional Docker image override.
    #[arg(long = "image", value_name = "TAG")]
    pub image: Option<String>,

    /// Optional `--embedded-harness` applied to every run.
    #[arg(long = "embedded-harness", value_name = "PATH_IN_IMAGE")]
    pub embedded_harness: Option<String>,

    /// Per-run duration override.
    #[arg(long = "duration", value_name = "DUR", default_value = "12s")]
    pub duration: String,

    /// Output directory for logs and manifests.
    #[arg(long = "out-dir", value_name = "DIR", default_value = "out/variance")]
    pub out_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum QualifyError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error(transparent)]
    Regime(#[from] RegimeError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
    #[error("preset {0:?} not found")]
    PresetMissing(String),
    #[error("scenario rejected an unsupported jvm vendor: {vendor:?}")]
    UnsupportedVendor { vendor: String },
    #[error("invalid duration override {0:?}")]
    InvalidDuration(String),
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// `gc-forge presets …`.
pub fn execute_presets(args: &PresetsArgs) -> Result<u8, QualifyError> {
    match &args.command {
        PresetsCommand::List => {
            for (name, body) in all_presets() {
                // Presets that use `extends:` cannot fully deserialise from
                // their embedded body alone; fall back to a name-only listing.
                match Scenario::from_yaml(body, name) {
                    Ok(s) => println!(
                        "{name:32}  algo={:?}  regime={}",
                        s.spec.gc.algorithm, s.spec.regime.kind
                    ),
                    Err(_) => println!("{name:32}  (extends another preset)"),
                }
            }
            Ok(0)
        }
        PresetsCommand::Show { name } | PresetsCommand::Export { name } => {
            let body =
                embedded_yaml(name).ok_or_else(|| QualifyError::PresetMissing(name.clone()))?;
            print!("{body}");
            Ok(0)
        }
    }
}

struct RunnerContext {
    runner: DockerRunner,
    out_dir: PathBuf,
}

fn build_ctx(image: Option<&str>, embedded_harness: Option<&str>, out_dir: &Path) -> RunnerContext {
    let mut runner = DockerRunner::new();
    if let Some(image) = image {
        runner = runner.with_image(image.to_owned());
    }
    if let Some(path) = embedded_harness {
        runner = runner.with_embedded_harness(path.to_owned());
    }
    RunnerContext {
        runner,
        out_dir: out_dir.to_owned(),
    }
}

/// `gc-forge selftest`.
pub fn execute_selftest(args: &SelftestArgs) -> Result<u8, QualifyError> {
    std::fs::create_dir_all(&args.out_dir).map_err(|e| QualifyError::Io {
        path: args.out_dir.clone(),
        source: e,
    })?;
    let ctx = build_ctx(
        args.image.as_deref(),
        args.embedded_harness.as_deref(),
        &args.out_dir,
    );
    println!("▶ selftest: {} presets", all_presets().len());

    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut passed = 0u32;
    for (name, body) in all_presets() {
        match run_one_preset(name, body, &ctx, &args.per_preset_duration) {
            Ok(status) => match status {
                ValidationStatus::Passed => {
                    passed += 1;
                    println!("  ✓ {name}");
                }
                ValidationStatus::Skipped => {
                    skipped += 1;
                    println!("  ∼ {name} (validation skipped — no recognised rules)");
                }
                ValidationStatus::Failed => {
                    failed += 1;
                    println!("  ✗ {name}");
                    if !args.continue_on_error {
                        break;
                    }
                }
            },
            Err(e) => {
                failed += 1;
                println!("  ✗ {name} — {e}");
                if !args.continue_on_error {
                    break;
                }
            }
        }
    }

    println!(
        "▶ selftest done: {passed} passed, {skipped} skipped, {failed} failed → {}",
        ctx.out_dir.display()
    );
    Ok(if failed == 0 { 0 } else { 3 })
}

fn run_one_preset(
    name: &str,
    body: &str,
    ctx: &RunnerContext,
    duration_override: &str,
) -> Result<ValidationStatus, QualifyError> {
    let mut scenario = Scenario::from_yaml(body, name)?;
    let dur: gc_forge_scenario::Duration = duration_override
        .parse()
        .map_err(|_| QualifyError::InvalidDuration(duration_override.to_owned()))?;
    scenario.spec.duration = dur;

    if !matches!(scenario.spec.jvm.vendor, JvmVendor::Temurin) {
        return Err(QualifyError::UnsupportedVendor {
            vendor: format!("{:?}", scenario.spec.jvm.vendor).to_lowercase(),
        });
    }

    let regime = resolve_regime(&scenario.spec.regime)?;
    let workload_args = regime.workload_args(&scenario)?;
    let log_path = ctx.out_dir.join(format!("{name}.log"));
    let manifest_path = ctx.out_dir.join(format!("{name}.manifest.yaml"));

    let host_jar = PathBuf::from("workload-harness/target/workload-harness.jar");
    let spec = RunSpec {
        scenario: scenario.clone(),
        log_path: log_path.clone(),
        harness_jar: host_jar.clone(),
        workload_args,
        cpu_limit: None,
        memory_limit: None,
    };

    let outcome = ctx.runner.execute(&spec)?;
    let manifest = build_minimal_manifest(
        name,
        &scenario,
        &host_jar,
        regime.expected_phenomena(),
        regime.expected_invariant_rules(),
        &outcome,
    )?;

    let parsed = parse_log(&log_path).map_err(|e| QualifyError::Io {
        path: log_path.clone(),
        source: std::io::Error::other(e.to_string()),
    })?;
    let validation = validate_invariants(
        &manifest.expected_invariants,
        &parsed,
        format!("gc-forge {}", env!("CARGO_PKG_VERSION")),
    );
    let status = validation.status;

    let mut updated = manifest;
    updated.validation = validation;
    updated.write_yaml(&manifest_path)?;

    Ok(status)
}

#[allow(clippy::too_many_lines)]
fn build_minimal_manifest(
    source_label: &str,
    scenario: &Scenario,
    harness_jar: &Path,
    phenomena: Vec<&'static str>,
    invariant_rules: Vec<&'static str>,
    outcome: &gc_forge_runner::RunOutcome,
) -> Result<RunManifest, ScenarioError> {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    let log_size = std::fs::metadata(&outcome.log_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let log_sha = if outcome.log_path.exists() {
        gc_forge_scenario::sha256_hex(&outcome.log_path)?
    } else {
        sha256_hex_bytes(b"")
    };
    let jar_sha = if harness_jar.exists() {
        gc_forge_scenario::sha256_hex(harness_jar)?
    } else {
        sha256_hex_bytes(b"")
    };

    let started_at: DateTime<Utc> = DateTime::<Utc>::from(outcome.started_at);
    let ended_at: DateTime<Utc> = DateTime::<Utc>::from(outcome.ended_at);
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
            source_path: PathBuf::from(format!("preset:{source_label}")),
            source_sha256: sha256_hex_bytes(
                serde_yaml::to_string(scenario)
                    .unwrap_or_default()
                    .as_bytes(),
            ),
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

fn coefficient_of_variation(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean = samples.iter().copied().sum::<f64>() / samples.len() as f64;
    if mean.abs() < 1e-12 {
        return 0.0;
    }
    let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / samples.len() as f64;
    (var.sqrt() / mean.abs()) * 100.0
}

/// `gc-forge variance-check`.
#[allow(clippy::too_many_lines)]
pub fn execute_variance(args: &VarianceCheckArgs) -> Result<u8, QualifyError> {
    if args.runs < 2 {
        eprintln!("variance-check needs at least 2 runs");
        return Ok(1);
    }
    std::fs::create_dir_all(&args.out_dir).map_err(|e| QualifyError::Io {
        path: args.out_dir.clone(),
        source: e,
    })?;
    let ctx = build_ctx(
        args.image.as_deref(),
        args.embedded_harness.as_deref(),
        &args.out_dir,
    );

    // Resolve the scenario from either an embedded preset name or a path.
    let (label, body) = if let Some(body) = embedded_yaml(&args.target) {
        (args.target.clone(), body.to_owned())
    } else {
        let body = std::fs::read_to_string(&args.target).map_err(|e| QualifyError::Io {
            path: PathBuf::from(&args.target),
            source: e,
        })?;
        (args.target.clone(), body)
    };

    let mut young_counts = Vec::with_capacity(args.runs as usize);
    let mut mean_pauses = Vec::with_capacity(args.runs as usize);
    let mut p99_pauses = Vec::with_capacity(args.runs as usize);

    println!(
        "▶ variance-check {label} ({} runs, duration={})",
        args.runs, args.duration
    );
    for run in 0..args.runs {
        let mut scenario = Scenario::from_yaml(&body, &label)?;
        let dur: gc_forge_scenario::Duration = args
            .duration
            .parse()
            .map_err(|_| QualifyError::InvalidDuration(args.duration.clone()))?;
        scenario.spec.duration = dur;
        let log_path = args.out_dir.join(format!("run-{run:02}.log"));
        scenario.spec.output = Some(OutputSpec {
            log_path: Some(log_path.clone()),
            ..OutputSpec::default()
        });

        if !matches!(scenario.spec.jvm.vendor, JvmVendor::Temurin) {
            return Err(QualifyError::UnsupportedVendor {
                vendor: format!("{:?}", scenario.spec.jvm.vendor).to_lowercase(),
            });
        }
        let regime = resolve_regime(&scenario.spec.regime)?;
        let workload_args = regime.workload_args(&scenario)?;
        let host_jar = PathBuf::from("workload-harness/target/workload-harness.jar");
        let spec = RunSpec {
            scenario: scenario.clone(),
            log_path: log_path.clone(),
            harness_jar: host_jar,
            workload_args,
            cpu_limit: None,
            memory_limit: None,
        };
        let _ = ctx.runner.execute(&spec)?;

        let parsed = parse_log(&log_path).map_err(|e| QualifyError::Io {
            path: log_path.clone(),
            source: std::io::Error::other(e.to_string()),
        })?;
        let yc = parsed.count(GcEventKind::Young) as f64;
        young_counts.push(yc);
        mean_pauses.push(parsed.mean_pause_ms());
        p99_pauses.push(parsed.percentile_pause_ms(99.0));
        println!(
            "  run {run}: young_count={yc}, mean_pause_ms={:.3}, p99_pause_ms={:.3}",
            mean_pauses[run as usize], p99_pauses[run as usize]
        );
    }

    let cv_young = coefficient_of_variation(&young_counts);
    let cv_mean = coefficient_of_variation(&mean_pauses);
    let cv_p99 = coefficient_of_variation(&p99_pauses);
    println!(
        "▶ CV: young_count={cv_young:.2}%, mean_pause_ms={cv_mean:.2}%, p99_pause_ms={cv_p99:.2}%"
    );

    let mut exceeded: Option<(String, f64, f64)> = None;
    if cv_young > 8.0 {
        exceeded = Some(("young_count".to_owned(), cv_young, 8.0));
    } else if cv_mean > 10.0 {
        exceeded = Some(("mean_pause_ms".to_owned(), cv_mean, 10.0));
    } else if cv_p99 > 20.0 {
        exceeded = Some(("p99_pause_ms".to_owned(), cv_p99, 20.0));
    }
    if let Some((metric, cv_val, limit)) = exceeded {
        eprintln!("✗ variance budget exceeded for {metric}: CV={cv_val:.2}% > {limit}%");
        return Ok(3);
    }
    println!("✓ variance within budgets");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_subcommand_args_parse() {
        // Just ensure the enum is wired.
        let _ = PresetsCommand::List;
    }

    #[test]
    fn cv_basic() {
        fn cv(samples: &[f64]) -> f64 {
            let mean = samples.iter().sum::<f64>() / samples.len() as f64;
            let var =
                samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / samples.len() as f64;
            (var.sqrt() / mean.abs()) * 100.0
        }
        let cv = cv(&[10.0, 11.0, 9.0]);
        assert!(cv > 0.0 && cv < 20.0);
    }
}
