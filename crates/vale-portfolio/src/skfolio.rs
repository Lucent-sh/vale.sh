use std::io::Write;
use std::process::{Command, Stdio};
use vale_core::error::{ValeError, ValeResult};

const PYTHON_SCRIPT: &str = r#"
import sys, json
import numpy as np

data = json.loads(sys.stdin.read())
method = data["method"]
returns = np.array(data["returns"])
tickers = data["tickers"]

try:
    if method == "hrp":
        from skfolio.optimization import HierarchicalRiskParity
        model = HierarchicalRiskParity()
    elif method == "risk_parity":
        from skfolio.optimization import RiskBudgeting
        model = RiskBudgeting()
    elif method == "black_litterman":
        from skfolio.optimization import MeanRisk
        model = MeanRisk()
    else:
        raise ValueError(f"Unknown method: {method}")
    model.fit(returns)
    weights = dict(zip(tickers, model.weights_.tolist()))
    print(json.dumps({"weights": weights}))
except Exception as e:
    print(json.dumps({"error": str(e)}), file=sys.stderr)
    sys.exit(1)
"#;

/// Call skfolio via Python subprocess with structured JSON I/O.
pub async fn optimize_via_skfolio(
    method: &str,
    returns_json: &str,
    tickers: &[String],
) -> ValeResult<Vec<(String, f64)>> {
    let python = std::env::var("VALE_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let mut child = Command::new(&python)
        .arg("-c")
        .arg(PYTHON_SCRIPT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            ValeError::AdapterUnavailable(format!(
                "python3 not found: {e}. Run `vale doctor` to see installation instructions."
            ))
        })?;

    let input = serde_json::json!({
        "method": method,
        "returns": serde_json::from_str::<serde_json::Value>(returns_json)
            .map_err(|e| ValeError::Parse(e.to_string()))?,
        "tickers": tickers,
    });
    let input_str = serde_json::to_string(&input)?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(input_str.as_bytes())
            .map_err(ValeError::Io)?;
    }

    let output = child.wait_with_output().map_err(ValeError::Io)?;

    if !output.status.success() {
        return Err(ValeError::AdapterUnavailable(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let weights = result["weights"]
        .as_object()
        .ok_or_else(|| ValeError::Parse("Invalid skfolio output".into()))?;

    Ok(weights
        .iter()
        .map(|(k, v)| (k.clone(), v.as_f64().unwrap_or(0.0)))
        .collect())
}
