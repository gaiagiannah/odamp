//! ODAMP Compliance — OFAC SDN feed.
//!
//! OFAC publishes the SDN list in multiple formats:
//! - JSON API: https://sanctions.ofac.treas.gov/api/sdn/v1/sdnList
//! - CSV: https://www.treasury.gov/ofac/downloads/sdn/sdn.csv
//! - XML (advanced): cross-referenced lookup tables for digital currency addresses
//!
//! This module handles the JSON API format (simplest, most reliable
//! for programmatic access). The XML format's digital-currency-address
//! cross-reference schema is documented but not fully implemented in
//! v0.x — the JSON API includes the addresses directly.

use serde::{Deserialize, Serialize};

/// A single entry in the OFAC SDN list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdnEntry {
    pub uid: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub addresses: Vec<String>,   // Digital currency addresses (if any)
    #[serde(default)]
    pub program: Option<String>,  // "SDN", "CAPTA", "SECT", etc.
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub date_listed: Option<String>,
    #[serde(default)]
    pub date_delisted: Option<String>,
}

/// The full SDN list response from the OFAC API.
#[derive(Debug, Deserialize)]
pub struct SdnListResponse {
    #[serde(rename = "results")]
    pub entries: Vec<SdnEntry>,
    pub count: usize,
}

/// Parse the OFAC JSON API response into structured entries.
pub fn parse_sdn_response(json: &str) -> Result<Vec<SdnEntry>, serde_json::Error> {
    let response: SdnListResponse = serde_json::from_str(json)?;
    Ok(response.entries)
}

/// Generate a version string for the loaded list.
pub fn list_version(entries: &[SdnEntry]) -> String {
    use std::collections::HashSet;
    let addr_count = entries.iter().map(|e| e.addresses.len()).sum::<usize>();
    format!("{} entries, {} addresses, loaded {}", entries.len(), addr_count, chrono::Utc::now().to_rfc3339())
}

/// Fetch the SDN list from the OFAC API.
///
/// This is called by the `load-ofac-sdn.sh` script or by the API's
/// scheduled refresh task. In v0.x, it's a simple HTTP GET.
pub async fn fetch_sdn_list(url: &str) -> Result<Vec<SdnEntry>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url)
        .await?
        .text()
        .await?;
    Ok(parse_sdn_response(&response)?)
}   