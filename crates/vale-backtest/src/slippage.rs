pub trait SlippageModel: Send + Sync {
    fn apply(&self, price: f64, quantity: f64, is_buy: bool) -> f64;
}

pub struct FixedSlippage {
    pub ticks: f64,
    pub tick_size: f64,
}

pub struct PercentageSlippage {
    pub rate: f64,
}

pub struct VolumeSlippage {
    pub rate: f64,
}

impl SlippageModel for FixedSlippage {
    fn apply(&self, price: f64, _quantity: f64, is_buy: bool) -> f64 {
        let slip = self.ticks * self.tick_size;
        if is_buy {
            price + slip
        } else {
            price - slip
        }
    }
}

impl SlippageModel for PercentageSlippage {
    fn apply(&self, price: f64, _quantity: f64, is_buy: bool) -> f64 {
        if is_buy {
            price * (1.0 + self.rate)
        } else {
            price * (1.0 - self.rate)
        }
    }
}

impl SlippageModel for VolumeSlippage {
    fn apply(&self, price: f64, quantity: f64, is_buy: bool) -> f64 {
        let slip = self.rate * quantity.abs();
        if is_buy {
            price + slip
        } else {
            price - slip
        }
    }
}
