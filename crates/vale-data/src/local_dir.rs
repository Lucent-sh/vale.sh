use async_trait::async_trait;
use std::path::{Path, PathBuf};
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{Bar, Resolution, TimeRange};

use crate::local::LocalCsvProvider;
use crate::provider::DataProvider;

/// Load `{data_dir}/{SYMBOL}.csv` (case-insensitive fallback).
pub struct LocalDirProvider {
    data_dir: PathBuf,
}

impl LocalDirProvider {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    fn resolve_csv(&self, symbol: &str) -> ValeResult<PathBuf> {
        let candidates = [
            self.data_dir.join(format!("{symbol}.csv")),
            self.data_dir.join(format!("{}.csv", symbol.to_lowercase())),
            self.data_dir.join(format!("{}.csv", symbol.to_uppercase())),
        ];
        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        Err(ValeError::Data(format!(
            "no CSV for {symbol} in {} (expected {{symbol}}.csv)",
            self.data_dir.display()
        )))
    }
}

#[async_trait]
impl DataProvider for LocalDirProvider {
    fn name(&self) -> &'static str {
        "local"
    }

    fn requires_auth(&self) -> bool {
        false
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        resolution: Resolution,
        range: &TimeRange,
    ) -> ValeResult<Vec<Bar>> {
        let path = self.resolve_csv(symbol)?;
        LocalCsvProvider::new(path)
            .fetch_ohlcv(symbol, resolution, range)
            .await
    }

    async fn ping(&self) -> ValeResult<()> {
        if self.data_dir.is_dir() {
            Ok(())
        } else {
            Err(ValeError::Data(format!(
                "local data directory not found: {}",
                self.data_dir.display()
            )))
        }
    }
}
