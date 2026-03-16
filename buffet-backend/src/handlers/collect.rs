use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::actors::collector::CollectData;
use crate::actors::messages::CollectHistorical;
use crate::error::Result;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CollectRequest {
    pub symbol: String,
    pub asset_type: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct CollectResponse {
    pub status: String,
    pub symbol: String,
}

pub async fn trigger_collection(
    State(state): State<AppState>,
    Json(req): Json<CollectRequest>,
) -> Result<(StatusCode, Json<CollectResponse>)> {
    let symbol = req.symbol.clone();

    if let (Some(start), Some(end)) = (req.start, req.end) {
        let _ = state
            .collector
            .tell(CollectHistorical {
                symbol: symbol.clone(),
                asset_type: req.asset_type.unwrap_or_else(|| "stock".to_string()),
                start,
                end,
            })
            .send()
            .await;
    } else {
        let _ = state
            .collector
            .tell(CollectData {
                symbol: symbol.clone(),
            })
            .send()
            .await;
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(CollectResponse {
            status: "accepted".into(),
            symbol,
        }),
    ))
}
