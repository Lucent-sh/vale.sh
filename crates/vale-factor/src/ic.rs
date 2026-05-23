use vale_risk::correlation::pearson;

/// Information coefficient at multiple lags.
pub fn information_coefficient(
    signals: &[f64],
    returns: &[f64],
    lags: &[usize],
) -> Vec<(usize, f64)> {
    lags.iter()
        .filter_map(|&lag| {
            if signals.len() <= lag || returns.len() <= lag {
                return None;
            }
            let sig = &signals[..signals.len() - lag];
            let ret = &returns[lag..];
            let n = sig.len().min(ret.len());
            if n == 0 {
                return None;
            }
            Some((lag, pearson(&sig[..n], &ret[..n])))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ic_perfect_correlation() {
        let signals: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let returns: Vec<f64> = (0..20).map(|i| i as f64 + 1.0).collect();
        let ics = information_coefficient(&signals, &returns, &[1]);
        assert!(!ics.is_empty());
        assert!(ics[0].1 > 0.99);
    }
}
