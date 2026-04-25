//! Regenerates `schemas/scenario-v1.json` and `schemas/run-manifest-v1.json`
//! from the typed models.
//!
//! Usage:
//!     cargo run -p gc-forge-scenario --bin gen-schema [--check]
//!
//! With `--check`, the binary exits non-zero if either on-disk schema differs
//! from a fresh regeneration (intended for CI).

use std::path::PathBuf;
use std::process::ExitCode;

use gc_forge_scenario::{render_manifest_schema, render_schema};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|a| a == "--check");

    let scenario_schema = match render_schema() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error rendering scenario schema: {e}");
            return ExitCode::from(2);
        }
    };
    let manifest_schema = match render_manifest_schema() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error rendering manifest schema: {e}");
            return ExitCode::from(2);
        }
    };

    let root = workspace_root().unwrap_or_else(|| PathBuf::from("."));
    let scenario_path = root.join("schemas/scenario-v1.json");
    let manifest_path = root.join("schemas/run-manifest-v1.json");

    if check_mode {
        let scenario_ok = check(&scenario_path, &scenario_schema);
        let manifest_ok = check(&manifest_path, &manifest_schema);
        if scenario_ok && manifest_ok {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        }
    } else {
        if write(&scenario_path, &scenario_schema).is_err() {
            return ExitCode::from(2);
        }
        if write(&manifest_path, &manifest_schema).is_err() {
            return ExitCode::from(2);
        }
        ExitCode::SUCCESS
    }
}

fn check(path: &PathBuf, fresh: &str) -> bool {
    match std::fs::read_to_string(path) {
        Ok(on_disk) if on_disk == fresh => {
            println!("✓ {} is up to date", path.display());
            true
        }
        Ok(_) => {
            eprintln!(
                "✗ {} has drifted from the typed schema. Re-run without --check to regenerate.",
                path.display()
            );
            false
        }
        Err(e) => {
            eprintln!("error reading {}: {e}", path.display());
            false
        }
    }
}

fn write(path: &PathBuf, body: &str) -> Result<(), ()> {
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error creating {}: {e}", parent.display());
            return Err(());
        }
    }
    if let Err(e) = std::fs::write(path, body.as_bytes()) {
        eprintln!("error writing {}: {e}", path.display());
        return Err(());
    }
    println!("Wrote {}", path.display());
    Ok(())
}

fn workspace_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
}
