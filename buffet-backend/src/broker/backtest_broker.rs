use crate::broker::{Broker, BrokerError, FillResult};
use crate::models::order::OrderSide;
use async_trait::async_trait;

/// A broker implementation for backtesting that applies configurable
/// slippage and commission to every fill.
pub struct BacktestBroker {
    /// Commission rate as a fraction of trade value (e.g. 0.001 = 0.1%)
    pub commission_rate: f64,
    /// Slippage in basis points (e.g. 10.0 = 0.1%)
    pub slippage_bps: f64,
    /// The current market price — must be set via `set_price` before each fill
    pub current_price: f64,
}

impl BacktestBroker {
    pub fn new(commission_rate: f64, slippage_bps: f64) -> Self {
        Self {
            commission_rate,
            slippage_bps,
            current_price: 0.0,
        }
    }

    /// Update the current market price before submitting an order.
    pub fn set_price(&mut self, price: f64) {
        self.current_price = price;
    }

    /// Apply slippage to a base price based on order side.
    ///
    /// - Buys fill slightly *higher* (adverse for the buyer).
    /// - Sells fill slightly *lower* (adverse for the seller).
    pub fn apply_slippage(&self, base: f64, side: &OrderSide) -> f64 {
        let factor = self.slippage_bps / 10_000.0;
        match side {
            OrderSide::Buy => base * (1.0 + factor),
            OrderSide::Sell => base * (1.0 - factor),
        }
    }

    /// Return the commission cost for a given trade value.
    pub fn apply_commission(&self, value: f64) -> f64 {
        value * self.commission_rate
    }
}

#[async_trait]
impl Broker for BacktestBroker {
    async fn submit_market_order(
        &self,
        _symbol: &str,
        side: &OrderSide,
        quantity: f64,
    ) -> Result<FillResult, BrokerError> {
        let fill_price = self.apply_slippage(self.current_price, side);
        let fill_value = fill_price * quantity;
        let commission = self.apply_commission(fill_value);

        tracing::debug!(
            "BacktestBroker: market {:?} {:.6} @ {:.4} (slippage applied), commission={:.4}",
            side,
            quantity,
            fill_price,
            commission
        );

        Ok(FillResult {
            fill_price,
            fill_quantity: quantity,
            filled: true,
            rejection_reason: None,
            commission: Some(commission),
        })
    }

    /// Limit orders in backtesting are treated the same as market orders —
    /// they fill immediately at the current price with slippage applied.
    async fn submit_limit_order(
        &self,
        _symbol: &str,
        side: &OrderSide,
        quantity: f64,
        _limit_price: f64,
    ) -> Result<FillResult, BrokerError> {
        let fill_price = self.apply_slippage(self.current_price, side);
        let fill_value = fill_price * quantity;
        let commission = self.apply_commission(fill_value);

        tracing::debug!(
            "BacktestBroker: limit {:?} {:.6} @ {:.4} (treated as market), commission={:.4}",
            side,
            quantity,
            fill_price,
            commission
        );

        Ok(FillResult {
            fill_price,
            fill_quantity: quantity,
            filled: true,
            rejection_reason: None,
            commission: Some(commission),
        })
    }

    fn name(&self) -> &str {
        "BacktestBroker"
    }
}
