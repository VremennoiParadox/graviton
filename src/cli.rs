//! CLI definition and subcommand dispatch.
//!
//! Owns all [`clap`] structures. Does not contain simulation logic.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use tracing::info;

use crate::app::App;
use crate::error::{OrreryTuiError, Result};
use crate::scenario::load;
use crate::simulation::{print_diagnostics, run_headless};

/// Terminal N-body gravitational simulator.
#[derive(Debug, Parser)]
#[command(
    name = "orrery-tui",
    version,
    about = "Terminal N-body gravitational simulator with NASA HORIZONS data",
    long_about = "Simulate Newtonian N-body gravity in the terminal.\n\
                  Load TOML scenarios, fetch real ephemeris from NASA JPL HORIZONS, \
                  and explore orbits with an interactive TUI."
)]
pub struct Cli {
    /// Log level filter (e.g. `info`, `debug`, `orrery-tui=trace`).
    #[arg(long, global = true, env = "RUST_LOG", default_value = "warn")]
    pub log: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a simulation from a scenario file.
    Run(RunArgs),

    /// Fetch Solar System state vectors from NASA JPL HORIZONS.
    Fetch(FetchArgs),

    /// Validate a scenario TOML file without running.
    Validate(ValidateArgs),

    /// List bundled scenario files shipped with orrery-tui.
    ListScenarios,

    /// Run physics and rendering benchmarks.
    Bench(BenchArgs),
}

#[derive(Debug, clap::Args)]
pub struct RunArgs {
    /// Path to a scenario TOML file.
    pub scenario: PathBuf,

    /// Run without TUI; print diagnostics and exit.
    #[arg(long)]
    pub headless: bool,

    /// Number of physics steps in headless mode.
    #[arg(long, default_value_t = 1000)]
    pub steps: u64,

    /// Override simulation time step in seconds.
    #[arg(long)]
    pub dt: Option<f64>,

    /// Integrator to use.
    #[arg(long, value_enum, default_value_t = IntegratorArg::Rk4)]
    pub integrator: IntegratorArg,

    /// Use Barnes-Hut approximation instead of direct O(n²) gravity.
    #[arg(long)]
    pub barnes_hut: bool,

    /// Barnes-Hut opening angle θ (smaller = more accurate).
    #[arg(long, default_value_t = 0.7)]
    pub theta: f64,

    /// Disable gravitational field heatmap overlay.
    #[arg(long)]
    pub no_heatmap: bool,

    /// Disable body trails.
    #[arg(long)]
    pub no_trails: bool,
}

#[derive(Debug, clap::Args)]
pub struct FetchArgs {
    /// Preset to fetch (currently only `solar-system`).
    #[arg(default_value = "solar-system")]
    pub preset: String,

    /// Ephemeris date (YYYY-MM-DD).
    #[arg(long)]
    pub date: Option<String>,

    /// HORIZONS reference center (default: Solar System barycenter `500@0`).
    #[arg(long, default_value = "500@0")]
    pub center: String,

    /// Output scenario path.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Use cache only; do not contact HORIZONS.
    #[arg(long)]
    pub offline: bool,

    /// Re-fetch even if cache exists.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct ValidateArgs {
    /// Scenario file or glob pattern.
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct BenchArgs {
    /// Benchmark filter (e.g. `direct`, `barnes_hut`).
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, ValueEnum)]
pub enum IntegratorArg {
    #[default]
    Rk4,
}

/// Parse CLI and run the selected subcommand.
pub fn run(cli: Cli) -> Result<()> {
    init_logging(&cli.log);

    match cli.command {
        Commands::Run(args) => run_simulation(args),
        Commands::Fetch(args) => fetch_horizons(args),
        Commands::Validate(args) => validate_scenarios(args),
        Commands::ListScenarios => list_scenarios(),
        Commands::Bench(args) => run_bench(args),
    }
}

fn init_logging(filter: &str) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
        .ok();
}

fn run_simulation(args: RunArgs) -> Result<()> {
    info!(scenario = %args.scenario.display(), headless = args.headless, "run requested");

    if !args.scenario.exists() {
        return Err(OrreryTuiError::Other(format!(
            "scenario file not found: {}",
            args.scenario.display()
        )));
    }

    if args.integrator != IntegratorArg::Rk4 {
        return Err(OrreryTuiError::Physics(
            crate::error::PhysicsError::NotImplemented(format!("{:?}", args.integrator)),
        ));
    }

    let _ = (args.no_heatmap, args.no_trails);

    let mut loaded = load(&args.scenario)?;
    if args.barnes_hut {
        loaded.system.settings.use_barnes_hut = true;
        loaded.system.settings.barnes_hut_theta = args.theta;
    }

    if args.headless {
        let (initial, final_diag) = run_headless(&mut loaded, args.steps, args.dt)?;
        print_diagnostics(&loaded.system, &initial, &final_diag, args.steps);
        return Ok(());
    }

    let scenario_path = args.scenario.clone();
    App::run_interactive(loaded, scenario_path, args)
}

fn fetch_horizons(args: FetchArgs) -> Result<()> {
    info!(preset = %args.preset, offline = args.offline, "fetch requested");
    let path = crate::horizons::run_fetch(crate::horizons::FetchOptions {
        preset: args.preset,
        date: args.date,
        center: args.center,
        output: args.output,
        offline: args.offline,
        force: args.force,
    })?;
    println!("Run: orrery-tui run {}", path.display());
    Ok(())
}

fn validate_scenarios(args: ValidateArgs) -> Result<()> {
    if args.paths.is_empty() {
        return Err(OrreryTuiError::Other(
            "provide at least one scenario path to validate".into(),
        ));
    }

    let mut any_error = false;
    for path in &args.paths {
        if !path.exists() {
            eprintln!("error: scenario file not found: {}", path.display());
            any_error = true;
            continue;
        }
        match load(path) {
            Ok(loaded) => {
                println!(
                    "ok: {} ({} bodies, dt = {:.6} s)",
                    path.display(),
                    loaded.system.bodies.len(),
                    loaded.system.settings.dt_s
                );
            }
            Err(err) => {
                eprintln!("error: {}: {err}", path.display());
                any_error = true;
            }
        }
    }

    if any_error {
        return Err(OrreryTuiError::Other(
            "one or more scenarios failed validation".into(),
        ));
    }
    Ok(())
}

fn list_scenarios() -> Result<()> {
    let scenarios_dir = PathBuf::from("scenarios");
    if !scenarios_dir.is_dir() {
        println!("No scenarios/ directory found.");
        return Ok(());
    }

    let mut found = false;
    for entry in std::fs::read_dir(&scenarios_dir).map_err(|e| OrreryTuiError::Io {
        path: scenarios_dir.clone(),
        source: e,
    })? {
        let entry = entry.map_err(|e| OrreryTuiError::Io {
            path: scenarios_dir.clone(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            println!("{}", path.display());
            found = true;
        }
    }

    if !found {
        println!("No .toml scenarios in scenarios/.");
    }

    Ok(())
}

fn run_bench(args: BenchArgs) -> Result<()> {
    crate::bench::run(args.filter.as_deref())
}
