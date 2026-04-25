//! `gc-forge validate <log> --manifest <manifest.yaml>` — re-checks a GC log
//! against the manifest's expected invariants.

use std::path::PathBuf;

use clap::Args;
use thiserror::Error;

use gc_forge_scenario::{RunManifest, ScenarioError, ValidationStatus};
use gc_forge_validate::{parse_log, validate_invariants, ParseError};

#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// GC log file to validate.
    pub log: PathBuf,

    /// Path to the run manifest containing the expected invariants.
    #[arg(long = "manifest", value_name = "PATH")]
    pub manifest: PathBuf,

    /// When set, the validation block is written back into the manifest.
    #[arg(long = "update-manifest")]
    pub update_manifest: bool,

    /// Manifest format on write. Has no effect without `--update-manifest`.
    #[arg(long = "manifest-format", value_name = "FMT", default_value = "yaml")]
    pub manifest_format: ManifestFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ManifestFormat {
    Yaml,
    Json,
}

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error("invalid manifest YAML at {path:?}: {source}")]
    BadManifest {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
}

/// Entry point for `gc-forge validate`.
///
/// Returns an exit code: `0` on Passed/Skipped, `3` on Failed (matching the
/// SPEC §10 exit code for invariant violations).
///
/// # Errors
/// Surfaces I/O, manifest parse, and log-parse errors.
pub fn execute(args: &ValidateArgs) -> Result<u8, ValidateError> {
    let manifest_text = std::fs::read_to_string(&args.manifest).map_err(|e| {
        ValidateError::Scenario(ScenarioError::Io {
            path: args.manifest.clone(),
            source: e,
        })
    })?;
    let mut manifest: RunManifest = match serde_yaml::from_str(&manifest_text) {
        Ok(m) => m,
        Err(yaml_err) => {
            // Try JSON as a fallback (manifests can be either format).
            match serde_json::from_str(&manifest_text) {
                Ok(m) => m,
                Err(_) => {
                    return Err(ValidateError::BadManifest {
                        path: args.manifest.clone(),
                        source: yaml_err,
                    })
                }
            }
        }
    };

    let parsed = parse_log(&args.log)?;

    let validator_version = format!("gc-forge {}", env!("CARGO_PKG_VERSION"));
    let record = validate_invariants(&manifest.expected_invariants, &parsed, validator_version);

    println!(
        "▶ validation: {:?} ({} rules, {} passed, {} skipped, {} failed)",
        record.status,
        record.results.len(),
        record.results.iter().filter(|r| r.passed).count(),
        record
            .results
            .iter()
            .filter(|r| !r.passed && matches!(r.observed, serde_yaml::Value::Null))
            .count(),
        record
            .results
            .iter()
            .filter(|r| !r.passed && !matches!(r.observed, serde_yaml::Value::Null))
            .count(),
    );
    for result in &record.results {
        let prefix = match (result.passed, &result.observed) {
            (true, _) => "✓",
            (false, serde_yaml::Value::Null) => "∼",
            (false, _) => "✗",
        };
        println!(
            "  {prefix} {}    threshold={}    observed={}",
            result.rule,
            yaml_inline(&result.threshold),
            yaml_inline(&result.observed),
        );
    }

    let exit_code = match record.status {
        ValidationStatus::Failed => 3,
        ValidationStatus::Passed | ValidationStatus::Skipped => 0,
    };

    if args.update_manifest {
        manifest.validation = record;
        match args.manifest_format {
            ManifestFormat::Yaml => manifest.write_yaml(&args.manifest)?,
            ManifestFormat::Json => manifest.write_json(&args.manifest)?,
        }
        println!("✓ manifest updated → {}", args.manifest.display());
    }

    Ok(exit_code)
}

fn yaml_inline(v: &serde_yaml::Value) -> String {
    serde_yaml::to_string(v).map_or_else(|_| "<unprintable>".to_owned(), |s| s.trim().to_owned())
}
