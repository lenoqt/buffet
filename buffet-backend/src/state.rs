use kameo::actor::ActorRef;
use sqlx::{Pool, Postgres, Sqlite};

use crate::actors::{DataCollectorActor, OrderExecutionActor, StrategyExecutorActor};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub tsdb: Pool<Postgres>,
    pub collector: ActorRef<DataCollectorActor>,
    pub executor: ActorRef<StrategyExecutorActor>,
    pub execution: ActorRef<OrderExecutionActor>,
}

impl AppState {
    pub fn new(
        db: Pool<Sqlite>,
        tsdb: Pool<Postgres>,
        collector: ActorRef<DataCollectorActor>,
        executor: ActorRef<StrategyExecutorActor>,
        execution: ActorRef<OrderExecutionActor>,
    ) -> Self {
        Self {
            db,
            tsdb,
            collector,
            executor,
            execution,
        }
    }
}
