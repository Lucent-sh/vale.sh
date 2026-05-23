# Vale.sh — Full Implementation Plan
## Agent Prompt + Complete Build Instructions for Cursor Composer

---

## AGENT SYSTEM PROMPT

```
You are a senior systems engineer building Vale — a professional-grade quantitative finance CLI
written in Rust. The binary is named `vale`. The product is Vale.sh.

Your priorities in order:
1. Correctness. Every number Vale produces must be mathematically exact.
2. Performance. Cold start under 50ms. Native backtest under 50ms for 10yr daily data.
3. Developer experience. The CLI must be the most beautiful terminal UI in the quant space.
4. Composability. Every output can be piped. JSON is always available via --output json.

Architecture rules:
- Workspace of crates. One binary: vale-cli. All logic in library crates.
- Every external tool (LEAN, VectorBT, QuantLib, skfolio, OpenBB, TA-Lib) is an optional
  adapter behind a feature flag. Vale works without any of them.
- The adapter pattern is strict: all engines implement the same trait. BacktestResult is
  identical regardless of which engine produced it.
- Async-first using Tokio. CPU-bound work (sweeps, risk metrics) uses Rayon.
- All errors are typed. No unwrap() in library code. Use thiserror for library errors,
  anyhow for binary-level error propagation.
- sled for local cache. Never hit the network twice for the same data.
- pyo3 for Python interop. Python processes are spawned with structured JSON stdin/stdout
  RPC, not raw subprocess hacks.

UI rules (non-negotiable):
- Use Ratatui for all interactive terminal UI (progress, dashboards, watch mode).
- Use indicatif for progress bars on non-interactive operations.
- Use owo-colors + supports-color for all colored output, never hardcoded ANSI codes.
- Use comfy-table for all tabular data.
- Use textplots for ASCII sparklines and equity curves in the terminal.
- Every operation that takes more than 200ms shows a spinner with elapsed time.
- Colors respect NO_COLOR env var automatically via supports-color.

When implementing a feature:
1. Write the trait/interface first in vale-core or the relevant library crate.
2. Write tests with concrete expected values before the implementation.
3. Implement the feature.
4. Wire it into the CLI last.

Never break the `vale doctor` command. It must always run and report status.
Never break `vale --help` or any subcommand `--help`. Help text is part of the product.
```

---

## PROJECT OVERVIEW

```
Product:    Vale.sh
Binary:     vale
Language:   Rust 2021 edition, stable toolchain
MSRV:       1.78.0
License:    MIT
Repo:       github.com/valesh/vale
```

---

## REPOSITORY STRUCTURE

```
vale/
├── Cargo.toml                  # workspace root
├── Cargo.lock
├── rust-toolchain.toml
├── .cargo/
│   └── config.toml             # profile settings, linker flags
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── crates/
│   ├── vale-cli/               # binary entrypoint
│   ├── vale-core/              # shared types, config, error, cache
│   ├── vale-adapters/          # external tool bridge layer
│   ├── vale-backtest/          # native Rust event-driven backtest engine
│   ├── vale-sweep/             # vectorized parameter sweep engine
│   ├── vale-data/              # market data providers
│   ├── vale-indicators/        # technical indicators (native + TA-Lib FFI)
│   ├── vale-portfolio/         # portfolio construction and optimization
│   ├── vale-risk/              # risk metrics engine
│   ├── vale-price/             # derivatives and instrument pricing
│   ├── vale-factor/            # factor analysis (FF3, FF5, alpha decomp)
│   ├── vale-report/            # output rendering
│   └── vale-watch/             # live monitoring
├── scripts/
│   └── install.sh
├── docs/
└── tests/                      # integration tests
    └── fixtures/
```

---

## WORKSPACE CARGO.TOML

```toml
[workspace]
resolver = "2"
members = [
    "crates/vale-cli",
    "crates/vale-core",
    "crates/vale-adapters",
    "crates/vale-backtest",
    "crates/vale-sweep",
    "crates/vale-data",
    "crates/vale-indicators",
    "crates/vale-portfolio",
    "crates/vale-risk",
    "crates/vale-price",
    "crates/vale-factor",
    "crates/vale-report",
    "crates/vale-watch",
]

[workspace.dependencies]
# async
tokio = { version = "1.37", features = ["full"] }
tokio-util = { version = "0.7" }

# serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# error handling
anyhow = "1"
thiserror = "1"

# data
polars = { version = "0.40", features = ["lazy", "csv", "parquet", "json", "temporal"] }
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.9"

# math / stats
statrs = "0.17"
nalgebra = "0.32"

# http
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }

# cache
sled = "0.34"

# parallel
rayon = "1.10"

# cli / ui
clap = { version = "4", features = ["derive", "env", "color", "wrap_help"] }
ratatui = "0.27"
indicatif = { version = "0.17", features = ["rayon"] }
console = "0.15"
owo-colors = "4"
supports-color = "3"
comfy-table = { version = "7", features = ["crossterm"] }
textplots = "0.8"
crossterm = { version = "0.27", features = ["event-stream"] }

# python interop
pyo3 = { version = "0.21", features = ["auto-initialize"] }

# logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# utils
dirs = "5"
shellexpand = "3"
uuid = { version = "1", features = ["v4"] }
async-trait = "0.1"

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true

[profile.dev]
opt-level = 1
```

---

## RUST TOOLCHAIN

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

---

## CRATE 1: vale-core

### Purpose
All shared types, config, error types, local cache, and the async runtime wrapper.
No business logic. No I/O except config file reading and cache operations.

### Cargo.toml

```toml
[package]
name = "vale-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
chrono = { workspace = true }
polars = { workspace = true }
sled = { workspace = true }
dirs = { workspace = true }
shellexpand = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
```

### src/lib.rs

```rust
pub mod config;
pub mod error;
pub mod types;
pub mod cache;
pub mod resolution;
```

### src/types.rs

All domain types used across crates. Implement fully.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single OHLCV bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub symbol: String,
}

/// A completed trade from a backtest or live session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub direction: TradeDirection,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub fees: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeDirection {
    Long,
    Short,
}

/// Full normalized result from any backtest engine.
/// This is the universal output type — LEAN, VectorBT, and the
/// native engine all produce this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: String,
    pub strategy_name: String,
    pub engine: BacktestEngine,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub initial_cash: f64,
    pub final_equity: f64,
    pub total_return: f64,
    pub cagr: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_drawdown: f64,
    pub max_drawdown_duration_days: u64,
    pub volatility_annual: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub equity_curve: Vec<(DateTime<Utc>, f64)>,
    pub benchmark_curve: Option<Vec<(DateTime<Utc>, f64)>>,
    pub trades: Vec<Trade>,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BacktestEngine {
    Native,
    Lean,
    VectorBT,
}

