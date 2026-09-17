//! API authentication: HMAC signing for exchange APIs.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose::STANDARD as B64, Engine};

type HmacSha256 = Hmac<Sha256>;

/// Signs a message with HMAC-SHA256 (Coinbase, Kraken style).
pub fn sign_hmac(secret: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Signs with base64-encoded HMAC (Binance style).
pub fn sign_hmac_b64(secret: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    B64.encode(mac.finalize().into_bytes())
}

/// Creates a timestamped signature string.
pub fn signed_message(secret: &str, method: &str, path: &str, params: &str, timestamp: &str) -> String {
    let message = format!("{}{}{}{}", timestamp, method, path, params);
    sign_hmac(secret, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signing() {
        let sig = sign_hmac("secret_key", "test_message");
        assert_eq!(sig.len(), 64); // 32 bytes hex = 64 chars

        let sig_b64 = sign_hmac_b64("secret_key", "test_message");
        assert!(!sig_b64.is_empty());
    }
}   