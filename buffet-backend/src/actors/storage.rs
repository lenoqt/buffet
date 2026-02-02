use crate::actors::messages::{ActorError, ActorResult, TimeSeriesRef};
use crate::models::market_data::OHLCV;
use crate::tsdb::TimescaleDb;
use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};
use sqlx::{Pool, Postgres};
use std::convert::Infallible;

pub struct TimeSeriesStorageActor {
    tsdb: TimescaleDb,
}

impl TimeSeriesStorageActor {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            tsdb: TimescaleDb::new(pool),
        }
    }
}

impl Actor for TimeSeriesStorageActor {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(state: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(state)
    }

    fn name() -> &'static str {
        "TimeSeriesStorageActor"
    }
}

#[derive(Debug, Clone)]
pub struct StoreOHLCV {
    pub symbol: String,
    pub asset_type: String,
    pub data: Vec<OHLCV>,
}

impl Message<StoreOHLCV> for TimeSeriesStorageActor {
    type Reply = ActorResult<()>;

    async fn handle(
        &mut self,
        msg: StoreOHLCV,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.tsdb
            .insert_ohlcv(&msg.symbol, &msg.asset_type, &msg.data)
            .await
            .map_err(|e| ActorError::TsdbError(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct QueryOHLCV {
    pub symbol: String,
    pub ts_ref: TimeSeriesRef,
}

impl Message<QueryOHLCV> for TimeSeriesStorageActor {
    type Reply = ActorResult<Vec<OHLCV>>;

    async fn handle(
        &mut self,
        msg: QueryOHLCV,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.tsdb
            .query_ohlcv(&msg.symbol, msg.ts_ref.start_time, msg.ts_ref.end_time)
            .await
            .map_err(|e| ActorError::TsdbError(e.to_string()))
    }
}
