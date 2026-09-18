# ODAMP — Phase 0 + 1 + 2

Per your instruction, Phases were built back-to-back without verifying each
one compiles first. That means Phase 2 (below) is built on top of Phase 1
code that has not been run yet. If Phase 1 has a bug, it may show up as a
confusing Phase 2 error instead of a clean Phase 1 error — when you do start
debugging, it's worth building `core/security`, `core/portfolio`, and
`core/compliance` individually first (`cargo build -p odamp-security` etc.)
before debugging the full `api` binary, so you know which layer an error is
actually coming from.

## Phase 2: real on-chain balances + real prices

New crate: `core/indexer`. Two real, non-mocked integrations:

1. **`EvmClient`** — reads native and ERC-20 balances directly from any EVM
   JSON-RPC endpoint via raw JSON-RPC calls (`eth_getBalance`, and
   `eth_call` against `balanceOf(address)` for ERC-20 tokens). You supply
   the RPC URL — a public endpoint, or your own Alchemy/Infura/QuickNode
   URL.
2. **`PriceClient`** — reads real USD prices from CoinGecko's public API
   (no key required for light usage as of this writing).

New endpoint: `POST /api/v1/portfolio/:user_id/sync`. You give it a list of
assets to check (chain, address, whether it's native or ERC-20, CoinGecko
id), and it fetches the real balance + real price for each and upserts the
resulting position into Postgres — this replaces the manual
`POST /positions` calls from Phase 1 with actual data.

### Trying it

You need an RPC URL. Easiest free option: a public Ethereum RPC like
`https://eth.llamarpc.com` (no signup, but not guaranteed uptime/rate limits
— fine for testing, get a real Alchemy/Infura key before relying on this).

```bash
curl -s -X POST "http://localhost:8080/api/v1/portfolio/$USER_ID/sync" \
  -H "Content-Type: application/json" \
  -d '[
    {
      "rpc_url": "https://eth.llamarpc.com",
      "chain": "ethereum",
      "holder_address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
      "kind": "native",
      "symbol": "ETH",
      "asset_type": "crypto",
      "coingecko_id": "ethereum",
      "avg_cost_usd": 2500
    },
    {
      "rpc_url": "https://eth.llamarpc.com",
      "chain": "ethereum",
      "holder_address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
      "kind": "erc20",
      "token_contract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "decimals": 6,
      "symbol": "USDC",
      "asset_type": "stablecoin",
      "coingecko_id": "usd-coin",
      "avg_cost_usd": 1.0
    }
  ]'
```

(That address is Ethereum's well-known "vitalik.eth" — public information,
used here only as a real address to test against; swap in your own.)

Then re-check the portfolio — it now reflects real on-chain state:

```bash
curl -s "http://localhost:8080/api/v1/portfolio/$USER_ID"
```

Re-running the same `sync` call updates the same positions (via a
deterministic position ID derived from user + chain + symbol) rather than
creating duplicates.

## Known simplifications in Phase 2

- **No auto-discovery.** You must tell it which assets to check — it
  doesn't walk a wallet and find everything it holds. That requires either
  a paid indexing API (Alchemy token-balances, etc.) or querying many token
  contracts one at a time. Reasonable Phase 3 addition, not done here.
- **ERC-20 decimals must be supplied manually.** Fetching a token's
  `decimals()` on-chain automatically is a small addition, skipped for
  Phase 2 scope.
- **No caching/rate-limit handling** for the price or RPC calls — syncing
  many assets quickly could hit CoinGecko's or a public RPC's rate limits.
- **This was written and never run** (no network access in the sandbox
  building it) against either a real RPC endpoint or CoinGecko's live API.
  The JSON-RPC and REST shapes are correct per their public docs as I
  understand them, but you are the first real test.

## Everything from Phase 0 and 1 still applies

See the crate-level doc comments in `core/security/src/lib.rs`,
`core/compliance/src/lib.rs`, and `api/src/db.rs` for the running list of
what's real vs. placeholder vs. flagged-for-later in each of those layers.

## Next steps

- Wire the real OFAC SDN feed into `core/compliance` (still a placeholder)
- Scaffold `frontend/web` as a Next.js dashboard against this API
- Add ERC-20 auto-decimals + basic multi-asset auto-discovery to the indexer
