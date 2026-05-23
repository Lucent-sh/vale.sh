use crate::context::Context;
use crate::order::{Order, OrderStatus, OrderType};
use crate::strategy::Strategy;
use vale_core::types::Bar;
use vale_indicators::native::sma;

pub struct SmaCrossover {
    pub symbol: String,
    pub fast: usize,
    pub slow: usize,
    closes: Vec<f64>,
    in_position: bool,
}

impl SmaCrossover {
    pub fn new(symbol: impl Into<String>, fast: usize, slow: usize) -> Self {
        Self {
            symbol: symbol.into(),
            fast,
            slow,
            closes: Vec::new(),
            in_position: false,
        }
    }
}

impl Strategy for SmaCrossover {
    fn name(&self) -> &str {
        "sma_crossover"
    }

    fn on_bar(&mut self, ctx: &mut Context, bar: &Bar) {
        if bar.symbol != self.symbol {
            return;
        }
        self.closes.push(bar.close);
        if self.closes.len() < self.slow {
            return;
        }

        let fast_sma = sma(&self.closes, self.fast);
        let slow_sma = sma(&self.closes, self.slow);
        if fast_sma.is_empty() || slow_sma.is_empty() {
            return;
        }
        let fast_val = *fast_sma.last().unwrap_or(&0.0);
        let slow_val = *slow_sma.last().unwrap_or(&0.0);

        if fast_val > slow_val && !self.in_position {
            let equity = ctx.equity();
            let qty = (equity * 0.99 / bar.close).floor();
            if qty > 0.0 {
                ctx.submit_order(Order {
                    symbol: bar.symbol.clone(),
                    quantity: qty,
                    order_type: OrderType::Market,
                    status: OrderStatus::Pending,
                    is_buy: true,
                });
                self.in_position = true;
            }
        } else if fast_val < slow_val && self.in_position {
            if let Some(pos) = ctx.portfolio.positions.get(&bar.symbol) {
                let qty = pos.quantity;
                if qty > 0.0 {
                    ctx.submit_order(Order {
                        symbol: bar.symbol.clone(),
                        quantity: qty,
                        order_type: OrderType::Market,
                        status: OrderStatus::Pending,
                        is_buy: false,
                    });
                    self.in_position = false;
                }
            }
        }
    }
}
