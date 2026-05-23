use crate::cli::RiskCommand;
use crate::theme;
use anyhow::{Context, Result};
use vale_core::types::OutputFormat;
use vale_risk::drawdown::max_drawdown;
use vale_risk::metrics::{
    cagr, historical_var, log_returns, sharpe_ratio, sortino_ratio, volatility_annual,
};

pub async fn handle(cmd: RiskCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        RiskCommand::Metrics(args) => {
            let mut rdr = csv::Reader::from_path(&args.input)?;
            let mut equity = Vec::new();
            for result in rdr.records() {
                let record = result?;
                if record.len() >= 2 {
                    if let Ok(e) = record[1].parse::<f64>() {
                        equity.push(e);
                    }
                }
            }
            let returns = log_returns(&equity);
            let ann = 252.0_f64.sqrt();
            let rf_daily = args.risk_free / 252.0;
            let years = (equity.len() as f64 / 252.0).max(1.0);

            let mut metrics: Vec<(&str, String)> = vec![
                (
                    "Sharpe",
                    format!("{:.4}", sharpe_ratio(&returns, rf_daily, ann)),
                ),
                (
                    "Sortino",
                    format!("{:.4}", sortino_ratio(&returns, rf_daily, ann)),
                ),
                ("CAGR", format!("{:.4}", cagr(&equity, years))),
                (
                    "Volatility (Ann.)",
                    format!("{:.4}", volatility_annual(&returns, ann)),
                ),
                ("Max Drawdown", format!("{:.4}", max_drawdown(&equity))),
            ];
            for &c in &args.var_confidence {
                metrics.push((
                    "VaR",
                    format!("{:.4} @ {:.0}%", historical_var(&returns, c), c * 100.0),
                ));
            }

            match output {
                OutputFormat::Json => {
                    let map: std::collections::HashMap<_, _> =
                        metrics.iter().map(|(k, v)| (*k, v.clone())).collect();
                    println!("{}", serde_json::to_string_pretty(&map)?);
                }
                OutputFormat::Csv => {
                    println!("metric,value");
                    for (k, v) in &metrics {
                        println!("{k},{v}");
                    }
                }
                OutputFormat::Table => {
                    theme::section_header("Risk Metrics");
                    let metrics_display: Vec<(String, String)> = metrics
                        .iter()
                        .map(|(k, v)| {
                            (
                                (*k).to_string(),
                                if k == &"Sharpe" || k == &"CAGR" {
                                    theme::colored_metric(v.parse().unwrap_or(0.0), true)
                                } else {
                                    v.clone()
                                },
                            )
                        })
                        .collect();
                    let metrics_refs: Vec<(&str, String)> = metrics_display
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.clone()))
                        .collect();
                    let mut table = vale_report::table::risk_table(&metrics_refs);
                    theme::table_style(&mut table);
                    println!("{table}");
                }
            }
        }
        RiskCommand::Stress(args) => {
            let content = std::fs::read_to_string(&args.portfolio)?;
            let portfolio: std::collections::HashMap<String, f64> =
                serde_json::from_str(&content).context("portfolio JSON")?;
            let scenarios = vale_risk::stress::builtin_scenarios();
            let mut rows = Vec::new();
            for name in &args.scenarios {
                if let Some(scenario) = scenarios.get(name.as_str()) {
                    let impact = vale_risk::stress::apply_scenario(&portfolio, scenario);
                    rows.push((name.clone(), format!("{:.2}%", impact * 100.0)));
                }
            }
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
                OutputFormat::Csv => {
                    println!("scenario,impact");
                    for (n, v) in &rows {
                        println!("{n},{v}");
                    }
                }
                OutputFormat::Table => {
                    theme::section_header("Stress Test");
                    for (n, v) in &rows {
                        theme::status_line(n, v, true);
                    }
                }
            }
        }
        RiskCommand::Correlation(args) => {
            theme::info(&format!(
                "Correlation for {} tickers ({}) — fetch data first",
                args.tickers.len(),
                args.method
            ));
            let _ = args.start;
        }
    }
    Ok(())
}
