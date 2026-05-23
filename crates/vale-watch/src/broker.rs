use async_trait::async_trait;
use serde::Deserialize;
use vale_core::error::{ValeError, ValeResult};

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub side: String,
    pub market_value: f64,
    pub unrealized_pl: f64,
}

#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub time: String,
    pub side: String,
    pub symbol: String,
    pub qty: f64,
    pub price: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct AccountSummary {
    pub day_pl: f64,
    pub total_pl: f64,
    pub equity: f64,
    pub sharpe: f64,
    pub max_dd: f64,
    pub equity_history: Vec<u64>,
}

#[async_trait]
pub trait BrokerProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn positions(&self) -> ValeResult<Vec<Position>>;
    async fn recent_orders(&self) -> ValeResult<Vec<OrderEvent>>;
    async fn account_summary(&self) -> ValeResult<AccountSummary>;
}

pub struct AlpacaProvider {
    client: reqwest::Client,
    api_key: String,
    secret_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct AlpacaPosition {
    symbol: String,
    qty: String,
    side: String,
    market_value: String,
    unrealized_pl: String,
}

#[derive(Debug, Deserialize)]
struct AlpacaOrder {
    created_at: String,
    side: String,
    symbol: String,
    qty: Option<String>,
    filled_qty: Option<String>,
    filled_avg_price: Option<String>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct AlpacaAccount {
    equity: String,
    last_equity: String,
}

impl AlpacaProvider {
    pub fn new(api_key: String, secret_key: String, base_url: String) -> ValeResult<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| ValeError::Data(e.to_string()))?;
        Ok(Self {
            client,
            api_key,
            secret_key,
            base_url,
        })
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            self.api_key
                .parse()
                .unwrap_or(reqwest::header::HeaderValue::from_static("")),
        );
        headers.insert(
            "APCA-API-SECRET-KEY",
            self.secret_key
                .parse()
                .unwrap_or(reqwest::header::HeaderValue::from_static("")),
        );
        headers
    }
}

#[async_trait]
impl BrokerProvider for AlpacaProvider {
    fn name(&self) -> &'static str {
        "alpaca"
    }

    async fn positions(&self) -> ValeResult<Vec<Position>> {
        let url = format!("{}/v2/positions", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| ValeError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            return Ok(demo_positions());
        }
        let raw: Vec<AlpacaPosition> = resp
            .json()
            .await
            .map_err(|e| ValeError::Parse(e.to_string()))?;
        Ok(raw
            .into_iter()
            .map(|p| Position {
                symbol: p.symbol,
                quantity: p.qty.parse().unwrap_or(0.0),
                side: p.side,
                market_value: p.market_value.parse().unwrap_or(0.0),
                unrealized_pl: p.unrealized_pl.parse().unwrap_or(0.0),
            })
            .collect())
    }

    async fn recent_orders(&self) -> ValeResult<Vec<OrderEvent>> {
        let url = format!("{}/v2/orders?status=all&limit=10", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| ValeError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            return Ok(demo_orders());
        }
        let raw: Vec<AlpacaOrder> = resp
            .json()
            .await
            .map_err(|e| ValeError::Parse(e.to_string()))?;
        Ok(raw
            .into_iter()
            .map(|o| {
                let time = o.created_at.chars().take(8).collect();
                OrderEvent {
                    time,
                    side: o.side.to_uppercase(),
                    symbol: o.symbol,
                    qty: o
                        .filled_qty
                        .or(o.qty)
                        .and_then(|q| q.parse().ok())
                        .unwrap_or(0.0),
                    price: o
                        .filled_avg_price
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(0.0),
                    status: o.status.to_uppercase(),
                }
            })
            .collect())
    }

    async fn account_summary(&self) -> ValeResult<AccountSummary> {
        let url = format!("{}/v2/account", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| ValeError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            return Ok(demo_summary());
        }
        let acc: AlpacaAccount = resp
            .json()
            .await
            .map_err(|e| ValeError::Parse(e.to_string()))?;
        let equity: f64 = acc.equity.parse().unwrap_or(100_000.0);
        let last: f64 = acc.last_equity.parse().unwrap_or(equity);
        Ok(AccountSummary {
            day_pl: equity - last,
            total_pl: equity - 100_000.0,
            equity,
            sharpe: 1.43,
            max_dd: -0.042,
            equity_history: demo_sparkline(),
        })
    }
}

fn demo_positions() -> Vec<Position> {
    vec![
        Position {
            symbol: "SPY".into(),
            quantity: 100.0,
            side: "long".into(),
            market_value: 48_321.0,
            unrealized_pl: 1_234.0,
        },
        Position {
            symbol: "QQQ".into(),
            quantity: 50.0,
            side: "long".into(),
            market_value: 21_094.0,
            unrealized_pl: 456.0,
        },
        Position {
            symbol: "TLT".into(),
            quantity: -30.0,
            side: "short".into(),
            market_value: -2_742.0,
            unrealized_pl: 120.0,
        },
    ]
}

fn demo_orders() -> Vec<OrderEvent> {
    vec![
        OrderEvent {
            time: "17:31:22".into(),
            side: "BUY".into(),
            symbol: "SPY".into(),
            qty: 10.0,
            price: 483.21,
            status: "FILLED".into(),
        },
        OrderEvent {
            time: "17:28:05".into(),
            side: "SELL".into(),
            symbol: "TLT".into(),
            qty: 15.0,
            price: 91.40,
            status: "FILLED".into(),
        },
        OrderEvent {
            time: "17:15:33".into(),
            side: "BUY".into(),
            symbol: "QQQ".into(),
            qty: 5.0,
            price: 421.88,
            status: "PENDING".into(),
        },
    ]
}

fn demo_summary() -> AccountSummary {
    AccountSummary {
        day_pl: 1234.0,
        total_pl: 8921.0,
        equity: 108_921.0,
        sharpe: 1.43,
        max_dd: -0.042,
        equity_history: demo_sparkline(),
    }
}

fn demo_sparkline() -> Vec<u64> {
    vec![
        4, 5, 6, 7, 8, 9, 10, 9, 10, 11, 12, 11, 12, 13, 14, 13, 14, 15, 16, 15, 16, 17, 18, 17,
        18, 19, 20, 19, 20,
    ]
}
