#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use vale_core::error::{ValeError, ValeResult};

/// In-process Python RPC via PyO3 (optional feature).
#[cfg(feature = "pyo3")]
pub fn call_python_fn(module: &str, function: &str, args_json: &str) -> ValeResult<String> {
    Python::with_gil(|py| -> PyResult<String> {
        let json_mod = py.import("json")?;
        let args: pyo3::PyObject = json_mod
            .getattr("loads")?
            .call1((args_json,))?
            .into();
        let m = py.import(module)?;
        let f = m.getattr(function)?;
        let out = f.call1((args,))?;
        out.to_string()
    })
    .map_err(|e| ValeError::AdapterUnavailable(e.to_string()))
}

#[cfg(not(feature = "pyo3"))]
pub fn call_python_fn(_module: &str, _function: &str, _args_json: &str) -> ValeResult<String> {
    Err(ValeError::AdapterUnavailable(
        "pyo3 bridge not enabled; rebuild with feature pyo3".into(),
    ))
}
