use crate::context::Context;
use crate::order::{Order, OrderStatus, OrderType};
use crate::strategy::Strategy;
use vale_core::types::Bar;

pub struct BuyAndHold {
    pub symbol: String,
    bought: bool,
}

impl BuyAndHold {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            bought: false,
        }
    }
}

impl Strategy for BuyAndHold {
    fn name(&self) -> &str {
        "buy_and_hold"
    }

    fn on_bar(&mut self, ctx: &mut Context, bar: &Bar) {
        if bar.symbol != self.symbol || self.bought {
            return;
        }
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
            self.bought = true;
        }
    }
}
