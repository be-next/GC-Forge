//! GC-Forge command-line entry point.
//!
//! Iteration 5 wires `gc-forge run`. Other subcommands (`validate`, `batch`,
//! `selftest`, `presets`) land in later iterations.

#![allow(clippy::unnecessary_wraps)]

mod run;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use gc_forge_scenario::{Override, Scenario, ScenarioError};

/// GC-Forge — declarative generator of Java GC logs.
#[derive(Debug, Parser)]
#[command(
    name = "gc-forge",
    version,
    about = "Declarative generator of Java GC logs.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validates a scenario file (syntax + apiVersion + extends resolution),
    /// without executing it.
    Lint(LintArgs),

    /// Runs a scenario through the configured runner and writes the GC log
    /// + run manifest to disk.
    Run(run::RunArgs),
}

#[derive(Debug, clap::Args)]
struct LintArgs {
    /// Path to the scenario YAML.
    path: PathBuf,

    /// Optional `KEY=VALUE` overrides applied after extends resolution. Same
    /// syntax as `gc-forge run --override`. Useful for catching issues that
    /// only manifest after substitution.
    #[arg(long = "override", value_name = "KEY=VALUE")]
    overrides: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Lint(args)) => match run_lint(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                report(&e);
                ExitCode::from(1)
            }
        },
        Some(Command::Run(args)) => match run::execute(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(run::RunCommandError::Scenario(e)) => {
                report(&e);
                ExitCode::from(1)
            }
            Err(other) => {
                eprintln!("error: {other}");
                ExitCode::from(2)
            }
        },
        None => ExitCode::SUCCESS, // bare `gc-forge` prints help via clap's default
    }
}

fn run_lint(args: &LintArgs) -> Result<(), ScenarioError> {
    let scenario = Scenario::resolve(&args.path)?;

    if !args.overrides.is_empty() {
        let parsed: Vec<Override> = args
            .overrides
            .iter()
            .map(|s| Override::parse(s))
            .collect::<Result<_, _>>()?;
        let _ = scenario.clone().apply_overrides(&parsed)?;
    }

    println!(
        "✓ {} parses cleanly (apiVersion={}, kind={})",
        args.path.display(),
        scenario.api_version,
        scenario.kind
    );
    Ok(())
}

fn report(err: &ScenarioError) {
    eprintln!("error: {err}");
    let mut source = std::error::Error::source(err);
    while let Some(s) = source {
        eprintln!("  caused by: {s}");
        source = s.source();
    }
}
