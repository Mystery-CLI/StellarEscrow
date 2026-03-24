use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::storage::{get_timeline, get_trade};
use crate::types::{TradeAction, TradeDetail, TradeStatus, TimelineEntry};

/// Return a full trade detail view for a given viewer address.
///
/// - `trade_id` : the trade to inspect
/// - `viewer`   : the address requesting the view (determines available actions)
pub fn get_trade_detail(
    env: &Env,
    trade_id: u64,
    viewer: Address,
) -> Result<TradeDetail, ContractError> {
    let trade = get_trade(env, trade_id)?;
    let timeline = get_timeline(env, trade_id);
    let seller_payout = trade.amount.saturating_sub(trade.fee);
    let available_actions = resolve_actions(env, &trade, &viewer);

    Ok(TradeDetail {
        trade,
        timeline,
        available_actions,
        seller_payout,
    })
}

/// Determine which actions are available to the viewer based on trade status and role.
fn resolve_actions(
    env: &Env,
    trade: &crate::types::Trade,
    viewer: &Address,
) -> Vec<TradeAction> {
    let mut actions: Vec<TradeAction> = Vec::new(env);

    let is_buyer = viewer == &trade.buyer;
    let is_seller = viewer == &trade.seller;
    let is_arbitrator = trade
        .arbitrator
        .as_ref()
        .map(|a| a == viewer)
        .unwrap_or(false);

    match &trade.status {
        TradeStatus::Created => {
            if is_buyer {
                actions.push_back(TradeAction::Fund);
            }
            if is_seller {
                actions.push_back(TradeAction::Cancel);
            }
        }
        TradeStatus::Funded => {
            if is_seller {
                actions.push_back(TradeAction::Complete);
            }
            if (is_buyer || is_seller) && trade.arbitrator.is_some() {
                actions.push_back(TradeAction::RaiseDispute);
            }
        }
        TradeStatus::Completed => {
            if is_buyer {
                actions.push_back(TradeAction::ConfirmReceipt);
            }
            if (is_buyer || is_seller) && trade.arbitrator.is_some() {
                actions.push_back(TradeAction::RaiseDispute);
            }
        }
        TradeStatus::Disputed => {
            if is_arbitrator {
                actions.push_back(TradeAction::ResolveDispute);
            }
        }
        TradeStatus::Cancelled => {}
    }

    actions
}

/// Export a single trade's detail as a CSV row string.
///
/// Columns: trade_id,amount,fee,seller_payout,status,created_at,updated_at
pub fn export_trade_csv(env: &Env, trade_id: u64) -> Result<soroban_sdk::String, ContractError> {
    let trade = get_trade(env, trade_id)?;
    let seller_payout = trade.amount.saturating_sub(trade.fee);

    let status_code: u32 = match trade.status {
        TradeStatus::Created => 0,
        TradeStatus::Funded => 1,
        TradeStatus::Completed => 2,
        TradeStatus::Disputed => 3,
        TradeStatus::Cancelled => 4,
    };

    let header = soroban_sdk::String::from_str(
        env,
        "trade_id,amount,fee,seller_payout,status,created_at,updated_at\n",
    );
    let row = format_row(
        env,
        trade.id,
        trade.amount,
        trade.fee,
        seller_payout,
        status_code,
        trade.created_at,
        trade.updated_at,
    );

    Ok(concat_str(env, &header, &row))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn concat_str(
    env: &Env,
    a: &soroban_sdk::String,
    b: &soroban_sdk::String,
) -> soroban_sdk::String {
    let a_len = a.len() as usize;
    let b_len = b.len() as usize;
    let total = a_len + b_len;
    let mut buf = [0u8; 512];
    if total <= 512 {
        a.copy_into_slice(&mut buf[..a_len]);
        b.copy_into_slice(&mut buf[a_len..total]);
        soroban_sdk::String::from_bytes(env, &buf[..total])
    } else {
        a.clone()
    }
}

fn format_row(
    env: &Env,
    trade_id: u64,
    amount: u64,
    fee: u64,
    payout: u64,
    status: u32,
    created_at: u32,
    updated_at: u32,
) -> soroban_sdk::String {
    let mut buf = [0u8; 128];
    let mut pos = 0;
    pos = write_u64(&mut buf, pos, trade_id);  buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, amount);    buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, fee);       buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, payout);    buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, status as u64); buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, created_at as u64); buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, updated_at as u64); buf[pos] = b'\n'; pos += 1;
    soroban_sdk::String::from_bytes(env, &buf[..pos])
}

fn write_u64(buf: &mut [u8], mut pos: usize, mut n: u64) -> usize {
    if n == 0 { buf[pos] = b'0'; return pos + 1; }
    let start = pos;
    while n > 0 { buf[pos] = b'0' + (n % 10) as u8; n /= 10; pos += 1; }
    buf[start..pos].reverse();
    pos
}
