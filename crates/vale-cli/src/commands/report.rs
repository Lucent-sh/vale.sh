use crate::cli::ReportCommand;
use crate::theme;
use anyhow::Result;
use vale_core::types::{BacktestResult, OutputFormat};

pub async fn handle(cmd: ReportCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        ReportCommand::Tearsheet(args) => {
            let text = std::fs::read_to_string(&args.input)?;
            let result: BacktestResult = serde_json::from_str(&text)?;
            if args.format == "html" {
                let html = vale_report::html::generate_tearsheet(&result);
                if let Some(path) = args.out {
                    std::fs::write(&path, &html)?;
                    theme::success(&format!("Wrote {}", path.display()));
                } else {
                    println!("{html}");
                }
            } else {
                vale_report::tearsheet::print_tearsheet(&result);
            }
            let _ = (output, args.open);
        }
        ReportCommand::Show(args) => {
            let text = std::fs::read_to_string(&args.result)?;
            let result: BacktestResult = serde_json::from_str(&text)?;
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
                OutputFormat::Csv => print!("{}", vale_report::csv::backtest_equity_curve(&result)),
                OutputFormat::Table => vale_report::tearsheet::print_tearsheet(&result),
            }
        }
    }
    Ok(())
}
