//! Error types for scenario loading and resolution.

use std::path::PathBuf;

use thiserror::Error;

/// Anything that can go wrong while loading or resolving a scenario.
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("YAML parse error in {path}: {source}")]
    Yaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("unsupported apiVersion {found:?}: expected {expected:?}")]
    UnsupportedApiVersion {
        found: String,
        expected: &'static str,
    },

    #[error("unsupported kind {found:?}: expected {expected:?}")]
    UnsupportedKind {
        found: String,
        expected: &'static str,
    },

    #[error("extends cycle detected involving {path}")]
    ExtendsCycle { path: PathBuf },

    #[error("override path {path:?} does not match the scenario structure")]
    InvalidOverridePath { path: String },

    #[error("override {raw:?} is not in KEY=VALUE form")]
    InvalidOverrideSyntax { raw: String },

    #[error("override value {value:?} could not be coerced to YAML at {path:?}: {message}")]
    OverrideCoercion {
        path: String,
        value: String,
        message: String,
    },

    #[error("schema generation/serialisation failed: {0}")]
    Schema(#[from] serde_json::Error),
}

/// Convenient alias.
pub type Result<T, E = ScenarioError> = std::result::Result<T, E>;
