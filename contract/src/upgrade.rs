/// Upgrade system: versioning, migration registry, authorization, and rollback.
///
/// # Upgrade flow
/// 1. Admin proposes an upgrade via `propose_upgrade()` — stores the new WASM hash
///    and a timelock expiry (ledger sequence).
/// 2. After the timelock passes, admin calls `execute_upgrade()` — deploys the WASM
///    and snapshots critical instance-storage fields for rollback.
/// 3. Admin calls `migrate(expected_version)` to run any state migrations and bump
///    the version counter.
/// 4. If something goes wrong, admin calls `rollback_upgrade()` within the rollback
///    window to restore the snapshot and revert the version.
///
/// # Storage keys (all instance storage — cheap, always available)
///   UP_PROP   – pending UpgradeProposal
///   UP_SNAP   – RollbackSnapshot (written on execute, cleared on migrate success)
///   UP_GUARD  – UpgradeGuard (written on execute, cleared on migrate / rollback)
///   VERSION   – u32 contract version (also managed by storage.rs helpers)

use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

use crate::errors::ContractError;
use crate::storage::{get_admin, get_fee_bps, get_version, set_version};

// ---------------------------------------------------------------------------
// Timelock: ~24 h at ~5 s/ledger
// ---------------------------------------------------------------------------
pub const UPGRADE_TIMELOCK_LEDGERS: u32 = 17_280;

// ---------------------------------------------------------------------------
// Rollback window: ~1 h after execute
// ---------------------------------------------------------------------------
pub const ROLLBACK_WINDOW_LEDGERS: u32 = 720;

// ---------------------------------------------------------------------------
// Storage key helpers
// ---------------------------------------------------------------------------

fn key_up_prop()  -> Symbol { symbol_short!("UP_PROP") }
fn key_up_snap()  -> Symbol { symbol_short!("UP_SNAP") }
fn key_up_guard() -> Symbol { symbol_short!("UP_GUARD") }

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A pending upgrade waiting for the timelock to expire.
#[contracttype]
#[derive(Clone, Debug)]
pub struct UpgradeProposal {
    /// WASM hash of the new contract code.
    pub new_wasm_hash: BytesN<32>,
    /// Ledger sequence after which the upgrade may be executed.
    pub executable_after: u32,
    /// Address that submitted the proposal.
    pub proposed_by: Address,
    /// Human-readable description / changelog (max 256 chars enforced off-chain).
    pub description: soroban_sdk::String,
}

/// Snapshot of critical instance-storage fields taken just before WASM swap.
/// Used to restore state on rollback.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RollbackSnapshot {
    /// Contract version before the upgrade.
    pub version_before: u32,
    /// Platform fee bps before the upgrade.
    pub fee_bps_before: u32,
    /// Ledger sequence at which the snapshot was taken.
    pub snapshot_ledger: u32,
    /// Last ledger at which a rollback is still permitted.
    pub rollback_deadline: u32,
}

/// Guard written between execute and migrate/rollback to prevent re-entrancy.
#[contracttype]
#[derive(Clone, Debug)]
pub struct UpgradeGuard {
    /// Version the contract was at when the WASM was swapped.
    pub pre_upgrade_version: u32,
    /// Ledger at which the WASM was swapped.
    pub upgraded_at_ledger: u32,
}

// ---------------------------------------------------------------------------
// Storage accessors
// ---------------------------------------------------------------------------

pub fn get_upgrade_proposal(env: &Env) -> Option<UpgradeProposal> {
    env.storage().instance().get(&key_up_prop())
}

fn set_upgrade_proposal(env: &Env, proposal: &UpgradeProposal) {
    env.storage().instance().set(&key_up_prop(), proposal);
}

fn clear_upgrade_proposal(env: &Env) {
    env.storage().instance().remove(&key_up_prop());
}

pub fn get_rollback_snapshot(env: &Env) -> Option<RollbackSnapshot> {
    env.storage().instance().get(&key_up_snap())
}

fn set_rollback_snapshot(env: &Env, snap: &RollbackSnapshot) {
    env.storage().instance().set(&key_up_snap(), snap);
}

fn clear_rollback_snapshot(env: &Env) {
    env.storage().instance().remove(&key_up_snap());
}

pub fn get_upgrade_guard(env: &Env) -> Option<UpgradeGuard> {
    env.storage().instance().get(&key_up_guard())
}

fn set_upgrade_guard(env: &Env, guard: &UpgradeGuard) {
    env.storage().instance().set(&key_up_guard(), guard);
}

