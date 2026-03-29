use crate::error::MobileSdkError;
use crate::types::{SignedTransaction, UnsignedTransaction};

/// Supported mobile wallet types.
#[derive(Debug, Clone, PartialEq)]
pub enum MobileWallet {
    /// LOBSTR — most popular Stellar mobile wallet
    Lobstr,
    /// Solar Wallet
    Solar,
    /// Freighter (browser extension, also available on mobile)
    Freighter,
    /// Generic SEP-0007 deep-link compatible wallet
    Sep0007,
}

impl MobileWallet {
    /// Returns the deep-link URI scheme used by this wallet.
    pub fn scheme(&self) -> &'static str {
        match self {
            MobileWallet::Lobstr => "lobstr",
            MobileWallet::Solar => "solar",
            MobileWallet::Freighter => "freighter",
            MobileWallet::Sep0007 => "web+stellar",
        }
    }

    /// Returns a human-readable wallet name.
    pub fn name(&self) -> &'static str {
        match self {
            MobileWallet::Lobstr => "LOBSTR",
            MobileWallet::Solar => "Solar Wallet",
            MobileWallet::Freighter => "Freighter",
            MobileWallet::Sep0007 => "SEP-0007 Wallet",
        }
    }
}

/// Build a SEP-0007 `tx` deep-link to request signing from a mobile wallet.
/// The wallet app opens, shows the transaction, and signs it.
///
/// # Arguments
/// * `unsigned` — the unsigned transaction XDR
/// * `callback_url` — optional URL the wallet posts the signed XDR to
/// * `wallet` — target wallet (determines URI scheme)
pub fn build_sign_deep_link(
    unsigned: &UnsignedTransaction,
    callback_url: Option<&str>,
    wallet: &MobileWallet,
) -> Result<String, MobileSdkError> {
    let mut uri = format!(
        "{}://stellar.org/tx?xdr={}&network_passphrase={}",
        wallet.scheme(),
        urlenccode(&unsigned.xdr),
        urlenccode(&unsigned.network_passphrase),
    );
    if let Some(cb) = callback_url {
        uri.push_str(&format!("&callback={}", urlenccode(cb)));
    }
    Ok(uri)
}

/// Build a SEP-0007 `pay` deep-link for a simple payment request.
/// Useful for the "Fund Trade" flow where the buyer approves USDC transfer.
pub fn build_pay_deep_link(
    destination: &str,
    amount: &str,
    asset_code: &str,
    asset_issuer: &str,
    memo: Option<&str>,
    wallet: &MobileWallet,
) -> String {
    let mut uri = format!(
        "{}://stellar.org/pay?destination={}&amount={}&asset_code={}&asset_issuer={}",
        wallet.scheme(),
        urlenccode(destination),
        urlenccode(amount),
        urlenccode(asset_code),
        urlenccode(asset_issuer),
    );
    if let Some(m) = memo {
        uri.push_str(&format!("&memo={}&memo_type=text", urlenccode(m)));
    }
    uri
}

/// Detect which wallets are likely installed based on a list of registered
/// URI schemes reported by the host platform (iOS/Android).
///
/// Pass the list of schemes your platform reports as installed (e.g. from
/// `UIApplication.canOpenURL` on iOS or `PackageManager.queryIntentActivities`
/// on Android).
pub fn detect_installed_wallets(registered_schemes: &[&str]) -> Vec<MobileWallet> {
    let candidates = [
        MobileWallet::Lobstr,
        MobileWallet::Solar,
        MobileWallet::Freighter,
        MobileWallet::Sep0007,
    ];
    candidates
        .into_iter()
        .filter(|w| registered_schemes.contains(&w.scheme()))
        .collect()
}

/// Parse a signed XDR returned by a wallet deep-link callback.
/// Wallets typically return `?xdr=<signed_xdr>&hash=<tx_hash>` query params.
pub fn parse_wallet_callback(query_string: &str) -> Result<SignedTransaction, MobileSdkError> {
    let xdr = extract_param(query_string, "xdr")
        .ok_or_else(|| MobileSdkError::BuildFailed("missing xdr in callback".into()))?;
    let hash = extract_param(query_string, "hash").unwrap_or_default();
    Ok(SignedTransaction { xdr, hash })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Minimal percent-encoding for URI query values (encodes space, &, =, +, %).
fn urlenccode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap_or('0'));
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap_or('0'));
            }
        }
    }
    out
}

fn extract_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        if k == key {
            Some(parts.next().unwrap_or("").to_string())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UnsignedTransaction;

    #[test]
    fn test_build_sign_deep_link_lobstr() {
        let tx = UnsignedTransaction {
            xdr: "AAAA".into(),
            network_passphrase: "Test SDF Network ; September 2015".into(),
            fee: 100,
            sequence: 1,
        };
        let link = build_sign_deep_link(&tx, None, &MobileWallet::Lobstr).unwrap();
        assert!(link.starts_with("lobstr://stellar.org/tx?xdr="));
    }

    #[test]
    fn test_detect_installed_wallets() {
        let schemes = ["lobstr", "solar"];
        let wallets = detect_installed_wallets(&schemes);
        assert!(wallets.contains(&MobileWallet::Lobstr));
        assert!(wallets.contains(&MobileWallet::Solar));
        assert!(!wallets.contains(&MobileWallet::Freighter));
    }

    #[test]
    fn test_parse_wallet_callback() {
        let qs = "xdr=AAABBB&hash=deadbeef";
        let signed = parse_wallet_callback(qs).unwrap();
        assert_eq!(signed.xdr, "AAABBB");
        assert_eq!(signed.hash, "deadbeef");
    }
}
