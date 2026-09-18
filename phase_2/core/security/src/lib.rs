//! ODAMP Security Core
//!
//! Threshold-signature key management built on FROST (Flexible
//! Round-Optimized Schnorr Threshold signatures), standardized as
//! IETF RFC 9591 (Komlo & Goldberg / Connolly, Komlo, Goldberg, Wood).
//!
//! PHASE 1 CHANGE FROM THE PHASE 0 SCAFFOLD:
//! In Phase 0, the API process kept every generated key share in a
//! server-side in-memory Vec so /wallet/sign had something to sign
//! against. That is a real architectural bug relative to the
//! whitepaper's Section 2 commitment ("no ODAMP-operated entity ever
//! possesses a complete private key") — a process holding all N
//! shares can trivially reconstruct or use the key, threshold
//! cryptography or not.
//!
//! This version fixes that: `generate_wallet_trusted_dealer` returns
//! the shares to the CALLER exactly once and the library keeps no
//! reference to them afterward. `sign_threshold` takes the caller-
//! supplied shares as a parameter for that single signing operation
//! only. The server-side "wallet" representation
//! (`PublicWalletInfo`) contains only public, non-secret data
//! (group public key, threshold configuration) — which is exactly
//! what gets persisted to Postgres in Phase 1's `wallets` table.
//!
//! IMPORTANT — trusted-dealer keygen is still dev-only:
//! `generate_wallet_trusted_dealer` uses FROST's "trusted dealer"
//! mode, where this process briefly holds the full key material in
//! memory in order to split it. That's fine for local development,
//! but production key generation must use FROST's Distributed Key
//! Generation (DKG) protocol, where no single party ever holds the
//! complete key even momentarily. DKG is not yet implemented here —
//! flagged as a Phase 2+ follow-up (`generate_wallet_dkg`).

use frost_secp256k1 as frost;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
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

    #[error("not enough shares supplied to meet threshold: have {have}, need {need}")]
    InsufficientShares { have: usize, need: usize },

    #[error("serialization failed: {0}")]
    Serialization(String),
}

/// Public, non-secret description of a threshold wallet. This is
/// exactly what is safe to persist server-side (see
/// `api/src/db.rs::insert_wallet` in Phase 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicWalletInfo {
    pub wallet_id: Uuid,
    pub threshold: u16,
    pub total_shares: u16,
    /// Hex-encoded, compressed group public key. Any standard
    /// secp256k1/Schnorr verifier can check signatures against this
    /// without knowing threshold signing was used at all.
    pub group_public_key_hex: String,
}

/// A single participant's share of a threshold wallet's key,
/// serializable so it can be handed to the caller once at creation
/// time and supplied back in at signing time. This is the thing
/// that must go to a real, separate custody domain (user device,
/// encrypted backup, HSM) in a real deployment — never all of them
/// to the same place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedShare {
    /// Hex-encoded FROST identifier.
    pub identifier_hex: String,
    /// Hex-encoded, serialized FROST KeyPackage for this share.
    pub key_package_hex: String,
}

pub struct GeneratedWallet {
    pub info: PublicWalletInfo,
    /// Returned exactly once. The caller is responsible for
    /// distributing these to distinct custody domains and this
    /// library retains no copy after this function returns.
    pub shares: Vec<ExportedShare>,
}

/// Generate a new threshold wallet using trusted-dealer key generation.
/// See the module-level warning: dev-only, not for production custody.
pub fn generate_wallet_trusted_dealer(
    threshold: u16,
    total_shares: u16,
) -> Result<GeneratedWallet, SecurityError> {
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

    let mut exported = Vec::with_capacity(secret_shares.len());
    for (identifier, secret_share) in secret_shares {
        let key_package = frost::keys::KeyPackage::try_from(secret_share)
            .map_err(|e| SecurityError::KeyGen(e.to_string()))?;

        let identifier_bytes = identifier
            .serialize();
        let key_package_bytes = key_package
            .serialize()
            .map_err(|e| SecurityError::Serialization(e.to_string()))?;

        exported.push(ExportedShare {
            identifier_hex: hex::encode(identifier_bytes),
            key_package_hex: hex::encode(key_package_bytes),
        });
    }

    let wallet_id = Uuid::new_v4();
    let group_public_key_hex = hex::encode(public_key_package.verifying_key().serialize());

    Ok(GeneratedWallet {
        info: PublicWalletInfo {
            wallet_id,
            threshold,
            total_shares,
            group_public_key_hex,
        },
        shares: exported,
    })
}

