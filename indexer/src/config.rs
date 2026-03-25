use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub stellar: StellarConfig,
    pub rate_limit: RateLimitConfig,
    pub storage: StorageConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
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
pub struct RateLimitConfig {
    /// Requests per minute for unauthenticated / default IPs.
    pub default_rpm: u64,
    /// Requests per minute for elevated-tier IPs.
    pub elevated_rpm: u64,
    /// Requests per minute for admin-tier IPs (effectively unlimited in practice).
    pub admin_rpm: u64,
    /// IPs that bypass rate limiting entirely.
    #[serde(default)]
    pub whitelist: Vec<IpAddr>,
    /// IPs that are always blocked.
    #[serde(default)]
    pub blacklist: Vec<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Shared API keys for regular clients.
    #[serde(default)]
    pub api_keys: Vec<String>,
    /// Admin API keys for privileged routes.
    #[serde(default)]
    pub admin_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for uploaded files
    pub base_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    // Email (SendGrid-compatible)
    pub email_api_url: String,
    pub email_api_key: String,
    pub email_from: String,
    // SMS (Twilio-compatible)
    pub sms_api_url: String,
    pub sms_account_sid: String,
    pub sms_auth_token: String,
    pub sms_from: String,
    // Push (FCM v1)
    pub push_api_url: String,
    pub push_project_id: String,
    pub push_server_key: String,
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
            rate_limit: RateLimitConfig {
                default_rpm: 60,
                elevated_rpm: 300,
                admin_rpm: 6000,
                whitelist: vec![],
                blacklist: vec![],
            },
            auth: AuthConfig {
                api_keys: vec!["demo-key-123".to_string()],
                admin_keys: vec!["admin-key-123".to_string()],
            },
            storage: StorageConfig {
                base_dir: "./uploads".to_string(),
            },
            notification: NotificationConfig {
                email_api_url: "https://api.sendgrid.com".to_string(),
                email_api_key: String::new(),
                email_from: "noreply@stellarescrow.io".to_string(),
                sms_api_url: "https://api.twilio.com".to_string(),
                sms_account_sid: String::new(),
                sms_auth_token: String::new(),
                sms_from: String::new(),
                push_api_url: "https://fcm.googleapis.com".to_string(),
                push_project_id: String::new(),
                push_server_key: String::new(),
            },
        }
    }
}