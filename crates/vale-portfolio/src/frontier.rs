use crate::native::{max_sharpe, min_variance};
use crate::weights::Weights;
use nalgebra::DMatrix;
use std::collections::HashMap;
use vale_risk::metrics::{mean, std_dev};

/// Efficient frontier: Vec of (return, volatility, weights).
pub fn efficient_frontier(
    returns_matrix: &DMatrix<f64>,
    tickers: &[String],
    points: usize,
    rf: f64,
) -> Vec<(f64, f64, Weights)> {
    let mut frontier = Vec::new();
    if points == 0 {
        return frontier;
    }

    let min_w = min_variance(returns_matrix, tickers);
    let max_w = max_sharpe(returns_matrix, tickers, rf);

    for i in 0..points {
        let t = i as f64 / (points - 1).max(1) as f64;
        let mut combined = HashMap::new();
        for ticker in tickers {
            let w_min = min_w
                .iter()
                .find(|(t, _)| t == ticker)
                .map(|(_, w)| *w)
                .unwrap_or(0.0);
            let w_max = max_w
                .iter()
                .find(|(t, _)| t == ticker)
                .map(|(_, w)| *w)
                .unwrap_or(0.0);
            combined.insert(ticker.clone(), (1.0 - t) * w_min + t * w_max);
        }
        let mut weights = Weights(combined);
        weights.normalize();
        let (ret, vol) = portfolio_stats(returns_matrix, tickers, &weights);
        frontier.push((ret, vol, weights));
    }
    frontier
}

fn portfolio_stats(returns: &DMatrix<f64>, tickers: &[String], weights: &Weights) -> (f64, f64) {
    let n_rows = returns.nrows();
    let mut port_returns = Vec::with_capacity(n_rows);
    for i in 0..n_rows {
        let mut r = 0.0;
        for (j, ticker) in tickers.iter().enumerate() {
            let w = weights.0.get(ticker).copied().unwrap_or(0.0);
            r += w * returns[(i, j)];
        }
        port_returns.push(r);
    }
    (
        mean(&port_returns) * 252.0,
        std_dev(&port_returns) * 252.0_f64.sqrt(),
    )
}
