//! TA-Lib FFI bindings (optional `talib` feature).

#[cfg(feature = "talib")]
compile_error!("TA-Lib FFI is not bundled; use native indicators.");
