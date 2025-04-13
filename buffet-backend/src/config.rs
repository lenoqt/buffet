use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

pub struct Config {
    pub database_url: String,
    pub server_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env file
        if let Err(e) = dotenvy::dotenv() {
            tracing::warn!("Failed to load .env file: {}", e);
            // Continue even if .env file is missing - might be in production with env vars
        }

        // Database URL
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set in environment or .env file"))?;

        // Server host and port
        let host = std::env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| anyhow::anyhow!("Invalid SERVER_PORT: {}", e))?;

        let ip = IpAddr::from_str(&host)
            .map_err(|e| anyhow::anyhow!("Invalid SERVER_HOST address: {}", e))?;
        let server_addr = SocketAddr::new(ip, port);

        Ok(Self {
            database_url,
            server_addr,
        })
    }
}