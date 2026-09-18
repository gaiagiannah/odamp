//! ODAMP Compliance — OFAC SDN feed (Phase 3)
//!
//! READ THIS BEFORE TREATING THIS MODULE'S OUTPUT AS AUTHORITATIVE.
//!
//! OFAC's actual published Specially Designated Nationals (SDN) data,
//! in its "advanced" XML format, represents digital-currency
//! addresses through a layer of cross-referenced lookup tables: each
//! sanctioned party has `Feature` entries that reference a
//! `FeatureType` by numeric ID, and that ID is resolved against a
//! *separate* reference-value table to determine whether the feature
//! means "Digital Currency Address - XBT" (Bitcoin), "... ETH"
//! (Ethereum), etc. Implementing that full schema correctly, from
//! memory, without a live copy of the file to test against, is
//! exactly the kind of thing this project's whitepaper (Section 8)
//! commits to not overclaiming.
//!
//! So this module does something more modest and more honest: it
//! fetches the real, live OFAC data over HTTP, and then applies a
//! REGEX-BASED HEURISTIC to pull out strings that are shaped like
//! known crypto address formats (Bitcoin, Ethereum/EVM) appearing
//! near the words "Digital Currency Address" in the raw text. This
//! will likely have both false positives (address-shaped strings
//! that aren't actually sanctioned addresses) and false negatives
//! (real sanctioned addresses in a format this regex doesn't
//! recognize, or structured in a way the proximity heuristic misses).
//!
//! BEFORE THIS TOUCHES A REAL TRANSACTION: validate `fetch_addresses`
//! output against OFAC's own published list of known sanctioned
//! digital-currency addresses (they publish human-readable summaries
//! alongside the XML) and confirm known entries are actually being
//! extracted. If you get real address counts back that look wrong
//! (zero, or wildly high), the heuristic needs fixing before this is
//! trustworthy — don't ship it to a blocking code path until then.

use regex::Regex;

use crate::SanctionsList;

/// OFAC's sanctions-data publication endpoint. OFAC migrated their
/// list-distribution infrastructure to a new "Sanctions List
/// Service" in 2024; this URL is my best knowledge of the current
/// advanced-XML export endpoint as of this writing, but endpoint
/// URLs for government data services do change — if this 404s,
/// check https://ofac.treasury.gov/sanctions-list-service for the
/// current published location before assuming the parsing logic is
/// what's broken.
pub const DEFAULT_OFAC_URL: &str =
    "https://sanctionslistservice.ofac.treas.gov/api/PublicationPreview/exports/SDN_ADVANCED.XML";

/// Fetch the raw OFAC data and heuristically extract candidate
/// digital-currency addresses. See module docs for the important
/// caveats on this being a heuristic, not a schema-correct parser.
pub async fn fetch_and_build(
    client: &reqwest::Client,
    url: &str,
) -> anyhow::Result<SanctionsList> {
    let raw = client.get(url).send().await?.text().await?;
    let addresses = extract_candidate_addresses(&raw);

    let json = serde_json::to_string(&addresses)?;
    SanctionsList::load_from_json(
        &json,
        &format!(
            "OFAC SDN (heuristic extraction, fetched from {url}) — see core/compliance/src/ofac.rs for accuracy caveats"
        ),
    )
}

/// Heuristic extraction: find all Bitcoin- and EVM-shaped address
/// strings anywhere within a bounded window of the text "Digital
/// Currency Address", which is how OFAC's human-readable remarks
/// typically label these fields even within the structured XML.
fn extract_candidate_addresses(raw_text: &str) -> Vec<String> {
    // EVM-style: 0x + 40 hex chars.
    let evm_re = Regex::new(r"0x[a-fA-F0-9]{40}").expect("static regex must compile");
    // Bitcoin legacy/P2SH: 1 or 3 prefix, base58, 25-34 chars.
    let btc_legacy_re =
        Regex::new(r"\b[13][a-km-zA-HJ-NP-Z1-9]{25,34}\b").expect("static regex must compile");
    // Bitcoin bech32 (SegWit): bc1 prefix.
    let btc_bech32_re = Regex::new(r"\bbc1[a-z0-9]{25,60}\b").expect("static regex must compile");

    let marker = "Digital Currency Address";
    let window: usize = 400; // chars of context searched around each marker occurrence

    let mut found = std::collections::HashSet::new();

    let mut search_from = 0usize;
    while let Some(pos) = raw_text[search_from..].find(marker) {
        let abs_pos = search_from + pos;
        let start = abs_pos.saturating_sub(window);
        let end = (abs_pos + marker.len() + window).min(raw_text.len());
        let slice = &raw_text[start..end];

        for m in evm_re.find_iter(slice) {
            found.insert(m.as_str().to_string());
        }
        for m in btc_legacy_re.find_iter(slice) {
            found.insert(m.as_str().to_string());
        }
        for m in btc_bech32_re.find_iter(slice) {
            found.insert(m.as_str().to_string());
        }

        search_from = abs_pos + marker.len();
    }

    found.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_evm_address_near_marker() {
        let sample = r#"<Feature><Comment>Digital Currency Address - ETH
            0x1234567890abcdef1234567890abcdef12345678</Comment></Feature>"#;
        let found = extract_candidate_addresses(sample);
        assert!(found.contains(&"0x1234567890abcdef1234567890abcdef12345678".to_string()));
    }

    #[test]
    fn does_not_extract_address_far_from_any_marker() {
        let sample = "some unrelated document text with an address-shaped string \
            0x1234567890abcdef1234567890abcdef12345678 but no marker anywhere near it, \
            padded with enough filler text to push it outside any reasonable window size \
            just in case, repeated, repeated, repeated, repeated, repeated, repeated.";
        // No "Digital Currency Address" marker present at all in this sample.
        let found = extract_candidate_addresses(sample);
        assert!(found.is_empty());
    }
}
