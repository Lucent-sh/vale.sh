use vale_core::types::BacktestResult;

pub fn backtest_equity_curve(result: &BacktestResult) -> String {
    let mut out = String::from("timestamp,equity\n");
    for (ts, equity) in &result.equity_curve {
        out.push_str(&format!("{},{}\n", ts.to_rfc3339(), equity));
    }
    out
}

pub fn backtest_trades(result: &BacktestResult) -> String {
    let mut out = String::from(
        "id,symbol,entry_time,exit_time,entry_price,exit_price,quantity,direction,pnl,pnl_pct,fees\n",
    );
    for t in &result.trades {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{:?},{},{},{}\n",
            t.id,
            t.symbol,
            t.entry_time.to_rfc3339(),
            t.exit_time.to_rfc3339(),
            t.entry_price,
            t.exit_price,
            t.quantity,
            t.direction,
            t.pnl,
            t.pnl_pct,
            t.fees
        ));
    }
    out
}
