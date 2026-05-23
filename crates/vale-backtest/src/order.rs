use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub quantity: f64,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub is_buy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub symbol: String,
    pub quantity: f64,
    pub price: f64,
    pub commission: f64,
    pub is_buy: bool,
}
