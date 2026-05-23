use vale_core::types::BacktestResult;

/// Print tearsheet to stdout (table + ASCII charts).
pub fn print_tearsheet(result: &BacktestResult) {
    let table = crate::table::backtest_summary(result);
    println!("{table}");
    println!();
    println!("Equity Curve");
    println!("{}", crate::chart::equity_curve(result, 120, 24));
    println!();
    println!("Drawdown");
    println!("{}", crate::chart::drawdown_chart(result, 120, 12));
}
