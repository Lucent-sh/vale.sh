use crate::frontier::efficient_frontier;
use crate::weights::Weights;
use nalgebra::DMatrix;

pub fn frontier_to_csv(
    frontier: &[(f64, f64, Weights)],
    tickers: &[String],
) -> String {
    let mut header = String::from("return,volatility");
    for t in tickers {
        header.push_str(&format!(",w_{t}"));
    }
    header.push('\n');

    let mut rows = String::new();
    for (ret, vol, w) in frontier {
        rows.push_str(&format!("{ret},{vol}"));
        for t in tickers {
            let weight = w.0.get(t).copied().unwrap_or(0.0);
            rows.push_str(&format!(",{weight}"));
        }
        rows.push('\n');
    }
    format!("{header}{rows}")
}

pub fn compute_frontier_csv(
    returns_matrix: &DMatrix<f64>,
    tickers: &[String],
    points: usize,
    rf: f64,
) -> String {
    let frontier = efficient_frontier(returns_matrix, tickers, points, rf);
    frontier_to_csv(&frontier, tickers)
}
