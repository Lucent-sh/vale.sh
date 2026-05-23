use async_trait::async_trait;
use vale_core::error::ValeResult;
use vale_core::types::{Bar, Resolution, TimeRange};

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

    async fn ping(&self) -> ValeResult<()>;
}
