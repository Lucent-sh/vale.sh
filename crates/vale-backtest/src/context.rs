use crate::order::Order;
use crate::portfolio::Portfolio;
use std::collections::HashMap;
use vale_core::types::Bar;

pub struct DataWindow {
    pub bars: Vec<Bar>,
    pub index: usize,
}

impl DataWindow {
    pub fn current(&self) -> Option<&Bar> {
        self.bars.get(self.index)
    }

    pub fn history(&self, lookback: usize) -> &[Bar] {
        let start = self.index.saturating_sub(lookback);
        &self.bars[start..=self.index.min(self.bars.len().saturating_sub(1))]
    }
}

pub struct Context<'a> {
    pub portfolio: &'a mut Portfolio,
    pub data: DataWindow,
    pub orders: Vec<Order>,
    pub prices: HashMap<String, f64>,
}

impl<'a> Context<'a> {
    pub fn submit_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn equity(&self) -> f64 {
        self.portfolio.equity(&self.prices)
    }
}
