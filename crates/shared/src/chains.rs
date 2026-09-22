pub struct ChainConfig {
    pub name: &'static str,
    pub chain_id: u64,
    pub rpc_env_var: &'static str,
    pub is_testnet: bool,
}

pub const ETHEREUM: ChainConfig = ChainConfig {
    name: "ethereum",
    chain_id: 1,
    rpc_env_var: "ETH_RPC_URL",
    is_testnet: false,
};

pub const BASE: ChainConfig = ChainConfig {
    name: "base",
    chain_id: 8453,
    rpc_env_var: "BASE_RPC_URL",
    is_testnet: false,
};

pub const ARBITRUM: ChainConfig = ChainConfig {
    name: "arbitrum",
    chain_id: 42161,
    rpc_env_var: "ARBITRUM_RPC_URL",
    is_testnet: false,
};

pub const BSC: ChainConfig = ChainConfig {
    name: "bsc",
    chain_id: 56,
    rpc_env_var: "BSC_RPC_URL",
    is_testnet: false,
};

pub const SEPOLIA: ChainConfig = ChainConfig {
    name: "sepolia",
    chain_id: 11155111,
    rpc_env_var: "SEPOLIA_RPC_URL",
    is_testnet: true,
};

pub const BASE_SEPOLIA: ChainConfig = ChainConfig {
    name: "base-sepolia",
    chain_id: 84532,
    rpc_env_var: "BASE_SEPOLIA_RPC_URL",
    is_testnet: true,
};

pub fn get_chain(name: &str) -> Option<&'static ChainConfig> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" => Some(&ETHEREUM),
        "base" => Some(&BASE),
        "arbitrum" | "arb" => Some(&ARBITRUM),
        "bsc" | "binance" => Some(&BSC),
        "sepolia" => Some(&SEPOLIA),
        "base-sepolia" | "base_sepolia" => Some(&BASE_SEPOLIA),
        _ => None,
    }
}

/// Well-known tokens for initial asset registry
pub struct TokenInfo {
    pub symbol: &'static str,
    pub name: &'static str,
    pub address: &'static str,
    pub decimals: u8,
    pub coingecko_id: &'static str,
}

pub const USDC_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "USDC",
    name: "USD Coin",
    address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    decimals: 6,
    coingecko_id: "usd-coin",
};

pub const USDT_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "USDT",
    name: "Tether USD",
    address: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
    decimals: 6,
    coingecko_id: "tether",
};

pub const WBTC_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "WBTC",
    name: "Wrapped Bitcoin",
    address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
    decimals: 8,
    coingecko_id: "wrapped-bitcoin",
};

pub const DAI_ETHEREUM: TokenInfo = TokenInfo {
    symbol: "DAI",
    name: "Dai Stablecoin",
    address: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
    decimals: 18,
    coingecko_id: "dai",
};   