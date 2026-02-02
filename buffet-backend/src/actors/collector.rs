use crate::actors::messages::{ActorError, ActorResult};
use crate::actors::storage::StoreOHLCV;
use crate::models::market_data::OHLCV;
use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use tracing::info;

#[derive(Actor)]
#[actor(name = "DataCollectorActor")]
pub struct DataCollectorActor {
    storage_ref: ActorRef<crate::actors::storage::TimeSeriesStorageActor>,
}

impl DataCollectorActor {
    pub fn new(storage_ref: ActorRef<crate::actors::storage::TimeSeriesStorageActor>) -> Self {
        Self { storage_ref }
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
        info!("Collecting data for symbol: {}", msg.symbol);

        // Mock data for now
        let mock_data = vec![OHLCV {
            timestamp: chrono::Utc::now(),
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
        }];

        self.storage_ref
            .ask(StoreOHLCV {
                symbol: msg.symbol.clone(),
                asset_type: "crypto".to_string(), // Default for mock
                data: mock_data,
            })
            .await
            .map_err(|e| ActorError::Internal(e.to_string()))
    }
}
