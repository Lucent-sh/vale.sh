/// Annualized Sharpe ratio.
pub fn sharpe_ratio(returns: &[f64], risk_free_daily: f64, annualization: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let excess: Vec<f64> = returns.iter().map(|r| r - risk_free_daily).collect();
    let mean = mean(&excess);
    let std = std_dev(&excess);
    if std == 0.0 {
        return 0.0;
    }
    (mean / std) * annualization
}

/// Sortino ratio (penalizes only downside deviation).
pub fn sortino_ratio(returns: &[f64], risk_free_daily: f64, annualization: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let excess: Vec<f64> = returns.iter().map(|r| r - risk_free_daily).collect();
    let mean = mean(&excess);
    let downside: Vec<f64> = excess.iter().filter(|&&r| r < 0.0).copied().collect();
    if downside.is_empty() {
        return f64::INFINITY;
    }
    let downside_std = std_dev(&downside);
    if downside_std == 0.0 {
        return 0.0;
    }
    (mean / downside_std) * annualization
}

/// Calmar ratio = CAGR / |max_drawdown|.
pub fn calmar_ratio(cagr: f64, max_drawdown: f64) -> f64 {
    if max_drawdown == 0.0 {
        return f64::INFINITY;
    }
    cagr / max_drawdown.abs()
}

/// Compound Annual Growth Rate from equity curve.
pub fn cagr(equity: &[f64], years: f64) -> f64 {
    if equity.len() < 2 || years == 0.0 {
        return 0.0;
    }
    let first = equity[0];
    let last = equity[equity.len() - 1];
    if first == 0.0 {
        return 0.0;
    }
    (last / first).powf(1.0 / years) - 1.0
}

/// Annual volatility from daily returns.
pub fn volatility_annual(returns: &[f64], annualization: f64) -> f64 {
    std_dev(returns) * annualization
}

/// Historical Value at Risk (percentile of loss distribution).
pub fn historical_var(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let tail_frac = 1.0 - confidence;
    let idx = (tail_frac * sorted.len() as f64 - 1.0).max(0.0) as usize;
    let idx = idx.min(sorted.len() - 1);
    -sorted[idx]
}

/// Conditional VaR (Expected Shortfall).
pub fn cvar(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let var = historical_var(returns, confidence);
    let tail: Vec<f64> = returns.iter().filter(|&&r| -r >= var).copied().collect();
    if tail.is_empty() {
        return var;
    }
    -mean(&tail)
}

/// Beta of strategy returns relative to benchmark.
pub fn beta(returns: &[f64], benchmark: &[f64]) -> f64 {
    if returns.len() != benchmark.len() || returns.is_empty() {
        return 0.0;
    }
    let n = returns.len() as f64;
    let mean_r = mean(returns);
    let mean_b = mean(benchmark);
    let cov: f64 = returns
        .iter()
        .zip(benchmark.iter())
        .map(|(r, b)| (r - mean_r) * (b - mean_b))
        .sum::<f64>()
        / n;
    let var_b = variance(benchmark);
    if var_b == 0.0 {
        return 0.0;
    }
    cov / var_b
}

/// Alpha annualized.
pub fn alpha(returns: &[f64], benchmark: &[f64], risk_free_daily: f64, annualization: f64) -> f64 {
    let b = beta(returns, benchmark);
    let mean_r = mean(returns);
    let mean_b = mean(benchmark);
    (mean_r - (risk_free_daily + b * (mean_b - risk_free_daily))) * annualization
}

/// Profit factor = sum of wins / |sum of losses|.
pub fn profit_factor(pnls: &[f64]) -> f64 {
    let wins: f64 = pnls.iter().filter(|&&p| p > 0.0).sum();
    let losses: f64 = pnls.iter().filter(|&&p| p < 0.0).map(|p| p.abs()).sum();
    if losses == 0.0 {
        return f64::INFINITY;
    }
    wins / losses
}

pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / data.len() as f64
}

pub fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

/// Convert price series to log returns.
pub fn log_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect()
}

/// Convert price series to simple returns.
pub fn simple_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect()
}
