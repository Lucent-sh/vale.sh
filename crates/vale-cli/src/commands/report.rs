use crate::cli::ReportCommand;
use crate::theme;
use anyhow::Result;
use std::process::Command;
use vale_core::types::{BacktestResult, OutputFormat};

pub async fn handle(cmd: ReportCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        ReportCommand::Tearsheet(args) => {
            let text = std::fs::read_to_string(&args.input)?;
            let result: BacktestResult = serde_json::from_str(&text)?;
            if args.format == "html" {
                let html = vale_report::html::generate_tearsheet(&result);
                let path = args
                    .out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("vale-tearsheet.html"));
                std::fs::write(&path, &html)?;
                theme::success(&format!("Wrote {}", path.display()));
                if args.open {
                    open_in_browser(&path)?;
                }
            } else {
                vale_report::tearsheet::print_tearsheet(&result);
            }
            let _ = output;
        }
        ReportCommand::Show(args) => {
            let text = std::fs::read_to_string(&args.result)?;
            let result: BacktestResult = serde_json::from_str(&text)?;
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
                OutputFormat::Csv => {
                    if std::env::var("VALE_CSV_TRADES").is_ok() {
                        print!("{}", vale_report::csv::backtest_trades(&result));
                    } else {
                        print!("{}", vale_report::csv::backtest_equity_curve(&result));
                    }
                }
                OutputFormat::Table => vale_report::tearsheet::print_tearsheet(&result),
            }
        }
        ReportCommand::Trades(args) => {
            let text = std::fs::read_to_string(&args.input)?;
            let result: BacktestResult = serde_json::from_str(&text)?;
            let csv = vale_report::csv::backtest_trades(&result);
            if let Some(path) = args.out {
                std::fs::write(&path, csv)?;
                theme::success(&format!("Wrote {}", path.display()));
            } else {
                print!("{csv}");
            }
        }
    }
    Ok(())
}

fn open_in_browser(path: &std::path::Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    #[cfg(target_os = "macos")]
    Command::new("open").arg(path_str.as_ref()).status()?;
    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(path_str.as_ref()).status()?;
    #[cfg(target_os = "windows")]
    Command::new("cmd")
        .args(["/C", "start", "", path_str.as_ref()])
        .status()?;
    Ok(())
}
