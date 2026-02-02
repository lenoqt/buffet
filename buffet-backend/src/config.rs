use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: SocketAddr,
    pub tsdb_url: String, // PostgreSQL connection string for TimescaleDB
    pub actor: ActorConfig,
}

#[derive(Debug, Clone)]
pub struct ActorConfig {
    pub mailbox_size: usize,
    pub timeout_ms: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Load .env file
        if let Err(e) = dotenvy::dotenv() {
            tracing::warn!("Failed to load .env file: {}", e);
            // Continue even if .env file is missing - might be in production with env vars
        }

        // Database URL (SQLite for metadata)
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set in environment or .env file"))?;

        // Server host and port
        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| anyhow::anyhow!("Invalid SERVER_PORT: {}", e))?;

        let ip = IpAddr::from_str(&host)
            .map_err(|e| anyhow::anyhow!("Invalid SERVER_HOST address: {}", e))?;
        let server_addr = SocketAddr::new(ip, port);

        // TSDB configuration (TimescaleDB - PostgreSQL connection string)
        let tsdb_url = std::env::var("TSDB_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/buffet_timeseries".to_string()
        });

        // Actor system configuration
        let mailbox_size = std::env::var("ACTOR_MAILBOX_SIZE")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .map_err(|e| anyhow::anyhow!("Invalid ACTOR_MAILBOX_SIZE: {}", e))?;

        let timeout_ms = std::env::var("ACTOR_TIMEOUT_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid ACTOR_TIMEOUT_MS: {}", e))?;

        let actor = ActorConfig {
            mailbox_size,
            timeout_ms,
        };

        Ok(Self {
            database_url,
            server_addr,
            tsdb_url,
            actor,
        })
    }

    /// Builder pattern for testing
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    database_url: Option<String>,
    server_addr: Option<SocketAddr>,
    tsdb_url: Option<String>,
    actor: Option<ActorConfig>,
}

impl ConfigBuilder {
    pub fn database_url(mut self, url: String) -> Self {
        self.database_url = Some(url);
        self
    }

    pub fn server_addr(mut self, addr: SocketAddr) -> Self {
        self.server_addr = Some(addr);
        self
    }

    pub fn tsdb_url(mut self, url: String) -> Self {
        self.tsdb_url = Some(url);
        self
    }

    pub fn actor(mut self, config: ActorConfig) -> Self {
        self.actor = Some(config);
        self
    }

    pub fn build(self) -> anyhow::Result<Config> {
        Ok(Config {
            database_url: self
                .database_url
                .ok_or_else(|| anyhow::anyhow!("database_url is required"))?,
            server_addr: self
                .server_addr
                .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000))),
            tsdb_url: self.tsdb_url.unwrap_or_else(|| {
                "postgres://postgres:postgres@localhost:5432/buffet_timeseries".to_string()
            }),
            actor: self.actor.unwrap_or_else(|| ActorConfig {
                mailbox_size: 1000,
                timeout_ms: 5000,
            }),
        })
    }
}
