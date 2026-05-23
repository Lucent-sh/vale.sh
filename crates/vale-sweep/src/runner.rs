use crate::result::SweepResult;
use rayon::prelude::*;
use std::collections::HashMap;
use vale_backtest::engine::BacktestEngine;
use vale_backtest::strategy::Strategy;
use vale_core::types::Bar;

/// Run all configurations in parallel via Rayon.
pub fn run_sweep<F>(
    configs: Vec<Vec<(String, f64)>>,
    strategy_factory: F,
    bars: &[Bar],
    engine: &BacktestEngine,
) -> Vec<SweepResult>
where
    F: Fn(&[(String, f64)]) -> Box<dyn Strategy> + Send + Sync,
{
    configs
        .par_iter()
        .filter_map(|config| {
            let mut strat = strategy_factory(config);
            match engine.run(strat.as_mut(), bars) {
                Ok(result) => Some(SweepResult {
                    params: config.iter().cloned().collect::<HashMap<_, _>>(),
                    result,
                }),
                Err(e) => {
                    tracing::warn!("config failed: {e:?}");
                    None
                }
            }
        })
        .collect()
}
