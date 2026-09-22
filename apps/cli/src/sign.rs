use colored::Colorized;
use std::collections::HashMap;

pub fn sign(message_hex: &str, shares_str: &str) -> anyhow::Result<()> {
    let password = std::env::var("KEY_ENCRYPTION_PASSWORD")
        .map_err(|_| anyhow::anyhow!("KEY_ENCRYPTION_PASSWORD not set"))?;

    let message_bytes = hex::decode(message_hex)
        .map_err(|e| anyhow::anyhow!("Invalid hex message: {}", e))?;

    let share_ids: Vec<u32> = shares_str
        .split(',')
        .map(|s| s.trim().parse::<u32>().map_err(|e| anyhow::anyhow!("Invalid share ID: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    println!("{}", "ODAMP — FROST Signing".bold());
    println!("{}", format!("  Shares: {:?}", share_ids).dimmed());
    println!("{}", format!("  Message: {} bytes", message_bytes.len()).dimmed());
    println!();

    // Load and decrypt shares
    let store = odamp_security_core::KeyStore::default_location()?;
    let mut encrypted_shares: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut passwords: HashMap<u32, String> = HashMap::new();

    for id in &share_ids {
        let file = store.load_share(*id)?;
        let decrypted = base64::decode(&file.ciphertext)
            .map_err(|e| anyhow::anyhow!("Failed to decode share {}: {}", id, e))?;
        encrypted_shares.push((*id, decrypted));
        passwords.insert(*id, password.clone());
    }

    println!("{}", "Running FROST signing ceremony...".yellow());

    let result = odamp_security_core::sign_message(
        &message_bytes,
        &encrypted_shares.iter().map(|(id, bytes)| (*id, bytes.as_slice())).collect::<Vec<_>>(),
        &passwords.iter().map(|(k, v)| (*k, v.as_str())).collect::<HashMap<u32, &str>>(),
        2, // threshold
    )?;

    println!();
    println!("{}", "✓ Signature produced".green());
    println!("{}", format!("  Signature:    {}", result.signature).bold());
    println!("{}", format!("  Msg Hash:     {}", result.message_hash).dimmed());
    println!("{}", format!("  Shares Used:  {:?}", result.shares_used).dimmed());

    Ok(())
}   