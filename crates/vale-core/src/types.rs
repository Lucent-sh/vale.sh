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
