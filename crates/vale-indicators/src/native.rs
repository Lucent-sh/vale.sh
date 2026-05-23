/// Simple Moving Average.
pub fn sma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period {
        return vec![];
    }
    data.windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

/// Exponential Moving Average.
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() {
        return vec![];
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
    result.push(data[0]);
    for &val in &data[1..] {
        let prev = *result.last().unwrap_or(&data[0]);
        result.push(val * k + prev * (1.0 - k));
    }
    result
}

/// Relative Strength Index (Wilder's smoothing).
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() <= period {
        return vec![];
    }
    let mut gains = Vec::new();
    let mut losses = Vec::new();
    for w in data.windows(2) {
        let change = w[1] - w[0];
        if change >= 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }
    if gains.len() < period {
        return vec![];
    }

    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    let mut result = Vec::with_capacity(gains.len() - period + 1);
    let rs = if avg_loss == 0.0 {
        f64::INFINITY
    } else {
        avg_gain / avg_loss
    };
    result.push(100.0 - 100.0 / (1.0 + rs));

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
        let rs = if avg_loss == 0.0 {
            f64::INFINITY
        } else {
            avg_gain / avg_loss
        };
        result.push(100.0 - 100.0 / (1.0 + rs));
    }
    result
}

/// Bollinger Bands. Returns (upper, middle, lower).
pub fn bollinger_bands(data: &[f64], period: usize, std_dev_mult: f64) -> Vec<(f64, f64, f64)> {
    if data.len() < period {
        return vec![];
    }
    data.windows(period)
        .map(|w| {
            let middle: f64 = w.iter().sum::<f64>() / period as f64;
            let variance: f64 = w.iter().map(|x| (x - middle).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            (
                middle + std_dev_mult * std,
                middle,
                middle - std_dev_mult * std,
            )
        })
        .collect()
}

/// MACD. Returns (macd_line, signal_line, histogram).
pub fn macd(data: &[f64], fast: usize, slow: usize, signal: usize) -> Vec<(f64, f64, f64)> {
    if data.len() < slow {
        return vec![];
    }
    let ema_fast = ema(data, fast);
    let ema_slow = ema(data, slow);
    let offset = slow - fast;
    let macd_line: Vec<f64> = ema_fast[offset..]
        .iter()
        .zip(ema_slow.iter())
        .map(|(f, s)| f - s)
        .collect();
    if macd_line.len() < signal {
        return vec![];
    }
    let signal_line = ema(&macd_line, signal);
    let offset2 = macd_line.len() - signal_line.len();
    signal_line
        .iter()
        .enumerate()
        .map(|(i, &sig)| {
            let macd = macd_line[i + offset2];
            (macd, sig, macd - sig)
        })
        .collect()
}

/// Average True Range.
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    if high.len() != low.len() || high.len() != close.len() || high.len() < 2 {
        return vec![];
    }
    let mut tr = Vec::with_capacity(high.len() - 1);
    for i in 1..high.len() {
        let hl = high[i] - low[i];
        let hc = (high[i] - close[i - 1]).abs();
        let lc = (low[i] - close[i - 1]).abs();
        tr.push(hl.max(hc).max(lc));
    }
    if tr.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    let initial: f64 = tr[..period].iter().sum::<f64>() / period as f64;
    result.push(initial);
    for &tr_val in tr.iter().skip(period) {
        let prev = *result.last().unwrap_or(&initial);
        let val = (prev * (period as f64 - 1.0) + tr_val) / period as f64;
        result.push(val);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sma_known() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&data, 3);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-10);
        assert!((result[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ema_starts_with_first() {
        let data = vec![10.0, 11.0, 12.0];
        let result = ema(&data, 2);
        assert_eq!(result[0], 10.0);
    }

    #[test]
    fn rsi_range() {
        let data: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64).sin() * 5.0).collect();
        let result = rsi(&data, 14);
        assert!(!result.is_empty());
        for v in &result {
            assert!(*v >= 0.0 && *v <= 100.0);
        }
    }

    #[test]
    fn bollinger_bands_ordering() {
        let data: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let bands = bollinger_bands(&data, 5, 2.0);
        for (upper, middle, lower) in bands {
            assert!(upper >= middle);
            assert!(middle >= lower);
        }
    }

    #[test]
    fn macd_produces_values() {
        let data: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 0.5).collect();
        let result = macd(&data, 12, 26, 9);
        assert!(!result.is_empty());
    }

    #[test]
    fn atr_positive() {
        let high: Vec<f64> = (0..20).map(|i| 105.0 + i as f64).collect();
        let low: Vec<f64> = (0..20).map(|i| 95.0 + i as f64).collect();
        let close: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let result = atr(&high, &low, &close, 14);
        assert!(!result.is_empty());
        assert!(result.iter().all(|&v| v > 0.0));
    }
}
