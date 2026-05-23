use textplots::{Chart, Plot, Shape};
use vale_core::types::BacktestResult;

/// Render ASCII equity curve.
pub fn equity_curve(result: &BacktestResult, width: u32, height: u32) -> String {
    if result.equity_curve.is_empty() {
        return String::new();
    }

    let start_ts = result.equity_curve[0].0.timestamp() as f64;
    let points: Vec<(f32, f32)> = result
        .equity_curve
        .iter()
        .map(|(ts, equity)| {
            let x = (ts.timestamp() as f64 - start_ts) / 86400.0;
            (x as f32, *equity as f32)
        })
        .collect();

    if points.is_empty() {
        return String::new();
    }

    let shape = Shape::Lines(points.as_slice());
    Chart::new(
        width,
        height,
        points[0].0,
        points.last().map(|p| p.0).unwrap_or(points[0].0),
    )
    .lineplot(&shape)
    .to_string()
}

/// Drawdown chart (underwater).
pub fn drawdown_chart(result: &BacktestResult, width: u32, height: u32) -> String {
    if result.equity_curve.is_empty() {
        return String::new();
    }
    let start_ts = result.equity_curve[0].0.timestamp() as f64;
    let mut peak = result.equity_curve[0].1;
    let points: Vec<(f32, f32)> = result
        .equity_curve
        .iter()
        .map(|(ts, equity)| {
            if *equity > peak {
                peak = *equity;
            }
            let dd = if peak > 0.0 {
                -(*equity - peak) / peak
            } else {
                0.0
            };
            let x = (ts.timestamp() as f64 - start_ts) / 86400.0;
            (x as f32, dd as f32)
        })
        .collect();

    let shape = Shape::Lines(points.as_slice());
    Chart::new(
        width,
        height,
        points[0].0,
        points.last().map(|p| p.0).unwrap_or(points[0].0),
    )
    .lineplot(&shape)
    .to_string()
}