/// Identifies a market instrument.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub ticker: String,
    pub asset_class: AssetClass,
    pub market: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetClass {
    Equity,
    Futures,
    Options,
    Forex,
    Crypto,
    Bond,
    Etf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum Resolution {
    Tick,
    Second,
    Minute,
    Hour,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Output format for CLI rendering.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}
```

### src/error.rs

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValeError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Data error: {0}")]
    Data(String),

    #[error("Backtest error: {0}")]
    Backtest(String),

    #[error("Adapter not available: {0}")]
    AdapterUnavailable(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Strategy error: {0}")]
    Strategy(String),
}

pub type ValeResult<T> = Result<T, ValeError>;
```

### src/config.rs

Full config struct with all fields. Loads from `~/.vale/config.toml` then merges
`./vale.toml` (project-level overrides).

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{ValeError, ValeResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub core: CoreConfig,
    pub providers: ProvidersConfig,
    pub lean: LeanConfig,
    pub risk: RiskConfig,
    pub report: ReportConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreConfig {
    pub default_engine: String,       // "native" | "lean" | "vectorbt"
    pub default_output: String,       // "table" | "json" | "csv"
    pub cache_dir: String,
    pub parallelism: usize,
    pub color: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProvidersConfig {
    pub default: String,
    pub polygon: PolygonConfig,
    pub alpaca: AlpacaConfig,
    pub yahoo: YahooConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolygonConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YahooConfig {
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LeanConfig {
    pub executable: String,
    pub docker_image: String,
    pub python_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RiskConfig {
    pub default_var_confidence: f64,
    pub risk_free_rate: f64,
    pub annualization_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportConfig {
    pub default_format: String,
    pub html_open_browser: bool,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: String,       // "dark" | "light" | "auto"
    pub sparklines: bool,
    pub animations: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            providers: ProvidersConfig::default(),
            lean: LeanConfig::default(),
            risk: RiskConfig::default(),
            report: ReportConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            default_engine: "native".into(),
            default_output: "table".into(),
            cache_dir: "~/.vale/cache".into(),
            parallelism: num_cpus::get(),
            color: true,
        }
    }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            default: "yahoo".into(),
            polygon: Default::default(),
            alpaca: AlpacaConfig {
                base_url: "https://paper-api.alpaca.markets".into(),
                ..Default::default()
            },
            yahoo: YahooConfig { timeout_secs: 10 },
        }
    }
}

impl Default for LeanConfig {
    fn default() -> Self {
        Self {
            executable: "lean".into(),
            docker_image: "quantconnect/lean:latest".into(),
            python_env: String::new(),
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            default_var_confidence: 0.95,
            risk_free_rate: 0.05,
            annualization_factor: 252.0,
        }
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            default_format: "table".into(),
            html_open_browser: false,
            output_dir: "./vale-reports".into(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            sparklines: true,
            animations: true,
        }
    }
}

impl Config {
    /// Load config: global (~/.vale/config.toml) merged with project (./vale.toml).
    pub fn load() -> ValeResult<Self> {
        let mut config = Self::default();

        // Load global config
        if let Some(home) = dirs::home_dir() {
            let global = home.join(".vale").join("config.toml");
            if global.exists() {
                let text = std::fs::read_to_string(&global)?;
                let parsed: Config = toml::from_str(&text)
                    .map_err(|e| ValeError::Config(e.to_string()))?;
                config = parsed;
            }
        }

        // Load project config (overrides global)
        let project = PathBuf::from("vale.toml");
        if project.exists() {
            let text = std::fs::read_to_string(&project)?;
            let parsed: Config = toml::from_str(&text)
                .map_err(|e| ValeError::Config(e.to_string()))?;
            // Selective merge: only override non-default fields
            // For simplicity in Phase 1, project config fully overrides global
            config = parsed;
        }

        Ok(config)
    }

    pub fn cache_dir(&self) -> PathBuf {
        PathBuf::from(shellexpand::tilde(&self.core.cache_dir).to_string())
    }

    pub fn init_global() -> ValeResult<()> {
        let home = dirs::home_dir()
            .ok_or_else(|| ValeError::Config("Cannot find home directory".into()))?;
        let vale_dir = home.join(".vale");
        std::fs::create_dir_all(&vale_dir)?;
        let config_path = vale_dir.join("config.toml");
        if !config_path.exists() {
            let default = toml::to_string_pretty(&Config::default())
                .map_err(|e| ValeError::Config(e.to_string()))?;
            std::fs::write(&config_path, default)?;
        }
        Ok(())
    }
}
```

### src/cache.rs

```rust
use crate::error::{ValeError, ValeResult};
use std::path::Path;

pub struct Cache {
    db: sled::Db,
}

impl Cache {
    pub fn open(path: &Path) -> ValeResult<Self> {
        std::fs::create_dir_all(path)?;
        let db = sled::open(path).map_err(|e| ValeError::Cache(e.to_string()))?;
        Ok(Self { db })
    }

    pub fn get(&self, key: &str) -> ValeResult<Option<Vec<u8>>> {
        self.db
            .get(key)
            .map(|v| v.map(|iv| iv.to_vec()))
            .map_err(|e| ValeError::Cache(e.to_string()))
    }

    pub fn set(&self, key: &str, value: &[u8]) -> ValeResult<()> {
        self.db
            .insert(key, value)
            .map(|_| ())
            .map_err(|e| ValeError::Cache(e.to_string()))
    }

    pub fn remove(&self, key: &str) -> ValeResult<()> {
        self.db
            .remove(key)
            .map(|_| ())
            .map_err(|e| ValeError::Cache(e.to_string()))
    }

    /// Cache key for market data.
    pub fn market_data_key(provider: &str, symbol: &str, resolution: &str, from: &str, to: &str) -> String {
        format!("data:{}:{}:{}:{}:{}", provider, symbol, resolution, from, to)
    }
}
```

---

## CRATE 2: vale-risk

### Purpose
Pure computation. No I/O, no async. All functions take slices of f64. Fully tested.

### src/lib.rs

```rust
pub mod metrics;
pub mod stress;
pub mod drawdown;
pub mod correlation;

pub use metrics::*;
pub use drawdown::*;
```

### src/metrics.rs — IMPLEMENT ALL OF THESE

```rust
/// Annualized Sharpe ratio.
/// returns: daily log returns
/// risk_free: daily risk-free rate (annual_rate / 252)
/// annualization: sqrt(252) for daily
pub fn sharpe_ratio(returns: &[f64], risk_free_daily: f64, annualization: f64) -> f64 {
    if returns.is_empty() { return 0.0; }
    let excess: Vec<f64> = returns.iter().map(|r| r - risk_free_daily).collect();
    let mean = mean(&excess);
    let std = std_dev(&excess);
    if std == 0.0 { return 0.0; }
    (mean / std) * annualization
}

/// Sortino ratio (penalizes only downside deviation).
pub fn sortino_ratio(returns: &[f64], risk_free_daily: f64, annualization: f64) -> f64 {
    if returns.is_empty() { return 0.0; }
    let excess: Vec<f64> = returns.iter().map(|r| r - risk_free_daily).collect();
    let mean = mean(&excess);
    let downside: Vec<f64> = excess.iter().filter(|&&r| r < 0.0).cloned().collect();
    if downside.is_empty() { return f64::INFINITY; }
    let downside_std = std_dev(&downside);
    if downside_std == 0.0 { return 0.0; }
    (mean / downside_std) * annualization
}

/// Calmar ratio = CAGR / |max_drawdown|.
pub fn calmar_ratio(cagr: f64, max_drawdown: f64) -> f64 {
    if max_drawdown == 0.0 { return f64::INFINITY; }
    cagr / max_drawdown.abs()
}

/// Compound Annual Growth Rate from equity curve.
pub fn cagr(equity: &[f64], years: f64) -> f64 {
    if equity.len() < 2 || years == 0.0 { return 0.0; }
    let first = equity[0];
    let last = equity[equity.len() - 1];
    if first == 0.0 { return 0.0; }
    (last / first).powf(1.0 / years) - 1.0
}

/// Annual volatility from daily returns.
pub fn volatility_annual(returns: &[f64], annualization: f64) -> f64 {
    std_dev(returns) * annualization
}

/// Historical Value at Risk (percentile of loss distribution).
/// confidence = 0.95 → returns the 5th percentile loss (positive number = loss).
pub fn historical_var(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() { return 0.0; }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((1.0 - confidence) * sorted.len() as f64) as usize;
    let idx = idx.min(sorted.len() - 1);
    -sorted[idx]
}

/// Conditional VaR (Expected Shortfall).
/// Average of losses worse than VaR at given confidence.
pub fn cvar(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() { return 0.0; }
    let var = historical_var(returns, confidence);
    let tail: Vec<f64> = returns.iter().filter(|&&r| -r >= var).cloned().collect();
    if tail.is_empty() { return var; }
    -mean(&tail)
}

/// Beta of strategy returns relative to benchmark.
pub fn beta(returns: &[f64], benchmark: &[f64]) -> f64 {
    if returns.len() != benchmark.len() || returns.is_empty() { return 0.0; }
    let n = returns.len() as f64;
    let mean_r = mean(returns);
    let mean_b = mean(benchmark);
    let cov: f64 = returns.iter().zip(benchmark.iter())
        .map(|(r, b)| (r - mean_r) * (b - mean_b))
        .sum::<f64>() / n;
    let var_b = variance(benchmark);
    if var_b == 0.0 { return 0.0; }
    cov / var_b
}

/// Alpha = strategy_mean - (risk_free + beta * (benchmark_mean - risk_free)), annualized.
pub fn alpha(returns: &[f64], benchmark: &[f64], risk_free_daily: f64, annualization: f64) -> f64 {
    let b = beta(returns, benchmark);
    let mean_r = mean(returns);
    let mean_b = mean(benchmark);
    (mean_r - (risk_free_daily + b * (mean_b - risk_free_daily))) * annualization
}

/// Profit factor = sum of wins / |sum of losses|.
pub fn profit_factor(pnls: &[f64]) -> f64 {
    let wins: f64 = pnls.iter().filter(|&&p| p > 0.0).sum();
    let losses: f64 = pnls.iter().filter(|&&p| p < 0.0).map(|p| p.abs()).sum();
    if losses == 0.0 { return f64::INFINITY; }
    wins / losses
}

// Statistical helpers
pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 { return 0.0; }
    let m = mean(data);
    data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / data.len() as f64
}

pub fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

/// Convert price series to log returns.
pub fn log_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2)
        .map(|w| (w[1] / w[0]).ln())
        .collect()
}

/// Convert price series to simple returns.
pub fn simple_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect()
}
```

### src/drawdown.rs

```rust
/// Maximum drawdown from peak. Returns positive number (magnitude of largest decline).
pub fn max_drawdown(equity: &[f64]) -> f64 {
    if equity.is_empty() { return 0.0; }
    let mut peak = equity[0];
    let mut max_dd = 0.0_f64;
    for &val in equity {
        if val > peak { peak = val; }
        let dd = (peak - val) / peak;
        if dd > max_dd { max_dd = dd; }
    }
    max_dd
}

/// All drawdown periods: (start_idx, end_idx, recovery_idx, magnitude).
pub fn drawdown_periods(equity: &[f64]) -> Vec<DrawdownPeriod> {
    let mut periods = Vec::new();
    let mut peak = equity[0];
    let mut peak_idx = 0;
    let mut in_drawdown = false;
    let mut trough_idx = 0;
    let mut trough_val = equity[0];

    for (i, &val) in equity.iter().enumerate() {
        if val >= peak {
            if in_drawdown {
                periods.push(DrawdownPeriod {
                    start: peak_idx,
                    trough: trough_idx,
                    end: i,
                    magnitude: (peak - trough_val) / peak,
                    duration_bars: i - peak_idx,
                });
                in_drawdown = false;
            }
            peak = val;
            peak_idx = i;
        } else {
            in_drawdown = true;
            if val < trough_val {
                trough_val = val;
                trough_idx = i;
            }
        }
    }
    periods
}

#[derive(Debug, Clone)]
pub struct DrawdownPeriod {
    pub start: usize,
    pub trough: usize,
    pub end: usize,
    pub magnitude: f64,
    pub duration_bars: usize,
}
```

---

## CRATE 3: vale-data

### Purpose
Market data fetching, caching, and normalization. All providers implement
`DataProvider` trait. Cache-first: always check sled before network.

### Cargo.toml

```toml
[package]
name = "vale-data"
version = "0.1.0"
edition = "2021"

[dependencies]
vale-core = { path = "../vale-core" }
tokio = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
polars = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
```

### src/lib.rs

```rust
pub mod provider;
pub mod yahoo;
pub mod polygon;
pub mod local;
pub mod cache_layer;

pub use provider::DataProvider;
pub use cache_layer::CachedProvider;
```

### src/provider.rs

```rust
use async_trait::async_trait;
use vale_core::{types::{Bar, Resolution, TimeRange}, error::ValeResult};

