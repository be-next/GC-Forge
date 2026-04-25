//! Regenerates `schemas/scenario-v1.json` from the typed `Scenario` model.
//!
//! Usage:
//!     cargo run -p gc-forge-scenario --bin gen-schema [--check]
//!
//! With `--check`, the binary exits non-zero if the on-disk schema differs
//! from what would be generated (intended for CI).

use std::path::PathBuf;
use std::process::ExitCode;

use gc_forge_scenario::render_schema;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|a| a == "--check");

    let schema = match render_schema() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error rendering schema: {e}");
            return ExitCode::from(2);
        }
    };

    let path = workspace_root().map_or_else(
        || PathBuf::from("schemas/scenario-v1.json"),
        |p| p.join("schemas/scenario-v1.json"),
    );

    if check_mode {
        match std::fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == schema => {
                println!("✓ {} is up to date", path.display());
                ExitCode::SUCCESS
            }
            Ok(_) => {
                eprintln!(
                    "✗ {} has drifted from the typed schema. Re-run without --check to regenerate.",
                    path.display()
                );
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("error reading {}: {e}", path.display());
                ExitCode::from(1)
            }
        }
    } else {
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error creating {}: {e}", parent.display());
                return ExitCode::from(2);
            }
        }
        if let Err(e) = std::fs::write(&path, schema.as_bytes()) {
            eprintln!("error writing {}: {e}", path.display());
            return ExitCode::from(2);
        }
        println!("Wrote {}", path.display());
        ExitCode::SUCCESS
    }
}

fn workspace_root() -> Option<PathBuf> {
    // CARGO_MANIFEST_DIR points to crates/gc-forge-scenario; go up two levels.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
}
