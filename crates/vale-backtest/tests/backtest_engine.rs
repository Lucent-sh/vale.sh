use chrono::{TimeZone, Utc};
use vale_backtest::commission::{FlatCommission, PercentageCommission};
use vale_backtest::engine::BacktestEngine;
use vale_backtest::slippage::PercentageSlippage;
use vale_backtest::strategies::buy_and_hold::BuyAndHold;
use vale_backtest::strategy::Strategy;
use vale_core::types::Bar;

fn make_bars(prices: &[f64]) -> Vec<Bar> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &close)| Bar {
            timestamp: Utc
                .with_ymd_and_hms(2020, 1, 1 + i as u32, 0, 0, 0)
                .unwrap(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 1_000_000.0,
            symbol: "SPY".to_string(),
        })
        .collect()
}

#[test]
fn buy_and_hold_produces_positive_return() {
    let bars = make_bars(&[100.0, 110.0, 120.0, 130.0, 140.0]);
    let engine = BacktestEngine {
        commission: Box::new(PercentageCommission { rate: 0.0 }),
        slippage: Box::new(PercentageSlippage { rate: 0.0 }),
        initial_cash: 100_000.0,
    };
    let mut strategy = BuyAndHold::new("SPY");
    let result = engine.run(&mut strategy, &bars).unwrap();
    assert!(result.total_return > 0.0);
    assert!(result.final_equity > result.initial_cash);
    assert_eq!(result.total_trades, 1);
    assert!(result.win_rate > 0.0);
}

#[test]
fn commission_is_deducted() {
    let bars = make_bars(&[100.0, 105.0, 110.0]);
    let engine_no_fee = BacktestEngine {
        commission: Box::new(PercentageCommission { rate: 0.0 }),
        slippage: Box::new(PercentageSlippage { rate: 0.0 }),
        initial_cash: 10_000.0,
    };
    let engine_fee = BacktestEngine {
        commission: Box::new(FlatCommission { per_trade: 50.0 }),
        slippage: Box::new(PercentageSlippage { rate: 0.0 }),
        initial_cash: 10_000.0,
    };
    let mut s1 = BuyAndHold::new("SPY");
    let mut s2 = BuyAndHold::new("SPY");
    let r1 = engine_no_fee.run(&mut s1, &bars).unwrap();
    let r2 = engine_fee.run(&mut s2, &bars).unwrap();
    assert!(r2.final_equity < r1.final_equity);
}

#[test]
fn short_pnl_correct_when_price_falls() {
    use vale_backtest::order::{Order, OrderStatus, OrderType};
    use vale_backtest::strategies::buy_and_hold::BuyAndHold;

    struct ShortOnFirstBar;
    impl Strategy for ShortOnFirstBar {
        fn name(&self) -> &str {
            "short_test"
        }
        fn on_bar(&mut self, ctx: &mut vale_backtest::context::Context, bar: &Bar) {
            if ctx.data.index == 0 {
                ctx.submit_order(Order {
                    symbol: bar.symbol.clone(),
                    quantity: 10.0,
                    order_type: OrderType::Market,
                    status: OrderStatus::Pending,
                    is_buy: false,
                });
            } else if ctx.data.index == 1 {
                ctx.submit_order(Order {
                    symbol: bar.symbol.clone(),
                    quantity: 10.0,
                    order_type: OrderType::Market,
                    status: OrderStatus::Pending,
                    is_buy: true,
                });
            }
        }
    }

    let bars = make_bars(&[100.0, 90.0]);
    let engine = BacktestEngine {
        commission: Box::new(PercentageCommission { rate: 0.0 }),
        slippage: Box::new(PercentageSlippage { rate: 0.0 }),
        initial_cash: 100_000.0,
    };
    let mut strategy = ShortOnFirstBar;
    let result = engine.run(&mut strategy, &bars).unwrap();
    assert!(!result.trades.is_empty());
    let trade = &result.trades[0];
    assert!(trade.pnl > 0.0);
    let _ = BuyAndHold::new("SPY");
}
