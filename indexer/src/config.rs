use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub meta: MetaConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub stellar: StellarConfig,
    pub rate_limit: RateLimitConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    pub gateway: GatewayConfig,
    pub integration: IntegrationConfig,
    #[serde(default)]
    pub compliance: ComplianceConfig,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub analytics: AnalyticsConfig,
    #[serde(default)]
    pub backup: BackupConfig,
    #[serde(default)]
    pub audit: AuditConfig,
}

// ---------------------------------------------------------------------------
// Compliance config (inlined to avoid circular module deps)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceConfig {
    #[serde(default)]
    pub kyc_provider_url: String,
    #[serde(default)]
    pub kyc_api_key: String,
    #[serde(default)]
    pub aml_provider_url: String,
    #[serde(default)]
    pub aml_api_key: String,
    #[serde(default = "default_kyc_level")]
    pub required_kyc_level: u8,
    #[serde(default = "default_aml_threshold")]
    pub aml_risk_threshold: u8,
    #[serde(default)]
    pub blocked_jurisdictions: Vec<String>,
    #[serde(default)]
    pub reporting_webhook_url: String,
    /// Maximum allowed trade amount in stroops (0 = unlimited)
    #[serde(default)]
    pub max_trade_amount: u64,
}

fn default_kyc_level() -> u8 { 1 }
fn default_aml_threshold() -> u8 { 70 }

// ---------------------------------------------------------------------------
// Monitoring config (inlined to avoid circular module deps)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
    #[serde(default)]
    pub alert_webhook_url: String,
    #[serde(default)]
    pub grafana_url: String,
    #[serde(default)]
    pub log_aggregation_url: String,
    #[serde(default)]
    pub incident_webhook_url: String,
    #[serde(default = "default_eval_interval")]
    pub alert_eval_interval_secs: u64,
}

fn default_metrics_port() -> u16 { 9090 }
fn default_eval_interval() -> u64 { 30 }

/// Metadata section — version tracking for the config itself.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaConfig {
    #[serde(default = "default_config_version")]
    pub version: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub environment: String,
}

fn default_config_version() -> String {
    "1.0.0".to_string()
}
fn default_schema_version() -> u32 {
    1
}

/// Non-secret subset of `Config` for operators and deployment validation (issue #313).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicConfigSnapshot {
    pub meta: MetaConfig,
    pub server: ServerConfig,
    pub stellar_network: String,
    pub stellar_horizon_url: String,
    pub stellar_contract_configured: bool,
    pub cache_redis_configured: bool,
    pub backup_interval_hours: u64,
    pub gateway_instance_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    /// sqlx connection pool max size (default: 10)
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
    /// sqlx connection pool min idle (default: 2)
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

fn default_pool_size() -> u32 {
    10
}
fn default_min_connections() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    /// Redis URL, e.g. redis://localhost:6379. Leave empty to disable.
    #[serde(default)]
    pub redis_url: String,
    /// Default TTL for cached API responses (seconds, default: 30)
    #[serde(default = "default_cache_ttl")]
    pub default_ttl_secs: u64,
    /// TTL for event list responses (seconds, default: 10)
    #[serde(default = "default_events_ttl")]
    pub events_ttl_secs: u64,
    /// TTL for search results (seconds, default: 30)
    #[serde(default = "default_search_ttl")]
    pub search_ttl_secs: u64,
    /// TTL for analytics dashboard (seconds, default: 60)
    #[serde(default = "default_analytics_ttl")]
    pub analytics_ttl_secs: u64,
    /// TTL for platform stats (seconds, default: 60)
    #[serde(default = "default_stats_ttl")]
    pub stats_ttl_secs: u64,
    /// TTL for Soroban RPC read-only call results (seconds, default: 30)
    #[serde(default = "default_rpc_ttl")]
    pub rpc_ttl_secs: u64,
}

