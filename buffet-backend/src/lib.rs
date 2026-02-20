pub mod actors;
pub mod broker;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod state;
pub mod telemetry;
pub mod tsdb;

// Re-export items needed for the binary
pub use error::{AppError, Result};
