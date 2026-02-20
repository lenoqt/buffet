use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::models::order::OrderSide;

/// Result of a broker fill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillResult {
    /// The price at which the order was filled
    pub fill_price: f64,
    /// The quantity that was filled (may differ from requested in partial fills)
    pub fill_quantity: f64,
    /// Whether the order was fully filled
    pub filled: bool,
    /// Optional rejection reason
    pub rejection_reason: Option<String>,
}

/// Trait for broker implementations.
/// Allows swapping between paper trading, backtesting, and real brokers.
#[async_trait]
pub trait Broker: Send + Sync {
    /// Submit a market order and return the fill result
    async fn submit_market_order(
        &self,
        symbol: &str,
        side: &OrderSide,
        quantity: f64,
    ) -> Result<FillResult, BrokerError>;

    /// Submit a limit order and return the fill result
    async fn submit_limit_order(
        &self,
        symbol: &str,
        side: &OrderSide,
        quantity: f64,
        limit_price: f64,
    ) -> Result<FillResult, BrokerError>;

    /// Returns the broker name (for logging)
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerError {
    /// Order was rejected
    Rejected(String),
    /// Connection to broker failed
    ConnectionError(String),
    /// Insufficient funds / margin
    InsufficientFunds,
    /// Unknown error
    Internal(String),
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokerError::Rejected(msg) => write!(f, "Order rejected: {}", msg),
            BrokerError::ConnectionError(msg) => write!(f, "Broker connection error: {}", msg),
            BrokerError::InsufficientFunds => write!(f, "Insufficient funds"),
            BrokerError::Internal(msg) => write!(f, "Broker internal error: {}", msg),
        }
    }
}

impl std::error::Error for BrokerError {}

/// Paper trading broker that simulates fills at a configurable price
pub struct PaperBroker {
    /// Simulated slippage in basis points (e.g., 10 = 0.1%)
    slippage_bps: f64,
    /// Default fill price when no market price is available
    default_price: f64,
}

impl PaperBroker {
    pub fn new(slippage_bps: f64, default_price: f64) -> Self {
        Self {
            slippage_bps,
            default_price,
        }
    }

    /// Apply slippage to a base price depending on order side
    fn apply_slippage(&self, base_price: f64, side: &OrderSide) -> f64 {
        let slippage_factor = self.slippage_bps / 10_000.0;
        match side {
            // Buys fill slightly higher (worse for buyer)
            OrderSide::Buy => base_price * (1.0 + slippage_factor),
            // Sells fill slightly lower (worse for seller)
            OrderSide::Sell => base_price * (1.0 - slippage_factor),
        }
    }
}

impl Default for PaperBroker {
    fn default() -> Self {
        Self {
            slippage_bps: 10.0, // 0.1% slippage
            default_price: 100.0,
        }
    }
}

#[async_trait]
impl Broker for PaperBroker {
    async fn submit_market_order(
        &self,
        _symbol: &str,
        side: &OrderSide,
        quantity: f64,
    ) -> Result<FillResult, BrokerError> {
        // TODO: Use last known market price from a price cache
        // For now, use default_price with slippage
        let fill_price = self.apply_slippage(self.default_price, side);

        tracing::info!(
            "PaperBroker: Filled market {} order for {:.4} units @ {:.2}",
            side,
            quantity,
            fill_price
        );

        Ok(FillResult {
            fill_price,
            fill_quantity: quantity,
            filled: true,
            rejection_reason: None,
        })
    }

    async fn submit_limit_order(
        &self,
        _symbol: &str,
        side: &OrderSide,
        quantity: f64,
        limit_price: f64,
    ) -> Result<FillResult, BrokerError> {
        // Paper trading: limit orders fill at limit price (optimistic)
        let fill_price = self.apply_slippage(limit_price, side);

        tracing::info!(
            "PaperBroker: Filled limit {} order for {:.4} units @ {:.2} (limit: {:.2})",
            side,
            quantity,
            fill_price,
            limit_price
        );

        Ok(FillResult {
            fill_price,
            fill_quantity: quantity,
            filled: true,
            rejection_reason: None,
        })
    }

    fn name(&self) -> &str {
        "PaperBroker"
    }
}
