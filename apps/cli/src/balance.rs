use colored::Colorized;

pub async fn check(address: &str, chain: &str) -> anyhow::Result<()> {
    let chain_config = odamp_shared::get_chain(chain)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {}", chain))?;

    let rpc_url = std::env::var(chain_config.rpc_env_var)
        .map_err(|_| anyhow::anyhow!("RPC URL not set ({} in .env)", chain_config.rpc_env_var))?;

    println!("{}", "ODAMP — Balance Check".bold());
    println!("{}", format!("  Address: {}", address).dimmed());
    println!("{}", format!("  Chain:   {}", chain).dimmed());
    println!();

    let client = odamp_indexer::EvmClient::new(&rpc_url);

    // Native balance
    let native = client.get_native_balance(address).await?;
    let native_f64: f64 = native.parse::<u128>().unwrap_or(0) as f64 / 1e18;

    println!("{}", format!("  Native:  {:.6} {}", native_f64, chain_config.name).green());

    // Check known tokens
    let tokens = [
        odamp_shared::USDC_ETHEREUM,
        odamp_shared::USDT_ETHEREUM,
        odamp_shared::WBTC_ETHEREUM,
        odamp_shared::DAI_ETHEREUM,
    ];

    for token in &tokens {
        if let Ok(balance) = client.get_erc20_balance(token.address, address).await {
            let bal_f64: f64 = balance.parse::<u128>().unwrap_or(0) as f64 / 10f64.powi(token.decimals as i32);
            if bal_f64 > 0.0 {
                println!("{}", format!("  {:<10} {:.4}", token.symbol, bal_f64).green());
            }
        }
    }

    Ok(())
}   