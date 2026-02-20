use crate::actors::messages::{ActorError, ActorResult, OrderRequest};
use crate::models::order::Order;
use kameo::Actor;
use kameo::message::{Context, Message};
use sqlx::{Pool, Sqlite};
use tracing::info;

#[derive(Actor)]
#[actor(name = "OrderExecutionActor")]
pub struct OrderExecutionActor {
    pool: Pool<Sqlite>,
}

impl OrderExecutionActor {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

impl Message<OrderRequest> for OrderExecutionActor {
    type Reply = ActorResult<Order>;

    async fn handle(
        &mut self,
        msg: OrderRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("Received order request for signal: {}", msg.signal_id);

        // 1. Create Open Order
        let mut order = Order::create(
            Some(msg.signal_id.clone()),
            &msg.symbol,
            msg.side.clone(),
            msg.quantity,
            msg.price,
            &self.pool,
        )
        .await
        .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        info!("Order created: {} ({})", order.id, order.status);

        // 2. Mock Execution (Fill immediately for now)
        // In real system, this would go to a broker API
        // Simulate some latency?

        order = Order::update_status(
            &order.id,
            crate::models::order::OrderStatus::Filled,
            &self.pool,
        )
        .await
        .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

        info!("Order filled: {}", order.id);

        Ok(order)
    }
}
