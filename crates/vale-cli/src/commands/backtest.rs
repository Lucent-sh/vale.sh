use crate::cli::{BacktestCommand, BacktestEngineArg, BacktestRunArgs};
use crate::strategy;
use crate::theme;
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vale_backtest::commission::PercentageCommission;
use vale_backtest::engine::BacktestEngine;
use vale_backtest::slippage::PercentageSlippage;
use vale_core::config::Config;
use vale_core::types::{BacktestResult, Bar, OutputFormat, TimeRange};
use vale_data::build_provider;
use vale_risk::metrics::{beta, log_returns};

#[cfg(feature = "lean")]
use vale_adapters::lean::LeanAdapter;
#[cfg(feature = "vectorbt")]
use vale_adapters::vectorbt::VectorBtAdapter;

pub async fn handle(cmd: BacktestCommand, output: OutputFormat, verbose: bool) -> Result<()> {
    match cmd {
        BacktestCommand::Run(args) => run(args, output, verbose).await,
        BacktestCommand::Compare(args) => compare(args, output).await,
        BacktestCommand::Validate(args) => validate(args).await,
    }
}

fn parse_range(start: &str, end: &str) -> Result<TimeRange> {
    let start_date = NaiveDate::parse_from_str(start, "%Y-%m-%d")
        .with_context(|| format!("invalid start date: {start}"))?;
    let end_date = NaiveDate::parse_from_str(end, "%Y-%m-%d")
        .with_context(|| format!("invalid end date: {end}"))?;
    Ok(TimeRange {
        start: Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).context("time")?),
        end: Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59).context("time")?),
    })
}

