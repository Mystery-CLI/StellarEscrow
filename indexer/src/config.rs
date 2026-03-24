use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub stellar: StellarConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarConfig {
    pub network: String, // "testnet" or "mainnet"
    pub contract_id: String,
    pub horizon_url: String,
    pub start_ledger: Option<u32>,
    pub poll_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for uploaded files
    pub base_dir: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig { port: 3000 },
            database: DatabaseConfig {
                url: "postgres://user:password@localhost/stellar_escrow".to_string(),
            },
            stellar: StellarConfig {
                network: "testnet".to_string(),
                contract_id: "".to_string(),
                horizon_url: "https://horizon-testnet.stellar.org".to_string(),
                start_ledger: None,
                poll_interval_seconds: 5,
            },
            storage: StorageConfig {
                base_dir: "./uploads".to_string(),
            },
        }
    }
}