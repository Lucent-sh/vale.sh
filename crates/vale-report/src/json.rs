use serde::Serialize;
use vale_core::types::BacktestResult;

#[derive(Debug, Serialize)]
pub struct FactorReportJson {
    pub alpha: f64,
    pub betas: Vec<f64>,
    pub t_stats: Vec<f64>,
    pub r_squared: f64,
    pub information_ratio: f64,
}

pub fn backtest_json(result: &BacktestResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".into())
}

pub fn to_pretty_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
}
