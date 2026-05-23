use statrs::distribution::{Continuous, ContinuousCDF, Normal};

fn norm_cdf(x: f64) -> f64 {
    Normal::new(0.0, 1.0).unwrap().cdf(x)
}

fn norm_pdf(x: f64) -> f64 {
    Normal::new(0.0, 1.0).unwrap().pdf(x)
}

pub fn d1_d2(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> (f64, f64) {
    let d1 = ((s / k).ln() + (r + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
    let d2 = d1 - sigma * t.sqrt();
    (d1, d2)
}

/// European call price.
pub fn bs_call(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    if t <= 0.0 {
        return (s - k).max(0.0);
    }
    let (d1, d2) = d1_d2(s, k, t, r, sigma);
    s * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
}

/// European put price.
pub fn bs_put(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    if t <= 0.0 {
        return (k - s).max(0.0);
    }
    let (d1, d2) = d1_d2(s, k, t, r, sigma);
    k * (-r * t).exp() * norm_cdf(-d2) - s * norm_cdf(-d1)
}

#[derive(Debug, Clone)]
pub struct BsGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

/// Black-Scholes Greeks for a call option.
pub fn bs_greeks(s: f64, k: f64, t: f64, r: f64, sigma: f64, is_call: bool) -> BsGreeks {
    if t <= 0.0 {
        return BsGreeks {
            delta: if is_call { 1.0 } else { -1.0 },
            gamma: 0.0,
            vega: 0.0,
            theta: 0.0,
            rho: 0.0,
        };
    }
    let (d1, d2) = d1_d2(s, k, t, r, sigma);
    let nd1 = norm_pdf(d1);
    let sqrt_t = t.sqrt();

    let delta = if is_call {
        norm_cdf(d1)
    } else {
        norm_cdf(d1) - 1.0
    };
    let gamma = nd1 / (s * sigma * sqrt_t);
    let vega = s * nd1 * sqrt_t / 100.0;
    let theta_call = -(s * nd1 * sigma) / (2.0 * sqrt_t) - r * k * (-r * t).exp() * norm_cdf(d2);
    let theta = if is_call {
        theta_call / 365.0
    } else {
        (theta_call + r * k * (-r * t).exp()) / 365.0
    };
    let rho = if is_call {
        k * t * (-r * t).exp() * norm_cdf(d2) / 100.0
    } else {
        -k * t * (-r * t).exp() * norm_cdf(-d2) / 100.0
    };

    BsGreeks {
        delta,
        gamma,
        vega,
        theta,
        rho,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atm_call_price_reasonable() {
        let price = bs_call(100.0, 100.0, 90.0 / 365.0, 0.05, 0.20);
        assert!(price > 2.0 && price < 10.0);
    }

    #[test]
    fn put_call_parity() {
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.20;
        let call = bs_call(s, k, t, r, sigma);
        let put = bs_put(s, k, t, r, sigma);
        let parity = call - put - s + k * (-r * t).exp();
        assert!(parity.abs() < 1e-6);
    }

    #[test]
    fn greeks_signs() {
        let g = bs_greeks(100.0, 100.0, 0.25, 0.05, 0.20, true);
        assert!(g.delta > 0.0 && g.delta < 1.0);
        assert!(g.gamma > 0.0);
        assert!(g.vega > 0.0);
    }
}
