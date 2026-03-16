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
    pub status: String,
    pub symbols: String,
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
    pub commission_rate: f64,
    pub slippage_bps: f64,
    pub run_config: Option<String>,
    pub trade_count: Option<i64>,
    pub win_rate: Option<f64>,
    pub profit_factor: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BacktestTradeDto {
    pub id: String,
    pub backtest_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub entry_time: String,
    pub exit_time: Option<String>,
    pub pnl: Option<f64>,
    pub percentage_return: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateStrategyDto {
    pub name: String,
    pub strategy_type: String,
    pub parameters: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RunBacktestDto {
    pub strategy_id: String,
    pub symbol: String,
    pub start_time: String,
    pub end_time: String,
    pub initial_balance: f64,
    pub commission_rate: Option<f64>,
    pub slippage_bps: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SignalDto {
    pub id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub signal_type: String,
    pub timestamp: String,
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

    pub async fn get_strategy(id: &str) -> Result<StrategyDto, String> {
        Self::fetch(&format!("{}/strategies/{}", API_BASE_URL, id)).await
    }

    pub async fn create_strategy(dto: &CreateStrategyDto) -> Result<StrategyDto, String> {
        let url = format!("{}/strategies", API_BASE_URL);
        let resp = Request::post(&url)
            .json(dto)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Error creating strategy: {}", resp.status()));
        }

        resp.json::<StrategyDto>().await.map_err(|e| e.to_string())
    }

    pub async fn activate_strategy(id: &str) -> Result<StrategyDto, String> {
        let url = format!("{}/strategies/{}/activate", API_BASE_URL, id);
        let resp = Request::put(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Error activating strategy: {}", resp.status()));
        }

        resp.json::<StrategyDto>().await.map_err(|e| e.to_string())
    }

    pub async fn deactivate_strategy(id: &str) -> Result<StrategyDto, String> {
        let url = format!("{}/strategies/{}/deactivate", API_BASE_URL, id);
        let resp = Request::put(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Error deactivating strategy: {}", resp.status()));
        }

        resp.json::<StrategyDto>().await.map_err(|e| e.to_string())
    }

    pub async fn delete_strategy(id: &str) -> Result<(), String> {
        let url = format!("{}/strategies/{}", API_BASE_URL, id);
        let resp = Request::delete(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 204 {
            return Ok(());
        }

        if !resp.ok() {
            return Err(format!("Error deleting strategy: {}", resp.status()));
        }

        Ok(())
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

    pub async fn get_backtest(id: &str) -> Result<BacktestDto, String> {
        Self::fetch(&format!("{}/backtests/{}", API_BASE_URL, id)).await
    }

    pub async fn run_backtest(dto: &RunBacktestDto) -> Result<BacktestDto, String> {
        let url = format!("{}/backtests", API_BASE_URL);
        let resp = Request::post(&url)
            .json(dto)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Error starting backtest: {}", resp.status()));
        }

        resp.json::<BacktestDto>().await.map_err(|e| e.to_string())
    }

    pub async fn get_backtest_trades(backtest_id: &str) -> Result<Vec<BacktestTradeDto>, String> {
        Self::fetch(&format!("{}/backtests/{}/trades", API_BASE_URL, backtest_id)).await
    }

    pub async fn trigger_collection(symbol: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/collect", API_BASE_URL);
        let body = serde_json::json!({ "symbol": symbol });

        let resp = Request::post(&url)
            .json(&body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Error triggering collection: {}", resp.status()));
        }

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| e.to_string())
    }
}
