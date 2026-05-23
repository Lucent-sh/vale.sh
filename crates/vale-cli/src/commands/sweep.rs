use crate::cli::SweepCommand;
use crate::theme;
use anyhow::{Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use vale_backtest::commission::PercentageCommission;
use vale_backtest::engine::BacktestEngine;
use vale_backtest::slippage::PercentageSlippage;
use vale_backtest::strategies::sma_crossover::SmaCrossover;
use vale_backtest::strategy::Strategy;
use vale_core::config::Config;
use vale_core::types::{OutputFormat, TimeRange};
use vale_data::build_provider;
use vale_sweep::{cartesian_product, rank_by_metric, ParamRange, SweepResult};

pub async fn handle(cmd: SweepCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        SweepCommand::Run(args) => {
            let config = Config::load()?;
            let start_date = NaiveDate::parse_from_str(&args.start, "%Y-%m-%d")?;
            let end_date = NaiveDate::parse_from_str(&args.end, "%Y-%m-%d")?;
            let range = TimeRange {
                start: Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).context("time")?),
                end: Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59).context("time")?),
            };

            let provider = build_provider(&config)?;
            let bars = provider
                .fetch_ohlcv(&args.ticker, args.resolution, &range)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            let mut param_ranges: Vec<ParamRange> = args
                .params
                .iter()
                .map(|s| ParamRange::parse(s))
                .collect::<Result<_, _>>()?;

            if param_ranges.is_empty() {
                param_ranges.push(ParamRange::parse("fast_ma:5:20:5")?);
                param_ranges.push(ParamRange::parse("slow_ma:20:60:10")?);
            }

            let configs = cartesian_product(&param_ranges);
            let total = configs.len();

            let (tx, rx) = mpsc::channel::<SweepResult>();
            let bars_arc = Arc::new(bars);
            let ticker = args.ticker.clone();

            let use_dashboard = matches!(output, OutputFormat::Table);
            let ui_handle = if use_dashboard {
                let rx_ui = rx;
                let metric = args.metric.clone();
                let top = args.top;
                Some(thread::spawn(move || {
                    let _ = crate::ui::sweep_dashboard::run_dashboard(rx_ui, total, metric, top);
                }))
            } else {
                None
            };

            let engine = BacktestEngine {
                commission: Box::new(PercentageCommission { rate: 0.001 }),
                slippage: Box::new(PercentageSlippage { rate: 0.0005 }),
                initial_cash: 100_000.0,
            };

            let results: Vec<SweepResult> = configs
                .into_iter()
                .filter_map(|config| {
                    let fast = config
                        .iter()
                        .find(|(k, _)| k == "fast_ma")
                        .map(|(_, v)| *v as usize)
                        .unwrap_or(10);
                    let slow = config
                        .iter()
                        .find(|(k, _)| k == "slow_ma")
                        .map(|(_, v)| *v as usize)
                        .unwrap_or(50);
                    let mut strat: Box<dyn Strategy> =
                        Box::new(SmaCrossover::new(&ticker, fast, slow));
                    match engine.run(strat.as_mut(), &bars_arc) {
                        Ok(result) => {
                            let sr = SweepResult {
                                params: config.into_iter().collect(),
                                result,
                            };
                            let _ = tx.send(sr.clone());
                            Some(sr)
                        }
                        Err(_) => None,
                    }
                })
                .collect();
            drop(tx);

            if let Some(h) = ui_handle {
                let _ = h.join();
            }

            let mut ranked = results;
            rank_by_metric(&mut ranked, &args.metric);
            ranked.truncate(args.top);

            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&ranked)?),
                OutputFormat::Csv => {
                    println!("fast_ma,slow_ma,sharpe,cagr,max_dd,win_rate");
                    for r in &ranked {
                        println!(
                            "{},{},{},{},{},{}",
                            r.params.get("fast_ma").unwrap_or(&0.0),
                            r.params.get("slow_ma").unwrap_or(&0.0),
                            r.result.sharpe_ratio,
                            r.result.cagr,
                            r.result.max_drawdown,
                            r.result.win_rate
                        );
                    }
                }
                OutputFormat::Table => {
                    if !use_dashboard {
                        theme::section_header("Top Sweep Results");
                        for (i, r) in ranked.iter().enumerate() {
                            theme::info(&format!(
                                "#{} sharpe={:.3} cagr={:.2}%",
                                i + 1,
                                r.result.sharpe_ratio,
                                r.result.cagr * 100.0
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
