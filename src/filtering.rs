/// Advanced multi-criteria filtering and sorting for trades.
///
/// Supports combining status, amount range, ledger range, and party address
/// filters, with flexible multi-field sorting and persistent filter presets.
use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    delete_preset, get_preset, get_preset_ids_for_user, get_trade, get_trade_counter,
    get_trade_ids_for_address, increment_preset_counter, index_preset_for_user,
    remove_preset_from_index, save_preset,
};
use crate::types::{
    FilterPreset, SortCriterion, SortOrder, TradeSortField, TradeFilter, TradeSearchPage,
    TransactionRecord, Trade, MAX_PRESETS_PER_USER, PRESET_NAME_MAX_LEN,
};

// ---------------------------------------------------------------------------
// Filter matching
// ---------------------------------------------------------------------------

/// Returns true if the trade satisfies all criteria in the filter.
/// All set fields are ANDed together.
fn matches(trade: &Trade, filter: &TradeFilter) -> bool {
    if let Some(ref status) = filter.status {
        if &trade.status != status {
            return false;
        }
    }
    if let Some(min) = filter.min_amount {
        if trade.amount < min {
            return false;
        }
    }
    if let Some(max) = filter.max_amount {
        if trade.amount > max {
            return false;
        }
    }
    if let Some(from) = filter.from_ledger {
        if trade.created_at < from {
            return false;
        }
    }
    if let Some(to) = filter.to_ledger {
        if trade.created_at > to {
            return false;
        }
    }
    if let Some(ref seller) = filter.seller {
        if &trade.seller != seller {
            return false;
        }
    }
    if let Some(ref buyer) = filter.buyer {
        if &trade.buyer != buyer {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Sorting
// ---------------------------------------------------------------------------

fn field_value(record: &TransactionRecord, field: &TradeSortField) -> u64 {
    match field {
        TradeSortField::CreatedAt => record.created_at as u64,
        TradeSortField::UpdatedAt => record.updated_at as u64,
        TradeSortField::Amount => record.amount,
        TradeSortField::Fee => record.fee,
    }
}

/// Insertion sort on a Soroban Vec<TransactionRecord> by a single criterion.
fn sort_records(records: &mut Vec<TransactionRecord>, criterion: &SortCriterion) {
    let len = records.len();
    let mut i: u32 = 1;
    while i < len {
        let mut j = i;
        while j > 0 {
            let a = records.get(j - 1).unwrap();
            let b = records.get(j).unwrap();
            let av = field_value(&a, &criterion.field);
            let bv = field_value(&b, &criterion.field);
            let should_swap = match criterion.order {
                SortOrder::Ascending => av > bv,
                SortOrder::Descending => av < bv,
            };
            if should_swap {
                records.set(j - 1, b);
                records.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
        i += 1;
    }
}

fn to_record(trade: &Trade) -> TransactionRecord {
    TransactionRecord {
        trade_id: trade.id,
        seller: trade.seller.clone(),
        buyer: trade.buyer.clone(),
        amount: trade.amount,
        fee: trade.fee,
        status: trade.status.clone(),
        created_at: trade.created_at,
        updated_at: trade.updated_at,
        metadata: trade.metadata.clone(),
        currency: trade.currency.clone(),
        expiry_time: trade.expiry_time,
    }
}

/// Search all trades on the platform using multi-criteria filtering.
///
/// Iterates the global trade counter — suitable for admin/platform-wide queries.
pub fn search_all_trades(
    env: &Env,
    filter: TradeFilter,
    sort: SortCriterion,
    offset: u32,
    limit: u32,
) -> Result<TradeSearchPage, ContractError> {
    validate_filter(&filter)?;
    let limit = clamp_limit(limit);

    let trade_count = crate::storage::get_trade_counter(env).unwrap_or(0);
    let mut records: Vec<TransactionRecord> = Vec::new(env);

    let mut id: u64 = 1;
    while id <= trade_count {
        if let Ok(trade) = get_trade(env, id) {
            if matches(&trade, &filter) {
                records.push_back(to_record(&trade));
            }
        }
        id += 1;
    }

    sort_records(&mut records, &sort);
    Ok(paginate(env, records, offset, limit))
}

/// Search trades for a specific address (seller or buyer) using multi-criteria filtering.
pub fn search_trades_for_address(
    env: &Env,
    address: &Address,
    filter: TradeFilter,
    sort: SortCriterion,
    offset: u32,
    limit: u32,
) -> Result<TradeSearchPage, ContractError> {
    validate_filter(&filter)?;
    let limit = clamp_limit(limit);

    let ids = get_trade_ids_for_address(env, address);
    let mut records: Vec<TransactionRecord> = Vec::new(env);

    for id in ids.iter() {
        if let Ok(trade) = get_trade(env, id) {
            if matches(&trade, &filter) {
                records.push_back(to_record(&trade));
            }
        }
    }

    sort_records(&mut records, &sort);
    Ok(paginate(env, records, offset, limit))
}

// ---------------------------------------------------------------------------
// Filter presets
// ---------------------------------------------------------------------------

/// Save a new filter preset for a user. Returns the new preset ID.
pub fn save_filter_preset(
    env: &Env,
    owner: &Address,
    name: String,
    filter: TradeFilter,
    sort: SortCriterion,
) -> Result<u64, ContractError> {
    if name.len() > PRESET_NAME_MAX_LEN {
        return Err(ContractError::PresetNameTooLong);
    }
    validate_filter(&filter)?;

    // Enforce per-user preset limit
    let existing = get_preset_ids_for_user(env, owner);
    if existing.len() >= MAX_PRESETS_PER_USER {
        return Err(ContractError::TooManyPresets);
    }

    let preset_id = increment_preset_counter(env)?;
    let now = env.ledger().sequence();
    let preset = FilterPreset {
        id: preset_id,
        owner: owner.clone(),
        name,
        filter,
        sort,
        created_at: now,
        updated_at: now,
    };
    save_preset(env, &preset);
    index_preset_for_user(env, owner, preset_id);
    events::emit_preset_saved(env, owner.clone(), preset_id);
    Ok(preset_id)
}

/// Update an existing preset (owner only).
pub fn update_filter_preset(
    env: &Env,
    caller: &Address,
    preset_id: u64,
    name: String,
    filter: TradeFilter,
    sort: SortCriterion,
) -> Result<(), ContractError> {
    if name.len() > PRESET_NAME_MAX_LEN {
        return Err(ContractError::PresetNameTooLong);
    }
    validate_filter(&filter)?;

    let mut preset = get_preset(env, preset_id)?;
    if &preset.owner != caller {
        return Err(ContractError::Unauthorized);
    }
    preset.name = name;
    preset.filter = filter;
    preset.sort = sort;
    preset.updated_at = env.ledger().sequence();
    save_preset(env, &preset);
    events::emit_preset_saved(env, caller.clone(), preset_id);
    Ok(())
}

/// Delete a preset (owner only).
pub fn delete_filter_preset(
    env: &Env,
    caller: &Address,
    preset_id: u64,
) -> Result<(), ContractError> {
    let preset = get_preset(env, preset_id)?;
    if &preset.owner != caller {
        return Err(ContractError::Unauthorized);
    }
    delete_preset(env, preset_id);
    remove_preset_from_index(env, caller, preset_id);
    events::emit_preset_deleted(env, caller.clone(), preset_id);
    Ok(())
}

/// Get a single preset by ID.
pub fn get_filter_preset(env: &Env, preset_id: u64) -> Result<FilterPreset, ContractError> {
    get_preset(env, preset_id)
}

/// List all presets for a user.
pub fn list_filter_presets(env: &Env, owner: &Address) -> Vec<FilterPreset> {
    let ids = get_preset_ids_for_user(env, owner);
    let mut presets: Vec<FilterPreset> = Vec::new(env);
    for id in ids.iter() {
        if let Ok(preset) = get_preset(env, id) {
            presets.push_back(preset);
        }
    }
    presets
}

/// Apply a saved preset to run a search for the preset owner's trades.
pub fn search_with_preset(
    env: &Env,
    preset_id: u64,
    offset: u32,
    limit: u32,
) -> Result<TradeSearchPage, ContractError> {
    let preset = get_preset(env, preset_id)?;
    search_trades_for_address(
        env,
        &preset.owner.clone(),
        preset.filter,
        preset.sort,
        offset,
        limit,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate_filter(filter: &TradeFilter) -> Result<(), ContractError> {
    if let (Some(from), Some(to)) = (filter.from_ledger, filter.to_ledger) {
        if from > to {
            return Err(ContractError::InvalidLedgerRange);
        }
    }
    if let (Some(min), Some(max)) = (filter.min_amount, filter.max_amount) {
        if min > max {
            return Err(ContractError::InvalidAmount);
        }
    }
    Ok(())
}

fn clamp_limit(limit: u32) -> u32 {
    if limit == 0 || limit > 100 { 100 } else { limit }
}

fn paginate(
    env: &Env,
    records: Vec<TransactionRecord>,
    offset: u32,
    limit: u32,
) -> TradeSearchPage {
    let total = records.len();
    let mut page: Vec<TransactionRecord> = Vec::new(env);
    let mut skipped: u32 = 0;
    let mut count: u32 = 0;
    for record in records.iter() {
        if skipped < offset {
            skipped += 1;
            continue;
        }
        if count >= limit {
            break;
        }
        page.push_back(record);
        count += 1;
    }
    TradeSearchPage { records: page, total, offset, limit }
}
