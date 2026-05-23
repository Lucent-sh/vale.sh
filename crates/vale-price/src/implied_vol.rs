use crate::blackscholes::bs_call;
use crate::blackscholes::bs_greeks;
use crate::blackscholes::bs_put;

/// Implied volatility via Newton-Raphson.
pub fn implied_volatility(market_price: f64, s: f64, k: f64, t: f64, r: f64, is_call: bool) -> f64 {
    let price_fn = |sigma: f64| {
        if is_call {
            bs_call(s, k, t, r, sigma)
        } else {
            bs_put(s, k, t, r, sigma)
        }
    };

    let mut sigma = 0.25;
    for _ in 0..100 {
        let model_price = price_fn(sigma);
        let diff = model_price - market_price;
        if diff.abs() < 1e-6 {
            return sigma;
        }
        let greeks = bs_greeks(s, k, t, r, sigma, is_call);
        let vega = greeks.vega * 100.0;
        if vega.abs() < 1e-12 {
            break;
        }
        sigma -= diff / vega;
        sigma = sigma.clamp(0.001, 5.0);
    }
    sigma
}