fn default_cache_ttl() -> u64 { 30 }
fn default_events_ttl() -> u64 { 10 }
fn default_search_ttl() -> u64 { 30 }
fn default_analytics_ttl() -> u64 { 60 }
fn default_stats_ttl() -> u64 { 60 }
fn default_rpc_ttl() -> u64 { 30 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarConfig {
    /// "testnet" or "mainnet"
    pub network: String,
    pub contract_id: String,
    pub horizon_url: String,
    pub start_ledger: Option<u32>,
    pub poll_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub default_rpm: u64,
    pub elevated_rpm: u64,
    pub admin_rpm: u64,
    #[serde(default)]
    pub whitelist: Vec<IpAddr>,
    #[serde(default)]
    pub blacklist: Vec<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default)]
    pub admin_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_dir: String,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
}

fn default_max_file_size() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationConfig {
    #[serde(default)]
    pub email_api_url: String,
    #[serde(default)]
    pub email_api_key: String,
    #[serde(default)]
    pub email_from: String,
    #[serde(default)]
    pub sms_api_url: String,
    #[serde(default)]
    pub sms_account_sid: String,
    #[serde(default)]
    pub sms_auth_token: String,
    #[serde(default)]
    pub sms_from: String,
    #[serde(default)]
    pub push_api_url: String,
    #[serde(default)]
    pub push_project_id: String,
    #[serde(default)]
    pub push_server_key: String,
}

// ---------------------------------------------------------------------------
// Validation errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ConfigValidationError(pub Vec<String>);

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config validation failed:\n  - {}",
            self.0.join("\n  - ")
        )
    }
}

impl std::error::Error for ConfigValidationError {}

// ---------------------------------------------------------------------------
// Loading & validation
// ---------------------------------------------------------------------------

/// Gateway configuration for API routing and load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Service instances for load balancing (host:port format)
    #[serde(default)]
    pub service_instances: Vec<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            service_instances: vec![],
        }
    }
}

// Integration config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorKind {
    Webhook,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub id: String,
    pub name: String,
    pub kind: ConnectorKind,
    pub url: String,
    #[serde(default)]
    pub auth_token: Option<String>,
    /// Event types to forward; empty means all events.
    #[serde(default)]
    pub event_filter: Vec<String>,
    #[serde(default = "default_connector_timeout")]
    pub timeout_secs: u64,
}

fn default_connector_timeout() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationConfig {
    #[serde(default)]
    pub connectors: Vec<ConnectorConfig>,
}

// ---------------------------------------------------------------------------
// Analytics config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsConfig {
    /// Retain analytics events for this many days (0 = forever)
    #[serde(default = "default_analytics_retention")]
    pub retention_days: u32,
    /// Export max rows per request
    #[serde(default = "default_export_limit")]
    pub export_limit: u64,
}

fn default_analytics_retention() -> u32 { 90 }
fn default_export_limit() -> u64 { 10_000 }

// ---------------------------------------------------------------------------
// Backup config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    #[serde(default)]
    pub database_url: String,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default)]
    pub backup_dir: Option<String>,
    #[serde(default)]
    pub s3_bucket: Option<String>,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    /// How often to run scheduled backups (hours, 0 = disabled)
    #[serde(default = "default_backup_interval")]
    pub interval_hours: u64,
    #[serde(default)]
    pub alert_webhook: Option<String>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            script_path: None,
            backup_dir: None,
            s3_bucket: None,
            retention_days: 30,
            interval_hours: 24,
            alert_webhook: None,
        }
    }
}

fn default_retention_days() -> u32 { 30 }
fn default_backup_interval() -> u64 { 24 }