#[async_trait]
pub trait DataProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn requires_auth(&self) -> bool;

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        resolution: Resolution,
        range: &TimeRange,
    ) -> ValeResult<Vec<Bar>>;

    /// Check if the provider is accessible right now.
    async fn ping(&self) -> ValeResult<()>;
}
```

### src/yahoo.rs — IMPLEMENT FULLY

Use the `yahoo_finance_api` crate or build direct HTTP calls to the Yahoo Finance
v8 API. Parse OHLCV JSON into `Vec<Bar>`. Handle rate limits with exponential backoff.
Map the following resolutions: Daily → "1d", Hour → "1h", Minute → "1m".

The Yahoo v8 endpoint:
```
https://query1.finance.yahoo.com/v8/finance/chart/{SYMBOL}?interval={INTERVAL}&range={RANGE}
```

For date range queries:
```
https://query1.finance.yahoo.com/v8/finance/chart/{SYMBOL}?interval={INTERVAL}&period1={UNIX_START}&period2={UNIX_END}
```

Implement retry with 3 attempts and 500ms / 1000ms / 2000ms backoff.

```rust
use async_trait::async_trait;
use vale_core::{types::{Bar, Resolution, TimeRange}, error::{ValeError, ValeResult}};
use crate::provider::DataProvider;

pub struct YahooProvider {
    client: reqwest::Client,
    timeout: std::time::Duration,
}

impl YahooProvider {
    pub fn new(timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap();
        Self { client, timeout: std::time::Duration::from_secs(timeout_secs) }
    }

    fn resolution_to_interval(r: Resolution) -> &'static str {
        match r {
            Resolution::Minute => "1m",
            Resolution::Hour => "1h",
            Resolution::Daily => "1d",
            Resolution::Weekly => "1wk",
            Resolution::Monthly => "1mo",
            _ => "1d",
        }
    }
}

#[async_trait]
impl DataProvider for YahooProvider {
    fn name(&self) -> &'static str { "yahoo" }
    fn requires_auth(&self) -> bool { false }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        resolution: Resolution,
        range: &TimeRange,
    ) -> ValeResult<Vec<Bar>> {
        // Implement full fetch + parse + Vec<Bar> construction
        // Parse timestamps from Yahoo's Unix timestamps
        // Parse OHLCV arrays (timestamps, opens, highs, lows, closes, volumes)
        // Return sorted by timestamp ascending
        todo!("implement Yahoo OHLCV fetch")
    }

    async fn ping(&self) -> ValeResult<()> {
        // HEAD request to Yahoo Finance to check connectivity
        todo!("implement Yahoo ping")
    }
}
```

### src/polygon.rs — IMPLEMENT FULLY

Use Polygon.io REST API v2:
```
GET https://api.polygon.io/v2/aggs/ticker/{TICKER}/range/{MULT}/{TIMESPAN}/{FROM}/{TO}
    ?apiKey={KEY}&adjusted=true&sort=asc&limit=50000
```

Timespans: minute, hour, day, week, month.

Handle pagination: if `next_url` is present in response, follow it until exhausted.
Respect Polygon rate limits (5 req/min on free tier, check `X-RateLimit-Remaining`).

### src/local.rs — IMPLEMENT FULLY

Read CSV files. Expected columns: `timestamp,open,high,low,close,volume`
with `timestamp` as ISO 8601 or Unix timestamp. Autodetect format.

---

## CRATE 4: vale-indicators

### Purpose
Technical indicators. Native Rust implementations for core indicators.
Optional TA-Lib FFI for the full 200+ indicator set.

### src/lib.rs

```rust
pub mod native;

#[cfg(feature = "talib")]
pub mod talib;
```

### src/native.rs — IMPLEMENT ALL

```rust
/// Simple Moving Average.
pub fn sma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period { return vec![]; }
    data.windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

/// Exponential Moving Average.
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() { return vec![]; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
    result.push(data[0]);
    for &val in &data[1..] {
        let prev = *result.last().unwrap();
        result.push(val * k + prev * (1.0 - k));
    }
    result
}

/// Relative Strength Index.
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    // Implement Wilder's RSI: initial avg gain/loss then smoothed rolling avg
    // Returns Vec starting at index `period`
    todo!("implement RSI")
}

/// Bollinger Bands. Returns (upper, middle, lower).
pub fn bollinger_bands(data: &[f64], period: usize, std_dev_mult: f64) -> Vec<(f64, f64, f64)> {
    // middle = SMA(period), upper = middle + std_dev_mult * std, lower = middle - std_dev_mult * std
    todo!("implement Bollinger Bands")
}

/// MACD. Returns (macd_line, signal_line, histogram).
pub fn macd(data: &[f64], fast: usize, slow: usize, signal: usize) -> Vec<(f64, f64, f64)> {
    todo!("implement MACD")
}

/// Average True Range.
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    todo!("implement ATR")
}
```

---

## CRATE 5: vale-backtest (Native Engine)

### Purpose
Full event-driven backtesting engine in Rust. No external dependencies for
the core simulation loop. Python strategy loading via pyo3 is optional.

### src/lib.rs

```rust
pub mod engine;
pub mod strategy;
pub mod portfolio;
pub mod commission;
pub mod slippage;
pub mod context;
pub mod order;
```

### src/strategy.rs

```rust
use crate::{context::Context, order::Order};
use vale_core::types::Bar;

pub trait Strategy: Send {
    fn name(&self) -> &str;
    fn on_start(&mut self, _ctx: &mut Context) {}
    fn on_bar(&mut self, ctx: &mut Context, bar: &Bar);
    fn on_end(&mut self, _ctx: &mut Context) {}
}
```

### src/engine.rs — IMPLEMENT FULLY

```rust
use vale_core::types::{BacktestResult, Bar, BacktestEngine};
use vale_core::error::ValeResult;
use crate::{strategy::Strategy, portfolio::Portfolio, commission::CommissionModel,
            slippage::SlippageModel, context::Context};

pub struct BacktestEngine {
    pub commission: Box<dyn CommissionModel>,
    pub slippage: Box<dyn SlippageModel>,
    pub initial_cash: f64,
}

impl BacktestEngine {
    pub fn run(
        &self,
        strategy: &mut dyn Strategy,
        bars: &[Bar],
    ) -> ValeResult<BacktestResult> {
        // 1. Sort bars by timestamp
        // 2. Initialize Portfolio with initial_cash
        // 3. Create Context
        // 4. Call strategy.on_start(&mut ctx)
        // 5. For each bar:
        //    a. Update portfolio mark-to-market
        //    b. Build Context for this bar
        //    c. Call strategy.on_bar(&mut ctx, bar)
        //    d. Process pending orders in ctx.orders
        //    e. Apply commission and slippage to filled orders
        //    f. Record equity point
        // 6. Call strategy.on_end(&mut ctx)
        // 7. Build BacktestResult from portfolio state
        todo!("implement engine run loop")
    }
}
```

### src/commission.rs

```rust
pub trait CommissionModel: Send + Sync {
    fn calculate(&self, quantity: f64, price: f64) -> f64;
}

pub struct PercentageCommission { pub rate: f64 }
pub struct FlatCommission { pub per_trade: f64 }
pub struct PerShareCommission { pub per_share: f64, pub min: f64 }

impl CommissionModel for PercentageCommission {
    fn calculate(&self, quantity: f64, price: f64) -> f64 {
        quantity * price * self.rate
    }
}
// Implement FlatCommission and PerShareCommission similarly.
```

### src/slippage.rs

```rust
pub trait SlippageModel: Send + Sync {
    fn apply(&self, price: f64, quantity: f64, is_buy: bool) -> f64;
}

pub struct FixedSlippage { pub ticks: f64, pub tick_size: f64 }
pub struct PercentageSlippage { pub rate: f64 }
pub struct VolumeSlippage { pub rate: f64 }  // slippage scales with quantity/volume

// Implement all three.
```

---

## CRATE 6: vale-sweep

### Purpose
Run thousands of strategy parameter configurations in parallel using Rayon.
Owns the parameter grid generation and result aggregation.

### src/lib.rs

```rust
pub mod grid;
pub mod runner;
pub mod result;
```

### src/grid.rs

```rust
/// A parameter range: name, start, end, step.
#[derive(Debug, Clone)]
pub struct ParamRange {
    pub name: String,
    pub start: f64,
    pub end: f64,
    pub step: f64,
}

impl ParamRange {
    /// Parse from CLI string "fast_ma:5:50:5"
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            anyhow::bail!("param must be name:start:end:step, got: {}", s);
        }
        Ok(Self {
            name: parts[0].to_string(),
            start: parts[1].parse()?,
            end: parts[2].parse()?,
            step: parts[3].parse()?,
        })
    }

    pub fn values(&self) -> Vec<f64> {
        let mut vals = Vec::new();
        let mut v = self.start;
        while v <= self.end {
            vals.push(v);
            v += self.step;
        }
        vals
    }
}

/// Generate all combinations (Cartesian product) of parameter values.
pub fn cartesian_product(params: &[ParamRange]) -> Vec<Vec<(String, f64)>> {
    // Implement iterative Cartesian product
    todo!("implement cartesian_product")
}
```

### src/runner.rs

```rust
use rayon::prelude::*;
use vale_core::types::BacktestResult;

pub struct SweepResult {
    pub params: std::collections::HashMap<String, f64>,
    pub result: BacktestResult,
}

/// Run all configurations in parallel via Rayon.
/// strategy_factory is a function that builds a fresh Strategy for each config.
pub fn run_sweep<F>(
    configs: Vec<Vec<(String, f64)>>,
    strategy_factory: F,
    bars: &[vale_core::types::Bar],
    engine: &vale_backtest::engine::BacktestEngine,
) -> Vec<SweepResult>
where
    F: Fn(&[(String, f64)]) -> Box<dyn vale_backtest::strategy::Strategy> + Send + Sync,
{
    configs.par_iter()
        .filter_map(|config| {
            let mut strat = strategy_factory(config);
            match engine.run(strat.as_mut(), bars) {
                Ok(result) => Some(SweepResult {
                    params: config.iter().cloned().collect(),
                    result,
                }),
                Err(e) => {
                    tracing::warn!("config failed: {:?}", e);
                    None
                }
            }
        })
        .collect()
}
```

---

## CRATE 7: vale-portfolio

### Purpose
Portfolio construction and optimization. Native implementations for equal weight,
min variance, and max sharpe. Delegates to skfolio via JSON subprocess RPC for
HRP, Black-Litterman, and risk parity.

### src/lib.rs

```rust
pub mod native;
pub mod skfolio;
pub mod weights;

