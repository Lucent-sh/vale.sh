use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{BacktestEngine, BacktestResult};

use crate::adapter::{Adapter, AdapterStatus};

pub struct LeanAdapter {
    pub executable: String,
    pub project_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LeanStatistics {
    #[serde(rename = "Sharpe Ratio")]
    sharpe_ratio: Option<String>,
    #[serde(rename = "Net Profit")]
    net_profit: Option<String>,
    #[serde(rename = "Drawdown")]
    drawdown: Option<String>,
    #[serde(rename = "Total Trades")]
    total_trades: Option<String>,
    #[serde(rename = "Win Rate")]
    win_rate: Option<String>,
    #[serde(rename = "Compounding Annual Return")]
    cagr: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LeanResultFile {
    #[serde(rename = "Statistics")]
    statistics: Option<LeanStatistics>,
}

impl LeanAdapter {
    pub fn new(executable: String, project_dir: PathBuf) -> Self {
        Self {
            executable,
            project_dir,
        }
    }

    pub fn detect_executable(config_path: &str) -> Option<String> {
        if !config_path.is_empty() && std::path::Path::new(config_path).exists() {
            return Some(config_path.to_string());
        }
        Command::new("which")
            .arg("lean")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    pub fn run_backtest(&self) -> ValeResult<BacktestResult> {
        let status = Command::new(&self.executable)
            .arg("backtest")
            .current_dir(&self.project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| {
                ValeError::AdapterUnavailable(format!(
                    "lean is not installed. Run `vale doctor` to see installation instructions. ({e})"
                ))
            })?;

        if !status.success() {
            return Err(ValeError::Backtest("lean backtest failed".into()));
        }

        self.parse_latest_result()
    }

    fn parse_latest_result(&self) -> ValeResult<BacktestResult> {
        let backtests_dir = self.project_dir.join("backtests");
        let mut latest: Option<PathBuf> = None;
        if backtests_dir.exists() {
            for entry in std::fs::read_dir(&backtests_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    latest = Some(match latest {
                        None => path,
                        Some(prev) if path > prev => path,
                        Some(prev) => prev,
                    });
                }
            }
        }
        let dir = latest.ok_or_else(|| ValeError::Backtest("no lean results found".into()))?;
        let json_files: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        let json_path = json_files
            .first()
            .map(|e| e.path())
            .ok_or_else(|| ValeError::Backtest("no lean JSON result".into()))?;

        let content = std::fs::read_to_string(json_path)?;
        self.parse_lean_json(&content)
    }

    fn parse_lean_json(&self, content: &str) -> ValeResult<BacktestResult> {
        let parsed: LeanResultFile =
            serde_json::from_str(content).map_err(|e| ValeError::Parse(e.to_string()))?;
        let stats = parsed
            .statistics
            .ok_or_else(|| ValeError::Parse("missing Statistics".into()))?;

        let parse_pct = |s: &Option<String>| -> f64 {
            s.as_ref()
                .and_then(|v| v.trim_end_matches('%').parse().ok())
                .map(|v: f64| v / 100.0)
                .unwrap_or(0.0)
        };

        use chrono::Utc;
        Ok(BacktestResult {
            id: uuid_simple(),
            strategy_name: "lean".into(),
            engine: BacktestEngine::Lean,
            start: Utc::now(),
            end: Utc::now(),
            initial_cash: 100_000.0,
            final_equity: 100_000.0,
            total_return: parse_pct(&stats.net_profit),
            cagr: parse_pct(&stats.cagr),
            sharpe_ratio: stats
                .sharpe_ratio
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            max_drawdown: parse_pct(&stats.drawdown),
            max_drawdown_duration_days: 0,
            volatility_annual: 0.0,
            total_trades: stats
                .total_trades
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            winning_trades: 0,
            losing_trades: 0,
            win_rate: parse_pct(&stats.win_rate),
            profit_factor: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            equity_curve: vec![],
            benchmark_curve: None,
            trades: vec![],
            params: serde_json::json!({}),
        })
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("lean-{t}")
}

#[async_trait]
impl Adapter for LeanAdapter {
    fn name(&self) -> &'static str {
        "lean"
    }

    async fn health_check(&self) -> ValeResult<AdapterStatus> {
        let output = Command::new(&self.executable)
            .arg("--version")
            .output()
            .map_err(|e| ValeError::AdapterUnavailable(e.to_string()))?;
        Ok(AdapterStatus {
            name: "lean".into(),
            available: output.status.success(),
            version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
            location: Some(self.executable.clone()),
            message: None,
        })
    }
}
