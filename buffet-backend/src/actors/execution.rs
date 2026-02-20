use crate::actors::messages::{ActorError, ActorResult, OrderRequest};
use crate::broker::{Broker, PaperBroker};
use crate::models::order::{Order, OrderStatus};
use crate::models::position::Position;
use kameo::Actor;
use kameo::message::{Context, Message};
use sqlx::{Pool, Sqlite};
use tracing::info;

#[derive(Actor)]
#[actor(name = "OrderExecutionActor")]
pub struct OrderExecutionActor {
    pool: Pool<Sqlite>,
    broker: Box<dyn Broker>,
}

impl OrderExecutionActor {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            broker: Box::new(PaperBroker::default()),
        }
    }

    pub fn with_broker(pool: Pool<Sqlite>, broker: Box<dyn Broker>) -> Self {
        Self { pool, broker }
    }
}

impl Message<OrderRequest> for OrderExecutionActor {
    type Reply = ActorResult<Order>;

    async fn handle(
        &mut self,
        msg: OrderRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!(
            "[{}] Received order request for signal: {}",
            self.broker.name(),
            msg.signal_id
        );

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

        // 2. Submit to broker
        let fill_result = if let Some(limit_price) = msg.price {
            self.broker
                .submit_limit_order(&msg.symbol, &msg.side, msg.quantity, limit_price)
                .await
        } else {
            self.broker
                .submit_market_order(&msg.symbol, &msg.side, msg.quantity)
                .await
        };

        match fill_result {
            Ok(fill) if fill.filled => {
                // 3. Update order status to Filled
                order = Order::update_status(&order.id, OrderStatus::Filled, &self.pool)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()))?;

                info!(
                    "Order filled: {} @ {:.2} (qty: {:.4})",
                    order.id, fill.fill_price, fill.fill_quantity
                );

                // 4. Track position
                if let Err(e) = Position::open_or_update(
                    &msg.symbol,
                    &msg.side.to_string(),
                    fill.fill_quantity,
                    fill.fill_price,
                    &self.pool,
                )
                .await
                {
                    tracing::error!("Failed to update position: {:?}", e);
                }
            }
            Ok(_fill) => {
                // Partial fill or not filled — keep as Open for now
                info!("Order {} not fully filled, keeping open", order.id);
            }
            Err(e) => {
                // Broker rejected the order
                tracing::error!("Broker rejected order {}: {}", order.id, e);
                order = Order::update_status(&order.id, OrderStatus::Rejected, &self.pool)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(order)
    }
}
