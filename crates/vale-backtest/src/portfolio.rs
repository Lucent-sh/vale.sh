use std::collections::HashMap;
use vale_core::types::{Bar, Trade};

#[derive(Debug, Clone)]
pub struct Position {
    pub quantity: f64,
    pub avg_price: f64,
}

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub cash: f64,
    pub positions: HashMap<String, Position>,
    pub trades: Vec<Trade>,
    pub initial_cash: f64,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            positions: HashMap::new(),
            trades: Vec::new(),
            initial_cash,
        }
    }

    pub fn equity(&self, prices: &HashMap<String, f64>) -> f64 {
        let positions_value: f64 = self
            .positions
            .iter()
            .map(|(sym, pos)| {
                let price = prices.get(sym).copied().unwrap_or(pos.avg_price);
                pos.quantity * price
            })
            .sum();
        self.cash + positions_value
    }

    pub fn mark_to_market(&self, bar: &Bar) -> f64 {
        let mut prices = HashMap::new();
        prices.insert(bar.symbol.clone(), bar.close);
        for (sym, pos) in &self.positions {
            prices.entry(sym.clone()).or_insert(pos.avg_price);
        }
        self.equity(&prices)
    }
}
