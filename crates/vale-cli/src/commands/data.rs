use crate::cli::{DataCommand, DataFetchArgs};
use crate::theme;
use anyhow::{Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use indicatif::ProgressBar;
use vale_core::config::Config;
use vale_core::types::{OutputFormat, TimeRange};
use vale_data::build_provider;
use vale_data::local::LocalCsvProvider;

use crate::cli::DataCommand::*;

pub async fn handle(cmd: DataCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        Fetch(args) => fetch(args, output).await,
        Inspect(args) => inspect(args, output).await,
        Export(args) => export(args).await,
        Sources => sources().await,
    }
}

fn parse_range(from: &str, to: Option<&str>) -> Result<TimeRange> {
    let start_date = NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .with_context(|| format!("invalid from date: {from}"))?;
    let end_date = match to {
        Some(t) => NaiveDate::parse_from_str(t, "%Y-%m-%d")?,
        None => Utc::now().date_naive(),
    };
    Ok(TimeRange {
        start: Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).context("time")?),
        end: Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59).context("time")?),
    })
}

async fn fetch(args: DataFetchArgs, output: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;
    if let Some(src) = &args.source {
        config.providers.default = src.clone();
    }

    let provider = build_provider(&config)?;
    let range = parse_range(&args.from, args.to.as_deref())?;
    let tickers = if args.ticker.is_empty() {
        vec!["SPY".to_string()]
    } else {
        args.ticker.clone()
    };

    let pb = if tickers.len() > 1 {
        let bar = indicatif::ProgressBar::new(tickers.len() as u64);
        bar.set_style(theme::progress_bar_style());
        bar
    } else {
        let bar = ProgressBar::new_spinner();
        bar.set_style(theme::spinner_style());
        bar.enable_steady_tick(std::time::Duration::from_millis(80));
        bar
    };

    let mut all_bars = Vec::new();
    for ticker in &tickers {
        pb.set_message(format!("fetching {ticker}…"));
        let bars = provider
            .fetch_ohlcv(ticker, args.resolution, &range)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        all_bars.extend(bars);
    }
    pb.finish_and_clear();

    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&all_bars)?),
        OutputFormat::Csv => {
            println!("timestamp,open,high,low,close,volume,symbol");
            for b in &all_bars {
                println!(
                    "{},{},{},{},{},{},{}",
                    b.timestamp.to_rfc3339(),
                    b.open,
                    b.high,
                    b.low,
                    b.close,
                    b.volume,
                    b.symbol
                );
            }
        }
        OutputFormat::Table => {
            theme::section_header("Market Data");
            theme::success(&format!("Fetched {} bars", all_bars.len()));
            if let Some(first) = all_bars.first() {
                theme::info(&format!(
                    "Range: {} → {}",
                    first.timestamp.format("%Y-%m-%d"),
                    all_bars
                        .last()
                        .map(|b| b.timestamp.format("%Y-%m-%d").to_string())
                        .unwrap_or_default()
                ));
            }
        }
    }

    if let Some(path) = args.out {
        std::fs::write(&path, serde_json::to_string_pretty(&all_bars)?)?;
        theme::success(&format!("Wrote {}", path.display()));
    }
    Ok(())
}

async fn inspect(args: crate::cli::DataInspectArgs, output: OutputFormat) -> Result<()> {
    let provider = LocalCsvProvider::new(&args.file);
    let bars = provider.read_bars("LOCAL")?;
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&bars)?),
        OutputFormat::Csv => {
            println!("timestamp,open,high,low,close,volume,symbol");
            for b in &bars {
                println!(
                    "{},{},{},{},{},{},{}",
                    b.timestamp.to_rfc3339(),
                    b.open,
                    b.high,
                    b.low,
                    b.close,
                    b.volume,
                    b.symbol
                );
            }
        }
        OutputFormat::Table => {
            theme::section_header("Data Inspect");
            theme::status_line("file", &args.file.display().to_string(), true);
            theme::status_line("bars", &bars.len().to_string(), true);
        }
    }
    Ok(())
}

async fn export(args: crate::cli::DataExportArgs) -> Result<()> {
    let config = Config::load()?;
    let provider = build_provider(&config)?;
    let range = parse_range(&args.from, args.to.as_deref())?;
    let bars = provider
        .fetch_ohlcv(&args.ticker, vale_core::types::Resolution::Daily, &range)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut out = String::from("timestamp,open,high,low,close,volume,symbol\n");
    for b in &bars {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            b.timestamp.to_rfc3339(),
            b.open,
            b.high,
            b.low,
            b.close,
            b.volume,
            b.symbol
        ));
    }
    std::fs::write(&args.out, out)?;
    theme::success(&format!("Exported to {}", args.out.display()));
    Ok(())
}

async fn sources() -> Result<()> {
    theme::section_header("Data Sources");
    theme::status_line("yahoo", "default, no key", true);
    theme::status_line("polygon", "requires API key", false);
    theme::status_line("local", "CSV files", true);
    Ok(())
}
