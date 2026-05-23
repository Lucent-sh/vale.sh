use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vale_core::cache::Cache;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{Bar, Resolution, TimeRange};

use crate::provider::DataProvider;

pub struct CachedProvider {
    inner: Arc<dyn DataProvider>,
    cache: Cache,
}

#[derive(Serialize, Deserialize)]
struct CachedBars {
    bars: Vec<Bar>,
    cached_at: i64,
}

impl CachedProvider {
    pub fn new(inner: Box<dyn DataProvider>, cache: Cache) -> Self {
        Self {
            inner: Arc::from(inner),
            cache,
        }
    }

    fn ttl_seconds(resolution: Resolution) -> u64 {
        match resolution {
            Resolution::Daily | Resolution::Weekly | Resolution::Monthly => 86400,
            Resolution::Hour => 3600,
            Resolution::Minute | Resolution::Second | Resolution::Tick => 300,
        }
    }

    fn cache_key(
        provider: &str,
        symbol: &str,
        resolution: Resolution,
        range: &TimeRange,
    ) -> String {
        Cache::market_data_key(
            provider,
            symbol,
            &resolution.to_string(),
            &range.start.to_rfc3339(),
            &range.end.to_rfc3339(),
        )
    }
}

#[async_trait]
impl DataProvider for CachedProvider {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn requires_auth(&self) -> bool {
        self.inner.requires_auth()
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        resolution: Resolution,
        range: &TimeRange,
    ) -> ValeResult<Vec<Bar>> {
        let key = Self::cache_key(self.inner.name(), symbol, resolution, range);
        if let Some(bytes) = self.cache.get(&key)? {
            if let Ok(cached) = serde_json::from_slice::<CachedBars>(&bytes) {
                let age = Utc::now().timestamp() - cached.cached_at;
                if age < Self::ttl_seconds(resolution) as i64 {
                    return Ok(cached.bars);
                }
            }
        }

        let bars = self.inner.fetch_ohlcv(symbol, resolution, range).await?;
        let entry = CachedBars {
            bars: bars.clone(),
            cached_at: Utc::now().timestamp(),
        };
        let bytes = serde_json::to_vec(&entry).map_err(ValeError::Json)?;
        self.cache.set(&key, &bytes)?;
        Ok(bars)
    }

    async fn ping(&self) -> ValeResult<()> {
        self.inner.ping().await
    }
}