fn clear_upgrade_guard(env: &Env) {
    env.storage().instance().remove(&key_up_guard());
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Step 1 — Admin proposes an upgrade with a timelock.
///
/// Replaces any existing pending proposal (admin can update before execution).
/// Emits `EvUpgradeProposed`.
pub fn propose_upgrade(
    env: &Env,
    admin: &Address,
    new_wasm_hash: BytesN<32>,
    description: soroban_sdk::String,
) -> Result<(), ContractError> {
    require_admin(env, admin)?;

    // Block new proposals while an upgrade guard is active (mid-upgrade state).
    if get_upgrade_guard(env).is_some() {
        return Err(ContractError::UpgradeInProgress);
    }

    let executable_after = env
        .ledger()
        .sequence()
        .checked_add(UPGRADE_TIMELOCK_LEDGERS)
        .ok_or(ContractError::Overflow)?;

    let proposal = UpgradeProposal {
        new_wasm_hash,
        executable_after,
        proposed_by: admin.clone(),
        description: description.clone(),
    };
    set_upgrade_proposal(env, &proposal);

    crate::events::emit_upgrade_proposed(env, admin.clone(), executable_after, description);
    Ok(())
}

/// Step 2 — Admin executes the upgrade after the timelock has passed.
///
/// Snapshots current state for rollback, swaps the WASM, and sets the guard.
/// Emits `EvUpgraded`.
pub fn execute_upgrade(env: &Env, admin: &Address) -> Result<(), ContractError> {
    require_admin(env, admin)?;

    // Prevent double-execution.
    if get_upgrade_guard(env).is_some() {
        return Err(ContractError::UpgradeInProgress);
    }

    let proposal = get_upgrade_proposal(env).ok_or(ContractError::NoUpgradeProposal)?;

    if env.ledger().sequence() < proposal.executable_after {
        return Err(ContractError::UpgradeTimelockActive);
    }

    let current_version = get_version(env);
    let current_fee_bps = get_fee_bps(env).unwrap_or(0);
    let now = env.ledger().sequence();

    // Snapshot before WASM swap so rollback can restore.
    let snap = RollbackSnapshot {
        version_before: current_version,
        fee_bps_before: current_fee_bps,
        snapshot_ledger: now,
        rollback_deadline: now
            .checked_add(ROLLBACK_WINDOW_LEDGERS)
            .ok_or(ContractError::Overflow)?,
    };
    set_rollback_snapshot(env, &snap);

    // Guard prevents re-entrancy and signals "upgrade in progress".
    set_upgrade_guard(env, &UpgradeGuard {
        pre_upgrade_version: current_version,
        upgraded_at_ledger: now,
    });

    // Clear the proposal — it has been consumed.
    clear_upgrade_proposal(env);

    // Deploy new WASM (Soroban built-in).
    env.deployer().update_current_contract_wasm(proposal.new_wasm_hash);

    crate::events::emit_upgraded(env, current_version);
    Ok(())
}

/// Step 3 — Admin runs post-upgrade state migrations and finalises the upgrade.
///
/// `expected_version` must equal the current stored version to prevent
/// accidental double-application. Bumps version and clears guard + snapshot.
/// Emits `EvMigrated`.
pub fn run_migration(env: &Env, admin: &Address, expected_version: u32) -> Result<(), ContractError> {
    require_admin(env, admin)?;

    // Guard must be present — migration only valid after execute_upgrade.
    let guard = get_upgrade_guard(env).ok_or(ContractError::NoUpgradeInProgress)?;

    let current = get_version(env);
    if current != expected_version {
        return Err(ContractError::MigrationVersionMismatch);
    }
    if current != guard.pre_upgrade_version {
        return Err(ContractError::MigrationVersionMismatch);
    }

    // -----------------------------------------------------------------------
    // Version-specific migration logic goes here.
    // Pattern: match on `expected_version` and apply idempotent state changes.
    //
    // Example:
    //   if expected_version == 1 {
    //       backfill_new_field(env);
    //   }
    // -----------------------------------------------------------------------

    let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
    set_version(env, next);

    // Upgrade complete — clear guard and snapshot.
    clear_upgrade_guard(env);
    clear_rollback_snapshot(env);

    crate::events::emit_migrated(env, current, next);
    Ok(())
}

/// Rollback — Admin reverts to the pre-upgrade state within the rollback window.
///
/// Restores the version counter and fee bps from the snapshot.
/// Does NOT re-deploy the old WASM (Soroban does not support WASM downgrade);
/// the admin must call `execute_upgrade` with the previous WASM hash separately.
/// Emits `EvUpgradeRolledBack`.
pub fn rollback_upgrade(env: &Env, admin: &Address) -> Result<(), ContractError> {
    require_admin(env, admin)?;

    let snap = get_rollback_snapshot(env).ok_or(ContractError::NoUpgradeInProgress)?;

    if env.ledger().sequence() > snap.rollback_deadline {
        return Err(ContractError::RollbackWindowExpired);
    }

    // Restore version.
    set_version(env, snap.version_before);

    // Restore fee bps.
    crate::storage::set_fee_bps(env, snap.fee_bps_before);

    // Clear guard and snapshot.
    clear_upgrade_guard(env);
    clear_rollback_snapshot(env);

    crate::events::emit_upgrade_rolled_back(env, admin.clone(), snap.version_before);
    Ok(())
}

/// Cancel a pending (not yet executed) upgrade proposal.
pub fn cancel_upgrade(env: &Env, admin: &Address) -> Result<(), ContractError> {
    require_admin(env, admin)?;
    if get_upgrade_proposal(env).is_none() {
        return Err(ContractError::NoUpgradeProposal);
    }
    clear_upgrade_proposal(env);
    crate::events::emit_upgrade_cancelled(env, admin.clone());
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn require_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let admin = get_admin(env)?;
    if &admin != caller {
        return Err(ContractError::Unauthorized);
    }
    caller.require_auth();
    Ok(())
}