impl Config {
    /// Load config from a TOML file, then apply environment variable overrides.
    ///
    /// Env var pattern: `STELLAR_ESCROW__<SECTION>__<KEY>=value`
    /// Examples:
    ///   STELLAR_ESCROW__DATABASE__URL=postgres://...
    ///   STELLAR_ESCROW__SERVER__PORT=8080
    ///   STELLAR_ESCROW__STELLAR__NETWORK=mainnet
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&contents)?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    /// Apply `STELLAR_ESCROW__<SECTION>__<KEY>` environment variable overrides.
    fn apply_env_overrides(&mut self) {
        const PREFIX: &str = "STELLAR_ESCROW__";

        for (key, val) in std::env::vars() {
            if !key.starts_with(PREFIX) {
                continue;
            }
            let rest = &key[PREFIX.len()..];
            let parts: Vec<&str> = rest.splitn(2, "__").collect();
            if parts.len() != 2 {
                continue;
            }
            let (section, field) = (parts[0].to_lowercase(), parts[1].to_lowercase());

            match (section.as_str(), field.as_str()) {
                ("server", "port") => {
                    if let Ok(v) = val.parse() {
                        self.server.port = v;
                    }
                }
                ("server", "host") => self.server.host = val,
                ("database", "url") => self.database.url = val,
                ("database", "max_connections") => {
                    if let Ok(v) = val.parse() {
                        self.database.max_connections = v;
                    }
                }
                ("stellar", "network") => self.stellar.network = val,
                ("stellar", "contract_id") => self.stellar.contract_id = val,
                ("stellar", "horizon_url") => self.stellar.horizon_url = val,
                ("stellar", "poll_interval_seconds") => {
                    if let Ok(v) = val.parse() {
                        self.stellar.poll_interval_seconds = v;
                    }
                }
                ("rate_limit", "default_rpm") => {
                    if let Ok(v) = val.parse() {
                        self.rate_limit.default_rpm = v;
                    }
                }
                ("rate_limit", "elevated_rpm") => {
                    if let Ok(v) = val.parse() {
                        self.rate_limit.elevated_rpm = v;
                    }
                }
                ("rate_limit", "admin_rpm") => {
                    if let Ok(v) = val.parse() {
                        self.rate_limit.admin_rpm = v;
                    }
                }
                ("storage", "base_dir") => self.storage.base_dir = val,
                ("notification", "email_api_key") => self.notification.email_api_key = val,
                ("notification", "sms_account_sid") => self.notification.sms_account_sid = val,
                ("notification", "sms_auth_token") => self.notification.sms_auth_token = val,
                ("notification", "push_server_key") => self.notification.push_server_key = val,
                ("notification", "push_project_id") => self.notification.push_project_id = val,
                _ => {} // Unknown keys are silently ignored
            }
        }
    }

    /// Validate required fields and semantic constraints.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        let mut errors = Vec::new();

        if self.database.url.is_empty() {
            errors.push("database.url must not be empty".to_string());
        }
        if self.stellar.horizon_url.is_empty() {
            errors.push("stellar.horizon_url must not be empty".to_string());
        }
        if !["testnet", "mainnet"].contains(&self.stellar.network.as_str()) {
            errors.push(format!(
                "stellar.network must be 'testnet' or 'mainnet', got '{}'",
                self.stellar.network
            ));
        }
        if self.stellar.poll_interval_seconds == 0 {
            errors.push("stellar.poll_interval_seconds must be > 0".to_string());
        }
        if self.server.port == 0 {
            errors.push("server.port must be > 0".to_string());
        }

        // Production-specific checks
        if self.meta.environment == "production" {
            if self.stellar.network != "mainnet" {
                errors.push(
                    "production environment requires stellar.network = 'mainnet'".to_string(),
                );
            }
            if self.stellar.contract_id.is_empty() {
                errors.push("stellar.contract_id must be set in production".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigValidationError(errors))
        }
    }

    /// Safe, privacy-preserving view for `/config/public` (no API keys, DB URLs, or webhook secrets).
    pub fn public_snapshot(&self) -> PublicConfigSnapshot {
        PublicConfigSnapshot {
            meta: self.meta.clone(),
            server: self.server.clone(),
            stellar_network: self.stellar.network.clone(),
            stellar_horizon_url: self.stellar.horizon_url.clone(),
            stellar_contract_configured: !self.stellar.contract_id.is_empty(),
            cache_redis_configured: !self.cache.redis_url.is_empty(),
            backup_interval_hours: self.backup.interval_hours,
            gateway_instance_count: self.gateway.service_instances.len(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            meta: MetaConfig {
                version: "1.0.0".to_string(),
                schema_version: 1,
                environment: "development".to_string(),
            },
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            database: DatabaseConfig {
                url: "postgres://indexer:password@localhost/stellar_escrow".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            stellar: StellarConfig {
                network: "testnet".to_string(),
                contract_id: String::new(),
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
            auth: AuthConfig::default(),
            storage: StorageConfig {
                base_dir: "./uploads".to_string(),
                max_file_size_mb: 10,
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
            cache: CacheConfig::default(),
            gateway: GatewayConfig::default(),
            integration: IntegrationConfig::default(),
            compliance: ComplianceConfig::default(),
            monitoring: MonitoringConfig::default(),
            analytics: AnalyticsConfig::default(),
            backup: BackupConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Audit config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Retention period in days (default 90). Logs older than this are purged.
    #[serde(default = "default_audit_retention")]
    pub retention_days: u32,
    /// How often to run the retention purge, in hours (0 = disabled, default 24).
    #[serde(default = "default_audit_purge_interval")]
    pub purge_interval_hours: u64,
}

fn default_audit_retention() -> u32 { 90 }
fn default_audit_purge_interval() -> u64 { 24 }

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            retention_days: default_audit_retention(),
            purge_interval_hours: default_audit_purge_interval(),
        }
    }
}

// ---------------------------------------------------------------------------
// Startup environment variable validation
// ---------------------------------------------------------------------------

/// Validates that all required environment variables are present and
/// non-empty at application startup. Exits with code 1 and descriptive
/// messages if any are missing.
///
/// Call this before `Config::load()` so failures are caught immediately.
pub fn validate_env_vars() {
    /// (env_var_name, description, required_always)
    const REQUIRED: &[(&str, &str)] = &[
        ("STELLAR_ESCROW__DATABASE__URL",
         "PostgreSQL connection string for the indexer database"),
        ("STELLAR_ESCROW__STELLAR__CONTRACT_ID",
         "Deployed Soroban contract address (C...)"),
        ("STELLAR_ESCROW__STELLAR__HORIZON_URL",
         "Stellar Horizon API base URL"),
        ("STELLAR_ESCROW__STELLAR__NETWORK",
         "Stellar network: 'testnet' or 'mainnet'"),
        ("STELLAR_ESCROW__AUTH__API_KEYS",
         "Comma-separated list of valid API keys"),
    ];

    // Also accept the legacy flat-name variants used in docker-compose
    const LEGACY_ALIASES: &[(&str, &str)] = &[
        ("DATABASE_URL", "STELLAR_ESCROW__DATABASE__URL"),
    ];

    let mut missing: Vec<String> = Vec::new();
    let mut invalid: Vec<String> = Vec::new();

    for (var, desc) in REQUIRED {
        // Check the canonical name first, then any legacy alias
        let alias = LEGACY_ALIASES.iter().find(|(_, canonical)| canonical == var).map(|(a, _)| *a);
        let value = std::env::var(var)
            .ok()
            .or_else(|| alias.and_then(|a| std::env::var(a).ok()));

        match value {
            None => missing.push(format!("  MISSING  {var}\n           {desc}")),
            Some(v) if v.trim().is_empty() => {
                missing.push(format!("  EMPTY    {var}\n           {desc}"))
            }
            Some(v) => {
                // Format-level checks
                if *var == "STELLAR_ESCROW__STELLAR__NETWORK"
                    && v != "testnet"
                    && v != "mainnet"
                {
                    invalid.push(format!(
                        "  INVALID  {var}={v:?}\n           must be 'testnet' or 'mainnet'"
                    ));
                }
                if *var == "STELLAR_ESCROW__STELLAR__CONTRACT_ID"
                    && !v.starts_with('C')
                {
                    invalid.push(format!(
                        "  INVALID  {var}={v:?}\n           Soroban contract IDs start with 'C'"
                    ));
                }
            }
        }
    }

    if missing.is_empty() && invalid.is_empty() {
        tracing::info!("Environment variable validation passed ({} required vars present)", REQUIRED.len());
        return;
    }

    eprintln!("\n[FATAL] Environment variable validation failed at startup\n");
    if !missing.is_empty() {
        eprintln!("Missing or empty variables:");
        for m in &missing { eprintln!("{m}"); }
    }
    if !invalid.is_empty() {
        eprintln!("\nInvalid variable values:");
        for i in &invalid { eprintln!("{i}"); }
    }
    eprintln!("\nSet the variables above and restart the service.\n");
    std::process::exit(1);
}
