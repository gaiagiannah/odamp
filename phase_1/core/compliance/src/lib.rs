//! ODAMP Compliance Core
//!
//! Sanctions screening against OFAC's Specially Designated Nationals
//! (SDN) list, which is public data. This module implements real
//! matching logic; it does not ship the actual SDN dataset (that is
//! fetched/refreshed at runtime — see `load_from_json`), and does
//! not implement KYC/AML identity verification, which requires a
//! licensed provider and is out of scope for an unlicensed project.
//!
//! OFAC publishes a machine-readable digital-currency-address list
//! as part of the SDN data (addresses OFAC has identified as
//! belonging to sanctioned persons/entities). This module treats
//! that list as the source of truth for crypto-address screening.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SanctionsList {
    /// Lowercased, normalized addresses known to be sanctioned.
    addresses: HashSet<String>,
    /// Free-text source/version label, e.g. "OFAC SDN, refreshed 2026-09-17".
    pub source_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningResult {
    pub address: String,
    pub is_sanctioned: bool,
    pub matched_list: Option<String>,
}

impl SanctionsList {
    pub fn empty() -> Self {
        Self {
            addresses: HashSet::new(),
            source_label: "uninitialized".to_string(),
        }
    }

    /// Load a sanctions list from a JSON array of address strings.
    /// In production this is called on a scheduled refresh against
    /// OFAC's published SDN digital-currency-address data (and
    /// equivalent EU/UN lists, merged into separate `SanctionsList`
    /// instances or a combined one) — not hardcoded here, because
    /// the list changes and must stay current.
    pub fn load_from_json(json: &str, source_label: &str) -> anyhow::Result<Self> {
        let raw: Vec<String> = serde_json::from_str(json)?;
        let addresses = raw.into_iter().map(|a| normalize(&a)).collect();
        Ok(Self {
            addresses,
            source_label: source_label.to_string(),
        })
    }

    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }

    pub fn screen(&self, address: &str) -> ScreeningResult {
        let normalized = normalize(address);
        let hit = self.addresses.contains(&normalized);
        ScreeningResult {
            address: address.to_string(),
            is_sanctioned: hit,
            matched_list: if hit {
                Some(self.source_label.clone())
            } else {
                None
            },
        }
    }

    /// Screen every address involved in a transaction (both sides).
    /// Returns true if the transaction should be blocked pending
    /// review because either party is a sanctions match.
    pub fn screen_transaction(&self, from: &str, to: &str) -> (bool, Vec<ScreeningResult>) {
        let results = vec![self.screen(from), self.screen(to)];
        let blocked = results.iter().any(|r| r.is_sanctioned);
        (blocked, results)
    }
}

fn normalize(address: &str) -> String {
    address.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_sanctioned_address() {
        let json = r#"["0xABCDEF1234567890abcdef1234567890ABCDEF12"]"#;
        let list = SanctionsList::load_from_json(json, "OFAC SDN test fixture").unwrap();

        let hit = list.screen("0xabcdef1234567890ABCDEF1234567890abcdef12");
        assert!(hit.is_sanctioned, "screening must be case-insensitive");

        let clean = list.screen("0x0000000000000000000000000000000000dEaD");
        assert!(!clean.is_sanctioned);
    }

    #[test]
    fn transaction_screening_blocks_if_either_side_matches() {
        let json = r#"["0x1111111111111111111111111111111111111"]"#;
        let list = SanctionsList::load_from_json(json, "test").unwrap();
        let (blocked, _) = list.screen_transaction(
            "0x1111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222",
        );
        assert!(blocked);
    }
}