pub use weights::Weights;
```

### src/weights.rs

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Weights(pub HashMap<String, f64>);

impl Weights {
    /// Normalize weights to sum to 1.0.
    pub fn normalize(&mut self) {
        let sum: f64 = self.0.values().sum();
        if sum == 0.0 { return; }
        for v in self.0.values_mut() { *v /= sum; }
    }

    pub fn equal(tickers: &[&str]) -> Self {
        let n = tickers.len() as f64;
        Self(tickers.iter().map(|t| (t.to_string(), 1.0 / n)).collect())
    }
}
```

### src/native.rs — IMPLEMENT

```rust
use nalgebra::{DMatrix, DVector};

/// Minimum variance portfolio weights (long-only, fully invested).
/// returns_matrix: rows = observations, cols = assets
pub fn min_variance(returns_matrix: &DMatrix<f64>, tickers: &[String]) -> Vec<(String, f64)> {
    // 1. Compute covariance matrix
    // 2. Solve the quadratic program: min w^T Σ w s.t. sum(w)=1, w>=0
    // 3. Use SLSQP or analytical solution for unconstrained, Lagrangian for constrained
    // For Phase 1: analytical solution via Lagrangian (unconstrained, then project to simplex)
    todo!("implement min_variance")
}

/// Maximum Sharpe ratio portfolio.
pub fn max_sharpe(returns_matrix: &DMatrix<f64>, tickers: &[String], rf: f64) -> Vec<(String, f64)> {
    // Tangency portfolio: w* = Σ^{-1}(μ - rf * 1) / (1^T Σ^{-1}(μ - rf * 1))
    todo!("implement max_sharpe")
}
```

### src/skfolio.rs — IMPLEMENT

Spawn Python process, pipe JSON, parse result.

```rust
use std::process::{Command, Stdio};
use std::io::Write;
use vale_core::error::{ValeError, ValeResult};

/// Call skfolio via a Python subprocess with structured JSON I/O.
/// Input: JSON with returns data and optimization method.
/// Output: JSON with weights.
pub async fn optimize_via_skfolio(
    method: &str,     // "hrp" | "risk_parity" | "black_litterman"
    returns_json: &str,
    tickers: &[String],
) -> ValeResult<Vec<(String, f64)>> {
    // 1. Find Python executable (check VALE_PYTHON env, then `python3`, then python in venv)
    // 2. Spawn process with the embedded Python script (see below)
    // 3. Write JSON to stdin
    // 4. Read JSON from stdout
    // 5. Parse weights

    let python_script = r#"
import sys, json
import numpy as np

data = json.loads(sys.stdin.read())
method = data["method"]
returns = np.array(data["returns"])
tickers = data["tickers"]

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
"#;

    let mut child = Command::new("python3")
        .arg("-c")
        .arg(python_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ValeError::AdapterUnavailable(format!("python3 not found: {}", e)))?;

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(returns_json.as_bytes())
        .map_err(|e| ValeError::Io(e))?;

    let output = child.wait_with_output()
        .map_err(|e| ValeError::Io(e))?;

    if !output.status.success() {
        return Err(ValeError::AdapterUnavailable(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let weights = result["weights"].as_object()
        .ok_or_else(|| ValeError::Parse("Invalid skfolio output".into()))?;

    Ok(weights.iter()
        .map(|(k, v)| (k.clone(), v.as_f64().unwrap_or(0.0)))
        .collect())
}
```

---

## CRATE 8: vale-adapters

### Purpose
All external tool integrations. Each adapter implements the relevant domain traits.
All adapters are gated behind feature flags.

### Cargo.toml

```toml
[package]
name = "vale-adapters"
version = "0.1.0"
edition = "2021"

[features]
lean = []
vectorbt = ["pyo3"]
quantlib = []
openbb = []

[dependencies]
vale-core = { path = "../vale-core" }
async-trait = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
reqwest = { workspace = true }
pyo3 = { workspace = true, optional = true }
```

### src/lib.rs

```rust
pub mod adapter;

#[cfg(feature = "lean")]
pub mod lean;

#[cfg(feature = "vectorbt")]
pub mod vectorbt;

pub mod doctor;
```

### src/adapter.rs

```rust
use async_trait::async_trait;
use vale_core::error::ValeResult;

#[derive(Debug, Clone)]
pub struct AdapterStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub location: Option<String>,
    pub message: Option<String>,
}

#[async_trait]
pub trait Adapter: Send + Sync {
    fn name(&self) -> &'static str;
    async fn health_check(&self) -> ValeResult<AdapterStatus>;
}
```

### src/lean.rs — IMPLEMENT FULLY

```rust
// 1. Detect lean binary via `which lean` or config path
// 2. Generate lean project config files (config.json, lean.json)
// 3. Spawn `lean backtest` subprocess
// 4. Stream stdout/stderr to indicatif progress bar
// 5. After completion, find results directory
// 6. Parse LEAN's JSON output into BacktestResult
// Key: LEAN outputs results to ./backtests/{id}/
// The main file is {id}.json with keys: Statistics, Charts, Orders

pub struct LeanAdapter {
    pub executable: String,
    pub project_dir: std::path::PathBuf,
}

// Parse LEAN statistics into BacktestResult fields:
// "Sharpe Ratio" -> sharpe_ratio
// "Net Profit" -> total_return
// "Drawdown" -> max_drawdown
// "Total Trades" -> total_trades
// "Win Rate" -> win_rate
// "Compounding Annual Return" -> cagr
```

### src/doctor.rs — IMPLEMENT FULLY

```rust
use crate::adapter::AdapterStatus;

pub struct DoctorReport {
    pub vale_version: String,
    pub config_path: Option<std::path::PathBuf>,
    pub cache_size_bytes: u64,
    pub data_providers: Vec<AdapterStatus>,
    pub backtest_engines: Vec<AdapterStatus>,
    pub portfolio_optimizers: Vec<AdapterStatus>,
    pub pricing_engines: Vec<AdapterStatus>,
}

impl DoctorReport {
    pub async fn run(config: &vale_core::config::Config) -> Self {
        // Check each integration:
        // lean: `lean --version` subprocess
        // vectorbt: `python3 -c "import vectorbt; print(vectorbt.__version__)"`
        // quantlib: `python3 -c "import QuantLib; print(QuantLib.__version__)"`
        // pyql: `python3 -c "import pyql; print(pyql.__version__)"`
        // skfolio: `python3 -c "import skfolio; print(skfolio.__version__)"`
        // ta-lib: `python3 -c "import talib; print(talib.__version__)"`
        // openbb: `python3 -c "import openbb; print('ok')"`
        // polygon key: check config
        // alpaca key: check config
        todo!("implement doctor checks")
    }
}
```

---

## CRATE 9: vale-report

### Purpose
All output rendering. Never contains computation logic. Renders `BacktestResult`,
`Weights`, risk metrics structs into table, JSON, CSV, or HTML.

### src/lib.rs

```rust
pub mod table;
pub mod chart;
pub mod html;
pub mod json;
pub mod csv;
pub mod tearsheet;
```

### src/table.rs — IMPLEMENT FULLY

Use `comfy-table` with custom styling.

```rust
use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
use vale_core::types::BacktestResult;

pub fn backtest_summary(result: &BacktestResult) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(comfy_table::presets::UTF8_FULL);

    // Header row with colored metric names
    // Color-code values: positive returns in green, negative in red
    // Show: Total Return, CAGR, Sharpe, Sortino, Max DD, Win Rate, Trades, Profit Factor

    table.set_header(vec![
        Cell::new("Metric").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);

    let metrics = [
        ("Total Return", format!("{:.2}%", result.total_return * 100.0)),
        ("CAGR", format!("{:.2}%", result.cagr * 100.0)),
        ("Sharpe Ratio", format!("{:.3}", result.sharpe_ratio)),
        ("Sortino Ratio", format!("{:.3}", result.sortino_ratio)),
        ("Calmar Ratio", format!("{:.3}", result.calmar_ratio)),
        ("Max Drawdown", format!("{:.2}%", result.max_drawdown * 100.0)),
        ("Volatility (Ann.)", format!("{:.2}%", result.volatility_annual * 100.0)),
        ("Total Trades", result.total_trades.to_string()),
        ("Win Rate", format!("{:.1}%", result.win_rate * 100.0)),
        ("Profit Factor", format!("{:.2}", result.profit_factor)),
    ];

    for (name, value) in &metrics {
        table.add_row(vec![Cell::new(name), Cell::new(value)]);
    }

    table
}
```

### src/chart.rs — IMPLEMENT FULLY

```rust
use textplots::{Chart, Plot, Shape};
use vale_core::types::BacktestResult;

/// Render ASCII equity curve. Returns multi-line string.
pub fn equity_curve(result: &BacktestResult, width: u32, height: u32) -> String {
    if result.equity_curve.is_empty() { return String::new(); }

    let start_ts = result.equity_curve[0].0.timestamp() as f64;
    let points: Vec<(f32, f32)> = result.equity_curve.iter()
        .map(|(ts, equity)| {
            let x = (ts.timestamp() as f64 - start_ts) / 86400.0;
            (x as f32, *equity as f32)
        })
        .collect();

    let mut output = String::new();
    Chart::new(width, height, points[0].0, points.last().unwrap().0)
        .lineplot(&Shape::Lines(&points))
        .to_string()
}
```

### src/html.rs — IMPLEMENT FULLY

Generate a self-contained HTML tearsheet using embedded CSS and Plotly.js loaded
from CDN. The HTML must be completely standalone (single file, no external deps
except Plotly CDN).

Sections:
1. Header: strategy name, date range, engine
2. Performance summary table
3. Equity curve chart (Plotly line chart)
4. Drawdown chart (Plotly filled area chart)
5. Monthly returns heatmap (Plotly annotated heatmap)
6. Trade statistics
7. Trades table

