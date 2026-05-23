use std::process::{Command, Stdio};
use vale_core::error::{ValeError, ValeResult};

const OPENBB_SCRIPT: &str = r#"
import json, sys
req = json.loads(sys.stdin.read())
try:
    from openbb import obb
    symbol = req["symbol"]
    result = obb.equity.price.historical(symbol, start_date=req["start"], end_date=req["end"])
    df = result.to_df()
    bars = []
    for row in df.itertuples():
        bars.append({
            "timestamp": str(row.date),
            "open": float(row.open),
            "high": float(row.high),
            "low": float(row.low),
            "close": float(row.close),
            "volume": float(getattr(row, "volume", 0)),
            "symbol": symbol,
        })
    print(json.dumps({"bars": bars}))
except Exception as e:
    print(json.dumps({"error": str(e)}), file=sys.stderr)
    sys.exit(1)
"#;

pub struct OpenBbAdapter {
    pub python: String,
}

impl OpenBbAdapter {
    pub fn detect() -> Option<Self> {
        for cmd in ["python3", "python"] {
            if Command::new(cmd)
                .args(["-c", "from openbb import obb"])
                .output()
                .is_ok_and(|o| o.status.success())
            {
                return Some(Self {
                    python: cmd.to_string(),
                });
            }
        }
        None
    }

    pub fn fetch_bars_json(
        &self,
        symbol: &str,
        start: &str,
        end: &str,
    ) -> ValeResult<String> {
        let payload = serde_json::json!({
            "symbol": symbol,
            "start": start,
            "end": end,
        });
        let mut child = Command::new(&self.python)
            .arg("-c")
            .arg(OPENBB_SCRIPT)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ValeError::AdapterUnavailable(format!("openbb: {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(payload.to_string().as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(ValeError::Data(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
