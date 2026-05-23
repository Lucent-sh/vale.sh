use crate::result::SweepResult;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
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
    run_sweep_with_hook(configs, strategy_factory, bars, engine, None::<fn(&SweepResult)>)
}

/// Parallel sweep with optional per-result hook (e.g. TUI channel).
pub fn run_sweep_with_hook<F, H>(
    configs: Vec<Vec<(String, f64)>>,
    strategy_factory: F,
    bars: &[Bar],
    engine: &BacktestEngine,
    hook: Option<H>,
) -> Vec<SweepResult>
where
    F: Fn(&[(String, f64)]) -> Box<dyn Strategy> + Send + Sync,
    H: Fn(&SweepResult) + Send + Sync,
{
    let hook = hook.map(Arc::new);

    configs
        .par_iter()
        .filter_map(|config| {
            let mut strat = strategy_factory(config);
            match engine.run(strat.as_mut(), bars) {
                Ok(mut result) => {
                    let params_map: HashMap<_, _> = config.iter().cloned().collect();
                    result.params =
                        serde_json::to_value(&params_map).unwrap_or(serde_json::json!({}));
                    let sr = SweepResult {
                        params: params_map,
                        result,
                    };
                    if let Some(ref h) = hook {
                        h(&sr);
                    }
                    Some(sr)
                }
                Err(e) => {
                    tracing::warn!("config failed: {e:?}");
                    None
                }
            }
        })
        .collect()
}
