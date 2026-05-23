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
        Command::Backtest(cmd) => backtest::handle(cmd, cli.output, cli.verbose).await,
        Command::Sweep(cmd) => sweep::handle(cmd, cli.output).await,
        Command::Data(cmd) => data::handle(cmd, cli.output).await,
        Command::Portfolio(cmd) => portfolio::handle(cmd, cli.output).await,
        Command::Risk(cmd) => risk::handle(cmd, cli.output).await,
        Command::Price(cmd) => price::handle(cmd, cli.output).await,
        Command::Factor(cmd) => factor::handle(cmd, cli.output).await,
        Command::Report(cmd) => report::handle(cmd, cli.output).await,
        Command::Strategy(cmd) => strategy::handle(cmd).await,
        Command::Watch(args) => watch::handle(args).await,
        Command::Doctor => doctor::handle(cli.output).await,
        Command::Config(cmd) => config::handle(cmd).await,
    }
}
