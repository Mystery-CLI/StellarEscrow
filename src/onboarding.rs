/// Onboarding flow for new users.
///
/// The flow has four skippable steps followed by a terminal Completed state:
///   0 – RegisterProfile
///   1 – AcknowledgeFees
///   2 – CreateFirstTemplate
///   3 – CreateFirstTrade
///
/// Users can advance, skip individual steps, or exit the entire flow at any
/// point without affecting existing contract state.  Progress is persisted
/// on-chain so users can resume exactly where they left off.
use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{get_onboarding, save_onboarding};
use crate::types::{OnboardingProgress, OnboardingStep, StepStatus};

// Total number of actionable steps (excludes the terminal Completed variant).
const STEP_COUNT: u32 = 4;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a step index (0-based) to its OnboardingStep variant.
fn step_from_index(index: u32) -> OnboardingStep {
    match index {
        0 => OnboardingStep::RegisterProfile,
        1 => OnboardingStep::AcknowledgeFees,
        2 => OnboardingStep::CreateFirstTemplate,
        _ => OnboardingStep::CreateFirstTrade,
    }
}

/// Derive the next pending step after the given index, or Completed if all done.
fn next_step(statuses: &Vec<StepStatus>) -> OnboardingStep {
    let mut i: u32 = 0;
    while i < STEP_COUNT {
        let s = statuses.get(i).unwrap_or(StepStatus::Pending);
        if s == StepStatus::Pending {
            return step_from_index(i);
        }
        i += 1;
    }
    OnboardingStep::Completed
}

/// Build the initial step-status vector (all Pending).
fn initial_statuses(env: &Env) -> Vec<StepStatus> {
    let mut v: Vec<StepStatus> = Vec::new(env);
    v.push_back(StepStatus::Pending);
    v.push_back(StepStatus::Pending);
    v.push_back(StepStatus::Pending);
    v.push_back(StepStatus::Pending);
    v
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start onboarding for a user.  Idempotent — calling again on an already-
/// started (but not finished) flow returns the existing progress unchanged.
/// Returns an error if the user has already finished onboarding.
pub fn start_onboarding(env: &Env, user: Address) -> Result<OnboardingProgress, ContractError> {
    user.require_auth();

    if let Some(existing) = get_onboarding(env, &user) {
        if existing.finished {
            return Err(ContractError::AlreadyInitialized);
        }
        // Resume: return existing progress without mutation
        return Ok(existing);
    }

    let now = env.ledger().sequence();
    let statuses = initial_statuses(env);
    let progress = OnboardingProgress {
        address: user.clone(),
        current_step: OnboardingStep::RegisterProfile,
        step_statuses: statuses,
        started_at: now,
        updated_at: now,
        finished: false,
    };
    save_onboarding(env, &progress);
    events::emit_onboarding_started(env, user);
    Ok(progress)
}

/// Mark the current pending step as completed and advance to the next one.
/// The `step_index` must match the index of the current pending step to
/// prevent accidental out-of-order completions.
pub fn complete_step(
    env: &Env,
    user: Address,
    step_index: u32,
) -> Result<OnboardingProgress, ContractError> {
    user.require_auth();

    let mut progress = get_onboarding(env, &user).ok_or(ContractError::NotInitialized)?;
    if progress.finished {
        return Err(ContractError::AlreadyInitialized);
    }
    if step_index >= STEP_COUNT {
        return Err(ContractError::InvalidAmount);
    }

    // Only allow completing the step that is currently Pending
    let current_status = progress.step_statuses.get(step_index).unwrap_or(StepStatus::Done);
    if current_status != StepStatus::Pending {
        return Err(ContractError::InvalidStatus);
    }

    progress.step_statuses.set(step_index, StepStatus::Done);
    progress.current_step = next_step(&progress.step_statuses);
    progress.updated_at = env.ledger().sequence();

    if progress.current_step == OnboardingStep::Completed {
        progress.finished = true;
        save_onboarding(env, &progress);
        events::emit_onboarding_step_done(env, user.clone(), step_index);
        events::emit_onboarding_completed(env, user);
    } else {
        save_onboarding(env, &progress);
        events::emit_onboarding_step_done(env, user, step_index);
    }

    Ok(progress)
}

/// Skip a specific step by index.  The step must still be Pending.
/// Skipping does not block progress — the flow advances to the next pending step.
pub fn skip_step(
    env: &Env,
    user: Address,
    step_index: u32,
) -> Result<OnboardingProgress, ContractError> {
    user.require_auth();

    let mut progress = get_onboarding(env, &user).ok_or(ContractError::NotInitialized)?;
    if progress.finished {
        return Err(ContractError::AlreadyInitialized);
    }
    if step_index >= STEP_COUNT {
        return Err(ContractError::InvalidAmount);
    }

    let current_status = progress.step_statuses.get(step_index).unwrap_or(StepStatus::Done);
    if current_status != StepStatus::Pending {
        return Err(ContractError::InvalidStatus);
    }

    progress.step_statuses.set(step_index, StepStatus::Skipped);
    progress.current_step = next_step(&progress.step_statuses);
    progress.updated_at = env.ledger().sequence();

    if progress.current_step == OnboardingStep::Completed {
        progress.finished = true;
        save_onboarding(env, &progress);
        events::emit_onboarding_step_skipped(env, user.clone(), step_index);
        events::emit_onboarding_completed(env, user);
    } else {
        save_onboarding(env, &progress);
        events::emit_onboarding_step_skipped(env, user, step_index);
    }

    Ok(progress)
}

/// Exit onboarding entirely without completing it.  All remaining Pending
/// steps are marked Skipped and `finished` is set to true.  Existing contract
/// state (trades, profiles, etc.) is never touched.
pub fn exit_onboarding(env: &Env, user: Address) -> Result<OnboardingProgress, ContractError> {
    user.require_auth();

    let mut progress = get_onboarding(env, &user).ok_or(ContractError::NotInitialized)?;
    if progress.finished {
        return Err(ContractError::AlreadyInitialized);
    }

    // Mark every remaining Pending step as Skipped
    let mut i: u32 = 0;
    while i < STEP_COUNT {
        if progress.step_statuses.get(i).unwrap_or(StepStatus::Pending) == StepStatus::Pending {
            progress.step_statuses.set(i, StepStatus::Skipped);
        }
        i += 1;
    }
    progress.current_step = OnboardingStep::Completed;
    progress.finished = true;
    progress.updated_at = env.ledger().sequence();

    save_onboarding(env, &progress);
    events::emit_onboarding_exited(env, user);
    Ok(progress)
}

/// Return the current onboarding progress for a user, or None if not started.
pub fn get_progress(env: &Env, user: &Address) -> Option<OnboardingProgress> {
    get_onboarding(env, user)
}

/// Return true if the user has an active (started, not finished) onboarding flow.
pub fn is_onboarding_active(env: &Env, user: &Address) -> bool {
    match get_onboarding(env, user) {
        Some(p) => !p.finished,
        None => false,
    }
}
