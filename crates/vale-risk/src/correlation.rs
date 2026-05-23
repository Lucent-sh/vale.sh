use crate::metrics::mean;

/// Pearson correlation coefficient.
pub fn pearson(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }
    let mx = mean(x);
    let my = mean(y);
    let mut num = 0.0;
    let mut dx2 = 0.0;
    let mut dy2 = 0.0;
    for (a, b) in x.iter().zip(y.iter()) {
        let da = a - mx;
        let db = b - my;
        num += da * db;
        dx2 += da * da;
        dy2 += db * db;
    }
    let denom = (dx2 * dy2).sqrt();
    if denom == 0.0 {
        0.0
    } else {
        num / denom
    }
}

/// Spearman rank correlation.
pub fn spearman(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }
    let rx = rank(x);
    let ry = rank(y);
    pearson(&rx, &ry)
}

fn rank(data: &[f64]) -> Vec<f64> {
    let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; data.len()];
    let mut i = 0;
    while i < indexed.len() {
        let mut j = i;
        while j + 1 < indexed.len() && (indexed[j + 1].1 - indexed[j].1).abs() < 1e-12 {
            j += 1;
        }
        let avg_rank = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j + 1;
    }
    ranks
}

/// Rolling Pearson correlation with given window size.
pub fn rolling_correlation(x: &[f64], y: &[f64], window: usize) -> Vec<f64> {
    if x.len() != y.len() || window == 0 || x.len() < window {
        return Vec::new();
    }
    (window..=x.len())
        .map(|end| pearson(&x[end - window..end], &y[end - window..end]))
        .collect()
}
