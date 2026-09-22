use colored::Colorize;
use odamp_security_core::KeyStore;

pub fn create(total_shares: u32, threshold: u32, chain: &str) -> anyhow::Result<()> {
    let password = std::env::var("KEY_ENCRYPTION_PASSWORD")
        .map_err(|_| anyhow::anyhow!("KEY_ENCRYPTION_PASSWORD not set in environment"))?;

    println!("{}", "ODAMP — Threshold Wallet Creation".bold());
    println!("{}", format!("  Shares:    {}", total_shares).dimmed());
    println!("{}", format!("  Threshold: {}", threshold).dimmed());
    println!("{}", format!("  Chain:     {}", chain).dimmed());
    println!();

    println!("{}", "Running DKG ceremony...".yellow());
    let result = odamp_security_core::generate_key_shares(total_shares, threshold, &password)?;

    // Store shares on disk
    let store = KeyStore::default_location()?;
    for share in &result.shares {
        let file = odamp_security_core::EncryptedShareFile {
            share_id: share.share_id,
            ciphertext: share.encrypted_share.clone(),
            nonce: String::new(), // embedded in ciphertext for v0.x
            salt: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        store.save_share(&file)?;
    }

    println!();
    println!("{}", "✓ Wallet created successfully".green());
    println!("{}", format!("  Group Public Key: {}", result.group_public_key).bold());
    println!("{}", format!("  Shares stored in:  ~/.odamp/keys/").dimmed());
    println!();
    println!("{}", "⚠  Store the key share files securely.".red().bold());
    println!("{}", "   You will need at least {} of {} to sign.".yellow(), threshold, total_shares);

    Ok(())
}

pub fn list() -> anyhow::Result<()> {
    let store = KeyStore::default_location()?;
    let shares = store.list_shares();

    if shares.is_empty() {
        println!("{}", "No key shares found in ~/.odamp/keys/".yellow());
        return Ok(());
    }

    println!("{}", "Stored Key Shares:".bold());
    for id in shares {
        println!("  share_{}.json", id);
    }

    Ok(())
}   