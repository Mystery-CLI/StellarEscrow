use crate::types::MobileError;

/// Map a contract error code (from ContractError repr) to a mobile-friendly error.
pub fn map_contract_error(code: u32) -> MobileError {
    let (message, retryable) = match code {
        1 => ("Contract already initialized.", false),
        2 => ("Contract not initialized.", false),
        3 => ("Invalid amount — must be greater than zero.", false),
        4 => ("Invalid fee — must be between 0 and 10000 bps.", false),
        5 => ("Arbitrator is not registered.", false),
        6 => ("Trade not found.", false),
        7 => ("Operation not allowed in the current trade status.", false),
        8 => ("Arithmetic overflow detected.", false),
        9 => ("No fees available to withdraw.", false),
        10 => ("You are not authorized for this operation.", false),
        11 => ("Contract is paused or too many metadata entries.", false),
        12 => ("Metadata value is too long.", false),
        13 => ("Invalid tier configuration.", false),
        14 => ("Tier not found.", false),
        15 => ("Template not found.", false),
        16 => ("Template is inactive.", false),
        17 => ("Template name is too long.", false),
        18 => ("Template version limit exceeded.", false),
        19 => ("Trade amount does not match template.", false),
        _ => ("An unexpected error occurred. Please try again.", true),
    };
    MobileError { code, message: message.to_string(), retryable }
}

/// Map a network/submission HTTP status to a mobile-friendly error.
pub fn map_http_error(status: u32, body: &str) -> MobileError {
    let (message, retryable) = match status {
        0 => ("No network connection. Transaction queued for later.", true),
        408 | 503 | 504 => ("Network timeout. Will retry automatically.", true),
        429 => ("Too many requests. Please wait a moment.", true),
        400 => (body, false),
        401 | 403 => ("Authentication failed.", false),
        _ => ("Submission failed. Please try again.", true),
    };
    MobileError { code: status, message: message.to_string(), retryable }
}