```rust
pub fn generate_tearsheet(result: &BacktestResult) -> String {
    // Build HTML string with embedded data as JSON in <script> tags
    // Plotly.js draws charts from the embedded data
    todo!("implement HTML tearsheet")
}
```

---

## CRATE 10: vale-cli (BINARY)

### Purpose
Command dispatch, output routing, UI rendering. No business logic.
All logic lives in library crates.

### Cargo.toml

```toml
[package]
name = "vale-cli"
version = "0.1.0"
edition = "2021"
default-run = "vale"

[[bin]]
name = "vale"
path = "src/main.rs"

[dependencies]
vale-core = { path = "../vale-core" }
vale-data = { path = "../vale-data" }
vale-backtest = { path = "../vale-backtest" }
vale-sweep = { path = "../vale-sweep" }
vale-portfolio = { path = "../vale-portfolio" }
vale-risk = { path = "../vale-risk" }
vale-report = { path = "../vale-report" }
vale-adapters = { path = "../vale-adapters", features = ["lean"] }
tokio = { workspace = true }
clap = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
indicatif = { workspace = true }
console = { workspace = true }
owo-colors = { workspace = true }
supports-color = { workspace = true }
comfy-table = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
chrono = { workspace = true }
```

### src/main.rs

```rust
mod cli;
mod commands;
mod ui;
mod theme;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Init tracing. Only show logs if VALE_LOG is set; otherwise silent.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("VALE_LOG")
                .unwrap_or_else(|_| EnvFilter::new("error"))
        )
        .init();

    let cli = cli::Cli::parse();
    commands::dispatch(cli).await
}
```

### src/cli.rs

Full clap derive struct for the entire command surface.

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};
use vale_core::types::{Resolution, OutputFormat};

#[derive(Parser)]
#[command(
    name = "vale",
    version,
    about = "Vale — quantitative finance at terminal speed",
    long_about = None,
    arg_required_else_help = true,
    propagate_version = true,
    styles = crate::theme::clap_styles(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output format
    #[arg(global = true, short, long, default_value = "table")]
    pub output: OutputFormat,

    /// Suppress colors
    #[arg(global = true, long, env = "NO_COLOR")]
    pub no_color: bool,

    /// Verbose mode (shows debug info)
    #[arg(global = true, short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run and manage backtests
    #[command(subcommand)]
    Backtest(BacktestCommand),

    /// Sweep parameter space across thousands of configs
    #[command(subcommand)]
    Sweep(SweepCommand),

    /// Fetch and inspect market data
    #[command(subcommand)]
    Data(DataCommand),

    /// Portfolio construction and optimization
    #[command(subcommand)]
    Portfolio(PortfolioCommand),

    /// Risk metrics and stress testing
    #[command(subcommand)]
    Risk(RiskCommand),

    /// Instrument pricing (options, bonds, derivatives)
    #[command(subcommand)]
    Price(PriceCommand),

    /// Factor analysis and alpha decomposition
    #[command(subcommand)]
    Factor(FactorCommand),

    /// Generate and view performance reports
    #[command(subcommand)]
    Report(ReportCommand),

    /// Scaffold and validate strategy files
    #[command(subcommand)]
    Strategy(StrategyCommand),

    /// Monitor live positions and paper trading
    Watch(WatchArgs),

    /// Verify installed integrations and config
    Doctor,

    /// Manage Vale configuration
    #[command(subcommand)]
    Config(ConfigCommand),
}

// --- Backtest ---

#[derive(Subcommand)]
pub enum BacktestCommand {
    /// Run a backtest
    Run(BacktestRunArgs),
    /// Compare two backtest result files side by side
    Compare(BacktestCompareArgs),
    /// Lint a strategy file for common bias issues
    Validate(BacktestValidateArgs),
}

#[derive(Args)]
pub struct BacktestRunArgs {
    /// Path to strategy file (.py for LEAN/VectorBT, or Rust strategy name)
    #[arg(short, long)]
    pub strategy: std::path::PathBuf,

    /// Backtest engine to use
    #[arg(short, long, default_value = "native")]
    pub engine: BacktestEngineArg,

    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    pub start: String,

    /// End date (YYYY-MM-DD)
    #[arg(long)]
    pub end: String,

    /// Data resolution
    #[arg(short, long, default_value = "daily")]
    pub resolution: Resolution,

    /// Initial cash
    #[arg(long, default_value = "100000")]
    pub cash: f64,

    /// Benchmark ticker for comparison
    #[arg(long)]
    pub benchmark: Option<String>,

    /// Save result JSON to this path
    #[arg(long)]
    pub save: Option<std::path::PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum BacktestEngineArg {
    Native,
    Lean,
    Vectorbt,
}

#[derive(Args)]
pub struct BacktestCompareArgs {
    /// Paths to result JSON files (2 or more)
    pub results: Vec<std::path::PathBuf>,
}

#[derive(Args)]
pub struct BacktestValidateArgs {
    pub strategy: std::path::PathBuf,
}

// --- Sweep ---

#[derive(Subcommand)]
pub enum SweepCommand {
    Run(SweepRunArgs),
}

#[derive(Args)]
pub struct SweepRunArgs {
    #[arg(short, long)]
    pub strategy: std::path::PathBuf,

    /// Parameter range: name:start:end:step (repeatable)
    #[arg(short, long = "param", num_args = 1..)]
    pub params: Vec<String>,

    /// Metric to rank by
    #[arg(short, long, default_value = "sharpe")]
    pub metric: String,

    /// Number of top results to show
    #[arg(long, default_value = "10")]
    pub top: usize,

    #[arg(long)]
    pub start: String,

    #[arg(long)]
    pub end: String,

    #[arg(short, long, default_value = "daily")]
    pub resolution: Resolution,
}

// --- Data ---

#[derive(Subcommand)]
pub enum DataCommand {
    /// Fetch OHLCV data
    Fetch(DataFetchArgs),
    /// Inspect a local data file
    Inspect(DataInspectArgs),
    /// Export cached data
    Export(DataExportArgs),
    /// List configured and available data sources
    Sources,
}

#[derive(Args)]
pub struct DataFetchArgs {
    /// Ticker symbol(s)
    #[arg(short, long, num_args = 1..)]
    pub ticker: Vec<String>,

    #[arg(short, long, default_value = "daily")]
    pub resolution: Resolution,

    #[arg(long)]
    pub from: String,

    #[arg(long)]
    pub to: Option<String>,

    /// Data source to use
    #[arg(long)]
    pub source: Option<String>,

    /// Output file path (if omitted, prints to stdout)
    #[arg(short, long)]
    pub out: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct DataInspectArgs {
    pub file: std::path::PathBuf,
}

#[derive(Args)]
pub struct DataExportArgs {
    #[arg(short, long)]
    pub ticker: String,

    #[arg(short, long, default_value = "csv")]
    pub format: String,

    #[arg(long)]
    pub from: String,

    #[arg(long)]
    pub to: Option<String>,

    #[arg(short, long)]
    pub out: std::path::PathBuf,
}

// --- Portfolio ---

#[derive(Subcommand)]
pub enum PortfolioCommand {
    Optimize(PortfolioOptimizeArgs),
    Backtest(PortfolioBacktestArgs),
    EfficientFrontier(EfficientFrontierArgs),
}

#[derive(Args)]
pub struct PortfolioOptimizeArgs {
    #[arg(short, long, num_args = 1..)]
    pub tickers: Vec<String>,

    #[arg(short, long, default_value = "max_sharpe")]
    pub method: String,

    #[arg(long)]
    pub start: String,

    #[arg(long)]
    pub end: Option<String>,

    #[arg(long, default_value = "0.05")]
    pub risk_free: f64,
}

#[derive(Args)]
pub struct PortfolioBacktestArgs {
    #[arg(long)]
    pub weights: std::path::PathBuf,

    #[arg(long, default_value = "monthly")]
    pub rebalance: String,

    #[arg(long)]
    pub start: String,

    #[arg(long)]
    pub end: Option<String>,
}

#[derive(Args)]
pub struct EfficientFrontierArgs {
    #[arg(short, long, num_args = 1..)]
    pub tickers: Vec<String>,

    #[arg(long, default_value = "200")]
    pub points: usize,

    #[arg(long)]
    pub start: String,

    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
}

// --- Risk ---

#[derive(Subcommand)]
pub enum RiskCommand {
    Metrics(RiskMetricsArgs),
    Stress(RiskStressArgs),
    Correlation(RiskCorrelationArgs),
}

#[derive(Args)]
pub struct RiskMetricsArgs {
    /// Path to equity curve CSV (columns: timestamp,equity)
    #[arg(short, long)]
    pub input: std::path::PathBuf,

    /// VaR confidence levels
    #[arg(long, num_args = 1.., default_values_t = vec![0.95_f64, 0.99_f64])]
    pub var_confidence: Vec<f64>,

    #[arg(long, default_value = "0.05")]
    pub risk_free: f64,
}

#[derive(Args)]
pub struct RiskStressArgs {
    #[arg(long)]
    pub portfolio: std::path::PathBuf,

    /// Scenario names (2008-crisis, 2020-covid, 2022-rate-shock, etc.)
    #[arg(long = "scenario", num_args = 1..)]
    pub scenarios: Vec<String>,
}

#[derive(Args)]
pub struct RiskCorrelationArgs {
    #[arg(short, long, num_args = 1..)]
    pub tickers: Vec<String>,

    #[arg(long, default_value = "pearson")]
    pub method: String,

    #[arg(long)]
    pub rolling: Option<usize>,

    #[arg(long)]
    pub start: String,
}

// --- Price ---

#[derive(Subcommand)]
pub enum PriceCommand {
    Option(PriceOptionArgs),
    Bond(PriceBondArgs),
    Greeks(PriceGreeksArgs),
}

#[derive(Args)]
pub struct PriceOptionArgs {
    #[arg(long)]
    pub type_: String,         // "european-call" | "european-put" | "american-call" | "american-put"

