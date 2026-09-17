use serde::{Deserialize, Serialize};
use crate::error::MpcError;

/// Security tier determines the MPC threshold and key share distribution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "tier")]
pub enum SecurityTier {
    /// < $1,000: Software MPC (2-of-2); device + encrypted backup
    Basic,
    /// $1,000 – $100,000: Software MPC (2-of-3); device + backup + social recovery
    Standard,
    /// $100,000 – $1,000,000: MPC (3-of-5) + HSM for Share 3
    Advanced,
    /// > $1,000,000: Full HSM-backed MPC (3-of-5) + Offline Signing Orchestrator
    Institutional,
}

impl SecurityTier {
    pub fn threshold(&self) -> (u32, u32) {
        match self {
            SecurityTier::Basic => (2, 2),
            SecurityTier::Standard => (2, 3),
            SecurityTier::Advanced => (3, 5),
            SecurityTier::Institutional => (3, 5),
        }
    }

    pub fn requires_hsm(&self) -> bool {
        matches!(self, SecurityTier::Advanced | SecurityTier::Institutional)
    }

    pub fn requires_offline_signing(&self) -> bool {
        matches!(self, SecurityTier::Institutional)
    }
}

/// MPC configuration for a user's wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcConfig {
    pub tier: SecurityTier,
    pub curve: SignatureCurve,
    pub pqc_enabled: bool,
    pub key_share_count: u32,
    pub threshold: u32,
    pub share_assignments: Vec<ShareAssignment>,
    pub recovery_config: Option<RecoveryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureCurve {
    /// secp256k1 (Bitcoin/Ethereum compatible)
    Secp256k1,
    /// P-256 (NIST, for institutional compatibility)
    P256,
    /// Ed25519 (modern, fast)
    Ed25519,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareAssignment {
    pub share_id: u32,
    pub location: ShareLocation,
    pub hsm_bound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShareLocation {
    /// User's primary device (mobile/PC)
    Device,
    /// Encrypted cloud backup
    CloudBackup,
    /// Hardware security module
    Hsm,
    /// Trusted contact (social recovery)
    TrustedContact { contact_id: String },
    /// Offline signing orchestrator
    OfflineSigner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub method: RecoveryMethod,
    pub threshold: u32,
    pub total_shares: u32,
    pub trusted_contacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecoveryMethod {
    /// Shamir's Secret Sharing
    Sss,
    /// Social recovery (trusted contacts)
    Social,
}

impl MpcConfig {
    pub fn for_tier(tier: SecurityTier) -> Result<Self, MpcError> {
        let (threshold, total) = tier.threshold();
        let curve = SignatureCurve::Secp256k1;

        let share_assignments = match tier {
            SecurityTier::Basic => vec![
                ShareAssignment { share_id: 1, location: ShareLocation::Device, hsm_bound: false },
                ShareAssignment { share_id: 2, location: ShareLocation::CloudBackup, hsm_bound: false },
            ],
            SecurityTier::Standard => vec![
                ShareAssignment { share_id: 1, location: ShareLocation::Device, hsm_bound: false },
                ShareAssignment { share_id: 2, location: ShareLocation::CloudBackup, hsm_bound: false },
                ShareAssignment { share_id: 3, location: ShareLocation::TrustedContact { contact_id: "pending".into() }, hsm_bound: false },
            ],
            SecurityTier::Advanced => vec![
                ShareAssignment { share_id: 1, location: ShareLocation::Device, hsm_bound: false },
                ShareAssignment { share_id: 2, location: ShareLocation::CloudBackup, hsm_bound: false },
                ShareAssignment { share_id: 3, location: ShareLocation::Hsm, hsm_bound: true },
                ShareAssignment { share_id: 4, location: ShareLocation::TrustedContact { contact_id: "pending".into() }, hsm_bound: false },
                ShareAssignment { share_id: 5, location: ShareLocation::TrustedContact { contact_id: "pending".into() }, hsm_bound: false },
            ],
            SecurityTier::Institutional => vec![
                ShareAssignment { share_id: 1, location: ShareLocation::Device, hsm_bound: false },
                ShareAssignment { share_id: 2, location: ShareLocation::CloudBackup, hsm_bound: false },
                ShareAssignment { share_id: 3, location: ShareLocation::Hsm, hsm_bound: true },
                ShareAssignment { share_id: 4, location: ShareLocation::OfflineSigner, hsm_bound: true },
                ShareAssignment { share_id: 5, location: ShareLocation::TrustedContact { contact_id: "pending".into() }, hsm_bound: false },
            ],
        };

        Ok(Self {
            tier: tier.clone(),
            curve,
            pqc_enabled: true,
            key_share_count: total,
            threshold,
            share_assignments,
            recovery_config: Some(RecoveryConfig {
                method: RecoveryMethod::Sss,
                threshold: 2,
                total_shares: 3,
                trusted_contacts: vec![],
            }),
        })
    }

    pub fn validate(&self) -> Result<(), MpcError> {
        if self.threshold > self.key_share_count {
            return Err(MpcError::InvalidThreshold {
                threshold: self.threshold,
                total: self.key_share_count,
            });
        }
        if self.share_assignments.len() as u32 != self.key_share_count {
            return Err(MpcError::ShareCountMismatch {
                expected: self.key_share_count,
                actual: self.share_assignments.len() as u32,
            });
        }
        if self.tier.requires_hsm() && !self.share_assignments.iter().any(|s| s.hsm_bound) {
            return Err(MpcError::HsmRequiredButNotConfigured);
        }
        Ok(())
    }
}   