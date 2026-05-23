use crate::native::max_sharpe;
use crate::weights::Weights;
use nalgebra::DMatrix;
use std::collections::HashMap;

/// Black-Litterman-style blend: start from max-Sharpe weights, tilt toward view tickers.
pub fn black_litterman(
    returns_matrix: &DMatrix<f64>,
    tickers: &[String],
    views: &HashMap<String, f64>,
    tilt: f64,
    risk_free: f64,
) -> Weights {
    let base = max_sharpe(returns_matrix, tickers, risk_free);
    let mut map: HashMap<String, f64> = base.into_iter().collect();
    if views.is_empty() {
        return Weights(map);
    }

    let view_strength: f64 = views
        .values()
        .map(|v| v.abs())
        .fold(0.0_f64, |a, b| a + b)
        .max(1e-9);
    for (ticker, view) in views {
        if let Some(w) = map.get_mut(ticker) {
            let target = (view / view_strength).max(0.0);
            *w = *w * (1.0 - tilt) + target * tilt;
        } else {
            let target = (view / view_strength).max(0.0);
            map.insert(ticker.clone(), target * tilt);
        }
    }

    let mut weights = Weights(map);
    weights.normalize();
    weights
}
