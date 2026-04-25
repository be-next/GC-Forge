//! GC-Forge command-line entry point.
//!
//! Iteration 1 (bootstrap): only `--version` is wired up. Subcommands land
//! starting iteration 2 (`scenario-parser`).

// Subcommands land in iter 2+ and will surface fallible operations.
#![allow(clippy::unnecessary_wraps)]

use clap::Parser;

/// GC-Forge — declarative generator of Java GC logs.
#[derive(Debug, Parser)]
#[command(
    name = "gc-forge",
    version,
    about = "Declarative generator of Java GC logs.",
    long_about = None,
)]
struct Cli {}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    // Iter 1: nothing to do beyond clap's auto --version / --help handling.
    Ok(())
}
