//! `gc-forge run` — orchestrates the scenario+regime+runner+manifest pipeline.

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
    sha256_hex, sha256_hex_bytes, ByteSize, ExitStatusRecord, ExpectedInvariantRecord, HostMeta,
    JvmRecord, JvmVendor, OutputRecord, Override, ReproducibilityRecord, RunManifest, RunMeta,
    Scenario, ScenarioError, ScenarioRecord,
};

/// `gc-forge run` arguments.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Path to the scenario YAML.
    pub path: PathBuf,

    /// Output directory. The log lands at `<out-dir>/<name>-<seed>.log` and
    /// the manifest at `<out-dir>/<name>-<seed>.manifest.<yaml|json>`.
    #[arg(long = "out-dir", value_name = "DIR", default_value = "out")]
    pub out_dir: PathBuf,

    /// `KEY=VALUE` overrides applied after extends resolution.
    #[arg(long = "override", value_name = "KEY=VALUE")]
    pub overrides: Vec<String>,

    /// Override the Docker image tag.
    #[arg(long = "image", value_name = "TAG")]
    pub image: Option<String>,

    /// Pass `--cpus=<N>` to `docker run`.
    #[arg(long = "docker-cpus", value_name = "N")]
    pub docker_cpus: Option<f64>,

    /// Pass `--memory=<bytes>` to `docker run`.
    #[arg(long = "docker-memory", value_name = "BYTES")]
    pub docker_memory: Option<u64>,

    /// Path to the workload-harness fat-jar on the host. Defaults to
    /// `workload-harness/target/workload-harness.jar` relative to the current
    /// working directory.
    #[arg(long = "harness-jar", value_name = "PATH")]
    pub harness_jar: Option<PathBuf>,

    /// When set, treat the image as an embedded-harness variant
    /// (e.g. `gc-forge-runner:*-jdk*`) and skip the host jar mount.
    #[arg(long = "embedded-harness", value_name = "PATH_IN_IMAGE")]
    pub embedded_harness: Option<String>,

    /// Manifest format: `yaml` (default) or `json`.
    #[arg(long = "manifest-format", value_name = "FMT", default_value = "yaml")]
    pub manifest_format: ManifestFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ManifestFormat {
    Yaml,
    Json,
}

#[derive(Debug, Error)]
pub enum RunCommandError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error(transparent)]
    Regime(#[from] RegimeError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
    #[error("harness jar {path:?} not found; pass --harness-jar or run `make build`")]
    HarnessJarMissing { path: PathBuf },
    #[error("scenario rejected an unsupported jvm vendor for the MVP: {vendor:?}")]
    UnsupportedVendor { vendor: String },
    #[error("invalid --docker-cpus value (must be > 0)")]
    InvalidCpuLimit,
}

/// Entry point for `gc-forge run`.
///
/// # Errors
/// Surfaces scenario, regime, and runner failures plus a small set of
/// CLI-specific errors (missing harness jar, unsupported vendor).
pub fn execute(args: &RunArgs) -> Result<(), RunCommandError> {
    let scenario = load_scenario(&args.path, &args.overrides)?;
    enforce_mvp_vendor(&scenario)?;

    let regime = resolve_regime(&scenario.spec.regime)?;
    let workload_args = regime.workload_args(&scenario)?;

    let (log_path, manifest_path) = output_paths(&scenario, &args.out_dir, args.manifest_format);
    // The host harness jar is only mounted when the image does not embed it;
    // skip the existence check in embedded-harness mode so callers using a
    // self-contained runner image don't need the host jar at all.
    let harness_jar = if args.embedded_harness.is_some() {
        args.harness_jar
            .clone()
            .unwrap_or_else(|| PathBuf::from("(embedded in image)"))
    } else {
        harness_jar_path(args.harness_jar.as_deref())?
    };

    let runner = build_runner(args);
    let cpu_limit = args.docker_cpus;
    if let Some(c) = cpu_limit {
        if c <= 0.0 {
            return Err(RunCommandError::InvalidCpuLimit);
        }
    }

    let spec = RunSpec {
        scenario: scenario.clone(),
        log_path: log_path.clone(),
        harness_jar: harness_jar.clone(),
        workload_args,
        cpu_limit,
        memory_limit: args.docker_memory.map(ByteSize),
    };

    println!(
        "▶ running {} (algorithm={:?}, regime={}, duration={})",
        scenario.metadata.name,
        scenario.spec.gc.algorithm,
        scenario.spec.regime.kind,
        scenario.spec.duration,
    );

    let outcome = runner.execute(&spec)?;

    let manifest = build_manifest(
        &args.path,
        &scenario,
        &harness_jar,
        regime.expected_phenomena(),
        regime.expected_invariant_rules(),
        &spec,
        &outcome,
    )?;

    match args.manifest_format {
        ManifestFormat::Yaml => manifest.write_yaml(&manifest_path)?,
        ManifestFormat::Json => manifest.write_json(&manifest_path)?,
    }

    println!("✓ log     → {}", outcome.log_path.display());
    println!("✓ manifest→ {}", manifest_path.display());
    if !outcome.exit_status.is_success() {
        println!(
            "  (note: JVM exit was {:?}; see manifest for the recorded status)",
            outcome.exit_status
        );
    }
    Ok(())
}

