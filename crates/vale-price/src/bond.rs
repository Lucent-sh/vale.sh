/// Bond price from face value, coupon rate, yield, and periods.
pub fn bond_price(face: f64, coupon_rate: f64, yield_rate: f64, periods: u32) -> f64 {
    if periods == 0 {
        return face;
    }
    let c = face * coupon_rate;
    let y = yield_rate;
    let mut price = 0.0;
    for t in 1..=periods {
        price += c / (1.0 + y).powi(t as i32);
    }
    price += face / (1.0 + y).powi(periods as i32);
    price
}

/// Yield to maturity via Newton-Raphson.
pub fn bond_ytm(face: f64, coupon_rate: f64, price: f64, periods: u32) -> f64 {
    let mut y = coupon_rate;
    for _ in 0..100 {
        let p = bond_price(face, coupon_rate, y, periods);
        let diff = p - price;
        if diff.abs() < 1e-8 {
            return y;
        }
        let p_up = bond_price(face, coupon_rate, y + 1e-6, periods);
        let deriv = (p_up - p) / 1e-6;
        if deriv.abs() < 1e-12 {
            break;
        }
        y -= diff / deriv;
        y = y.clamp(0.0001, 0.5);
    }
    y
}

/// Modified duration.
pub fn bond_duration(face: f64, coupon_rate: f64, yield_rate: f64, periods: u32) -> f64 {
    let price = bond_price(face, coupon_rate, yield_rate, periods);
    if price == 0.0 {
        return 0.0;
    }
    let c = face * coupon_rate;
    let y = yield_rate;
    let mut weighted = 0.0;
    for t in 1..=periods {
        let cf = if t == periods { c + face } else { c };
        weighted += t as f64 * cf / (1.0 + y).powi(t as i32);
    }
    weighted / (price * (1.0 + y))
}

/// Convexity.
pub fn bond_convexity(face: f64, coupon_rate: f64, yield_rate: f64, periods: u32) -> f64 {
    let price = bond_price(face, coupon_rate, yield_rate, periods);
    if price == 0.0 {
        return 0.0;
    }
    let c = face * coupon_rate;
    let y = yield_rate;
    let mut conv = 0.0;
    for t in 1..=periods {
        let cf = if t == periods { c + face } else { c };
        conv += t as f64 * (t as f64 + 1.0) * cf / (1.0 + y).powi(t as i32 + 2);
    }
    conv / price
}