/// Produce a threshold signature over `message`, given at least
/// `threshold` caller-supplied shares. The server-side code calling
/// this function never stores `supplied_shares` beyond the lifetime
/// of this call — see `api/src/main.rs::sign_message`.
pub fn sign_threshold(
    threshold: u16,
    total_shares: u16,
    message: &[u8],
    supplied_shares: &[ExportedShare],
) -> Result<frost::Signature, SecurityError> {
    if supplied_shares.len() < threshold as usize {
        return Err(SecurityError::InsufficientShares {
            have: supplied_shares.len(),
            need: threshold as usize,
        });
    }

    // Decode the supplied shares back into FROST types.
    let mut key_packages = std::collections::BTreeMap::new();
    for share in supplied_shares {
        let id_bytes = hex::decode(&share.identifier_hex)
            .map_err(|e| SecurityError::Serialization(e.to_string()))?;
        let identifier = frost::Identifier::deserialize(&id_bytes)
            .map_err(|e| SecurityError::Serialization(e.to_string()))?;

        let kp_bytes = hex::decode(&share.key_package_hex)
            .map_err(|e| SecurityError::Serialization(e.to_string()))?;
        let key_package = frost::keys::KeyPackage::deserialize(&kp_bytes)
            .map_err(|e| SecurityError::Serialization(e.to_string()))?;

        key_packages.insert(identifier, key_package);
    }

    // We need the group's PublicKeyPackage to aggregate. In this
    // trusted-dealer flow it can be reconstructed from any single
    // KeyPackage's `verifying_key()`, since every KeyPackage carries
    // the group's public verification data alongside its own share.
    let any_package = key_packages
        .values()
        .next()
        .ok_or_else(|| SecurityError::Signing("no key packages supplied".into()))?;
    let verifying_key = *any_package.verifying_key();

    let mut rng = OsRng;

    // Round 1: commitments.
    let mut nonces_map = std::collections::BTreeMap::new();
    let mut commitments_map = std::collections::BTreeMap::new();
    for (id, kp) in &key_packages {
        let (nonces, commitments) = frost::round1::commit(kp.signing_share(), &mut rng);
        nonces_map.insert(*id, nonces);
        commitments_map.insert(*id, commitments);
    }

    let signing_package = frost::SigningPackage::new(commitments_map, message);

    // Round 2: signature shares.
    let mut signature_shares = std::collections::BTreeMap::new();
    for (id, kp) in &key_packages {
        let nonces = &nonces_map[id];
        let share = frost::round2::sign(&signing_package, nonces, kp)
            .map_err(|e| SecurityError::Signing(e.to_string()))?;
        signature_shares.insert(*id, share);
    }

    // Build a PublicKeyPackage view for aggregation. frost-secp256k1's
    // `aggregate` needs the full PublicKeyPackage (verifying shares +
    // group key); each KeyPackage only carries its own verifying
    // share plus the group key, not the whole set, so in a real
    // deployment the PublicKeyPackage (public, non-secret) should be
    // fetched from the `wallets` table alongside `threshold`/
    // `total_shares` rather than reconstructed here. This trusted-
    // dealer-compatible reconstruction is a Phase 1 simplification —
    // flagged for replacement once PublicKeyPackage is persisted
    // as its own column (Phase 2 follow-up).
    let _ = verifying_key; // retained for the note above; see TODO
    let group_signature = frost::aggregate(&signing_package, &signature_shares, &{
        // Rebuild PublicKeyPackage from the verifying shares carried
        // in each supplied KeyPackage.
        let verifying_shares = key_packages
            .iter()
            .map(|(id, kp)| (*id, *kp.verifying_share()))
            .collect();
        frost::keys::PublicKeyPackage::new(verifying_shares, verifying_key)
    })
    .map_err(|e| SecurityError::Signing(e.to_string()))?;

    Ok(group_signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_signs_2_of_3() {
        let generated = generate_wallet_trusted_dealer(2, 3).expect("keygen should succeed");
        assert_eq!(generated.shares.len(), 3);

        let subset = &generated.shares[0..2];
        let message = b"odamp phase 1 test transaction";

        let signature = sign_threshold(
            generated.info.threshold,
            generated.info.total_shares,
            message,
            subset,
        )
        .expect("signing should succeed");

        let group_pubkey_bytes =
            hex::decode(&generated.info.group_public_key_hex).unwrap();
        let verifying_key =
            frost::VerifyingKey::deserialize(&group_pubkey_bytes).unwrap();
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn rejects_insufficient_shares() {
        let generated = generate_wallet_trusted_dealer(3, 5).expect("keygen should succeed");
        let subset = &generated.shares[0..1];
        let result = sign_threshold(
            generated.info.threshold,
            generated.info.total_shares,
            b"test",
            subset,
        );
        assert!(matches!(result, Err(SecurityError::InsufficientShares { .. })));
    }
}