fn load_scenario(path: &Path, overrides_raw: &[String]) -> Result<Scenario, ScenarioError> {
    let scenario = Scenario::resolve(path)?;
    if overrides_raw.is_empty() {
        return Ok(scenario);
    }
    let overrides: Vec<Override> = overrides_raw
        .iter()
        .map(|s| Override::parse(s))
        .collect::<Result<_, _>>()?;
    scenario.apply_overrides(&overrides)
}

fn enforce_mvp_vendor(scenario: &Scenario) -> Result<(), RunCommandError> {
    match scenario.spec.jvm.vendor {
        JvmVendor::Temurin => Ok(()),
        other => Err(RunCommandError::UnsupportedVendor {
            vendor: format!("{other:?}").to_lowercase(),
        }),
    }
}

fn output_paths(scenario: &Scenario, out_dir: &Path, fmt: ManifestFormat) -> (PathBuf, PathBuf) {
    let stem = format!(
        "{}-{:x}",
        scenario.metadata.name,
        scenario.spec.seed.as_u64()
    );
    let log = out_dir.join(format!("{stem}.log"));
    let manifest_ext = match fmt {
        ManifestFormat::Yaml => "manifest.yaml",
        ManifestFormat::Json => "manifest.json",
    };
    let manifest = out_dir.join(format!("{stem}.{manifest_ext}"));
    (log, manifest)
}

fn harness_jar_path(arg: Option<&Path>) -> Result<PathBuf, RunCommandError> {
    let candidate = arg.map_or_else(
        || PathBuf::from("workload-harness/target/workload-harness.jar"),
        Path::to_path_buf,
    );
    if !candidate.exists() {
        return Err(RunCommandError::HarnessJarMissing { path: candidate });
    }
    Ok(candidate)
}

fn build_runner(args: &RunArgs) -> DockerRunner {
    let mut runner = DockerRunner::new();
    if let Some(image) = &args.image {
        runner = runner.with_image(image.clone());
    }
    if let Some(path) = &args.embedded_harness {
        runner = runner.with_embedded_harness(path.clone());
    }
    runner
}

fn build_manifest(
    source_path: &Path,
    scenario: &Scenario,
    harness_jar: &Path,
    phenomena: Vec<&'static str>,
    invariant_rules: Vec<&'static str>,
    spec: &RunSpec,
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

    let resolved_for_record = scenario.clone();
    let log_for_jvm_flags = log_path_in_container(&outcome.log_path);
    let jvm_flags = build_jvm_command(scenario, &log_for_jvm_flags);

    let invariants = invariant_rules
        .into_iter()
        .map(|rule| ExpectedInvariantRecord {
            rule: rule.to_owned(),
            threshold: serde_yaml::Value::Null,
        })
        .collect();

    let manifest = RunManifest::new(
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
            resolved: resolved_for_record,
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
    );
    let _ = spec; // currently only the scenario subtree is recorded
    Ok(manifest)
}

fn system_time_to_utc(t: SystemTime) -> DateTime<Utc> {
    DateTime::<Utc>::from(t)
}

fn log_path_in_container(host_log: &Path) -> PathBuf {
    let name = host_log.file_name().map_or_else(
        || std::ffi::OsString::from("gc.log"),
        std::ffi::OsStr::to_os_string,
    );
    let mut p = PathBuf::from("/work");
    p.push(name);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use gc_forge_scenario::Scenario;

    fn scenario() -> Scenario {
        Scenario::from_yaml(
            r"apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: my-scenario
spec:
  jvm: { vendor: temurin, major: 21 }
  gc:
    algorithm: G1
    options:
      heap: { min: 1g, max: 2g }
  regime:
    kind: steady-state-healthy
  duration: 30s
  seed: 0xC0FFEE
",
            "<test>",
        )
        .unwrap()
    }

    #[test]
    fn output_paths_use_name_and_seed() {
        let s = scenario();
        let (log, manifest) = output_paths(&s, Path::new("out"), ManifestFormat::Yaml);
        assert_eq!(log, PathBuf::from("out/my-scenario-c0ffee.log"));
        assert_eq!(
            manifest,
            PathBuf::from("out/my-scenario-c0ffee.manifest.yaml")
        );
    }

    #[test]
    fn output_paths_switch_to_json_extension() {
        let s = scenario();
        let (_, manifest) = output_paths(&s, Path::new("out"), ManifestFormat::Json);
        assert_eq!(
            manifest,
            PathBuf::from("out/my-scenario-c0ffee.manifest.json")
        );
    }

    #[test]
    fn unsupported_vendor_is_rejected() {
        let s = scenario();
        let mut s2 = s;
        s2.spec.jvm.vendor = JvmVendor::Corretto;
        let err = enforce_mvp_vendor(&s2).unwrap_err();
        assert!(matches!(err, RunCommandError::UnsupportedVendor { .. }));
    }

    #[test]
    fn missing_harness_jar_returns_typed_error() {
        let err = harness_jar_path(Some(Path::new("/nonexistent.jar"))).unwrap_err();
        assert!(matches!(err, RunCommandError::HarnessJarMissing { .. }));
    }

    #[test]
    fn log_path_in_container_uses_work_prefix() {
        let p = log_path_in_container(Path::new("out/foo-1.log"));
        assert_eq!(p, PathBuf::from("/work/foo-1.log"));
    }
}
