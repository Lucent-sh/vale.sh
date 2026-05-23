use crate::commission::CommissionModel;
use crate::context::{Context, DataWindow};
use crate::order::{Fill, OrderStatus, OrderType};
use crate::portfolio::{Portfolio, Position};
use crate::slippage::SlippageModel;
use crate::strategy::Strategy;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::{BacktestEngine as EngineKind, BacktestResult, Bar, Trade, TradeDirection};
use vale_risk::drawdown::max_drawdown;
use vale_risk::metrics::{
    cagr, calmar_ratio, log_returns, profit_factor, sharpe_ratio, sortino_ratio, volatility_annual,
};

pub struct BacktestEngine {
    pub commission: Box<dyn CommissionModel>,
    pub slippage: Box<dyn SlippageModel>,
    pub initial_cash: f64,
}

impl BacktestEngine {
    pub fn run(&self, strategy: &mut dyn Strategy, bars: &[Bar]) -> ValeResult<BacktestResult> {
        if bars.is_empty() {
            return Err(ValeError::Backtest("no bars provided".into()));
        }

        let mut sorted: Vec<Bar> = bars.to_vec();
        sorted.sort_by_key(|b| b.timestamp);

        let mut portfolio = Portfolio::new(self.initial_cash);
        let mut equity_curve: Vec<(DateTime<Utc>, f64)> = Vec::new();
        let mut open_entries: HashMap<String, (DateTime<Utc>, f64, f64, TradeDirection)> =
            HashMap::new();

        {
            let mut ctx = Context {
                portfolio: &mut portfolio,
                data: DataWindow {
                    bars: sorted.clone(),
                    index: 0,
                },
                orders: Vec::new(),
                prices: HashMap::new(),
            };
            strategy.on_start(&mut ctx);
        }

        for i in 0..sorted.len() {
            let bar = sorted[i].clone();
            let orders;
            {
                let mut ctx = Context {
                    portfolio: &mut portfolio,
                    data: DataWindow {
                        bars: sorted.clone(),
                        index: i,
                    },
                    orders: Vec::new(),
                    prices: HashMap::from([(bar.symbol.clone(), bar.close)]),
                };
                strategy.on_bar(&mut ctx, &bar);
                orders = std::mem::take(&mut ctx.orders);
            }

            for order in orders {
                if order.status == OrderStatus::Cancelled {
                    continue;
                }
                let fill_price = match order.order_type {
                    OrderType::Market => {
                        self.slippage.apply(bar.close, order.quantity, order.is_buy)
                    }
                    OrderType::Limit { price } => {
                        self.slippage.apply(price, order.quantity, order.is_buy)
                    }
                };
                let commission = self.commission.calculate(order.quantity, fill_price);
                Self::apply_fill(
                    &mut portfolio,
                    &mut open_entries,
                    Fill {
                        symbol: order.symbol.clone(),
                        quantity: order.quantity,
                        price: fill_price,
                        commission,
                        is_buy: order.is_buy,
                    },
                    bar.timestamp,
                );
            }

            let equity = portfolio.mark_to_market(&bar);
            equity_curve.push((bar.timestamp, equity));
        }

        {
            let mut ctx = Context {
                portfolio: &mut portfolio,
                data: DataWindow {
                    bars: sorted.clone(),
                    index: sorted.len().saturating_sub(1),
                },
                orders: Vec::new(),
                prices: HashMap::new(),
            };
            strategy.on_end(&mut ctx);
        }

        let start = equity_curve
            .first()
            .map(|(t, _)| *t)
            .unwrap_or(sorted[0].timestamp);
        let end = equity_curve
            .last()
            .map(|(t, _)| *t)
            .unwrap_or(sorted[sorted.len() - 1].timestamp);
        let final_equity = equity_curve
            .last()
            .map(|(_, e)| *e)
            .unwrap_or(self.initial_cash);
        let total_return = (final_equity - self.initial_cash) / self.initial_cash;

        let years = (end - start).num_days().max(1) as f64 / 365.25;
        let equities: Vec<f64> = equity_curve.iter().map(|(_, e)| *e).collect();
        let returns = log_returns(&equities);
        let ann = 252.0_f64.sqrt();
        let rf_daily = 0.05 / 252.0;
        let sharpe = sharpe_ratio(&returns, rf_daily, ann);
        let sortino = sortino_ratio(&returns, rf_daily, ann);
        let max_dd = max_drawdown(&equities);
        let cagr_val = cagr(&equities, years);
        let calmar = calmar_ratio(cagr_val, max_dd);
        let vol = volatility_annual(&returns, ann);

        let pnls: Vec<f64> = portfolio.trades.iter().map(|t| t.pnl).collect();
        let winning = portfolio.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing = portfolio.trades.iter().filter(|t| t.pnl < 0.0).count();
        let total_trades = portfolio.trades.len();
        let win_rate = if total_trades > 0 {
            winning as f64 / total_trades as f64
        } else {
            0.0
        };
        let pf = profit_factor(&pnls);
        let wins: Vec<f64> = pnls.iter().filter(|&&p| p > 0.0).copied().collect();
        let losses: Vec<f64> = pnls.iter().filter(|&&p| p < 0.0).copied().collect();
        let avg_win = if wins.is_empty() {
            0.0
        } else {
            wins.iter().sum::<f64>() / wins.len() as f64
        };
        let avg_loss = if losses.is_empty() {
            0.0
        } else {
            losses.iter().sum::<f64>() / losses.len() as f64
        };

        Ok(BacktestResult {
            id: Uuid::new_v4().to_string(),
            strategy_name: strategy.name().to_string(),
            engine: EngineKind::Native,
            start,
            end,
            initial_cash: self.initial_cash,
            final_equity,
            total_return,
            cagr: cagr_val,
            sharpe_ratio: sharpe,
            sortino_ratio: sortino,
            calmar_ratio: calmar,
            max_drawdown: max_dd,
            max_drawdown_duration_days: 0,
            volatility_annual: vol,
            total_trades,
            winning_trades: winning,
            losing_trades: losing,
            win_rate,
            profit_factor: pf,
            avg_win,
            avg_loss,
            equity_curve,
            benchmark_curve: None,
            trades: portfolio.trades,
            params: serde_json::json!({}),
        })
    }

