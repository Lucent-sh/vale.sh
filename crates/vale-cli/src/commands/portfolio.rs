use crate::cli::PortfolioCommand;
use crate::theme;
use anyhow::{Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use nalgebra::DMatrix;
use std::collections::HashMap;
use vale_core::config::Config;
use vale_core::types::{OutputFormat, TimeRange};
use vale_data::build_provider;
use vale_portfolio::native::{equal_weight, max_sharpe, min_variance};
use vale_portfolio::weights::Weights;
use vale_portfolio::{efficient_frontier, portfolio_backtest};

pub async fn handle(cmd: PortfolioCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        PortfolioCommand::Optimize(args) => {
            let config = Config::load()?;
            let returns_matrix =
                fetch_returns_matrix(&config, &args.tickers, &args.start, args.end.as_deref())
                    .await?;
            let tickers: Vec<String> = args.tickers.clone();

            let weights = match args.method.as_str() {
                "min_variance" => min_variance(&returns_matrix, &tickers),
                "max_sharpe" => max_sharpe(&returns_matrix, &tickers, args.risk_free),
                "equal_weight" | "equal" => equal_weight(&tickers),
                "hrp" | "risk_parity" | "black_litterman" => {
                    let json = serde_json::to_string(&returns_matrix_to_vec(&returns_matrix))?;
                    vale_portfolio::skfolio::optimize_via_skfolio(&args.method, &json, &tickers)
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))?
                }
                other => anyhow::bail!("unknown method: {other}"),
            };

            let w = Weights(weights.into_iter().collect());
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&w.0)?),
                OutputFormat::Csv => {
                    println!("ticker,weight");
                    for (t, v) in &w.0 {
                        println!("{t},{v}");
                    }
                }
                OutputFormat::Table => {
                    theme::section_header("Portfolio Weights");
                    let mut table = vale_report::table::portfolio_table(&w);
                    theme::table_style(&mut table);
                    println!("{table}");
                }
            }
        }
        PortfolioCommand::Backtest(args) => {
            let content = std::fs::read_to_string(&args.weights)?;
            let map: HashMap<String, f64> = serde_json::from_str(&content)?;
            let weights = Weights(map);
            let config = Config::load()?;
            let mut bars_by_ticker = HashMap::new();
            for ticker in weights.0.keys() {
                let bars =
                    fetch_ticker_bars(&config, ticker, &args.start, args.end.as_deref()).await?;
                bars_by_ticker.insert(ticker.clone(), bars);
            }
            let rebalance_days = match args.rebalance.as_str() {
                "daily" => 1,
                "weekly" => 7,
                _ => 30,
            };
            let result = portfolio_backtest(&bars_by_ticker, &weights, rebalance_days, 100_000.0);
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
                OutputFormat::Csv => {
                    println!("timestamp,equity");
                    for (t, e) in &result.equity_curve {
                        println!("{},{}", t.to_rfc3339(), e);
                    }
                }
                OutputFormat::Table => {
                    theme::section_header("Portfolio Backtest");
                    theme::status_line(
                        "total return",
                        &format!("{:.2}%", result.total_return * 100.0),
                        true,
                    );
                    theme::status_line("sharpe", &format!("{:.3}", result.sharpe_ratio), true);
                }
            }
        }
        PortfolioCommand::EfficientFrontier(args) => {
            let config = Config::load()?;
            let returns_matrix =
                fetch_returns_matrix(&config, &args.tickers, &args.start, None).await?;
            let frontier = efficient_frontier(&returns_matrix, &args.tickers, args.points, 0.05);
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&frontier)?),
                OutputFormat::Csv => {
                    println!("return,volatility");
                    for (r, v, _) in &frontier {
                        println!("{r},{v}");
                    }
                }
                OutputFormat::Table => {
                    theme::section_header("Efficient Frontier");
                    theme::info(&format!("{} points computed", frontier.len()));
                }
            }
            if let Some(path) = args.output {
                std::fs::write(&path, serde_json::to_string_pretty(&frontier)?)?;
            }
        }
    }
    Ok(())
}

async fn fetch_returns_matrix(
    config: &vale_core::config::Config,
    tickers: &[String],
    start: &str,
    end: Option<&str>,
) -> Result<DMatrix<f64>> {
    let mut all_returns = Vec::new();
    let mut min_len = usize::MAX;
    for ticker in tickers {
        let bars = fetch_ticker_bars(config, ticker, start, end).await?;
        let prices: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let rets = vale_risk::metrics::simple_returns(&prices);
        min_len = min_len.min(rets.len());
        all_returns.push(rets);
    }
    let n = min_len;
    let k = tickers.len();
    let mut matrix = DMatrix::zeros(n, k);
    for (j, rets) in all_returns.iter().enumerate() {
        let offset = rets.len() - n;
        for i in 0..n {
            matrix[(i, j)] = rets[offset + i];
        }
    }
    Ok(matrix)
}

async fn fetch_ticker_bars(
    config: &vale_core::config::Config,
    ticker: &str,
    start: &str,
    end: Option<&str>,
) -> Result<Vec<vale_core::types::Bar>> {
    let start_date = NaiveDate::parse_from_str(start, "%Y-%m-%d")?;
    let end_date = match end {
        Some(e) => NaiveDate::parse_from_str(e, "%Y-%m-%d")?,
        None => Utc::now().date_naive(),
    };
    let range = TimeRange {
        start: Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).context("time")?),
        end: Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59).context("time")?),
    };
    let provider = build_provider(config)?;
    provider
        .fetch_ohlcv(ticker, vale_core::types::Resolution::Daily, &range)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

fn returns_matrix_to_vec(matrix: &DMatrix<f64>) -> Vec<Vec<f64>> {
    let mut out = vec![vec![]; matrix.nrows()];
    for i in 0..matrix.nrows() {
        for j in 0..matrix.ncols() {
            out[i].push(matrix[(i, j)]);
        }
    }
    out
}
