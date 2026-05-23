use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vale_backtest::commission::PercentageCommission;
use vale_backtest::engine::BacktestEngine;
use vale_backtest::slippage::PercentageSlippage;
use vale_backtest::strategies::buy_and_hold::BuyAndHold;
use vale_core::types::Bar;

fn make_bars(n: usize) -> Vec<Bar> {
    (0..n)
        .map(|i| {
            let close = 100.0 + i as f64 * 0.1;
            Bar {
                timestamp: Utc
                    .with_ymd_and_hms(2010, 1, 1, 0, 0, 0)
                    .unwrap()
                    + chrono::Duration::days(i as i64),
                open: close,
                high: close,
                low: close,
                close,
                volume: 1_000_000.0,
                symbol: "SPY".into(),
            }
        })
        .collect()
}

fn bench_buy_and_hold_10yr(c: &mut Criterion) {
    let bars = make_bars(252 * 10);
    let engine = BacktestEngine {
        commission: Box::new(PercentageCommission { rate: 0.001 }),
        slippage: Box::new(PercentageSlippage { rate: 0.0005 }),
        initial_cash: 100_000.0,
    };
    c.bench_function("buy_and_hold_10yr_daily", |b| {
        b.iter(|| {
            let mut strategy = BuyAndHold::new("SPY");
            black_box(engine.run(&mut strategy, &bars).unwrap());
        });
    });
}

criterion_group!(benches, bench_buy_and_hold_10yr);
criterion_main!(benches);
