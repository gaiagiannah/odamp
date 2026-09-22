use colored::Colorized;

pub fn screen(address: &str) -> anyhow::Result<()> {
    println!("{}", "ODAMP — OFAC SDN Screening".bold());
    println!("{}", format!("  Address: {}", address).dimmed());
    println!();

    let mut engine = odamp_compliance::ComplianceEngine::new();

    // Load SDN list from DB (if available)
    // For CLI, we load from a local JSON cache if it exists
    let cache_path = std::path::Path::new(std::env::var("HOME").as_deref().unwrap_or("/root"))
        .join(".odamp").join("sdn_cache.json");

    if cache_path.exists() {
        let json = std::fs::read_to_string(&cache_path)?;
        let entries: Vec<odamp_compliance::ofac::SdnEntry> = serde_json::from_str(&json)?;
        let version = odamp_compliance::ofac::list_version(&entries);
        engine.load_sdn_list(entries, version);
    } else {
        println!("{}", "  ⚠ No local SDN cache found. Run load-ofac-sdn.sh first.".yellow());
        println!("{}", "  Result will be FLAGGED (fail-closed).".yellow());
    }

    let result = engine.screen(address)?;

    match result.status {
        odamp_shared::ScreenResult::Clean => {
            println!("{}", "  ✓ CLEAN — No match found".green().bold());
        }
        odamp_shared::ScreenResult::Flagged => {
            println!("{}", "  ⚠ FLAGGED — Manual review required".yellow().bold());
            if let Some(entity) = &result.matched_entity {
                println!("{}", format!("  Reason: {}", entity).yellow());
            }
        }
        odamp_shared::ScreenResult::Blocked => {
            println!("{}", "  ✗ BLOCKED — Sanctioned entity match".red().bold());
            if let Some(entity) = &result.matched_entity {
                println!("{}", format!("  Entity: {}", entity).red());
            }
        }
    }

    println!("{}", format!("  List:    {}", result.list_version).dimmed());
    println!("{}", format!("  Screened: {}", result.screened_at.to_rfc3339()).dimmed());

    Ok(())
}   