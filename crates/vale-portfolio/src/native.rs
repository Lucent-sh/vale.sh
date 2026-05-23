use nalgebra::{DMatrix, DVector};

/// Minimum variance portfolio (long-only via simplex projection).
pub fn min_variance(returns_matrix: &DMatrix<f64>, tickers: &[String]) -> Vec<(String, f64)> {
    let n = tickers.len();
    if n == 0 {
        return vec![];
    }
    let cov = covariance_matrix(returns_matrix);
    let ones = DVector::from_element(n, 1.0);
    let inv = match cov.clone().try_inverse() {
        Some(inv) => inv,
        None => return equal_weights(tickers),
    };
    let numerator = &inv * &ones;
    let denom = ones.dot(&numerator);
    if denom.abs() < 1e-12 {
        return equal_weights(tickers);
    }
    let mut w = numerator / denom;
    project_simplex(&mut w);
    w.iter()
        .enumerate()
        .map(|(i, &v)| (tickers[i].clone(), v.max(0.0)))
        .collect()
}

/// Maximum Sharpe tangency portfolio.
pub fn max_sharpe(
    returns_matrix: &DMatrix<f64>,
    tickers: &[String],
    rf: f64,
) -> Vec<(String, f64)> {
    let n = tickers.len();
    if n == 0 {
        return vec![];
    }
    let mu = mean_returns(returns_matrix);
    let cov = covariance_matrix(returns_matrix);
    let inv = match cov.try_inverse() {
        Some(inv) => inv,
        None => return equal_weights(tickers),
    };
    let excess = mu - DVector::from_element(n, rf);
    let numerator = inv * excess;
    let ones = DVector::from_element(n, 1.0);
    let denom = ones.dot(&numerator);
    if denom.abs() < 1e-12 {
        return equal_weights(tickers);
    }
    let w = numerator / denom;
    w.iter()
        .enumerate()
        .map(|(i, &v)| (tickers[i].clone(), v))
        .collect()
}

pub fn equal_weight(tickers: &[String]) -> Vec<(String, f64)> {
    equal_weights(tickers)
}

fn equal_weights(tickers: &[String]) -> Vec<(String, f64)> {
    let w = 1.0 / tickers.len() as f64;
    tickers.iter().map(|t| (t.clone(), w)).collect()
}

fn mean_returns(returns: &DMatrix<f64>) -> DVector<f64> {
    let n = returns.ncols();
    DVector::from_iterator(n, (0..n).map(|j| returns.column(j).mean()))
}

fn covariance_matrix(returns: &DMatrix<f64>) -> DMatrix<f64> {
    let n = returns.ncols();
    let t = returns.nrows() as f64;
    let mu = mean_returns(returns);
    let mut cov = DMatrix::zeros(n, n);
    for i in 0..returns.nrows() {
        for a in 0..n {
            for b in 0..n {
                let ra = returns[(i, a)] - mu[a];
                let rb = returns[(i, b)] - mu[b];
                cov[(a, b)] += ra * rb;
            }
        }
    }
    cov /= t.max(1.0);
    cov
}

fn project_simplex(w: &mut DVector<f64>) {
    let mut sorted: Vec<f64> = w.iter().copied().collect();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let mut cumsum = 0.0;
    let mut rho = 0usize;
    for (i, &v) in sorted.iter().enumerate() {
        cumsum += v;
        if v + (1.0 - cumsum) / (i as f64 + 1.0) > 0.0 {
            rho = i;
        }
    }
    let theta = (sorted[..=rho].iter().sum::<f64>() - 1.0) / (rho as f64 + 1.0);
    for wi in w.iter_mut() {
        *wi = (*wi - theta).max(0.0);
    }
    let sum: f64 = w.iter().sum();
    if sum > 0.0 {
        for wi in w.iter_mut() {
            *wi /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_weight_sums_to_one() {
        let tickers = vec!["A".into(), "B".into(), "C".into()];
        let w = equal_weight(&tickers);
        let sum: f64 = w.iter().map(|(_, v)| v).sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn min_variance_weights_sum() {
        let data = DMatrix::from_row_slice(
            5,
            2,
            &[0.01, 0.02, -0.01, 0.03, 0.02, -0.02, 0.01, 0.04, 0.00, 0.01],
        );
        let tickers = vec!["A".into(), "B".into()];
        let w = min_variance(&data, &tickers);
        let sum: f64 = w.iter().map(|(_, v)| v).sum();
        assert!((sum - 1.0).abs() < 0.01);
    }
}
