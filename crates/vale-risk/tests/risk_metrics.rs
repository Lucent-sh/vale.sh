use vale_risk::drawdown::*;
use vale_risk::metrics::*;

#[test]
fn sharpe_ratio_known_value() {
    let returns = vec![0.01_f64, -0.005, 0.008, -0.003, 0.012];
    let rf = 0.0;
    let s = sharpe_ratio(&returns, rf, 1.0);
    assert!(
        s > 0.0,
        "Sharpe should be positive for positive-mean returns"
    );
}

#[test]
fn sharpe_ratio_empty() {
    assert_eq!(sharpe_ratio(&[], 0.0, 1.0), 0.0);
}

#[test]
fn sharpe_ratio_single_element() {
    let s = sharpe_ratio(&[0.01], 0.0, 1.0);
    assert_eq!(s, 0.0);
}

#[test]
fn max_drawdown_flat() {
    let equity = vec![100.0, 100.0, 100.0];
    assert_eq!(max_drawdown(&equity), 0.0);
}

#[test]
fn max_drawdown_single_peak() {
    let equity = vec![100.0, 120.0, 100.0, 80.0, 90.0, 110.0];
    let dd = max_drawdown(&equity);
    assert!((dd - 0.3333).abs() < 0.001, "expected ~0.333, got {dd}");
}

#[test]
fn max_drawdown_empty() {
    assert_eq!(max_drawdown(&[]), 0.0);
}

#[test]
fn historical_var_known() {
    let mut returns: Vec<f64> = (0..95).map(|i| i as f64 * 0.001).collect();
    returns.extend([-0.10, -0.09, -0.08, -0.07, -0.06]);
    let var = historical_var(&returns, 0.95);
    assert!(var > 0.05 && var < 0.12, "VaR should be in tail: got {var}");
}

#[test]
fn historical_var_empty() {
    assert_eq!(historical_var(&[], 0.95), 0.0);
}

#[test]
fn cagr_doubles_in_7_years() {
    let equity = vec![100.0, 200.0];
    let c = cagr(&equity, 7.0);
    assert!((c - 0.1041).abs() < 0.001, "expected ~0.1041, got {c}");
}

#[test]
fn log_returns_basic() {
    let prices = vec![100.0, 110.0, 99.0];
    let returns = log_returns(&prices);
    assert_eq!(returns.len(), 2);
    assert!((returns[0] - (110.0_f64 / 100.0).ln()).abs() < 1e-10);
}

#[test]
fn simple_returns_all_positive() {
    let prices = vec![100.0, 110.0, 120.0];
    let returns = simple_returns(&prices);
    assert!(returns.iter().all(|&r| r > 0.0));
}

#[test]
fn simple_returns_all_negative() {
    let prices = vec![120.0, 110.0, 100.0];
    let returns = simple_returns(&prices);
    assert!(returns.iter().all(|&r| r < 0.0));
}

#[test]
fn profit_factor_all_wins() {
    assert_eq!(profit_factor(&[1.0, 2.0, 3.0]), f64::INFINITY);
}

#[test]
fn profit_factor_all_losses() {
    let pf = profit_factor(&[-1.0, -2.0]);
    assert_eq!(pf, 0.0);
}

#[test]
fn sortino_all_positive() {
    let returns = vec![0.01, 0.02, 0.03];
    assert_eq!(sortino_ratio(&returns, 0.0, 1.0), f64::INFINITY);
}
