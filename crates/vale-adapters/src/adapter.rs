use async_trait::async_trait;
use vale_core::error::ValeResult;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
