use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use serde::Deserialize;
use std::time::Duration;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{Bar, Resolution, TimeRange};

use crate::provider::DataProvider;

pub struct YahooProvider {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct YahooResult {
    timestamp: Option<Vec<i64>>,
    indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    open: Option<Vec<Option<f64>>>,
    high: Option<Vec<Option<f64>>>,
    low: Option<Vec<Option<f64>>>,
    close: Option<Vec<Option<f64>>>,
    volume: Option<Vec<Option<f64>>>,
}

impl YahooProvider {
    pub fn new(timeout_secs: u64) -> ValeResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Vale/0.1)")
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| ValeError::Data(e.to_string()))?;
        Ok(Self {
            client,
            base_url: "https://query1.finance.yahoo.com".into(),
        })
    }

    fn resolution_to_interval(r: Resolution) -> &'static str {
        match r {
            Resolution::Minute => "1m",
            Resolution::Hour => "1h",
            Resolution::Daily => "1d",
            Resolution::Weekly => "1wk",
            Resolution::Monthly => "1mo",
            _ => "1d",
        }
    }

    async fn fetch_with_retry(&self, url: &str) -> ValeResult<reqwest::Response> {
        let backoffs = [500u64, 1000, 2000];
        let mut last_err = String::new();
        for (attempt, &ms) in backoffs.iter().enumerate() {
            match self.client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(resp),
                Ok(resp) => {
                    last_err = format!("HTTP {}", resp.status());
                }
                Err(e) => {
                    last_err = e.to_string();
                }
            }
            if attempt + 1 < backoffs.len() {
                tokio::time::sleep(Duration::from_millis(ms)).await;
            }
        }
        Err(ValeError::Http(format!(
            "could not reach yahoo: {last_err}. Check your connection or run `vale doctor`."
        )))
    }

    fn parse_response(symbol: &str, body: &str) -> ValeResult<Vec<Bar>> {
        let parsed: YahooChartResponse =
            serde_json::from_str(body).map_err(|e| ValeError::Parse(format!("yahoo JSON: {e}")))?;

        if let Some(err) = parsed.chart.error {
            return Err(ValeError::Data(format!("yahoo API error: {err}")));
        }

        let result = parsed
            .chart
            .result
            .and_then(|mut r| r.pop())
            .ok_or_else(|| ValeError::Data("yahoo returned no data".into()))?;

        let timestamps = result
            .timestamp
            .ok_or_else(|| ValeError::Data("yahoo missing timestamps".into()))?;
        let quote = result
            .indicators
            .quote
            .into_iter()
            .next()
            .ok_or_else(|| ValeError::Data("yahoo missing quote".into()))?;

        let opens = quote.open.unwrap_or_default();
        let highs = quote.high.unwrap_or_default();
        let lows = quote.low.unwrap_or_default();
        let closes = quote.close.unwrap_or_default();
        let volumes = quote.volume.unwrap_or_default();

        let mut bars = Vec::new();
        for (i, &ts) in timestamps.iter().enumerate() {
            let close = closes.get(i).and_then(|o| *o);
            let Some(close) = close else { continue };
            let timestamp = Utc
                .timestamp_opt(ts, 0)
                .single()
                .ok_or_else(|| ValeError::Parse("invalid timestamp".into()))?;
            bars.push(Bar {
                timestamp,
                open: opens.get(i).and_then(|o| *o).unwrap_or(close),
                high: highs.get(i).and_then(|o| *o).unwrap_or(close),
                low: lows.get(i).and_then(|o| *o).unwrap_or(close),
                close,
                volume: volumes.get(i).and_then(|o| *o).unwrap_or(0.0),
                symbol: symbol.to_string(),
            });
        }
        bars.sort_by_key(|b| b.timestamp);
        Ok(bars)
    }
}

#[async_trait]
impl DataProvider for YahooProvider {
    fn name(&self) -> &'static str {
        "yahoo"
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
        let interval = Self::resolution_to_interval(resolution);
        let period1 = range.start.timestamp();
        let period2 = range.end.timestamp();
        let url = format!(
            "{}/v8/finance/chart/{symbol}?interval={interval}&period1={period1}&period2={period2}",
            self.base_url
        );
        let resp = self.fetch_with_retry(&url).await?;
        let body = resp
            .text()
            .await
            .map_err(|e| ValeError::Http(e.to_string()))?;
        Self::parse_response(symbol, &body)
    }

    async fn ping(&self) -> ValeResult<()> {
        let url = format!(
            "{}/v8/finance/chart/SPY?interval=1d&range=1d",
            self.base_url
        );
        self.fetch_with_retry(&url).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use httpmock::prelude::*;

    #[tokio::test]
    async fn yahoo_parses_mock_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path_contains("/v8/finance/chart/SPY");
            then.status(200).body(
                r#"{
                "chart": {
                    "result": [{
                        "timestamp": [1609459200, 1609545600],
                        "indicators": {
                            "quote": [{
                                "open": [375.0, 376.0],
                                "high": [380.0, 381.0],
                                "low": [374.0, 375.0],
                                "close": [378.0, 379.0],
                                "volume": [1000000.0, 1100000.0]
                            }]
                        }
                    }]
                }
            }"#,
            );
        });

        let client = reqwest::Client::builder().build().expect("client");
        let provider = YahooProvider {
            client,
            base_url: server.base_url(),
        };
        let range = TimeRange {
            start: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2021, 1, 2, 0, 0, 0).unwrap(),
        };
        let bars = provider
            .fetch_ohlcv("SPY", Resolution::Daily, &range)
            .await
            .expect("fetch");
        mock.assert();
        assert_eq!(bars.len(), 2);
        assert!((bars[0].close - 378.0).abs() < 1e-6);
    }
}
