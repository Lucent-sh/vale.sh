use comfy_table::{Attribute, Cell, Color, Table};
use vale_core::types::BacktestResult;
use vale_portfolio::weights::Weights;

pub fn apply_vale_style(table: &mut Table) {
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
}

pub fn backtest_summary(result: &BacktestResult) -> Table {
    let mut table = Table::new();
    apply_vale_style(&mut table);
    table.set_header(vec![
        Cell::new("Metric")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 176,
                b: 0,
            }),
        Cell::new("Value")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 176,
                b: 0,
            }),
    ]);

    let metrics = [
        (
            "Total Return",
            format!("{:.2}%", result.total_return * 100.0),
            result.total_return >= 0.0,
        ),
        (
            "CAGR",
            format!("{:.2}%", result.cagr * 100.0),
            result.cagr >= 0.0,
        ),
        (
            "Sharpe Ratio",
            format!("{:.3}", result.sharpe_ratio),
            result.sharpe_ratio > 0.0,
        ),
        (
            "Sortino Ratio",
            format!("{:.3}", result.sortino_ratio),
            result.sortino_ratio > 0.0,
        ),
        (
            "Calmar Ratio",
            format!("{:.3}", result.calmar_ratio),
            result.calmar_ratio > 0.0,
        ),
        (
            "Max Drawdown",
            format!("{:.2}%", result.max_drawdown * 100.0),
            false,
        ),
        (
            "Volatility (Ann.)",
            format!("{:.2}%", result.volatility_annual * 100.0),
            false,
        ),
        ("Total Trades", result.total_trades.to_string(), true),
        (
            "Win Rate",
            format!("{:.1}%", result.win_rate * 100.0),
            result.win_rate > 0.5,
        ),
        (
            "Profit Factor",
            format!("{:.2}", result.profit_factor),
            result.profit_factor > 1.0,
        ),
    ];

    for (name, value, good) in &metrics {
        let color = if *good {
            Color::Rgb {
                r: 80,
                g: 200,
                b: 120,
            }
        } else if name == &"Max Drawdown" {
            Color::Rgb {
                r: 220,
                g: 80,
                b: 80,
            }
        } else {
            Color::Reset
        };
        table.add_row(vec![Cell::new(name), Cell::new(value).fg(color)]);
    }
    table
}

pub fn sweep_table(rows: &[(&str, Vec<(String, String)>)]) -> Table {
    let mut table = Table::new();
    apply_vale_style(&mut table);
    if rows.is_empty() {
        return table;
    }
    let headers: Vec<_> = rows[0].1.iter().map(|(k, _)| k.as_str()).collect();
    table.set_header(
        std::iter::once("#")
            .chain(headers)
            .map(Cell::new)
            .collect::<Vec<_>>(),
    );
    for (rank, cols) in rows {
        let mut row = vec![Cell::new(rank)];
        for (_, v) in cols {
            row.push(Cell::new(v));
        }
        table.add_row(row);
    }
    table
}

pub fn portfolio_table(weights: &Weights) -> Table {
    let mut table = Table::new();
    apply_vale_style(&mut table);
    table.set_header(vec!["Ticker", "Weight"]);
    let mut items: Vec<_> = weights.0.iter().collect();
    items.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (ticker, weight) in items {
        table.add_row(vec![
            Cell::new(ticker),
            Cell::new(format!("{:.2}%", weight * 100.0)),
        ]);
    }
    table
}

pub fn risk_table(metrics: &[(&str, String)]) -> Table {
    let mut table = Table::new();
    apply_vale_style(&mut table);
    table.set_header(vec!["Metric", "Value"]);
    for (name, value) in metrics {
        table.add_row(vec![Cell::new(name), Cell::new(value)]);
    }
    table
}

pub fn factor_table(result: &crate::json::FactorReportJson) -> Table {
    let mut table = Table::new();
    apply_vale_style(&mut table);
    table.set_header(vec!["Factor", "Beta", "T-Stat"]);
    table.add_row(vec![
        Cell::new("Alpha"),
        Cell::new(format!("{:.4}", result.alpha)),
        Cell::new(""),
    ]);
    for (i, beta) in result.betas.iter().enumerate() {
        let t = result
            .t_stats
            .get(i)
            .map(|t| format!("{:.2}", t))
            .unwrap_or_default();
        table.add_row(vec![
            Cell::new(format!("F{}", i + 1)),
            Cell::new(format!("{:.4}", beta)),
            Cell::new(t),
        ]);
    }
    table.add_row(vec![
        Cell::new("R²"),
        Cell::new(format!("{:.4}", result.r_squared)),
        Cell::new(""),
    ]);
    table
}
