pub mod alpaca_market;
pub mod cache_layer;
pub mod local;
pub mod local_dir;
pub mod polygon;
pub mod polars_export;
pub mod provider;
pub mod yahoo;

pub use cache_layer::CachedProvider;
pub use provider::DataProvider;

use vale_core::config::Config;
use vale_core::error::{ValeError, ValeResult};

/// Build a data provider from configuration.
pub fn build_provider(config: &Config) -> ValeResult<Box<dyn DataProvider>> {
    let name = config.providers.default.as_str();
    let inner: Box<dyn DataProvider> = match name {
        "yahoo" => Box::new(yahoo::YahooProvider::new(
            config.providers.yahoo.timeout_secs,
        )?),
        "polygon" => {
            if config.providers.polygon.api_key.is_empty() {
                return Err(ValeError::Config(
                    "providers.polygon.api_key is not configured. Run: vale config set providers.polygon.api_key <value>".into(),
                ));
            }
            Box::new(polygon::PolygonProvider::new(
                config.providers.polygon.api_key.clone(),
            )?)
        }
        "alpaca" => {
            if config.providers.alpaca.api_key.is_empty() {
                return Err(ValeError::Config(
                    "providers.alpaca.api_key is not configured".into(),
                ));
            }
            Box::new(alpaca_market::AlpacaMarketProvider::new(
                config.providers.alpaca.api_key.clone(),
                config.providers.alpaca.secret_key.clone(),
                true,
            )?)
        }
        "local" => Box::new(local_dir::LocalDirProvider::new(config.local_data_dir())),
        other => {
            return Err(ValeError::Data(format!("unknown data provider: {other}")));
        }
    };
    let cache = vale_core::cache::Cache::open(&config.cache_dir())?;
    Ok(Box::new(CachedProvider::new(inner, cache)))
}
