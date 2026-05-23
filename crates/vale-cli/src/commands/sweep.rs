use crate::cli::SweepCommand;
use crate::strategy;
use crate::theme;
use anyhow::{Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use vale_backtest::commission::PercentageCommission;
use vale_backtest::engine::BacktestEngine;
use vale_backtest::slippage::PercentageSlippage;
use vale_core::config::Config;
use vale_core::types::{OutputFormat, TimeRange};
use vale_data::build_provider;
use vale_sweep::{
    append_checkpoint, cartesian_product, load_checkpoint, rank_by_metric, run_sweep_with_hook,
    save_checkpoint, ParamRange, SweepResult,
};

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

            let mut configs = cartesian_product(&param_ranges);
            if let Some(ref ckpt) = args.checkpoint {
                if ckpt.exists() {
                    let done = load_checkpoint(ckpt).map_err(|e| anyhow::anyhow!("{e}"))?;
                    let done_keys: std::collections::HashSet<String> = done
                        .iter()
                        .filter_map(|r| serde_json::to_string(&r.params).ok())
                        .collect();
                    configs.retain(|c| {
                        let m = strategy::params_from_grid(c);
                        let key = serde_json::to_string(&m).unwrap_or_default();
                        !done_keys.contains(&key)
                    });
                    theme::info(&format!(
                        "Resuming sweep: {} configs remaining",
                        configs.len()
                    ));
                }
            }

            let total = configs.len();
            let resolved = strategy::resolve_strategy(&args.strategy, &args.ticker)?;
            let bars_arc = Arc::new(bars);
            let ticker = resolved.ticker.clone();
            let strategy_path = resolved.builtin.clone();
            let strategy_params = resolved.params.clone();

            let engine = BacktestEngine {
                commission: Box::new(PercentageCommission { rate: 0.001 }),
                slippage: Box::new(PercentageSlippage { rate: 0.0005 }),
                initial_cash: 100_000.0,
            };

            let use_dashboard = matches!(output, OutputFormat::Table);
            let checkpoint_path = args.checkpoint.clone();
            let strategy_factory = {
                let strategy_path = strategy_path.clone();
                let ticker = ticker.clone();
                let base_params = strategy_params.clone();
                move |config: &[(String, f64)]| {
                    let mut params = strategy::params_from_grid(config);
                    for (k, v) in &base_params {
                        params.entry(k.clone()).or_insert(*v);
                    }
                    strategy::build_strategy_from_map(&strategy_path, &ticker, &params)
                        .expect("strategy params")
                }
            };

            let results: Vec<SweepResult> = if use_dashboard {
                let (tx, rx) = mpsc::channel::<SweepResult>();
                let metric = args.metric.clone();
                let top = args.top;
                let ui_handle = thread::spawn(move || {
                    let _ = crate::ui::sweep_dashboard::run_dashboard(rx, total, metric, top);
                });

                let hook_tx = tx.clone();
                let ckpt = checkpoint_path.clone();
                let ranked = run_sweep_with_hook(
                    configs,
                    strategy_factory,
                    &bars_arc,
                    &engine,
                    Some(move |sr: &SweepResult| {
                        let _ = hook_tx.send(sr.clone());
                        if let Some(ref path) = ckpt {
                            let _ = append_checkpoint(path, sr);
                        }
                    }),
                );
                drop(tx);
                let _ = ui_handle.join();
                ranked
            } else {
                let ckpt = checkpoint_path.clone();
                run_sweep_with_hook(
                    configs,
                    strategy_factory,
                    &bars_arc,
                    &engine,
                    ckpt.as_ref().map(|path| {
                        let path = path.clone();
                        move |sr: &SweepResult| {
                            let _ = append_checkpoint(&path, sr);
                        }
                    }),
                )
            };

            let mut ranked = results;
            rank_by_metric(&mut ranked, &args.metric);
            ranked.truncate(args.top);

            if let Some(ref path) = checkpoint_path {
                save_checkpoint(path, &ranked).ok();
            }

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
