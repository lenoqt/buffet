use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "http://localhost:3000/api";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StrategyDto {
    pub id: String,
    pub name: String,
    pub strategy_type: String,
    pub parameters: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OrderDto {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub price: Option<f64>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PositionDto {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: Option<f64>,
    pub unrealized_pnl: Option<f64>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BacktestDto {
    pub id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub status: String,
    pub initial_balance: f64,
    pub final_balance: Option<f64>,
    pub total_return: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
}

pub struct ApiService;

impl ApiService {
    async fn fetch<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
        let resp = Request::get(url).send().await.map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!(
                "Server responded with status {}: {}",
                resp.status(),
                resp.status_text()
            ));
        }

        resp.json::<T>().await.map_err(|e| e.to_string())
    }

    pub async fn get_strategies() -> Result<Vec<StrategyDto>, String> {
        Self::fetch(&format!("{}/strategies", API_BASE_URL)).await
    }

    pub async fn get_orders() -> Result<Vec<OrderDto>, String> {
        Self::fetch(&format!("{}/orders", API_BASE_URL)).await
    }

    pub async fn get_positions() -> Result<Vec<PositionDto>, String> {
        Self::fetch(&format!("{}/positions", API_BASE_URL)).await
    }

    pub async fn get_backtests() -> Result<Vec<BacktestDto>, String> {
        Self::fetch(&format!("{}/backtests", API_BASE_URL)).await
    }

    pub async fn run_backtest(
        strategy_id: &str,
        symbol: &str,
        start: &str,
        end: &str,
        balance: f64,
    ) -> Result<BacktestDto, String> {
        let url = format!("{}/backtests", API_BASE_URL);
        let body = serde_json::json!({
            "strategy_id": strategy_id,
            "symbol": symbol,
            "start_time": start,
            "end_time": end,
            "initial_balance": balance
        });

        let resp = Request::post(&url)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Error starting backtest: {}", resp.status()));
        }

        resp.json::<BacktestDto>().await.map_err(|e| e.to_string())
    }
}
