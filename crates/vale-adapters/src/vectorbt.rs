use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{BacktestEngine, BacktestResult};

#[derive(Debug, Deserialize)]
struct VectorBtOutput {
    strategy_name: Option<String>,
    total_return: f64,
    cagr: Option<f64>,
    sharpe_ratio: f64,
    sortino_ratio: Option<f64>,
    max_drawdown: f64,
    total_trades: Option<usize>,
    win_rate: Option<f64>,
    final_equity: Option<f64>,
    initial_cash: Option<f64>,
    error: Option<String>,
}

pub struct VectorBtAdapter {
    pub python: String,
    pub script_path: PathBuf,
}

impl VectorBtAdapter {
    pub fn script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("python/vectorbt_runner.py")
    }

    pub fn new(python: impl Into<String>) -> Self {
        Self {
            python: python.into(),
            script_path: Self::script_path(),
        }
    }

    pub fn detect_python() -> Option<String> {
        for cmd in ["python3", "python"] {
            if Command::new(cmd)
                .args(["-c", "import vectorbt"])
                .output()
                .is_ok_and(|o| o.status.success())
            {
                return Some(cmd.to_string());
            }
        }
        None
    }

    pub fn run_backtest(
        &self,
        ticker: &str,
        start: &str,
        end: &str,
        strategy: &str,
        params: &serde_json::Value,
        cash: f64,
    ) -> ValeResult<BacktestResult> {
        if !self.script_path.exists() {
            return Err(ValeError::AdapterUnavailable(format!(
                "vectorbt runner not found at {}",
                self.script_path.display()
            )));
        }

        let payload = serde_json::json!({
            "ticker": ticker,
            "start": start,
            "end": end,
            "strategy": strategy,
            "params": params,
            "cash": cash,
        });

        let mut child = Command::new(&self.python)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ValeError::AdapterUnavailable(format!(
                    "vectorbt is not installed. Run `vale doctor`. ({e})"
                ))
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(payload.to_string().as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(ValeError::Backtest(format!(
                "vectorbt backtest failed: {err}"
            )));
        }

        let parsed: VectorBtOutput = serde_json::from_slice(&output.stdout)
            .map_err(|e| ValeError::Parse(format!("vectorbt output: {e}")))?;

        if let Some(msg) = parsed.error {
            return Err(ValeError::Backtest(msg));
        }

        use chrono::Utc;
        let initial = parsed.initial_cash.unwrap_or(cash);
        let final_eq = parsed.final_equity.unwrap_or(initial * (1.0 + parsed.total_return));

        Ok(BacktestResult {
            id: format!("vbt-{}", Utc::now().timestamp_millis()),
            strategy_name: parsed
                .strategy_name
                .unwrap_or_else(|| strategy.to_string()),
            engine: BacktestEngine::VectorBT,
            start: Utc::now(),
            end: Utc::now(),
            initial_cash: initial,
            final_equity: final_eq,
            total_return: parsed.total_return,
            cagr: parsed.cagr.unwrap_or(parsed.total_return),
            sharpe_ratio: parsed.sharpe_ratio,
            sortino_ratio: parsed.sortino_ratio.unwrap_or(0.0),
            calmar_ratio: 0.0,
            max_drawdown: parsed.max_drawdown,
            max_drawdown_duration_days: 0,
            volatility_annual: 0.0,
            total_trades: parsed.total_trades.unwrap_or(0),
            winning_trades: 0,
            losing_trades: 0,
            win_rate: parsed.win_rate.unwrap_or(0.0),
            profit_factor: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            equity_curve: vec![],
            benchmark_curve: None,
            trades: vec![],
            params: params.clone(),
        })
    }
}
