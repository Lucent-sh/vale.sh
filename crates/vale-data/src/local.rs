use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use std::path::Path;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{Bar, Resolution, TimeRange};

use crate::provider::DataProvider;

pub struct LocalCsvProvider {
    path: std::path::PathBuf,
}

impl LocalCsvProvider {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn parse_timestamp(s: &str) -> ValeResult<DateTime<Utc>> {
        if let Ok(ts) = s.parse::<i64>() {
            if ts > 1_000_000_000_000 {
                return Utc
                    .timestamp_millis_opt(ts)
                    .single()
                    .ok_or_else(|| ValeError::Parse(format!("invalid unix ms: {s}")));
            }
            return Utc
                .timestamp_opt(ts, 0)
                .single()
                .ok_or_else(|| ValeError::Parse(format!("invalid unix: {s}")));
        }
        if let Ok(nd) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Ok(nd.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| ValeError::Parse(format!("timestamp parse: {e}")))
    }

    pub fn read_bars(&self, symbol: &str) -> ValeResult<Vec<Bar>> {
        let mut rdr = csv::Reader::from_path(&self.path).map_err(|e| ValeError::Io(e.into()))?;
        let headers = rdr.headers().map_err(|e| ValeError::Parse(e.to_string()))?;
        let headers: Vec<String> = headers.iter().map(|s| s.to_lowercase()).collect();

        let idx_ts = headers
            .iter()
            .position(|h| h == "timestamp" || h == "date" || h == "time");
        let idx_o = headers.iter().position(|h| h == "open");
        let idx_h = headers.iter().position(|h| h == "high");
        let idx_l = headers.iter().position(|h| h == "low");
        let idx_c = headers.iter().position(|h| h == "close");
        let idx_v = headers.iter().position(|h| h == "volume");

        let mut bars = Vec::new();
        for result in rdr.records() {
            let record = result.map_err(|e| ValeError::Parse(e.to_string()))?;
            let ts_idx =
                idx_ts.ok_or_else(|| ValeError::Parse("missing timestamp column".into()))?;
            let timestamp = Self::parse_timestamp(record.get(ts_idx).unwrap_or(""))?;
            let parse_f = |idx: Option<usize>| -> f64 {
                idx.and_then(|i| record.get(i))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0)
            };
            let close = parse_f(idx_c);
            bars.push(Bar {
                timestamp,
                open: parse_f(idx_o).max(if close > 0.0 { close } else { parse_f(idx_o) }),
                high: parse_f(idx_h),
                low: parse_f(idx_l),
                close,
                volume: parse_f(idx_v),
                symbol: symbol.to_string(),
            });
        }
        bars.sort_by_key(|b| b.timestamp);
        Ok(bars)
    }
}

#[async_trait]
impl DataProvider for LocalCsvProvider {
    fn name(&self) -> &'static str {
        "local"
    }

    fn requires_auth(&self) -> bool {
        false
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        _resolution: Resolution,
        range: &TimeRange,
    ) -> ValeResult<Vec<Bar>> {
        let bars = self.read_bars(symbol)?;
        Ok(bars
            .into_iter()
            .filter(|b| b.timestamp >= range.start && b.timestamp <= range.end)
            .collect())
    }

    async fn ping(&self) -> ValeResult<()> {
        if self.path.exists() {
            Ok(())
        } else {
            Err(ValeError::Data(format!(
                "file not found: {}",
                self.path.display()
            )))
        }
    }
}
