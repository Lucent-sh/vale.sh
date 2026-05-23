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
    Http(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Strategy error: {0}")]
    Strategy(String),
}

pub type ValeResult<T> = Result<T, ValeError>;
