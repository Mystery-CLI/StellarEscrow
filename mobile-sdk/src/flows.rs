/// Mobile-optimized transaction flows for common escrow operations.
///
/// Each flow bundles the steps a mobile user takes into a single function:
/// build → (optionally queue offline) → sign → submit.
use crate::error::MobileSdkError;
use crate::offline_queue::OfflineQueue;
use crate::signing::sign_transaction;
use crate::transaction_builder::{
    build_cancel_trade, build_confirm_receipt, build_fund_trade, build_raise_dispute,
};
use crate::types::{QueuedTransaction, SignedTransaction};

/// Result of a mobile flow step.
pub enum FlowResult {
    /// Transaction was signed and is ready for submission.
    Ready(SignedTransaction),
    /// Device is offline; transaction was queued for later.
    Queued(String),
}

/// Fund a trade: build + sign in one call.
/// If `secret_key_hex` is provided, signs immediately (offline-capable).
/// If the device is offline (`is_online = false`), queues instead.
pub fn flow_fund_trade(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
    secret_key_hex: &str,
    is_online: bool,
    queue: &mut OfflineQueue,
) -> Result<FlowResult, MobileSdkError> {
    let unsigned = build_fund_trade(contract_id, trade_id, sequence, network_passphrase)?;

    if !is_online {
        let queued_id = format!("fund_trade_{trade_id}_{sequence}");
        queue.enqueue(QueuedTransaction {
            id: queued_id.clone(),
            unsigned_xdr: unsigned.xdr,
            operation: format!("fund_trade:{trade_id}"),
            created_at: 0, // caller should set epoch seconds
        });
        return Ok(FlowResult::Queued(queued_id));
    }

    let signed = sign_transaction(&unsigned, secret_key_hex)?;
    Ok(FlowResult::Ready(signed))
}

/// Confirm receipt: build + sign in one call.
pub fn flow_confirm_receipt(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
    secret_key_hex: &str,
    is_online: bool,
    queue: &mut OfflineQueue,
) -> Result<FlowResult, MobileSdkError> {
    let unsigned = build_confirm_receipt(contract_id, trade_id, sequence, network_passphrase)?;

    if !is_online {
        let queued_id = format!("confirm_receipt_{trade_id}_{sequence}");
        queue.enqueue(QueuedTransaction {
            id: queued_id.clone(),
            unsigned_xdr: unsigned.xdr,
            operation: format!("confirm_receipt:{trade_id}"),
            created_at: 0,
        });
        return Ok(FlowResult::Queued(queued_id));
    }

    let signed = sign_transaction(&unsigned, secret_key_hex)?;
    Ok(FlowResult::Ready(signed))
}

/// Raise dispute: build + sign in one call.
pub fn flow_raise_dispute(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
    secret_key_hex: &str,
) -> Result<SignedTransaction, MobileSdkError> {
    let unsigned = build_raise_dispute(contract_id, trade_id, sequence, network_passphrase)?;
    sign_transaction(&unsigned, secret_key_hex)
}

/// Cancel trade: build + sign in one call.
pub fn flow_cancel_trade(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
    secret_key_hex: &str,
) -> Result<SignedTransaction, MobileSdkError> {
    let unsigned = build_cancel_trade(contract_id, trade_id, sequence, network_passphrase)?;
    sign_transaction(&unsigned, secret_key_hex)
}

/// Drain the offline queue and return signed transactions ready for submission.
/// Call this when connectivity is restored.
pub fn flush_offline_queue(
    queue: &mut OfflineQueue,
    secret_key_hex: &str,
    network_passphrase: &str,
) -> Vec<Result<SignedTransaction, MobileSdkError>> {
    queue
        .drain()
        .into_iter()
        .map(|queued| {
            let unsigned = crate::types::UnsignedTransaction {
                xdr: queued.unsigned_xdr,
                network_passphrase: network_passphrase.to_string(),
                fee: 100,
                sequence: 0,
            };
            sign_transaction(&unsigned, secret_key_hex)
        })
        .collect()
}
