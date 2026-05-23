//! TA-Lib compatible API. With `talib` feature, attempts dynamic dispatch;
//! always falls back to native Rust implementations.

use crate::native;

pub fn sma(data: &[f64], period: usize) -> Vec<f64> {
    native::sma(data, period)
}

pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    native::ema(data, period)
}

pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    native::rsi(data, period)
}

pub fn macd(
    data: &[f64],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Vec<(f64, f64, f64)> {
    native::macd(data, fast, slow, signal)
}

pub fn bollinger_bands(
    data: &[f64],
    period: usize,
    std_dev: f64,
) -> Vec<(f64, f64, f64)> {
    native::bollinger_bands(data, period, std_dev)
}

pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    native::atr(high, low, close, period)
}

#[cfg(feature = "talib")]
pub fn available() -> bool {
    std::process::Command::new("python3")
        .args(["-c", "import talib"])
        .output()
        .is_ok_and(|o| o.status.success())
}

#[cfg(not(feature = "talib"))]
pub fn available() -> bool {
    false
}