    #[arg(long)]
    pub spot: f64,

    #[arg(long)]
    pub strike: f64,

    #[arg(long)]
    pub expiry: String,        // "90d" | "2025-12-19"

    #[arg(long)]
    pub vol: f64,

    #[arg(long)]
    pub rate: f64,

    #[arg(long, default_value = "black-scholes")]
    pub model: String,

    #[arg(long)]
    pub iv: Option<f64>,       // If set, compute implied vol from market price
}

#[derive(Args)]
pub struct PriceBondArgs {
    #[arg(long)]
    pub face: f64,

    #[arg(long)]
    pub coupon: f64,

    #[arg(long)]
    pub maturity: String,

    #[arg(long)]
    pub rate: f64,
}

#[derive(Args)]
pub struct PriceGreeksArgs {
    #[arg(long)]
    pub type_: String,

    #[arg(long)]
    pub spot: f64,

    #[arg(long)]
    pub strike: f64,

    #[arg(long)]
    pub expiry: String,

    #[arg(long)]
    pub vol: f64,

    #[arg(long)]
    pub rate: f64,
}

// --- Factor ---

#[derive(Subcommand)]
pub enum FactorCommand {
    Analyze(FactorAnalyzeArgs),
    Ic(FactorIcArgs),
}

#[derive(Args)]
pub struct FactorAnalyzeArgs {
    #[arg(short, long)]
    pub returns: std::path::PathBuf,

    #[arg(long, default_value = "ff3")]
    pub model: String,      // "ff3" | "ff5" | "carhart4"

    #[arg(long)]
    pub benchmark: Option<String>,
}

#[derive(Args)]
pub struct FactorIcArgs {
    #[arg(long)]
    pub signals: std::path::PathBuf,

    #[arg(long)]
    pub returns: std::path::PathBuf,

    #[arg(long, num_args = 1.., default_values_t = vec![1_usize, 5, 21])]
    pub periods: Vec<usize>,
}

// --- Report ---

#[derive(Subcommand)]
pub enum ReportCommand {
    /// Generate tearsheet from a backtest result JSON
    Tearsheet(ReportTearsheetArgs),
    Show(ReportShowArgs),
}

#[derive(Args)]
pub struct ReportTearsheetArgs {
    #[arg(short, long)]
    pub input: std::path::PathBuf,

    #[arg(long, default_value = "html")]
    pub format: String,

    #[arg(short, long)]
    pub out: Option<std::path::PathBuf>,

    #[arg(long)]
    pub open: bool,
}

#[derive(Args)]
pub struct ReportShowArgs {
    pub result: std::path::PathBuf,
}

// --- Strategy ---

#[derive(Subcommand)]
pub enum StrategyCommand {
    Scaffold(StrategyScaffoldArgs),
    Validate(StrategyValidateArgs),
    List,
}

#[derive(Args)]
pub struct StrategyScaffoldArgs {
    pub name: String,

    #[arg(long, default_value = "lean-python")]
    pub template: String,

    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct StrategyValidateArgs {
    pub strategy: std::path::PathBuf,
}

// --- Watch ---

#[derive(Args)]
pub struct WatchArgs {
    #[arg(long, default_value = "alpaca")]
    pub broker: String,

    #[arg(long)]
    pub strategy: Option<std::path::PathBuf>,

    #[arg(long, default_value = "paper")]
    pub mode: String,
}

// --- Config ---

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Initialize global config at ~/.vale/config.toml
    Init,
    /// Get a config value
    Get { key: String },
    /// Set a config value
    Set { key: String, value: String },
    /// Print full config
    Show,
    /// Open config file in $EDITOR
    Edit,
}
```

---

## CLI UI DESIGN SYSTEM

### src/theme.rs — IMPLEMENT IN FULL

This is where Vale's visual identity lives. Every color, every border, every
spacing decision is defined here. The aesthetic is: **dark terminal, amber
accents, precise monospace data, zero decoration that doesn't carry information.**

```rust
use owo_colors::OwoColorize;
use supports_color::Stream;

/// Vale color palette — RGB values for true-color terminals.
/// Falls back to ANSI 256/16 when true-color is not supported.
pub mod palette {
    // Primary amber: Vale's brand accent
    pub const AMBER: (u8, u8, u8) = (255, 176, 0);
    // Muted amber for secondary text
    pub const AMBER_DIM: (u8, u8, u8) = (180, 120, 0);
    // Bright white for primary text
    pub const WHITE: (u8, u8, u8) = (240, 240, 235);
    // Slate for secondary text and borders
    pub const SLATE: (u8, u8, u8) = (120, 120, 130);
    // Positive: green for gains, good metrics
    pub const GREEN: (u8, u8, u8) = (80, 200, 120);
    // Negative: red for losses, bad metrics
    pub const RED: (u8, u8, u8) = (220, 80, 80);
    // Info: cyan for neutral highlights
    pub const CYAN: (u8, u8, u8) = (80, 190, 220);
    // Background: shown in Ratatui context
    pub const BG_DARK: (u8, u8, u8) = (18, 18, 22);
    pub const BG_CARD: (u8, u8, u8) = (26, 26, 32);
    pub const BORDER: (u8, u8, u8) = (50, 50, 65);
}

/// Returns true if the terminal supports color output.
pub fn color_enabled() -> bool {
    supports_color::on(Stream::Stdout).is_some()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("VALE_NO_COLOR").is_err()
}

/// Custom clap styling with amber headers.
pub fn clap_styles() -> clap::builder::Styles {
    use clap::builder::styling::*;
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().bold())
        .usage(AnsiColor::Yellow.on_default().bold())
        .literal(AnsiColor::White.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}

/// Print the Vale banner on first run and help screens.
///
/// Design: monospace ASCII wordmark with amber gradient effect.
/// Dimensions: 8 lines, 60 chars wide. No external font rendering.
pub fn print_banner() {
    if !color_enabled() {
        println!("vale — quantitative finance at terminal speed");
        return;
    }
    // Full 8-line ASCII art wordmark for "VALE"
    // Each character rendered in block style, amber colored
    let lines = [
        r"  ██╗   ██╗ █████╗ ██╗     ███████╗",
        r"  ██║   ██║██╔══██╗██║     ██╔════╝",
        r"  ██║   ██║███████║██║     █████╗  ",
        r"  ╚██╗ ██╔╝██╔══██║██║     ██╔══╝  ",
        r"   ╚████╔╝ ██║  ██║███████╗███████╗",
        r"    ╚═══╝  ╚═╝  ╚═╝╚══════╝╚══════╝",
    ];
    for line in &lines {
        println!("{}", line.truecolor(255, 176, 0));
    }
    println!(
        "  {}",
        "quantitative finance at terminal speed"
            .truecolor(120, 120, 130)
    );
    println!();
}

/// Spinner style used by indicatif.
pub fn spinner_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template(
        "{spinner:.yellow} {msg} {elapsed_precise:.dim}"
    )
    .unwrap()
    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}

/// Progress bar style for long operations with known total.
pub fn progress_bar_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template(
        "{spinner:.yellow} [{bar:40.yellow/dim}] {pos}/{len} {msg} ({eta})"
    )
    .unwrap()
    .progress_chars("█▉▊▋▌▍▎▏ ")
    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}

/// Multi-progress bar style (for sweep operations).
pub fn sweep_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template(
        "  {msg:<40} [{bar:30.yellow/dim}] {pos}/{len}"
    )
    .unwrap()
    .progress_chars("█▉ ")
}

/// Print a section header line.
///     ── Section Name ──────────────────────────────
pub fn section_header(title: &str) {
    if !color_enabled() {
        println!("── {} ──", title);
        return;
    }
    let line = format!("── {} ", title);
    let pad = "─".repeat(60_usize.saturating_sub(line.len()));
    println!(
        "{}{}",
        line.truecolor(255, 176, 0).bold(),
        pad.truecolor(50, 50, 65)
    );
}

/// Print a key-value status line with colored value.
pub fn status_line(key: &str, value: &str, ok: bool) {
    if !color_enabled() {
        println!("  [{}] {}: {}", if ok { "ok" } else { "--" }, key, value);
        return;
    }
    let indicator = if ok {
        "[ok]".truecolor(80, 200, 120).bold().to_string()
    } else {
        "[--]".truecolor(120, 120, 130).to_string()
    };
    println!(
        "  {} {:<28} {}",
        indicator,
        key.truecolor(240, 240, 235),
        value.truecolor(120, 120, 130)
    );
}

/// Format a metric value with color based on whether higher is better.
pub fn colored_metric(value: f64, is_positive_good: bool) -> String {
    if !color_enabled() {
        return format!("{:.4}", value);
    }
    if is_positive_good {
        if value > 0.0 {
            format!("{:.4}", value).truecolor(80, 200, 120).to_string()
        } else {
            format!("{:.4}", value).truecolor(220, 80, 80).to_string()
        }
    } else {
        // For drawdown, beta, etc. — lower is better
        if value < 0.1 {
            format!("{:.4}", value).truecolor(80, 200, 120).to_string()
        } else {
            format!("{:.4}", value).truecolor(220, 80, 80).to_string()
        }
    }
}

/// Print a success message.
pub fn success(msg: &str) {
    if color_enabled() {
        println!("  {} {}", "✓".truecolor(80, 200, 120).bold(), msg);
    } else {
        println!("  [ok] {}", msg);
    }
}

/// Print an error message (to stderr).
pub fn error(msg: &str) {
    if color_enabled() {
        eprintln!("  {} {}", "✗".truecolor(220, 80, 80).bold(), msg.truecolor(220, 80, 80));
    } else {
        eprintln!("  [error] {}", msg);
    }
}

/// Print a warning message.
pub fn warning(msg: &str) {
    if color_enabled() {
        println!("  {} {}", "⚠".truecolor(255, 176, 0), msg.truecolor(255, 176, 0));
    } else {
        println!("  [warn] {}", msg);
    }
}