    fn apply_fill(
        portfolio: &mut Portfolio,
        open_entries: &mut HashMap<String, (DateTime<Utc>, f64, f64, TradeDirection)>,
        fill: Fill,
        timestamp: DateTime<Utc>,
    ) {
        let cost = fill.quantity * fill.price;
        if fill.is_buy {
            portfolio.cash -= cost + fill.commission;
        } else {
            portfolio.cash += cost - fill.commission;
        }

        let pos = portfolio
            .positions
            .entry(fill.symbol.clone())
            .or_insert(Position {
                quantity: 0.0,
                avg_price: 0.0,
            });

        let prev_qty = pos.quantity;
        let new_qty = if fill.is_buy {
            prev_qty + fill.quantity
        } else {
            prev_qty - fill.quantity
        };

        if prev_qty == 0.0 {
            open_entries.insert(
                fill.symbol.clone(),
                (
                    timestamp,
                    fill.price,
                    fill.quantity.abs(),
                    if fill.is_buy {
                        TradeDirection::Long
                    } else {
                        TradeDirection::Short
                    },
                ),
            );
            pos.avg_price = fill.price;
            pos.quantity = new_qty;
        } else if (prev_qty > 0.0 && !fill.is_buy) || (prev_qty < 0.0 && fill.is_buy) {
            let close_qty = fill.quantity.abs().min(prev_qty.abs());
            if let Some((entry_time, entry_price, _qty, direction)) =
                open_entries.remove(&fill.symbol)
            {
                let exit_price = fill.price;
                let pnl = match direction {
                    TradeDirection::Long => (exit_price - entry_price) * close_qty,
                    TradeDirection::Short => (entry_price - exit_price) * close_qty,
                };
                let pnl_pct = if entry_price != 0.0 {
                    pnl / (entry_price * close_qty)
                } else {
                    0.0
                };
                portfolio.trades.push(Trade {
                    id: Uuid::new_v4().to_string(),
                    symbol: fill.symbol.clone(),
                    entry_time,
                    exit_time: timestamp,
                    entry_price,
                    exit_price,
                    quantity: close_qty,
                    direction,
                    pnl,
                    pnl_pct,
                    fees: fill.commission,
                });
            }
            pos.quantity = new_qty;
            if pos.quantity == 0.0 {
                pos.avg_price = 0.0;
            }
        } else {
            let total_cost = pos.avg_price * prev_qty.abs() + fill.price * fill.quantity;
            pos.quantity = new_qty;
            if pos.quantity != 0.0 {
                pos.avg_price = total_cost / pos.quantity.abs();
            }
        }
    }
}
