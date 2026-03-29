use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{get_template, increment_template_counter, save_template};
use crate::types::{
    TemplateTerms, TemplateVersion, TradeTemplate, TEMPLATE_MAX_VERSIONS,
    TEMPLATE_NAME_MAX_LEN,
};

fn validate_name(name: &String) -> Result<(), ContractError> {
    if name.len() > TEMPLATE_NAME_MAX_LEN {
        return Err(ContractError::TemplateNameTooLong);
    }
    Ok(())
}

pub fn create_template(
    env: &Env,
    owner: &Address,
    name: String,
    terms: TemplateTerms,
) -> Result<u64, ContractError> {
    validate_name(&name)?;
    let template_id = increment_template_counter(env)?;
    let now = env.ledger().sequence();
    let version = TemplateVersion { version: 1, terms, created_at: now };
    let mut versions: Vec<TemplateVersion> = Vec::new(env);
    versions.push_back(version);
    let template = TradeTemplate {
        id: template_id,
        owner: owner.clone(),
        name,
        current_version: 1,
        versions,
        active: true,
        created_at: now,
        updated_at: now,
    };
    save_template(env, template_id, &template);
    events::emit_template_created(env, template_id, owner.clone());
    Ok(template_id)
}

pub fn update_template(
    env: &Env,
    caller: &Address,
    template_id: u64,
    name: String,
    terms: TemplateTerms,
) -> Result<(), ContractError> {
    validate_name(&name)?;
    let mut template = get_template(env, template_id)?;
    if template.owner != *caller {
        return Err(ContractError::Unauthorized);
    }
    if template.versions.len() >= TEMPLATE_MAX_VERSIONS {
        // Drop the oldest entry by rebuilding from index 1
        let mut trimmed: Vec<TemplateVersion> = Vec::new(env);
        let len = template.versions.len();
        for i in 1..len {
            // unwrap is safe: index is within bounds
            trimmed.push_back(template.versions.get_unchecked(i));
        }
        template.versions = trimmed;
    }
    let new_version = template
        .current_version
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    let now = env.ledger().sequence();
    template.versions.push_back(TemplateVersion { version: new_version, terms, created_at: now });
    template.current_version = new_version;
    template.name = name;
    template.updated_at = now;
    save_template(env, template_id, &template);
    events::emit_template_updated(env, template_id, new_version);
    Ok(())
}

pub fn deactivate_template(
    env: &Env,
    caller: &Address,
    template_id: u64,
) -> Result<(), ContractError> {
    let mut template = get_template(env, template_id)?;
    if template.owner != *caller {
        return Err(ContractError::Unauthorized);
    }
    template.active = false;
    template.updated_at = env.ledger().sequence();
    save_template(env, template_id, &template);
    events::emit_template_deactivated(env, template_id);
    Ok(())
}

pub fn resolve_terms(
    env: &Env,
    template_id: u64,
) -> Result<(TemplateTerms, u32), ContractError> {
    let template = get_template(env, template_id)?;
    if !template.active {
        return Err(ContractError::TemplateInactive);
    }
    // versions is never empty for a valid template; last entry is always current
    let len = template.versions.len();
    if len == 0 {
        return Err(ContractError::TemplateNotFound);
    }
    let version = template.versions.get_unchecked(len - 1);
    Ok((version.terms, version.version))
}