/// Print an info message (dimmed).
pub fn info(msg: &str) {
    if color_enabled() {
        println!("  {} {}", "·".truecolor(120, 120, 130), msg.truecolor(120, 120, 130));
    } else {
        println!("  {}", msg);
    }
}

/// comfy-table preset with Vale styling (dark borders, amber header).
pub fn table_style(table: &mut comfy_table::Table) {
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
}
```

### src/ui/mod.rs — RATATUI COMPONENTS

```rust
pub mod spinner;
pub mod progress;
pub mod watch_dashboard;
pub mod sweep_dashboard;
```

### src/ui/watch_dashboard.rs

Full Ratatui TUI for `vale watch`. This is the most complex UI component.

Layout:
```
┌─ vale watch ─────────────────────────────────────────────────────────────────┐
│  STRATEGY: momentum.py   BROKER: alpaca-paper   MODE: paper   17:32:44 UTC  │
├──────────────────┬───────────────────────┬──────────────────────────────────┤
│  POSITIONS       │  P&L SUMMARY          │  EQUITY CURVE                    │
│                  │                       │                                  │
│  SPY   100  long │  Day P&L:  +$1,234    │  ▂▃▄▅▆▇█▇▆▇█▇▆▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇   │
│  QQQ    50  long │  Total:    +$8,921    │                                  │
│  TLT   -30  short│  Sharpe:   1.43       │                                  │
│                  │  Max DD:  -4.2%       │                                  │
├──────────────────┴───────────────────────┴──────────────────────────────────┤
│  RECENT ORDERS                                                               │
│  17:31:22  BUY   SPY   10 @ 483.21  FILLED                                  │
│  17:28:05  SELL  TLT   15 @ 91.40   FILLED                                  │
│  17:15:33  BUY   QQQ    5 @ 421.88  PENDING                                 │
├──────────────────────────────────────────────────────────────────────────────┤
│  [q] quit   [r] refresh   [h] help                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

Implement using:
- `ratatui::widgets::Block` with `BorderType::Rounded`
- `ratatui::widgets::Table` for positions and orders
- `ratatui::widgets::Sparkline` for equity curve
- `ratatui::widgets::Paragraph` for summary stats
- `crossterm::event::EventStream` for non-blocking input
- Refresh on a `tokio::time::interval` of 2 seconds

### src/ui/sweep_dashboard.rs

Ratatui TUI for `vale sweep run`. Shows live progress of parameter sweep.

Layout:
```
┌─ vale sweep ─────────────────────────────────────────────────────────────────┐
│  strategy: momentum.py   params: fast_ma × slow_ma   configs: 324           │
├──────────────────────────────────────────────────────────────────────────────┤
│  Progress  ████████████████████████░░░░░░░░  218/324  67%   eta: 0:00:42    │
├──────────────────────────────────────────────────────────────────────────────┤
│  TOP RESULTS (by sharpe)                                                     │
│                                                                              │
│  #   fast_ma  slow_ma  sharpe  cagr     max_dd  win_rate                    │
│  1   10       50       2.14    34.2%    -8.1%   61.4%                       │
│  2   10       40       2.08    31.7%    -8.9%   59.8%                       │
│  3   15       50       1.97    29.1%    -9.2%   58.3%                       │
│  4   10       60       1.91    28.4%   -10.1%   57.9%                       │
│  5   20       60       1.88    27.8%    -9.8%   57.1%                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

Updates live as results come in from the Rayon thread pool.
Use `std::sync::mpsc::channel` to send results from worker threads to the UI thread.

---

## COMMAND IMPLEMENTATIONS

### src/commands/mod.rs

```rust
pub mod backtest;
pub mod sweep;
pub mod data;
pub mod portfolio;
pub mod risk;
pub mod price;
pub mod factor;
pub mod report;
pub mod strategy;
pub mod watch;
pub mod doctor;
pub mod config;

use anyhow::Result;
use crate::cli::{Cli, Command};

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Backtest(cmd) => backtest::handle(cmd, cli.output, cli.verbose).await,
        Command::Sweep(cmd) => sweep::handle(cmd, cli.output).await,
        Command::Data(cmd) => data::handle(cmd, cli.output).await,
        Command::Portfolio(cmd) => portfolio::handle(cmd, cli.output).await,
        Command::Risk(cmd) => risk::handle(cmd, cli.output).await,
        Command::Price(cmd) => price::handle(cmd, cli.output).await,
        Command::Factor(cmd) => factor::handle(cmd, cli.output).await,
        Command::Report(cmd) => report::handle(cmd).await,
        Command::Strategy(cmd) => strategy::handle(cmd).await,
        Command::Watch(args) => watch::handle(args).await,
        Command::Doctor => doctor::handle().await,
        Command::Config(cmd) => config::handle(cmd).await,
    }
}
```

### src/commands/doctor.rs — IMPLEMENT FULLY

This command must always work. It is the first thing users run.

```rust
pub async fn handle() -> anyhow::Result<()> {
    use crate::theme;

    theme::print_banner();
    theme::section_header("Core");

    let config = vale_core::config::Config::load()
        .unwrap_or_default();

    // vale version
    theme::status_line("vale", env!("CARGO_PKG_VERSION"), true);

    // config file
    let config_path = dirs::home_dir()
        .map(|h| h.join(".vale").join("config.toml"))
        .filter(|p| p.exists());
    theme::status_line(
        "config",
        &config_path.map(|p| p.display().to_string())
            .unwrap_or_else(|| "not found — run: vale config init".into()),
        config_path.is_some(),
    );

    // cache
    let cache_dir = config.cache_dir();
    let cache_size = dir_size(&cache_dir).unwrap_or(0);
    theme::status_line(
        "cache",
        &format!("{} ({:.1} MB)", cache_dir.display(), cache_size as f64 / 1_000_000.0),
        cache_dir.exists(),
    );

    println!();
    theme::section_header("Data Providers");

    check_tool_status("yahoo", "no key required", || {
        // Try a HEAD request to Yahoo
        std::process::Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}",
                   "https://query1.finance.yahoo.com"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout) == "200")
            .unwrap_or(false)
    });

    let polygon_ok = !config.providers.polygon.api_key.is_empty();
    theme::status_line(
        "polygon",
        if polygon_ok { "key configured" } else {
            "key not configured — run: vale config set providers.polygon.api_key <KEY>"
        },
        polygon_ok,
    );

    let alpaca_ok = !config.providers.alpaca.api_key.is_empty();
    theme::status_line(
        "alpaca",
        if alpaca_ok { "key configured" } else {
            "key not configured — run: vale config set providers.alpaca.api_key <KEY>"
        },
        alpaca_ok,
    );

    println!();
    theme::section_header("Backtest Engines");
    theme::status_line("native", "built-in", true);
    check_python_package("lean", "lean", "lean --version");
    check_python_package("vectorbt", "vectorbt", "import vectorbt; print(vectorbt.__version__)");

    println!();
    theme::section_header("Portfolio Optimizers");
    theme::status_line("native (equal_weight, min_variance, max_sharpe)", "built-in", true);
    check_python_package("skfolio", "skfolio", "import skfolio; print(skfolio.__version__)");
    check_python_package("pypfopt", "pypfopt", "import pypfopt; print(pypfopt.__version__)");

    println!();
    theme::section_header("Pricing Engines");
    check_python_package("quantlib (pyql)", "pyql", "import pyql; print('available')");

    println!();
    Ok(())
}

fn check_python_package(display_name: &str, _pkg: &str, code: &str) {
    let result = std::process::Command::new("python3")
        .args(["-c", &format!("{}", code)])
        .output();

    match result {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            crate::theme::status_line(display_name, &ver, true);
        }
        _ => {
            crate::theme::status_line(
                display_name,
                &format!("not found — install: pip install {}", display_name),
                false,
            );
        }
    }
}
```

### src/commands/backtest.rs

```rust
use indicatif::ProgressBar;
use crate::{cli::{BacktestCommand, BacktestRunArgs}, theme};
use vale_core::types::OutputFormat;

pub async fn handle(cmd: BacktestCommand, output: OutputFormat, verbose: bool) -> anyhow::Result<()> {
    match cmd {
        BacktestCommand::Run(args) => run(args, output, verbose).await,
        BacktestCommand::Compare(args) => compare(args, output).await,
        BacktestCommand::Validate(args) => validate(args).await,
    }
}

async fn run(args: BacktestRunArgs, output: OutputFormat, _verbose: bool) -> anyhow::Result<()> {
    let config = vale_core::config::Config::load()?;

    // 1. Show spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(theme::spinner_style());
    pb.set_message(format!("loading data for backtest…"));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // 2. Parse dates
    let start = chrono::NaiveDate::parse_from_str(&args.start, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("invalid start date: {}", args.start))?;
    let end = chrono::NaiveDate::parse_from_str(&args.end, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("invalid end date: {}", args.end))?;

    // 3. Determine tickers from strategy or args
    // (For native engine, strategy file declares tickers in a manifest comment or config)

    // 4. Fetch data for all tickers
    // let provider = vale_data::build_provider(&config)?;
    // let bars = provider.fetch_ohlcv(...).await?;

    pb.set_message("running backtest…");

    // 5. Dispatch to engine
    // let result = match args.engine {
    //     BacktestEngineArg::Native => { ... }
    //     BacktestEngineArg::Lean => { ... }
    //     BacktestEngineArg::Vectorbt => { ... }
    // };

    pb.finish_and_clear();

    // 6. Print result
    // match output {
    //     OutputFormat::Table => {
    //         theme::section_header("Backtest Result");
    //         println!("{}", vale_report::table::backtest_summary(&result));
    //         println!();
    //         theme::section_header("Equity Curve");
    //         println!("{}", vale_report::chart::equity_curve(&result, 120, 24));
    //     }
    //     OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&result)?),
    //     OutputFormat::Csv => vale_report::csv::backtest_equity_curve(&result)?,
    // }

    Ok(())
}
```

---

## CI/CD CONFIGURATION

### .github/workflows/ci.yml

```yaml
name: CI

on:
  push:
    branches: [main, dev]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy (deny warnings)
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Tests
        run: cargo test --workspace

  test-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace
