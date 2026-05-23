use crate::weights::Weights;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use vale_core::types::Bar;
use vale_risk::drawdown::max_drawdown;
use vale_risk::metrics::{cagr, log_returns, sharpe_ratio, volatility_annual};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PortfolioBacktestResult {
    pub equity_curve: Vec<(DateTime<Utc>, f64)>,
    pub total_return: f64,
    pub cagr: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub volatility_annual: f64,
}

/// Portfolio backtest with periodic rebalancing.
pub fn portfolio_backtest(
    bars_by_ticker: &HashMap<String, Vec<Bar>>,
    weights: &Weights,
    rebalance_days: u32,
    initial_cash: f64,
) -> PortfolioBacktestResult {
    let mut dates: Vec<DateTime<Utc>> = Vec::new();
    for bars in bars_by_ticker.values() {
        for b in bars {
            if !dates.contains(&b.timestamp) {
                dates.push(b.timestamp);
            }
        }
    }
    dates.sort();

    let mut equity = initial_cash;
    let mut equity_curve = Vec::new();
    let mut last_rebalance = dates.first().copied();

    for &date in &dates {
        let should_rebalance = match last_rebalance {
            None => true,
            Some(last) => (date - last).num_days() as u32 >= rebalance_days,
        };

        if should_rebalance {
            last_rebalance = Some(date);
        }

        let mut day_return = 0.0;
        for (ticker, w) in &weights.0 {
            if let Some(bars) = bars_by_ticker.get(ticker) {
                if let Some((prev, curr)) = find_bar_pair(bars, date) {
                    if prev.close > 0.0 {
                        let r = (curr.close - prev.close) / prev.close;
                        day_return += w * r;
                    }
                }
            }
        }
        equity *= 1.0 + day_return;
        equity_curve.push((date, equity));
    }

    let equities: Vec<f64> = equity_curve.iter().map(|(_, e)| *e).collect();
    let returns = log_returns(&equities);
    let years = if dates.len() > 1 {
        (dates[dates.len() - 1] - dates[0]).num_days().max(1) as f64 / 365.25
    } else {
        1.0
    };

    PortfolioBacktestResult {
        total_return: (equity - initial_cash) / initial_cash,
        cagr: cagr(&equities, years),
        sharpe_ratio: sharpe_ratio(&returns, 0.05 / 252.0, 252.0_f64.sqrt()),
        max_drawdown: max_drawdown(&equities),
        volatility_annual: volatility_annual(&returns, 252.0_f64.sqrt()),
        equity_curve,
    }
}

fn find_bar_pair(bars: &[Bar], date: DateTime<Utc>) -> Option<(&Bar, &Bar)> {
    let idx = bars.iter().position(|b| b.timestamp == date)?;
    if idx > 0 {
        Some((&bars[idx - 1], &bars[idx]))
    } else {
        None
    }
}
