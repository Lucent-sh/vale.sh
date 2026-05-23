mod cli;
mod commands;
mod strategy;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let show_banner = args.len() == 1
        || args
            .get(1)
            .is_some_and(|a| a == "--help" || a == "-h" || a == "help");
    if show_banner {
        theme::print_banner();
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("VALE_LOG").unwrap_or_else(|_| EnvFilter::new("error")),
        )
        .init();

    let cli = cli::Cli::parse();
    if let Err(e) = commands::dispatch(cli).await {
        theme::error(&e.to_string());
        std::process::exit(1);
    }
    Ok(())
}
