use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{Bar, Resolution, TimeRange};

use crate::provider::DataProvider;

pub struct AlpacaMarketProvider {
    client: reqwest::Client,
    api_key: String,
    secret_key: String,
    data_url: String,
}

#[derive(Debug, Deserialize)]
struct AlpacaBarsResponse {
    bars: Option<Vec<AlpacaBar>>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlpacaBar {
    t: DateTime<Utc>,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: u64,
}

impl AlpacaMarketProvider {
    pub fn new(api_key: String, secret_key: String, paper: bool) -> ValeResult<Self> {
        let _paper = paper;
        let data_url = "https://data.alpaca.markets".to_string();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ValeError::Data(e.to_string()))?;
        Ok(Self {
            client,
            api_key,
            secret_key,
            data_url,
        })
    }

    fn timeframe(resolution: Resolution) -> &'static str {
        match resolution {
            Resolution::Minute => "1Min",
            Resolution::Hour => "1Hour",
            Resolution::Daily => "1Day",
            Resolution::Weekly => "1Week",
            Resolution::Monthly => "1Month",
            _ => "1Day",
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        let _ = headers.insert(
            "APCA-API-KEY-ID",
            self.api_key.parse().unwrap_or(reqwest::header::HeaderValue::from_static("")),
        );
        let _ = headers.insert(
            "APCA-API-SECRET-KEY",
            self.secret_key
                .parse()
                .unwrap_or(reqwest::header::HeaderValue::from_static("")),
        );
        headers
    }
}

#[async_trait]
impl DataProvider for AlpacaMarketProvider {
    fn name(&self) -> &'static str {
        "alpaca"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        resolution: Resolution,
        range: &TimeRange,
    ) -> ValeResult<Vec<Bar>> {
        let tf = Self::timeframe(resolution);
        let start = range.start.to_rfc3339();
        let end = range.end.to_rfc3339();
        let mut url = format!(
            "{}/v2/stocks/{symbol}/bars?timeframe={tf}&start={start}&end={end}&limit=10000",
            self.data_url
        );

        let mut all = Vec::new();
        loop {
            let resp = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| ValeError::Http(e.to_string()))?;

            if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            if !resp.status().is_success() {
                return Err(ValeError::Http(format!("alpaca data HTTP {}", resp.status())));
            }

            let parsed: AlpacaBarsResponse = resp
                .json()
                .await
                .map_err(|e| ValeError::Parse(e.to_string()))?;

            if let Some(bars) = parsed.bars {
                for b in bars {
                    all.push(Bar {
                        timestamp: b.t,
                        open: b.o,
                        high: b.h,
                        low: b.l,
                        close: b.c,
                        volume: b.v as f64,
                        symbol: symbol.to_string(),
                    });
                }
            }

            match parsed.next_page_token {
                Some(token) => {
                    url = format!(
                        "{}/v2/stocks/{symbol}/bars?timeframe={tf}&start={start}&end={end}&limit=10000&page_token={token}",
                        self.data_url
                    );
                }
                None => break,
            }
        }

        all.sort_by_key(|b| b.timestamp);
        Ok(all)
    }

    async fn ping(&self) -> ValeResult<()> {
        let url = format!("{}/v2/stocks/SPY/bars?timeframe=1Day&limit=1", self.data_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| ValeError::Http(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ValeError::Http(format!("alpaca ping: {}", resp.status())))
        }
    }
}
