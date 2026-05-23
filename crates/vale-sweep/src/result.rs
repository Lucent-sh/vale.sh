use std::collections::HashMap;
use vale_core::types::BacktestResult;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SweepResult {
    pub params: HashMap<String, f64>,
    pub result: BacktestResult,
}

/// Rank sweep results by metric name (sharpe, cagr, total_return, max_drawdown inverted).
pub fn rank_by_metric(results: &mut [SweepResult], metric: &str) {
    results.sort_by(|a, b| {
        let va = metric_value(&a.result, metric);
        let vb = metric_value(&b.result, metric);
        vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn metric_value(result: &BacktestResult, metric: &str) -> f64 {
    match metric {
        "sharpe" | "sharpe_ratio" => result.sharpe_ratio,
        "cagr" => result.cagr,
        "total_return" | "return" => result.total_return,
        "sortino" | "sortino_ratio" => result.sortino_ratio,
        "calmar" | "calmar_ratio" => result.calmar_ratio,
        "max_dd" | "max_drawdown" => -result.max_drawdown,
        "win_rate" => result.win_rate,
        "profit_factor" => result.profit_factor,
        _ => result.sharpe_ratio,
    }
}
