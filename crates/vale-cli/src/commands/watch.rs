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
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "buy_and_hold".into());

    let broker: Arc<dyn vale_watch::broker::BrokerProvider> = match args.broker.as_str() {
        "alpaca" => {
            let provider = AlpacaProvider::new(
                config.providers.alpaca.api_key.clone(),
                config.providers.alpaca.secret_key.clone(),
                config.providers.alpaca.base_url.clone(),
            )?;
            Arc::new(provider)
        }
        other => anyhow::bail!("unknown broker: {other}"),
    };

    let mode = if broker.name() == "alpaca" {
        let demo = config.providers.alpaca.api_key.is_empty();
        if demo {
            format!("{} [DEMO DATA]", args.mode)
        } else {
            args.mode
        }
    } else {
        args.mode
    };

    run_dashboard(broker, strategy, mode)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
