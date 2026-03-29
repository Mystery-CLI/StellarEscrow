use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};

use crate::error::MobileSdkError;
use crate::types::{SignedTransaction, UnsignedTransaction};

/// Sign a transaction XDR with a raw secret key (hex-encoded 32-byte seed).
/// Runs entirely offline — no network required.
pub fn sign_transaction(
    unsigned: &UnsignedTransaction,
    secret_key_hex: &str,
) -> Result<SignedTransaction, MobileSdkError> {
    let key_bytes: [u8; 32] = hex::decode(secret_key_hex)
        .map_err(|e| MobileSdkError::InvalidKeypair(e.to_string()))?
        .try_into()
        .map_err(|_| MobileSdkError::InvalidKeypair("key must be exactly 32 bytes".into()))?;

    // network_hash = SHA-256(network_passphrase)
    let network_hash: [u8; 32] = Sha256::digest(unsigned.network_passphrase.as_bytes()).into();

    // tx_hash = SHA-256(network_hash || xdr_bytes)
    let xdr_bytes = STANDARD
        .decode(&unsigned.xdr)
        .map_err(|e| MobileSdkError::BuildFailed(e.to_string()))?;
    let mut h = Sha256::new();
    h.update(network_hash);
    h.update(&xdr_bytes);
    let tx_hash: [u8; 32] = h.finalize().into();

    // Sign with ed25519
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes);
    use ed25519_dalek::Signer;
    let signature = signing_key.sign(&tx_hash);

    // Append signature bytes to XDR (real impl would wrap in XDR TransactionEnvelope)
    let mut signed_bytes = xdr_bytes;
    signed_bytes.extend_from_slice(&signature.to_bytes());

    Ok(SignedTransaction {
        xdr: STANDARD.encode(&signed_bytes),
        hash: hex::encode(tx_hash),
    })
}
