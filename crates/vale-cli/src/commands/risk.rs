use crate::cli::RiskCommand;
use crate::theme;
use anyhow::{Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use vale_core::config::Config;
use vale_core::types::{OutputFormat, TimeRange};
use vale_data::build_provider;
use vale_risk::correlation::correlation_matrix;
use vale_risk::drawdown::max_drawdown;
use vale_risk::metrics::{
    alpha, beta, cagr, cvar, historical_var, log_returns, sharpe_ratio, sortino_ratio,
    volatility_annual,
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

            let mut metrics: Vec<(String, String)> = vec![
                (
                    "Sharpe".into(),
                    format!("{:.4}", sharpe_ratio(&returns, rf_daily, ann)),
                ),
                (
                    "Sortino".into(),
                    format!("{:.4}", sortino_ratio(&returns, rf_daily, ann)),
                ),
                ("CAGR".into(), format!("{:.4}", cagr(&equity, years))),
                (
                    "Volatility (Ann.)".into(),
                    format!("{:.4}", volatility_annual(&returns, ann)),
                ),
                (
                    "Max Drawdown".into(),
                    format!("{:.4}", max_drawdown(&equity)),
                ),
            ];
            for &c in &args.var_confidence {
                metrics.push((
                    format!("VaR @ {:.0}%", c * 100.0),
                    format!("{:.4}", historical_var(&returns, c)),
                ));
                metrics.push((
                    format!("CVaR @ {:.0}%", c * 100.0),
                    format!("{:.4}", cvar(&returns, c)),
                ));
            }

            if let Some(bench_path) = &args.benchmark {
                let mut bench_equity = Vec::new();
                let mut rdr = csv::Reader::from_path(bench_path)?;
                for result in rdr.records() {
                    let record = result?;
                    if record.len() >= 2 {
                        if let Ok(e) = record[1].parse::<f64>() {
                            bench_equity.push(e);
                        }
                    }
                }
                let bench_returns = log_returns(&bench_equity);
                let len = returns.len().min(bench_returns.len());
                if len > 1 {
                    let r = &returns[returns.len() - len..];
                    let b = &bench_returns[bench_returns.len() - len..];
                    metrics.push(("Beta".into(), format!("{:.4}", beta(r, b))));
                    metrics.push((
                        "Alpha (Ann.)".into(),
                        format!("{:.4}", alpha(r, b, rf_daily, 252.0)),
                    ));
                }
            }

            match output {
                OutputFormat::Json => {
                    let map: std::collections::HashMap<_, _> =
                        metrics.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
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
                            let colored = if k.starts_with("Sharpe") || k == "CAGR" {
                                theme::colored_metric(v.parse().unwrap_or(0.0), true)
                            } else {
                                v.clone()
                            };
                            (k.clone(), colored)
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
            let config = Config::load()?;
            let start_date = NaiveDate::parse_from_str(&args.start, "%Y-%m-%d")?;
            let end = Utc::now();
            let start = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).context("time")?);
            let range = TimeRange { start, end };

            let provider = build_provider(&config)?;
            let mut series = Vec::new();
            let mut labels = Vec::new();

            for ticker in &args.tickers {
                let bars = provider
                    .fetch_ohlcv(ticker, vale_core::types::Resolution::Daily, &range)
                    .await
                    .map_err(|e| anyhow::anyhow!("{ticker}: {e}"))?;
                let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
                series.push(log_returns(&closes));
                labels.push(ticker.clone());
            }

            if let Some(window) = args.rolling {
                if args.tickers.len() == 2 {
                    let rolling =
                        vale_risk::correlation::rolling_correlation(&series[0], &series[1], window);
                    match output {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&rolling)?);
                        }
                        OutputFormat::Csv => {
                            println!("rolling_correlation");
                            for v in rolling {
                                println!("{v}");
                            }
                        }
                        OutputFormat::Table => {
                            theme::section_header(&format!(
                                "Rolling {} correlation (window={window})",
                                args.method
                            ));
                            for (i, v) in rolling.iter().enumerate() {
                                theme::info(&format!("t+{}: {v:.4}", i + window));
                            }
                        }
                    }
                    return Ok(());
                }
            }

            let matrix = correlation_matrix(&series, &args.method);

            match output {
                OutputFormat::Json => {
                    let payload = serde_json::json!({
                        "labels": labels,
                        "matrix": matrix,
                        "method": args.method,
                    });
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
                OutputFormat::Csv => {
                    print!(",");
                    for label in &labels {
                        print!("{label},");
                    }
                    println!();
                    for (i, row) in matrix.iter().enumerate() {
                        print!("{},", labels[i]);
                        for v in row {
                            print!("{v:.6},");
                        }
                        println!();
                    }
                }
                OutputFormat::Table => {
                    theme::section_header(&format!("Correlation ({})", args.method));
                    print!("{:>8}", "");
                    for label in &labels {
                        print!(" {:>8}", label);
                    }
                    println!();
                    for (i, row) in matrix.iter().enumerate() {
                        print!("{:>8}", labels[i]);
                        for v in row {
                            print!(" {:>8.3}", v);
                        }
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}
