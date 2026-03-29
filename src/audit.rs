/// Comprehensive audit logging for security, compliance, and debugging.
///
/// On-chain audit entries are stored in persistent storage keyed by a
/// monotonically increasing counter.  Each entry captures who did what,
/// on which resource, with what outcome, and at which ledger.
///
/// Off-chain consumers (the indexer) can replay these entries via the
/// `get_audit_log` / `get_audit_logs` query functions.
use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Vec};

use crate::errors::ContractError;

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------
const AUDIT_COUNTER: &str = "AUDIT_CTR";
const AUDIT_PREFIX: &str = "AUDIT";
/// Maximum entries returned in a single page query.
pub const AUDIT_PAGE_LIMIT: u32 = 100;
/// Default retention window in ledgers (~90 days at 5 s/ledger).
pub const AUDIT_RETENTION_LEDGERS: u32 = 1_555_200;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditCategory {
    Security,
    Trade,
    Admin,
    Governance,
    System,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditSeverity {
    Info,
    Warn,
    ErrorLevel,
    Critical,
}

/// A single immutable audit log entry stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    /// Sequential ID (1-based).
    pub id: u64,
    /// Address that triggered the action.
    pub actor: Address,
    /// High-level category.
    pub category: AuditCategory,
    /// Specific action name, e.g. "trade.created".
    pub action: String,
    /// Optional resource type, e.g. "trade".
    pub resource_type: Option<String>,
    /// Optional resource identifier, e.g. trade_id as string.
    pub resource_id: Option<u64>,
    /// Outcome of the action.
    pub outcome: AuditOutcome,
    /// Severity level.
    pub severity: AuditSeverity,
    /// Ledger sequence at time of recording.
    pub ledger: u32,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn get_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&AUDIT_COUNTER)
        .unwrap_or(0u64)
}

fn set_counter(env: &Env, v: u64) {
    env.storage().instance().set(&AUDIT_COUNTER, &v);
}

fn save_entry(env: &Env, entry: &AuditEntry) {
    let key = (AUDIT_PREFIX, entry.id);
    env.storage().persistent().set(&key, entry);
}

fn load_entry(env: &Env, id: u64) -> Option<AuditEntry> {
    let key = (AUDIT_PREFIX, id);
    env.storage().persistent().get(&key)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Record an audit entry and emit an on-chain event.
/// Returns the new entry's ID.
pub fn record(
    env: &Env,
    actor: Address,
    category: AuditCategory,
    action: String,
    resource_type: Option<String>,
    resource_id: Option<u64>,
    outcome: AuditOutcome,
    severity: AuditSeverity,
) -> Result<u64, ContractError> {
    let id = get_counter(env)
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    set_counter(env, id);

    let entry = AuditEntry {
        id,
        actor: actor.clone(),
        category: category.clone(),
        action: action.clone(),
        resource_type,
        resource_id,
        outcome: outcome.clone(),
        severity: severity.clone(),
        ledger: env.ledger().sequence(),
    };

    save_entry(env, &entry);

    // Emit a lightweight event so the indexer can pick it up without
    // having to scan all persistent storage.
    env.events().publish(
        (symbol_short!("audit"),),
        (id, actor, category, action, outcome, severity),
    );

    Ok(id)
}

/// Retrieve a single audit entry by ID.
pub fn get_audit_log(env: &Env, id: u64) -> Option<AuditEntry> {
    load_entry(env, id)
}

/// Retrieve a page of audit entries in reverse-chronological order
/// (newest first).  `from_id` is the highest ID to start from
/// (inclusive); pass `0` to start from the latest.
pub fn get_audit_logs(env: &Env, from_id: u64, limit: u32) -> Vec<AuditEntry> {
    let limit = limit.min(AUDIT_PAGE_LIMIT);
    let latest = get_counter(env);
    let start = if from_id == 0 || from_id > latest {
        latest
    } else {
        from_id
    };

    let mut results = Vec::new(env);
    let mut id = start;
    let mut count = 0u32;

    while id > 0 && count < limit {
        if let Some(entry) = load_entry(env, id) {
            results.push_back(entry);
            count += 1;
        }
        id -= 1;
    }

    results
}

/// Return the total number of audit entries recorded.
pub fn audit_count(env: &Env) -> u64 {
    get_counter(env)
}

// ---------------------------------------------------------------------------
// Convenience wrappers used by other modules
// ---------------------------------------------------------------------------

/// Log a successful trade action.
pub fn log_trade(
    env: &Env,
    actor: Address,
    action: String,
    trade_id: u64,
) -> Result<(), ContractError> {
    record(
        env,
        actor,
        AuditCategory::Trade,
        action,
        Some(String::from_str(env, "trade")),
        Some(trade_id),
        AuditOutcome::Success,
        AuditSeverity::Info,
    )?;
    Ok(())
}

/// Log an admin action.
pub fn log_admin(
    env: &Env,
    actor: Address,
    action: String,
) -> Result<(), ContractError> {
    record(
        env,
        actor,
        AuditCategory::Admin,
        action,
        None,
        None,
        AuditOutcome::Success,
        AuditSeverity::Info,
    )?;
    Ok(())
}

/// Log a security event (unauthorized access, denied action, etc.).
pub fn log_security(
    env: &Env,
    actor: Address,
    action: String,
    outcome: AuditOutcome,
) -> Result<(), ContractError> {
    let severity = match outcome {
        AuditOutcome::Denied  => AuditSeverity::Warn,
        AuditOutcome::Failure => AuditSeverity::ErrorLevel,
        AuditOutcome::Success => AuditSeverity::Info,
    };
    record(
        env,
        actor,
        AuditCategory::Security,
        action,
        None,
        None,
        outcome,
        severity,
    )?;
    Ok(())
}
