pub mod backtest;
pub mod config;
pub mod data;
pub mod doctor;
pub mod factor;
pub mod portfolio;
pub mod price;
pub mod report;
pub mod risk;
pub mod strategy;
pub mod sweep;
pub mod watch;

use crate::cli::{Cli, Command};
use anyhow::Result;

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        None => {
            crate::theme::print_banner();
            let mut cmd = <Cli as clap::CommandFactory>::command();
            cmd.print_help()?;
            println!();
            Ok(())
        }
        Some(Command::Backtest(cmd)) => backtest::handle(cmd, cli.output, cli.verbose).await,
        Some(Command::Sweep(cmd)) => sweep::handle(cmd, cli.output).await,
        Some(Command::Data(cmd)) => data::handle(cmd, cli.output).await,
        Some(Command::Portfolio(cmd)) => portfolio::handle(cmd, cli.output).await,
        Some(Command::Risk(cmd)) => risk::handle(cmd, cli.output).await,
        Some(Command::Price(cmd)) => price::handle(cmd, cli.output).await,
        Some(Command::Factor(cmd)) => factor::handle(cmd, cli.output).await,
        Some(Command::Report(cmd)) => report::handle(cmd, cli.output).await,
        Some(Command::Strategy(cmd)) => strategy::handle(cmd).await,
        Some(Command::Watch(args)) => watch::handle(args).await,
        Some(Command::Doctor) => doctor::handle(cli.output).await,
        Some(Command::Config(cmd)) => config::handle(cmd).await,
        Some(Command::Completions { shell }) => {
            use clap::CommandFactory;
            use clap_complete::generate;
            let mut app = Cli::command();
            generate(shell, &mut app, "vale", &mut std::io::stdout());
            Ok(())
        }
    }
}