```

### .github/workflows/release.yml

```yaml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            cross: true
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross (if needed)
        if: matrix.cross
        run: cargo install cross --git https://github.com/cross-rs/cross
      - name: Build
        run: |
          if [ "${{ matrix.cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }} -p vale-cli
          else
            cargo build --release --target ${{ matrix.target }} -p vale-cli
          fi
        shell: bash
      - name: Package binary
        run: |
          cd target/${{ matrix.target }}/release
          tar czf vale-${{ matrix.target }}.tar.gz vale${{ matrix.os == 'windows-latest' && '.exe' || '' }}
        shell: bash
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/vale-*.tar.gz
```

---

## TESTS — WRITE THESE BEFORE IMPLEMENTING

### tests/risk_metrics.rs

```rust
#[cfg(test)]
mod tests {
    use vale_risk::metrics::*;
    use vale_risk::drawdown::*;

    #[test]
    fn sharpe_ratio_known_value() {
        // Annual returns: 10%, RF: 5%, Vol: 15%
        // Daily: mean ~ 0.000378, rf ~ 0.000198, std ~ 0.00945
        // Sharpe = (0.000378 - 0.000198) / 0.00945 * sqrt(252) ≈ 0.302
        // Use synthetic daily returns that produce known Sharpe
        let returns = vec![0.01_f64, -0.005, 0.008, -0.003, 0.012];
        let rf = 0.0;
        let s = sharpe_ratio(&returns, rf, 1.0);  // annualization = 1 for unit test
        assert!(s > 0.0, "Sharpe should be positive for positive-mean returns");
    }

    #[test]
    fn max_drawdown_flat() {
        let equity = vec![100.0, 100.0, 100.0];
        assert_eq!(max_drawdown(&equity), 0.0);
    }

    #[test]
    fn max_drawdown_single_peak() {
        let equity = vec![100.0, 120.0, 100.0, 80.0, 90.0, 110.0];
        // Peak at 120, trough at 80: DD = (120 - 80) / 120 = 0.3333
        let dd = max_drawdown(&equity);
        assert!((dd - 0.3333).abs() < 0.001, "expected ~0.333, got {}", dd);
    }

    #[test]
    fn historical_var_known() {
        // 100 returns, worst 5 are -0.10 to -0.06
        // VaR at 95% should be ~0.06
        let mut returns: Vec<f64> = (0..95).map(|i| i as f64 * 0.001).collect();
        returns.extend([-0.10, -0.09, -0.08, -0.07, -0.06]);
        let var = historical_var(&returns, 0.95);
        assert!(var > 0.05 && var < 0.12, "VaR should be in tail: got {}", var);
    }

    #[test]
    fn cagr_doubles_in_7_years() {
        // Starting at 100, ending at 200, over 7 years
        // CAGR = (200/100)^(1/7) - 1 ≈ 0.1041
        let equity = vec![100.0, 200.0];
        let c = cagr(&equity, 7.0);
        assert!((c - 0.1041).abs() < 0.001, "expected ~0.1041, got {}", c);
    }

    #[test]
    fn log_returns_basic() {
        let prices = vec![100.0, 110.0, 99.0];
        let returns = log_returns(&prices);
        assert_eq!(returns.len(), 2);
        assert!((returns[0] - (110.0_f64 / 100.0).ln()).abs() < 1e-10);
    }
}
```

### tests/backtest_engine.rs

```rust
// Test: buy-and-hold strategy on synthetic data produces correct return
// Test: strategy with no trades produces zero return minus fees
// Test: commission is correctly deducted from portfolio value
// Test: short position PnL is correct when price falls
```

---

## INSTALL SCRIPT

### scripts/install.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="valesh/vale"
INSTALL_DIR="${VALE_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VALE_VERSION:-latest}"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

case "$OS" in
  linux) TARGET="${ARCH}-unknown-linux-gnu" ;;
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | cut -d'"' -f4)
fi

URL="https://github.com/$REPO/releases/download/$VERSION/vale-$TARGET.tar.gz"

echo "Installing vale $VERSION for $TARGET..."
mkdir -p "$INSTALL_DIR"
curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR" vale
chmod +x "$INSTALL_DIR/vale"

echo "vale installed to $INSTALL_DIR/vale"
echo "Run 'vale doctor' to verify your installation."
```

---

## IMPLEMENTATION ORDER

Follow this exact order. Do not skip steps. Do not implement a later crate
before all tests for the current crate pass.

### Week 1-2: Foundation
1. Create workspace with `vale-core`. All types, config, error, cache. Tests pass.
2. Create `vale-risk`. All metric functions. All tests in `tests/risk_metrics.rs` pass.
3. Create `vale-data` with `YahooProvider`. Integration test: fetch SPY daily 2020-2024.

### Week 3-4: Engine
4. Create `vale-indicators`. SMA, EMA, RSI, BB, MACD, ATR. Tests with known values.
5. Create `vale-backtest`. Strategy trait, engine, portfolio, commission, slippage. Tests pass.
6. Wire a `BuyAndHold` example strategy through the engine. Verify output.

### Week 5: CLI Phase 1
7. Create `vale-cli` with `vale doctor`, `vale config init/show/set/get`.
8. Wire `vale backtest run --engine native`.
9. Wire `vale data fetch` with Yahoo.
10. Wire `vale risk metrics`.
11. Full `vale-report` table and JSON output. ASCII equity curve.

### Week 6: Sweep + Portfolio
12. Create `vale-sweep`. Grid generator, Rayon runner, Ratatui sweep dashboard.
13. Create `vale-portfolio`. Native min_variance and max_sharpe. Tests.
14. Wire `vale portfolio optimize`. Wire `vale sweep run`.

### Week 7: Adapters
15. Create `vale-adapters`. Implement LEAN adapter (subprocess + JSON parsing).
16. Wire `vale backtest run --engine lean`.
17. Implement skfolio adapter in `vale-portfolio`.
18. Wire HRP and risk_parity portfolio methods.

### Week 8: Polish + Ship
19. HTML tearsheet (`vale-report` html module).
20. `vale watch` Ratatui dashboard (paper mode only).
21. `vale price option` with native Black-Scholes implementation.
22. CI passes on all platforms. Install script tested.
23. `vale doctor` shows complete status on clean machine.

---

## ADDITIONAL IMPLEMENTATION NOTES

### Black-Scholes Implementation (vale-price)

Implement natively without QuantLib for Phase 1. Add QuantLib adapter in Phase 3.

```
d1 = (ln(S/K) + (r + σ²/2) * T) / (σ * √T)
d2 = d1 - σ * √T
Call = S * N(d1) - K * e^(-rT) * N(d2)
Put  = K * e^(-rT) * N(-d2) - S * N(-d1)
Delta(call) = N(d1)
Delta(put)  = N(d1) - 1
Gamma = N'(d1) / (S * σ * √T)
Vega  = S * N'(d1) * √T / 100  (per 1% vol change)
Theta = -(S * N'(d1) * σ) / (2√T) - r * K * e^(-rT) * N(d2)  (call, per day)
Rho   = K * T * e^(-rT) * N(d2) / 100  (call, per 1% rate change)
```

Use `statrs::distribution::Normal` for N(x) and N'(x).

### Implied Volatility (Newton-Raphson)

```
Given market_price, solve for σ: BS(σ) = market_price
σ_{n+1} = σ_n - (BS(σ_n) - market_price) / Vega(σ_n)
Iterate until |BS(σ_n) - market_price| < 1e-6 or 100 iterations.
```

### Strategy Scaffolding Templates

`vale strategy scaffold --template lean-python`:
Generates a minimal LEAN Python project with:
- `main.py` with `QCAlgorithm` subclass
- `config.json` for LEAN CLI
- `.vale/strategy.toml` with ticker list and parameter declarations

`vale strategy scaffold --template native-rust`:
Generates a Rust file with the `Strategy` trait implemented as a template.

### Factor Data Sources

Fama-French data: Download from Kenneth French's library at
`https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Research_Data_Factors_daily_CSV.zip`
Cache in `~/.vale/cache/ff/`.

### Stress Scenarios

Built-in scenarios stored as `HashMap<&str, HashMap<&str, f64>>` mapping
scenario name → (asset → period return). Sources:

- `2008-crisis`: Sep 2008 - Mar 2009. SPY: -51%, TLT: +20%, GLD: +5%
- `2020-covid`: Feb 2020 - Mar 2020. SPY: -34%, TLT: +10%, GLD: -3%
- `2022-rate-shock`: Jan 2022 - Dec 2022. SPY: -19%, TLT: -30%, GLD: -2%
- `2000-dotcom`: Mar 2000 - Oct 2002. SPY: -49%, TLT: +30%, GLD: +15%

---

## PERFORMANCE CHECKLIST

Before shipping each component, verify:

- [ ] `vale doctor` runs in < 3 seconds (all subprocess checks combined)
- [ ] `vale data fetch SPY --from 2015-01-01 --resolution daily` hits cache in < 10ms on second run
- [ ] `vale backtest run --engine native` completes 10yr daily single-asset in < 50ms
- [ ] `vale risk metrics` on a 2500-row CSV completes in < 10ms
- [ ] `vale sweep run --param fast:5:50:5 --param slow:20:200:10` (380 configs) completes in < 30s on 8 cores
- [ ] `vale portfolio optimize --method max_sharpe` on 5 assets completes in < 200ms
- [ ] Cold start (`vale --help`) in < 50ms

---

## DEFINITION OF DONE

A feature is ship-ready when:
1. All unit tests pass with `cargo test --workspace`.
2. `cargo clippy --workspace -- -D warnings` passes with zero warnings.
3. `cargo fmt --all` produces no diff.
4. The command runs end-to-end on a clean machine with `vale doctor` showing the feature available.
5. `--output json` produces valid JSON that `jq` can parse.
6. `--help` text is complete, accurate, and matches the actual behavior.
7. Errors print to stderr with `[error]` prefix and exit code 1.
8. Success output prints to stdout and exits with code 0.
