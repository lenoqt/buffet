use crate::actors::messages::{ActorError, ActorResult, CollectHistorical, MarketDataUpdate};
use crate::actors::storage::StoreOHLCV;
use crate::providers::normalize::normalize_ohlcv;
use crate::providers::{MarketDataProvider, YahooProvider};
use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use tracing::{error, info, warn};

#[derive(Actor)]
#[actor(name = "DataCollectorActor")]
pub struct DataCollectorActor {
    storage_ref: ActorRef<crate::actors::storage::TimeSeriesStorageActor>,
    strategy_ref: ActorRef<crate::actors::strategy::StrategyExecutorActor>,
    provider: YahooProvider,
}

impl DataCollectorActor {
    pub fn new(
        storage_ref: ActorRef<crate::actors::storage::TimeSeriesStorageActor>,
        strategy_ref: ActorRef<crate::actors::strategy::StrategyExecutorActor>,
    ) -> Self {
        Self {
            storage_ref,
            strategy_ref,
            provider: YahooProvider::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectData {
    pub symbol: String,
}

impl Message<CollectData> for DataCollectorActor {
    type Reply = ActorResult<()>;

    async fn handle(
        &mut self,
        msg: CollectData,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!(symbol = %msg.symbol, "Collecting last 30 days of OHLCV data");

        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(30);

        let raw = self
            .provider
            .fetch_ohlcv(&msg.symbol, start, end)
            .await
            .map_err(|e| {
                error!(symbol = %msg.symbol, error = %e, "Provider fetch failed");
                ActorError::Internal(e.to_string())
            })?;

        let data = normalize_ohlcv(raw).map_err(|e| {
            error!(symbol = %msg.symbol, error = %e, "OHLCV normalization failed");
            ActorError::Internal(e.to_string())
        })?;

        info!(symbol = %msg.symbol, count = data.len(), "Fetched and normalized OHLCV records");

        self.storage_ref
            .ask(StoreOHLCV {
                symbol: msg.symbol.clone(),
                asset_type: "stock".to_string(),
                data: data.clone(),
            })
            .await
            .map_err(|e| ActorError::Internal(e.to_string()))?;

        info!(symbol = %msg.symbol, "Stored OHLCV data; forwarding to strategy executor");

        for point in data {
            if let Err(e) = self
                .strategy_ref
                .tell(MarketDataUpdate {
                    symbol: msg.symbol.clone(),
                    data: point,
                })
                .send()
                .await
            {
                warn!(symbol = %msg.symbol, error = %e, "Failed to forward data point to strategy executor");
            }
        }

        Ok(())
    }
}

impl Message<CollectHistorical> for DataCollectorActor {
    type Reply = ActorResult<()>;

    async fn handle(
        &mut self,
        msg: CollectHistorical,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!(
            symbol = %msg.symbol,
            asset_type = %msg.asset_type,
            start = %msg.start,
            end = %msg.end,
            "Collecting historical OHLCV data"
        );

        let raw = self
            .provider
            .fetch_ohlcv(&msg.symbol, msg.start, msg.end)
            .await
            .map_err(|e| {
                error!(symbol = %msg.symbol, error = %e, "Provider fetch failed for historical range");
                ActorError::Internal(e.to_string())
            })?;

        let data = normalize_ohlcv(raw).map_err(|e| {
            error!(symbol = %msg.symbol, error = %e, "OHLCV normalization failed");
            ActorError::Internal(e.to_string())
        })?;

        info!(
            symbol = %msg.symbol,
            count = data.len(),
            "Fetched and normalized historical OHLCV records"
        );

        self.storage_ref
            .ask(StoreOHLCV {
                symbol: msg.symbol.clone(),
                asset_type: msg.asset_type.clone(),
                data: data.clone(),
            })
            .await
            .map_err(|e| ActorError::Internal(e.to_string()))?;

        info!(symbol = %msg.symbol, "Stored historical OHLCV data; forwarding to strategy executor");

        for point in data {
            if let Err(e) = self
                .strategy_ref
                .tell(MarketDataUpdate {
                    symbol: msg.symbol.clone(),
                    data: point,
                })
                .send()
                .await
            {
                warn!(symbol = %msg.symbol, error = %e, "Failed to forward historical data point to strategy executor");
            }
        }

        Ok(())
    }
}
