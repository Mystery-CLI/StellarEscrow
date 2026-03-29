use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::warn;

use crate::config::RateLimitConfig;

// ---------------------------------------------------------------------------
// Tier definitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RateTier {
    Default,
    Elevated,
    Admin,
}

impl RateTier {
    /// Requests allowed per window.
    pub fn capacity(&self, cfg: &RateLimitConfig) -> u64 {
        match self {
            Self::Default => cfg.default_rpm,
            Self::Elevated => cfg.elevated_rpm,
            Self::Admin => cfg.admin_rpm,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-IP bucket (token bucket, refilled every minute)
// ---------------------------------------------------------------------------

struct Bucket {
    tokens: u64,
    last_refill: Instant,
    tier: RateTier,
}

impl Bucket {
    fn new(tier: RateTier, capacity: u64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            tier,
        }
    }

    /// Consume one token. Returns true if allowed.
    fn consume(&mut self, capacity: u64) -> bool {
        let elapsed = self.last_refill.elapsed();
        if elapsed >= Duration::from_secs(60) {
            self.tokens = capacity;
            self.last_refill = Instant::now();
        }
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn remaining(&self, capacity: u64) -> u64 {
        let elapsed = self.last_refill.elapsed();
        if elapsed >= Duration::from_secs(60) {
            capacity
        } else {
            self.tokens
        }
    }

    fn reset_secs(&self) -> u64 {
        let elapsed = self.last_refill.elapsed().as_secs();
        60u64.saturating_sub(elapsed)
    }
}

// ---------------------------------------------------------------------------
// Global monitoring counters
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct RateLimitCounters {
    pub total_allowed: AtomicU64,
    pub total_blocked: AtomicU64,
    pub total_blacklisted: AtomicU64,
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

pub struct RateLimiter {
    cfg: RateLimitConfig,
    buckets: DashMap<IpAddr, Bucket>,
    /// IPs with elevated or admin tier overrides.
    tier_overrides: DashMap<IpAddr, RateTier>,
    whitelist: Arc<RwLock<Vec<IpAddr>>>,
    blacklist: Arc<RwLock<Vec<IpAddr>>>,
    pub counters: Arc<RateLimitCounters>,
}

impl RateLimiter {
    pub fn new(cfg: RateLimitConfig) -> Self {
        let mut whitelist = cfg.whitelist.clone();
        let mut blacklist = cfg.blacklist.clone();

        Self {
            cfg,
            buckets: DashMap::new(),
            tier_overrides: DashMap::new(),
            whitelist: Arc::new(RwLock::new(whitelist)),
            blacklist: Arc::new(RwLock::new(blacklist)),
            counters: Arc::new(RateLimitCounters::default()),
        }
    }

    /// Check whether a request from `ip` is allowed.
    /// Returns Ok(headers) with rate-limit headers, or Err if blocked.
    pub async fn check(&self, ip: IpAddr) -> Result<RateLimitHeaders, RateLimitError> {
        // Whitelist bypasses everything
        if self.whitelist.read().await.contains(&ip) {
            self.counters.total_allowed.fetch_add(1, Ordering::Relaxed);
            return Ok(RateLimitHeaders::unlimited());
        }

        // Blacklist — hard block
        if self.blacklist.read().await.contains(&ip) {
            self.counters
                .total_blacklisted
                .fetch_add(1, Ordering::Relaxed);
            warn!("Blacklisted IP blocked: {}", ip);
            return Err(RateLimitError::Blacklisted);
        }

        let tier = self
            .tier_overrides
            .get(&ip)
            .map(|t| t.clone())
            .unwrap_or(RateTier::Default);

        let capacity = tier.capacity(&self.cfg);

        let mut bucket = self
            .buckets
            .entry(ip)
            .or_insert_with(|| Bucket::new(tier.clone(), capacity));

        let remaining = bucket.remaining(capacity);
        let reset_secs = bucket.reset_secs();

        if bucket.consume(capacity) {
            self.counters.total_allowed.fetch_add(1, Ordering::Relaxed);
            Ok(RateLimitHeaders {
                limit: capacity,
                remaining: remaining.saturating_sub(1),
                reset_secs,
            })
        } else {
            self.counters.total_blocked.fetch_add(1, Ordering::Relaxed);
            warn!("Rate limit exceeded for IP: {}", ip);
            Err(RateLimitError::LimitExceeded {
                limit: capacity,
                reset_secs,
            })
        }
    }

    // ---- Whitelist / Blacklist management ----

    pub async fn add_whitelist(&self, ip: IpAddr) {
        let mut wl = self.whitelist.write().await;
        if !wl.contains(&ip) {
            wl.push(ip);
        }
    }

    pub async fn remove_whitelist(&self, ip: IpAddr) {
        let mut wl = self.whitelist.write().await;
        wl.retain(|x| *x != ip);
    }

    pub async fn add_blacklist(&self, ip: IpAddr) {
        let mut bl = self.blacklist.write().await;
        if !bl.contains(&ip) {
            bl.push(ip);
        }
        // Remove from whitelist if present
        self.remove_whitelist(ip).await;
    }

    pub async fn remove_blacklist(&self, ip: IpAddr) {
        let mut bl = self.blacklist.write().await;
        bl.retain(|x| *x != ip);
    }

    pub async fn set_tier(&self, ip: IpAddr, tier: RateTier) {
        self.tier_overrides.insert(ip, tier);
        // Reset bucket so new tier takes effect immediately
        self.buckets.remove(&ip);
    }

    // ---- Monitoring snapshot ----

    pub async fn snapshot(&self) -> RateLimitSnapshot {
        RateLimitSnapshot {
            total_allowed: self.counters.total_allowed.load(Ordering::Relaxed),
            total_blocked: self.counters.total_blocked.load(Ordering::Relaxed),
            total_blacklisted: self.counters.total_blacklisted.load(Ordering::Relaxed),
            active_buckets: self.buckets.len(),
            whitelist: self
                .whitelist
                .read()
                .await
                .iter()
                .map(|ip| ip.to_string())
                .collect(),
            blacklist: self
                .blacklist
                .read()
                .await
                .iter()
                .map(|ip| ip.to_string())
                .collect(),
            tier_overrides: self
                .tier_overrides
                .iter()
                .map(|e| (e.key().to_string(), format!("{:?}", e.value())))
                .collect(),
            config: RateLimitConfigSnapshot {
                default_rpm: self.cfg.default_rpm,
                elevated_rpm: self.cfg.elevated_rpm,
                admin_rpm: self.cfg.admin_rpm,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RateLimitError {
    LimitExceeded { limit: u64, reset_secs: u64 },
    Blacklisted,
}

#[derive(Debug, Clone)]
pub struct RateLimitHeaders {
    pub limit: u64,
    pub remaining: u64,
    pub reset_secs: u64,
}

impl RateLimitHeaders {
    fn unlimited() -> Self {
        Self {
            limit: u64::MAX,
            remaining: u64::MAX,
            reset_secs: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RateLimitSnapshot {
    pub total_allowed: u64,
    pub total_blocked: u64,
    pub total_blacklisted: u64,
    pub active_buckets: usize,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub tier_overrides: Vec<(String, String)>,
    pub config: RateLimitConfigSnapshot,
}

#[derive(Debug, Serialize)]
pub struct RateLimitConfigSnapshot {
    pub default_rpm: u64,
    pub elevated_rpm: u64,
    pub admin_rpm: u64,
}
