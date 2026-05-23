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
    pub fn market_data_key(
        provider: &str,
        symbol: &str,
        resolution: &str,
        from: &str,
        to: &str,
    ) -> String {
        format!("data:{provider}:{symbol}:{resolution}:{from}:{to}")
    }
}
