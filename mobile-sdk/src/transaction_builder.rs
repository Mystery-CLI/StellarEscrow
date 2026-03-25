use base64::{engine::general_purpose::STANDARD, Engine};

use crate::error::MobileSdkError;
use crate::types::UnsignedTransaction;

/// Network passphrases
pub const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";
pub const MAINNET_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";

/// Common mobile fee (in stroops). Kept low for mobile UX.
const MOBILE_BASE_FEE: u32 = 100;

/// Build an unsigned transaction XDR for `fund_trade`.
/// On mobile the buyer calls this, signs offline, then submits when online.
pub fn build_fund_trade(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
) -> Result<UnsignedTransaction, MobileSdkError> {
    build_invoke(contract_id, "fund_trade", &[trade_id.to_string()], sequence, network_passphrase)
}

/// Build an unsigned transaction XDR for `confirm_receipt`.
pub fn build_confirm_receipt(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
) -> Result<UnsignedTransaction, MobileSdkError> {
    build_invoke(
        contract_id,
        "confirm_receipt",
        &[trade_id.to_string()],
        sequence,
        network_passphrase,
    )
}

/// Build an unsigned transaction XDR for `raise_dispute`.
pub fn build_raise_dispute(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
) -> Result<UnsignedTransaction, MobileSdkError> {
    build_invoke(
        contract_id,
        "raise_dispute",
        &[trade_id.to_string()],
        sequence,
        network_passphrase,
    )
}

/// Build an unsigned transaction XDR for `cancel_trade`.
pub fn build_cancel_trade(
    contract_id: &str,
    trade_id: u64,
    sequence: i64,
    network_passphrase: &str,
) -> Result<UnsignedTransaction, MobileSdkError> {
    build_invoke(
        contract_id,
        "cancel_trade",
        &[trade_id.to_string()],
        sequence,
        network_passphrase,
    )
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Produce a minimal unsigned transaction envelope (XDR placeholder).
/// A real implementation would use stellar-xdr or horizon to build the envelope.
/// This encodes the intent as JSON-in-base64 so the signing module can hash it
/// deterministically without pulling in a full XDR library.
fn build_invoke(
    contract_id: &str,
    function: &str,
    args: &[String],
    sequence: i64,
    network_passphrase: &str,
) -> Result<UnsignedTransaction, MobileSdkError> {
    let payload = serde_json::json!({
        "contract": contract_id,
        "function": function,
        "args": args,
        "sequence": sequence,
    });
    let xdr = STANDARD.encode(
        serde_json::to_string(&payload)
            .map_err(|e| MobileSdkError::BuildFailed(e.to_string()))?,
    );
    Ok(UnsignedTransaction {
        xdr,
        network_passphrase: network_passphrase.to_string(),
        fee: MOBILE_BASE_FEE,
        sequence,
    })
}
