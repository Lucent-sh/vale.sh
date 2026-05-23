use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use vale_backtest::strategies::buy_and_hold::BuyAndHold;
use vale_backtest::strategies::sma_crossover::SmaCrossover;
use vale_backtest::strategy::Strategy;

pub fn build_strategy(
    name: &Path,
    ticker: &str,
    params: &[(String, f64)],
) -> Result<Box<dyn Strategy>> {
    let stem = name
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("buy_and_hold");
    match stem {
        "buy_and_hold" => Ok(Box::new(BuyAndHold::new(ticker))),
        "sma_crossover" => {
            let fast = param_usize(params, "fast_ma", 10);
            let slow = param_usize(params, "slow_ma", 50);
            Ok(Box::new(SmaCrossover::new(ticker, fast, slow)))
        }
        other => anyhow::bail!("unknown strategy: {other}"),
    }
}

pub fn build_strategy_from_map(
    name: &Path,
    ticker: &str,
    params: &HashMap<String, f64>,
) -> Result<Box<dyn Strategy>> {
    let vec: Vec<(String, f64)> = params.iter().map(|(k, v)| (k.clone(), *v)).collect();
    build_strategy(name, ticker, &vec)
}

fn param_usize(params: &[(String, f64)], key: &str, default: usize) -> usize {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| *v as usize)
        .unwrap_or(default)
}

pub fn params_from_grid(config: &[(String, f64)]) -> HashMap<String, f64> {
    config.iter().cloned().collect()
}
