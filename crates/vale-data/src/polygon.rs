use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{Bar, Resolution, TimeRange};

use crate::provider::DataProvider;

pub struct PolygonProvider {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct PolygonAggResponse {
    results: Option<Vec<PolygonBar>>,
    next_url: Option<String>,
    #[allow(dead_code)]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PolygonBar {
    t: i64,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
}

impl PolygonProvider {
    pub fn new(api_key: String) -> ValeResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ValeError::Data(e.to_string()))?;
        Ok(Self { client, api_key })
    }

    fn resolution_params(r: Resolution) -> (&'static str, &'static str) {
        match r {
            Resolution::Minute => ("1", "minute"),
            Resolution::Hour => ("1", "hour"),
            Resolution::Daily => ("1", "day"),
            Resolution::Weekly => ("1", "week"),
            Resolution::Monthly => ("1", "month"),
            _ => ("1", "day"),
        }
    }

    fn format_date(dt: DateTime<Utc>) -> String {
        dt.format("%Y-%m-%d").to_string()
    }
}

#[async_trait]
impl DataProvider for PolygonProvider {
    fn name(&self) -> &'static str {
        "polygon"
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
        let (mult, span) = Self::resolution_params(resolution);
        let from = Self::format_date(range.start);
        let to = Self::format_date(range.end);
        let mut url = format!(
            "https://api.polygon.io/v2/aggs/ticker/{symbol}/range/{mult}/{span}/{from}/{to}?adjusted=true&sort=asc&limit=50000&apiKey={}",
            self.api_key
        );

        let mut all_bars = Vec::new();
        loop {
            let resp = self.client.get(&url).send().await.map_err(|e| {
                ValeError::Http(format!(
                    "could not reach polygon: {e}. Check your connection or run `vale doctor`."
                ))
            })?;

            if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let wait = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2);
                tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                continue;
            }

            if let Some(remaining) = resp.headers().get("x-ratelimit-remaining") {
                if let Ok(s) = remaining.to_str() {
                    if s == "0" {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
            }

            if !resp.status().is_success() {
                return Err(ValeError::Http(format!(
                    "could not reach polygon: HTTP {}. Check your connection or run `vale doctor`.",
                    resp.status()
                )));
            }

            let body = resp
                .text()
                .await
                .map_err(|e| ValeError::Http(e.to_string()))?;
            let parsed: PolygonAggResponse =
                serde_json::from_str(&body).map_err(|e| ValeError::Parse(e.to_string()))?;

            if let Some(results) = parsed.results {
                for b in results {
                    let timestamp = Utc
                        .timestamp_millis_opt(b.t)
                        .single()
                        .ok_or_else(|| ValeError::Parse("invalid polygon timestamp".into()))?;
                    all_bars.push(Bar {
                        timestamp,
                        open: b.o,
                        high: b.h,
                        low: b.l,
                        close: b.c,
                        volume: b.v,
                        symbol: symbol.to_string(),
                    });
                }
            }

            match parsed.next_url {
                Some(next) => {
                    url = if next.contains("apiKey=") {
                        next
                    } else {
                        format!("{next}&apiKey={}", self.api_key)
                    };
                }
                None => break,
            }
        }

        all_bars.sort_by_key(|b| b.timestamp);
        Ok(all_bars)
    }

    async fn ping(&self) -> ValeResult<()> {
        let url = format!(
            "https://api.polygon.io/v2/aggs/ticker/SPY/range/1/day/2024-01-01/2024-01-02?apiKey={}",
            self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| {
            ValeError::Http(format!(
                "could not reach polygon: {e}. Check your connection or run `vale doctor`."
            ))
        })?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ValeError::Http(format!(
                "could not reach polygon: HTTP {}. Check your connection or run `vale doctor`.",
                resp.status()
            )))
        }
    }
}
