/// Maximum drawdown from peak. Returns positive number (magnitude of largest decline).
pub fn max_drawdown(equity: &[f64]) -> f64 {
    if equity.is_empty() {
        return 0.0;
    }
    let mut peak = equity[0];
    let mut max_dd = 0.0_f64;
    for &val in equity {
        if val > peak {
            peak = val;
        }
        let dd = (peak - val) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    max_dd
}

/// All drawdown periods.
pub fn drawdown_periods(equity: &[f64]) -> Vec<DrawdownPeriod> {
    if equity.is_empty() {
        return Vec::new();
    }
    let mut periods = Vec::new();
    let mut peak = equity[0];
    let mut peak_idx = 0;
    let mut in_drawdown = false;
    let mut trough_idx = 0;
    let mut trough_val = equity[0];

    for (i, &val) in equity.iter().enumerate() {
        if val >= peak {
            if in_drawdown {
                periods.push(DrawdownPeriod {
                    start: peak_idx,
                    trough: trough_idx,
                    end: i,
                    magnitude: (peak - trough_val) / peak,
                    duration_bars: i - peak_idx,
                });
                in_drawdown = false;
            }
            peak = val;
            peak_idx = i;
        } else {
            in_drawdown = true;
            if val < trough_val {
                trough_val = val;
                trough_idx = i;
            }
        }
    }
    periods
}

#[derive(Debug, Clone)]
pub struct DrawdownPeriod {
    pub start: usize,
    pub trough: usize,
    pub end: usize,
    pub magnitude: f64,
    pub duration_bars: usize,
}
