use crate::cli::WatchArgs;
use anyhow::Result;
use std::sync::Arc;
use vale_core::config::Config;
use vale_watch::broker::AlpacaProvider;
use vale_watch::run_dashboard;

pub async fn handle(args: WatchArgs) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let strategy = args
        .strategy
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "momentum.py".into());

    let broker: Arc<dyn vale_watch::broker::BrokerProvider> = match args.broker.as_str() {
        "alpaca" => {
            if config.providers.alpaca.api_key.is_empty() {
                Arc::new(AlpacaProvider::new(
                    String::new(),
                    String::new(),
                    config.providers.alpaca.base_url.clone(),
                )?)
            } else {
                Arc::new(AlpacaProvider::new(
                    config.providers.alpaca.api_key.clone(),
                    config.providers.alpaca.secret_key.clone(),
                    config.providers.alpaca.base_url.clone(),
                )?)
            }
        }
        other => anyhow::bail!("unknown broker: {other}"),
    };

    run_dashboard(broker, strategy, args.mode)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
