use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights(pub HashMap<String, f64>);

impl Weights {
    /// Normalize weights to sum to 1.0.
    pub fn normalize(&mut self) {
        let sum: f64 = self.0.values().sum();
        if sum == 0.0 {
            return;
        }
        for v in self.0.values_mut() {
            *v /= sum;
        }
    }

    pub fn equal(tickers: &[&str]) -> Self {
        let n = tickers.len() as f64;
        Self(tickers.iter().map(|t| (t.to_string(), 1.0 / n)).collect())
    }
}
