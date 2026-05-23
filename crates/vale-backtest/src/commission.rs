pub trait CommissionModel: Send + Sync {
    fn calculate(&self, quantity: f64, price: f64) -> f64;
}

pub struct PercentageCommission {
    pub rate: f64,
}

pub struct FlatCommission {
    pub per_trade: f64,
}

pub struct PerShareCommission {
    pub per_share: f64,
    pub min: f64,
}

impl CommissionModel for PercentageCommission {
    fn calculate(&self, quantity: f64, price: f64) -> f64 {
        quantity.abs() * price * self.rate
    }
}

impl CommissionModel for FlatCommission {
    fn calculate(&self, _quantity: f64, _price: f64) -> f64 {
        self.per_trade
    }
}

impl CommissionModel for PerShareCommission {
    fn calculate(&self, quantity: f64, _price: f64) -> f64 {
        (quantity.abs() * self.per_share).max(self.min)
    }
}
