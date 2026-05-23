use chrono::{DateTime, Datelike, Utc};
use std::collections::BTreeMap;

/// Monthly simple returns keyed by (year, month).
pub fn monthly_returns(equity_curve: &[(DateTime<Utc>, f64)]) -> BTreeMap<(i32, u32), f64> {
    let mut by_month: BTreeMap<(i32, u32), Vec<f64>> = BTreeMap::new();
    for (ts, eq) in equity_curve {
        by_month
            .entry((ts.year(), ts.month()))
            .or_default()
            .push(*eq);
    }

    let mut out = BTreeMap::new();
    let mut prev_end: Option<f64> = None;
    for (key, values) in by_month {
        let end = *values.last().unwrap_or(&0.0);
        let ret = match prev_end {
            Some(p) if p > 0.0 => (end - p) / p,
            _ => 0.0,
        };
        out.insert(key, ret);
        prev_end = Some(end);
    }
    out
}

/// Heatmap z-matrix [month][year] for Plotly (months 1-12).
pub fn monthly_heatmap_matrix(
    monthly: &BTreeMap<(i32, u32), f64>,
) -> (Vec<String>, Vec<String>, Vec<Vec<f64>>) {
    let years: Vec<String> = monthly
        .keys()
        .map(|(y, _)| *y)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|y| y.to_string())
        .collect();
    let months: Vec<String> = (1..=12).map(month_label).collect();
    let mut z = vec![vec![0.0; years.len()]; 12];
    for ((year, month), ret) in monthly {
        if let Some(yi) = years.iter().position(|y| y == &year.to_string()) {
            let mi = (*month as usize).saturating_sub(1);
            if mi < 12 {
                z[mi][yi] = *ret * 100.0;
            }
        }
    }
    (months, years, z)
}

fn month_label(m: u32) -> String {
    match m {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        _ => "Dec",
    }
    .to_string()
}
