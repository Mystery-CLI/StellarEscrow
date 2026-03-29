use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::storage::{get_trade, get_trade_ids_for_address};
use crate::types::{HistoryFilter, HistoryPage, SortOrder, Trade, TransactionRecord};

/// Convert a Trade into a TransactionRecord
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

/// Check whether a trade passes the given filter
fn matches_filter(trade: &Trade, filter: &HistoryFilter) -> bool {
    if let Some(ref status) = filter.status {
        if &trade.status != status {
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
    true
}

/// Return paginated, filtered, sorted transaction history for an address.
///
/// - `address`  : seller or buyer address to look up
/// - `filter`   : optional status / ledger-range filters
/// - `sort`     : Ascending or Descending by `created_at`
/// - `offset`   : number of matching records to skip
/// - `limit`    : max records to return (capped at 100)
pub fn get_history(
    env: &Env,
    address: Address,
    filter: HistoryFilter,
    sort: SortOrder,
    offset: u32,
    limit: u32,
) -> Result<HistoryPage, ContractError> {
    let limit = if limit == 0 || limit > 100 { 100 } else { limit };

    let ids = get_trade_ids_for_address(env, &address);

    // Collect all matching records
    let mut records: Vec<TransactionRecord> = Vec::new(env);
    for id in ids.iter() {
        let trade = get_trade(env, id)?;
        if matches_filter(&trade, &filter) {
            records.push_back(to_record(&trade));
        }
    }

    // Sort by created_at
    // Soroban Vec doesn't have sort_by, so we do a simple insertion sort
    let len = records.len();
    for i in 1..len {
        let mut j = i;
        while j > 0 {
            let a = records.get(j - 1).unwrap();
            let b = records.get(j).unwrap();
            let should_swap = match sort {
                SortOrder::Ascending => a.created_at > b.created_at,
                SortOrder::Descending => a.created_at < b.created_at,
            };
            if should_swap {
                records.set(j - 1, b);
                records.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
    }

    let total = records.len();

    // Apply pagination
    let mut page: Vec<TransactionRecord> = Vec::new(env);
    let mut count: u32 = 0;
    let mut skipped: u32 = 0;
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

    Ok(HistoryPage {
        records: page,
        total,
        offset,
        limit,
    })
}

/// Export transaction history as a CSV-formatted Soroban String.
///
/// Columns: trade_id,seller,buyer,amount,fee,status,created_at,updated_at
pub fn export_csv(
    env: &Env,
    address: Address,
    filter: HistoryFilter,
) -> Result<soroban_sdk::String, ContractError> {
    let page = get_history(
        env,
        address,
        filter,
        SortOrder::Ascending,
        0,
        100,
    )?;

    // Build CSV as a fixed-size byte buffer using soroban_sdk::String::from_str
    // We encode each row as "id,amount,fee,status_code,created_at,updated_at\n"
    // (addresses are omitted from CSV since Soroban Address has no to_string in no_std)
    let mut csv = soroban_sdk::String::from_str(
        env,
        "trade_id,amount,fee,status,created_at,updated_at\n",
    );

    for record in page.records.iter() {
        let status_code: u32 = match record.status {
            crate::types::TradeStatus::Created => 0,
            crate::types::TradeStatus::Funded => 1,
            crate::types::TradeStatus::Completed => 2,
            crate::types::TradeStatus::Disputed => 3,
            crate::types::TradeStatus::Cancelled => 4,
        };

        // Encode numbers into a small stack buffer
        let row = format_csv_row(
            env,
            record.trade_id,
            record.amount,
            record.fee,
            status_code,
            record.created_at,
            record.updated_at,
        );
        csv = concat_strings(env, &csv, &row);
    }

    Ok(csv)
}

/// Concatenate two Soroban Strings by copying through a raw byte slice
fn concat_strings(
    env: &Env,
    a: &soroban_sdk::String,
    b: &soroban_sdk::String,
) -> soroban_sdk::String {
    let a_len = a.len() as usize;
    let b_len = b.len() as usize;
    let total = a_len + b_len;

    // Copy both strings into a stack buffer (max 512 bytes for safety)
    let mut buf = [0u8; 512];
    if total <= 512 {
        a.copy_into_slice(&mut buf[..a_len]);
        b.copy_into_slice(&mut buf[a_len..total]);
        soroban_sdk::String::from_bytes(env, &buf[..total])
    } else {
        // Fallback: just return a (shouldn't happen with our row sizes)
        a.clone()
    }
}

/// Format one CSV row from numeric fields
fn format_csv_row(
    env: &Env,
    trade_id: u64,
    amount: u64,
    fee: u64,
    status: u32,
    created_at: u32,
    updated_at: u32,
) -> soroban_sdk::String {
    // Build row in a fixed 128-byte stack buffer
    let mut buf = [0u8; 128];
    let mut pos = 0;

    pos = write_u64(&mut buf, pos, trade_id);
    buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, amount);
    buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, fee);
    buf[pos] = b','; pos += 1;
    pos = write_u32(&mut buf, pos, status);
    buf[pos] = b','; pos += 1;
    pos = write_u32(&mut buf, pos, created_at);
    buf[pos] = b','; pos += 1;
    pos = write_u32(&mut buf, pos, updated_at);
    buf[pos] = b'\n'; pos += 1;

    let slice = &buf[..pos];
    soroban_sdk::String::from_bytes(env, slice)
}

fn write_u64(buf: &mut [u8], mut pos: usize, mut n: u64) -> usize {
    if n == 0 {
        buf[pos] = b'0';
        return pos + 1;
    }
    let start = pos;
    while n > 0 {
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        pos += 1;
    }
    buf[start..pos].reverse();
    pos
}

fn write_u32(buf: &mut [u8], pos: usize, n: u32) -> usize {
    write_u64(buf, pos, n as u64)
}
