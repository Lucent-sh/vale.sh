use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone)]
pub struct OlsResult {
    pub alpha: f64,
    pub betas: Vec<f64>,
    pub t_stats: Vec<f64>,
    pub r_squared: f64,
    pub information_ratio: f64,
}

/// OLS regression: y = alpha + X * betas + epsilon
pub fn ols(y: &[f64], x: &[Vec<f64>]) -> OlsResult {
    let n = y.len();
    if n == 0 || x.is_empty() || x[0].len() != n {
        return OlsResult {
            alpha: 0.0,
            betas: vec![],
            t_stats: vec![],
            r_squared: 0.0,
            information_ratio: 0.0,
        };
    }

    let k = x.len() + 1;
    let mut design = DMatrix::zeros(n, k);
    for i in 0..n {
        design[(i, 0)] = 1.0;
        for (j, factor) in x.iter().enumerate() {
            design[(i, j + 1)] = factor[i];
        }
    }
    let y_vec = DVector::from_column_slice(y);

    let xt_x = design.transpose() * &design;
    let xt_y = design.transpose() * &y_vec;
    let coeffs = match xt_x.clone().try_inverse() {
        Some(inv) => inv * xt_y,
        None => DVector::zeros(k),
    };

    let y_hat = &design * &coeffs;
    let residuals = &y_vec - &y_hat;
    let ss_res: f64 = residuals.iter().map(|r| r * r).sum();
    let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
    let ss_tot: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    let mse = ss_res / (n - k).max(1) as f64;
    let mut t_stats = vec![0.0; k];
    if let Some(inv) = xt_x.try_inverse() {
        for j in 0..k {
            let se = (mse * inv[(j, j)]).sqrt();
            t_stats[j] = if se > 0.0 { coeffs[j] / se } else { 0.0 };
        }
    }

    let alpha = coeffs[0];
    let betas: Vec<f64> = (1..k).map(|j| coeffs[j]).collect();
    let ir = if residuals.std_dev() > 0.0 {
        alpha / residuals.std_dev()
    } else {
        0.0
    };

    OlsResult {
        alpha,
        betas,
        t_stats: t_stats[1..].to_vec(),
        r_squared,
        information_ratio: ir,
    }
}

trait StdDev {
    fn std_dev(&self) -> f64;
}

impl StdDev for DVector<f64> {
    fn std_dev(&self) -> f64 {
        let n = self.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let mean = self.iter().sum::<f64>() / n;
        let var: f64 = self.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        var.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ols_recovers_linear_relationship() {
        let y: Vec<f64> = (0..50)
            .map(|i| 0.1 + 0.5 * i as f64 + 0.01 * (i as f64).sin())
            .collect();
        let x = vec![(0..50).map(|i| i as f64).collect::<Vec<_>>()];
        let result = ols(&y, &x);
        assert!(result.betas[0] > 0.0);
        assert!(result.r_squared > 0.5);
    }
}
