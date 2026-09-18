//! ODAMP Security Core
//!
//! Threshold-signature key management built on FROST (Flexible
//! Round-Optimized Schnorr Threshold signatures), standardized as
//! IETF RFC 9591 (Komlo & Goldberg / Connolly, Komlo, Goldberg, Wood).
//!
//! This crate wraps the `frost-secp256k1` implementation so the rest
//! of ODAMP never touches raw key shares directly.
//!
//! IMPORTANT — READ BEFORE USING BEYOND LOCAL DEVELOPMENT:
//! `generate_wallet_trusted_dealer` below uses FROST's "trusted dealer"
//! key generation mode, where a single process briefly holds the full
//! key material in memory in order to split it into shares. This is
//! fine for local development and tests, but it reintroduces exactly
//! the single-point-of-compromise problem threshold signatures exist
//! to remove. Production key generation MUST use FROST's Distributed
//! Key Generation (DKG) protocol instead, where no party ever
//! possesses the complete key — that is a Phase 1 follow-up
//! (`generate_wallet_dkg`, not yet implemented in this scaffold).

use std::collections::BTreeMap;

use frost_secp256k1 as frost;
use rand_core::OsRng;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("threshold configuration invalid: need min <= max and min >= 1, got min={min}, max={max}")]
    InvalidThreshold { min: u16, max: u16 },

    #[error("frost keygen failed: {0}")]
    KeyGen(String),

    #[error("frost signing failed: {0}")]
    Signing(String),

    #[error("not enough participants supplied to meet threshold: have {have}, need {need}")]
    InsufficientParticipants { have: usize, need: usize },
}

/// A single participant's share of a threshold wallet's key.
/// This struct is what gets distributed to (and stored by) each
/// custody domain — e.g. user device, encrypted backup, HSM.
pub struct KeyShare {
    pub identifier: frost::Identifier,
    pub key_package: frost::keys::KeyPackage,
}

/// The public, non-secret material describing a threshold wallet:
/// its group public key and the verification data needed to check
/// individual signature shares during signing.
pub struct ThresholdWallet {
    pub wallet_id: Uuid,
    pub threshold: u16,
    pub total_shares: u16,
    pub public_key_package: frost::keys::PublicKeyPackage,
    pub shares: BTreeMap<frost::Identifier, KeyShare>,
}

/// Generate a new threshold wallet using trusted-dealer key generation.
///
/// `threshold` = minimum number of shares required to sign (e.g. 2)
/// `total_shares` = total number of shares created (e.g. 3)
///
/// See the module-level warning above: this mode is for local
/// development only.
pub fn generate_wallet_trusted_dealer(
    threshold: u16,
    total_shares: u16,
) -> Result<ThresholdWallet, SecurityError> {
    if threshold == 0 || threshold > total_shares {
        return Err(SecurityError::InvalidThreshold {
            min: threshold,
            max: total_shares,
        });
    }

    let mut rng = OsRng;

    let (secret_shares, public_key_package) = frost::keys::generate_with_dealer(
        total_shares,
        threshold,
        frost::keys::IdentifierList::Default,
        &mut rng,
    )
    .map_err(|e| SecurityError::KeyGen(e.to_string()))?;

    let mut shares = BTreeMap::new();
    for (identifier, secret_share) in secret_shares {
        let key_package = frost::keys::KeyPackage::try_from(secret_share)
            .map_err(|e| SecurityError::KeyGen(e.to_string()))?;
        shares.insert(
            identifier,
            KeyShare {
                identifier,
                key_package,
            },
        );
    }

    Ok(ThresholdWallet {
        wallet_id: Uuid::new_v4(),
        threshold,
        total_shares,
        public_key_package,
        shares,
    })
}

/// Produce a threshold signature over `message` using a subset of
/// the wallet's shares. `signer_ids` must contain at least
/// `wallet.threshold` identifiers, all of which must be keys in
/// `wallet.shares`.
///
/// This performs both FROST rounds (commit, then sign+aggregate) in
/// a single call for simplicity. A real multi-device deployment
/// would split these two rounds across a network round-trip between
/// the devices holding each share — this synchronous version is
/// correct for local testing but not the final signing-ceremony
/// transport.
pub fn sign_threshold(
    wallet: &ThresholdWallet,
    message: &[u8],
    signer_ids: &[frost::Identifier],
) -> Result<frost::Signature, SecurityError> {
    if signer_ids.len() < wallet.threshold as usize {
        return Err(SecurityError::InsufficientParticipants {
            have: signer_ids.len(),
            need: wallet.threshold as usize,
        });
    }

    let mut rng = OsRng;

    // Round 1: each participant generates signing nonces + commitments.
    let mut nonces_map = BTreeMap::new();
    let mut commitments_map = BTreeMap::new();

    for id in signer_ids {
        let key_package = &wallet
            .shares
            .get(id)
            .ok_or_else(|| SecurityError::Signing(format!("unknown signer id {:?}", id)))?
            .key_package;

        let (nonces, commitments) =
            frost::round1::commit(key_package.signing_share(), &mut rng);
        nonces_map.insert(*id, nonces);
        commitments_map.insert(*id, commitments);
    }

    let signing_package = frost::SigningPackage::new(commitments_map, message);

    // Round 2: each participant produces a signature share.
    let mut signature_shares = BTreeMap::new();
    for id in signer_ids {
        let key_package = &wallet.shares[id].key_package;
        let nonces = &nonces_map[id];
        let share = frost::round2::sign(&signing_package, nonces, key_package)
            .map_err(|e| SecurityError::Signing(e.to_string()))?;
        signature_shares.insert(*id, share);
    }

    // Aggregate into a single, standard Schnorr signature that any
    // secp256k1 verifier can check against the wallet's group public
    // key — the verifier does not need to know threshold signing was
    // used at all.
    let group_signature = frost::aggregate(
        &signing_package,
        &signature_shares,
        &wallet.public_key_package,
    )
    .map_err(|e| SecurityError::Signing(e.to_string()))?;

    Ok(group_signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_signs_2_of_3() {
        let wallet = generate_wallet_trusted_dealer(2, 3).expect("keygen should succeed");
        assert_eq!(wallet.shares.len(), 3);

        let ids: Vec<_> = wallet.shares.keys().take(2).copied().collect();
        let message = b"odamp phase 0 test transaction";

        let signature =
            sign_threshold(&wallet, message, &ids).expect("signing should succeed");

        let verified = wallet
            .public_key_package
            .verifying_key()
            .verify(message, &signature);
        assert!(verified.is_ok(), "aggregated signature must verify");
    }

    #[test]
    fn rejects_insufficient_signers() {
        let wallet = generate_wallet_trusted_dealer(3, 5).expect("keygen should succeed");
        let ids: Vec<_> = wallet.shares.keys().take(1).copied().collect();
        let result = sign_threshold(&wallet, b"test", &ids);
        assert!(matches!(
            result,
            Err(SecurityError::InsufficientParticipants { .. })
        ));
    }
}
