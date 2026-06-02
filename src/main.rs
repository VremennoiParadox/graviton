//! orrery-tui — terminal N-body gravitational simulator (binary entry point).

mod app;
mod bench;
mod cli;
mod error;
mod horizons;
mod input;
mod physics;
mod render;
mod scenario;
mod simulation;
mod time;

use anyhow::Context;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    cli::run(cli).context("orrery-tui failed")
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};

    #[test]
    fn cli_parses_without_subcommand_errors() {
        let cli = Cli::try_parse_from(["orrery-tui", "list-scenarios"]).unwrap();
        assert!(matches!(cli.command, Commands::ListScenarios));
    }

    #[test]
    fn run_subcommand_parses_flags() {
        let cli = Cli::try_parse_from([
            "orrery-tui",
            "run",
            "scenarios/earth-moon.toml",
            "--headless",
            "--steps",
            "500",
            "--barnes-hut",
        ])
        .unwrap();
        match cli.command {
            Commands::Run(args) => {
                assert!(args.headless);
                assert_eq!(args.steps, 500);
                assert!(args.barnes_hut);
            }
            _ => panic!("expected Run subcommand"),
        }
    }

    #[test]
    fn fetch_subcommand_parses_date() {
        let cli =
            Cli::try_parse_from(["orrery-tui", "fetch", "solar-system", "--date", "2026-06-01"])
                .unwrap();
        assert!(matches!(cli.command, Commands::Fetch(_)));
    }
}