fn resolve_lean_project(strategy: &Path) -> PathBuf {
    if strategy.is_dir() {
        strategy.to_path_buf()
    } else if strategy.is_file() {
        strategy
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

fn benchmark_equity_curve(bars: &[Bar], initial_cash: f64) -> Vec<(DateTime<Utc>, f64)> {
    if bars.is_empty() {
        return Vec::new();
    }
    let first = bars[0].close;
    if first == 0.0 {
        return Vec::new();
    }
    bars.iter()
        .map(|b| (b.timestamp, initial_cash * b.close / first))
        .collect()
}

fn attach_benchmark(
    mut result: BacktestResult,
    bench_bars: &[Bar],
    initial_cash: f64,
) -> BacktestResult {
    let bench_curve = benchmark_equity_curve(bench_bars, initial_cash);
    let bench_by_day: HashMap<NaiveDate, f64> = bench_bars
        .iter()
        .map(|b| (b.timestamp.date_naive(), b.close))
        .collect();

    let mut asset_returns = Vec::new();
    let mut bench_returns = Vec::new();
    let equities: Vec<f64> = result.equity_curve.iter().map(|(_, e)| *e).collect();
    let asset_rets = log_returns(&equities);

    for (i, (ts, _)) in result.equity_curve.iter().enumerate().skip(1) {
        if let Some(&bench_close) = bench_by_day.get(&ts.date_naive()) {
            if let Some(prev_ts) = result.equity_curve.get(i - 1) {
                if let Some(&prev_bench) = bench_by_day.get(&prev_ts.0.date_naive()) {
                    if prev_bench > 0.0 && i - 1 < asset_rets.len() {
                        asset_returns.push(asset_rets[i - 1]);
                        bench_returns.push((bench_close / prev_bench).ln());
                    }
                }
            }
        }
    }

    let beta_val = beta(&asset_returns, &bench_returns);
    result.benchmark_curve = Some(bench_curve);
    if let Some(obj) = result.params.as_object_mut() {
        obj.insert("beta".into(), serde_json::json!(beta_val));
        if let Some(bench) = &result.benchmark_curve {
            if let Some((_, be)) = bench.last() {
                obj.insert(
                    "benchmark_return".into(),
                    serde_json::json!((be - initial_cash) / initial_cash),
                );
            }
        }
    }
    result
}

async fn run(args: BacktestRunArgs, output: OutputFormat, _verbose: bool) -> Result<()> {
    let config = Config::load()?;
    let range = parse_range(&args.start, &args.end)?;
    let resolved = strategy::resolve_strategy(&args.strategy, &args.ticker)?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(theme::spinner_style());
    pb.set_message(format!("loading data for {}…", resolved.ticker));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let provider = build_provider(&config)?;
    let bars = provider
        .fetch_ohlcv(&resolved.ticker, args.resolution, &range)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    pb.set_message("running backtest…");

    let engine = BacktestEngine {
        commission: Box::new(PercentageCommission { rate: 0.001 }),
        slippage: Box::new(PercentageSlippage { rate: 0.0005 }),
        initial_cash: args.cash,
    };

    let mut result = match args.engine {
        BacktestEngineArg::Native => {
            let mut strategy = strategy::build_resolved(&resolved)?;
            engine
                .run(strategy.as_mut(), &bars)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        BacktestEngineArg::Lean => {
            #[cfg(feature = "lean")]
            {
                let exe = LeanAdapter::detect_executable(&config.lean.executable).ok_or_else(|| {
                    anyhow::anyhow!(
                        "lean is not installed. Run `vale doctor` to see installation instructions."
                    )
                })?;
                let project = resolve_lean_project(&args.strategy);
                let adapter = LeanAdapter::new(exe, project);
                adapter.run_backtest().map_err(|e| anyhow::anyhow!("{e}"))?
            }
            #[cfg(not(feature = "lean"))]
            {
                anyhow::bail!("lean engine not enabled in this build");
            }
        }
        BacktestEngineArg::Vectorbt => {
            #[cfg(feature = "vectorbt")]
            {
                let python = VectorBtAdapter::detect_python().ok_or_else(|| {
                    anyhow::anyhow!(
                        "vectorbt is not installed. Run `vale doctor` to see installation instructions."
                    )
                })?;
                let stem = resolved
                    .builtin
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("sma_crossover");
                let params: serde_json::Value = resolved
                    .params
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                    .collect();
                VectorBtAdapter::new(python).run_backtest(
                    &resolved.ticker,
                    &args.start,
                    &args.end,
                    stem,
                    &params,
                    args.cash,
                )?
            }
            #[cfg(not(feature = "vectorbt"))]
            {
                anyhow::bail!("vectorbt engine not enabled in this build");
            }
        }
    };

    if let Some(bench_ticker) = &args.benchmark {
        pb.set_message(format!("loading benchmark {bench_ticker}…"));
        let bench_bars = provider
            .fetch_ohlcv(bench_ticker, args.resolution, &range)
            .await
            .map_err(|e| anyhow::anyhow!("benchmark fetch: {e}"))?;
        result = attach_benchmark(result, &bench_bars, args.cash);
    }

    pb.finish_and_clear();

    if let Some(path) = &args.save {
        std::fs::write(path, serde_json::to_string_pretty(&result)?)?;
        theme::success(&format!("Saved result to {}", path.display()));
    }

    match output {
        OutputFormat::Table => {
            theme::section_header("Backtest Result");
            let mut table = vale_report::table::backtest_summary(&result);
            theme::table_style(&mut table);
            println!("{table}");
            if let Some(beta) = result.params.get("beta") {
                theme::info(&format!("Beta vs benchmark: {beta:.4}"));
            }
            println!();
            theme::section_header("Equity Curve");
            println!("{}", vale_report::chart::equity_curve(&result, 120, 24));
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
        OutputFormat::Csv => print!("{}", vale_report::csv::backtest_equity_curve(&result)),
    }
    Ok(())
}

async fn compare(args: crate::cli::BacktestCompareArgs, output: OutputFormat) -> Result<()> {
    let mut results = Vec::new();
    for path in &args.results {
        let text = std::fs::read_to_string(path)?;
        let r: vale_core::types::BacktestResult = serde_json::from_str(&text)?;
        results.push((path.display().to_string(), r));
    }
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&results)?),
        OutputFormat::Csv => {
            println!("name,total_return,cagr,sharpe,max_drawdown,win_rate,trades");
            for (name, r) in &results {
                println!(
                    "{},{},{},{},{},{},{}",
                    csv_escape(name),
                    r.total_return,
                    r.cagr,
                    r.sharpe_ratio,
                    r.max_drawdown,
                    r.win_rate,
                    r.total_trades
                );
            }
        }
        OutputFormat::Table => {
            theme::section_header("Backtest Comparison");
            for (name, r) in &results {
                theme::info(name);
                let mut table = vale_report::table::backtest_summary(r);
                theme::table_style(&mut table);
                println!("{table}");
            }
        }
    }
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

async fn validate(args: crate::cli::BacktestValidateArgs) -> Result<()> {
    let findings = strategy::validate_strategy(&args.strategy)?;
    if findings.is_empty() {
        theme::success("Strategy validation passed (no issues found)");
        return Ok(());
    }

    theme::section_header("Strategy Validation");
    let mut errors = 0usize;
    for f in &findings {
        match f.level {
            "error" => {
                theme::error(&f.message);
                errors += 1;
            }
            _ => theme::warning(&f.message),
        }
    }

    if errors > 0 {
        anyhow::bail!("{errors} validation error(s)");
    }
    theme::success("Strategy validation passed with warnings");
    Ok(())
}
