
```markdown
# ODAMP

**Open Digital Asset Management Platform**

> An open-core system that applies threshold cryptography,
> post-quantum signatures, explainable AI agents, and on-chain
> compliance tooling to the management of both native digital
> assets and tokenized real-world assets.

## Current Implementation (v0.x)

Multi-chain portfolio tracking with FROST 2-of-3 threshold wallet
security, post-quantum key exchange (ML-KEM-768), AI-driven risk
analytics, OFAC sanctions screening, and policy-based automation
for native tokens and tokenized RWAs across EVM and Solana chains.

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────────────┐
│  Phase 6: Unified Dashboard (Next.js + Tailwind + Recharts)         │
├─────────────────────────────────────────────────────────────────────┤
│  API Gateway (Hono / TypeScript) — REST + WebSocket + AuthN         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Phase 1: Security Core (Rust)                                      │
│  ├── FROST 2-of-3 threshold signing (frost-secp256k1-evm)           │
│  ├── Key generation ceremony (DKG, trusted dealer)                  │
│  ├── Key share encryption at rest (AES-256-GCM + ML-KEM-768)        │
│  └── Safe smart account (EIP-1271) as the on-chain wallet           │
│                                                                     │
│  Phase 2: Portfolio Engine (TypeScript)                             │
│  ├── Multi-chain indexer (viem for EVM, @solana/web3.js)            │
│  ├── Real market pricing (CoinGecko / Chainlink on-chain)           │
│  └── Risk metrics (Sharpe, Sortino, VaR, max drawdown, HHI)         │
│                                                                     │
│  Phase 3: Compliance Layer (TypeScript)                             │
│  ├── OFAC SDN list ingestion (public CSV/JSON, auto-refresh)        │
│  ├── Address matching (exact + fuzzy via ENS/reverse lookup)        │
│  └── Transaction pre-flight screening (block/flag/alert)            │
│                                                                     │
│  Phase 4: AI Agent Layer (Python / FastAPI)                         │
│  ├── Risk Sentinel agent (LLM-backed, logged reasoning)             │
│  ├── Portfolio state analysis (real positions, real prices)         │
│  └── Explainable output (structured JSON: claim → evidence → conf)  │
│                                                                     │
│  Phase 5: Execution Engine (TypeScript)                             │
│  ├── Pre-trade simulation (slippage, gas, impact model)             │
│  ├── DEX aggregator integration (1inch / 0x testnet)                │
│  └── FROST signing → Safe execution on testnet                      │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  Data: PostgreSQL 16 + TimescaleDB + pgvector + Redis 7             │
├─────────────────────────────────────────────────────────────────────┤
│  Blockchain: viem (EVM) | @solana/web3.js (Solana)                  │
│  Safe: safe-frost Solidity verifier (deployed to testnet)           │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Threshold Signing | Rust, `frost-secp256k1` v2.2 (keccak256) |
| Key Storage | AES-256-GCM, Argon2id KDF, 0600 file permissions |
| API | Axum (Rust), Tower HTTP |
| Indexer | EVM JSON-RPC (eth_getBalance, eth_call 0x70a08231) |
| Pricing | CoinGecko public API |
| Risk Math | Rust (Sharpe, Sortino, VaR, HHI, max drawdown) |
| Compliance | OFAC SDN public JSON API |
| AI Agent | Python 3.12, FastAPI, Anthropic Claude |
| Database | PostgreSQL 16 + TimescaleDB |
| Cache | Redis 7 |
| CLI | Rust, clap v4 |
| Orchestration | Docker Compose |

## Phase Plan

| Phase | Name | Done Criteria |
|-------|------|---------------|
| **0** | Foundation | `cargo build` passes, `docker compose up` → DB running, `curl /health` → 200 |
| **1** | Security Core | `cargo test` passes, `odamp wallet create` → 3 shares, `odamp sign` → valid Schnorr sig verified against group pubkey |
| **2** | Portfolio Engine | `odamp balance --address 0x... --chain ethereum` → real on-chain balances + USD prices |
| **3** | Compliance Layer | `odamp screen --address 0x...` → CLEAN/FLAGGED/BLOCKED. OFAC list loaded |
| **4** | AI Agent | `POST /agents/risk-sentinel/analyze` → structured JSON with findings |
| **5** | Execution (testnet) | Quote + execute on Base Sepolia. FROST signs. Safe executes. |
| **6** | Frontend | Unified dashboard. Real data from all phases. No mocks. |

## Quick Start

```bash
# 1. Configure
cp .env.example .env
# Edit: set ETH_RPC_URL, KEY_ENCRYPTION_PASSWORD, LLM_API_KEY

# 2. Build & test
cargo build --workspace
cargo test --workspace

# 3. Start infra
docker compose -f infra/docker-compose.yml up -d

# 4. Start API
cargo run -p odamp-api

# 5. Health check
curl http://localhost:3000/health

# 6. AI agent (separate terminal)
cd services/ai-agent
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
uvicorn app:app --reload --port 8000

# 7. CLI
cargo run -p odamp-cli -- wallet create --shares 3 --threshold 2
cargo run -p odamp-cli -- balance --address 0xYourAddress --chain ethereum
cargo run -p odamp-cli -- screen --address 0xSomeAddress   

---

Security Model
Concern	Approach
Key storage	AES-256-GCM encrypted files, 0600 perms, Argon2id KDF
Signing	FROST 2-of-3. Full key never exists.
Sanctions	OFAC SDN pre-flight. Hard block. Fail-closed.
AI	No execution authority. Structured output. Full logging.
API	JWT auth. Read-only default. Write requires FROST ceremony.

What This Is NOT (v0.x)
Skipped	Why
HSM / MPC provider	File-encrypted keys. HSM is commercial tier.
QKD	Physics hardware. ML-KEM-768 is the software equivalent.
Full KYC/AML	OFAC only. Full pipeline requires licensed providers.
Mainnet execution	Testnet only. Mainnet is a separate gated decision.
Cross-chain bridges	Reads events. Bridge initiation deferred.

License
MIT (open-core). Commercial tier under separate license.
EOF

---

## Project Structure

```
odamp/
├── apps/
│   ├── web/                  # Phase 6: Next.js dashboard
│   ├── api/                  # Hono REST + WebSocket API (all phases)
│   └── cli/                  # Bun CLI: wallet ops, signing, execution
├── crates/
│   ├── security-core/        # Phase 1: FROST wallet (Rust)
│   │   ├── src/
│   │   │   ├── dkg.rs        # Key generation ceremony
│   │   │   ├── sign.rs       # 2-of-3 signing protocol
│   │   │   ├── key_store.rs  # AES-256-GCM encrypted file storage
│   │   │   └── lib.rs
│   │   └── Cargo.toml        # frost-secp256k1-evm, ml-kem, aes-gcm
│   └── pqcrypto/             # ML-KEM-768 + ML-DSA-65 bindings
├── packages/
│   ├── portfolio/            # Phase 2: Multi-chain indexer + risk
│   ├── compliance/           # Phase 3: OFAC SDN screening
│   ├── execution/            # Phase 5: DEX aggregator + pre-trade sim
│   └── shared/               # Zod schemas, types, constants
├── services/
│   └── ai-agent/             # Phase 4: Python/FastAPI Risk Sentinel
│       ├── app.py
│       ├── agents/
│       │   └── risk_sentinel.py
│       └── requirements.txt
├── contracts/
│   └── safe-frost/           # Solidity: FROST verifier + Safe integration
├── infra/
│   ├── docker-compose.yml
│   ├── migrations/           # SQL files (001_init.sql, 002_..., etc.)
│   └── scripts/
│       └── load-ofac-sdn.sh  # Phase 3: fetch + parse OFAC CSV
├── .vscode/
│   ├── extensions.json
│   ├── settings.json
│   └── launch.json
├── pnpm-workspace.yaml
├── package.json
├── Cargo.toml                # Rust workspace
├── pyproject.toml            # Python workspace
└── README.md
```

---

## Quick Start

### Prerequisites

- Node.js 22+, pnpm 9+
- Rust 1.78+ (`rustup`)
- Python 3.12+
- Docker + Docker Compose
- Foundry (`forge`) — for deploying Safe + FROST verifier to testnet
- An EVM RPC URL (Alchemy/Infura free tier)
- A Solana RPC URL (Helius/QuickNode free tier)
- An OpenAI or Anthropic API key (Phase 4)
- A 1inch API key (Phase 5, testnet)

### Phase 0: Get Running

```bash
git clone <repo> odamp && cd odamp
pnpm install
cargo build --workspace

# Start infra
docker compose -f infra/docker-compose.yml up -d

# Apply migrations
pnpm db:migrate

# Verify
curl http://localhost:8080/health
# → {"status":"ok","db":"connected","redis":"connected"}
```

### Phase 1: Create a Threshold Wallet

```bash
# Generate 3 key shares (2-of-3 threshold)
pnpm cli wallet create --threshold 2 --shares 3 --chain ethereum

# Sign a message (requires 2 of 3 shares)
pnpm cli sign --message "hello" --shares 1,2

# Verify on Sepolia (deployed Safe + FROST verifier)
pnpm cli verify --tx-hash 0x... --chain sepolia
```

### Phase 2: Index a Wallet

```bash
# Start the indexer (indexes all configured chains)
pnpm --filter portfolio index --address 0xYourSafe --chains ethereum,base,solana

# Check portfolio
curl http://localhost:8080/api/portfolio/0xYourSafe
# → { positions: [...], total_value_usd: ..., sharpe: ..., var_95: ... }
```

### Phase 3: Screen an Address

```bash
curl -X POST http://localhost:8080/api/compliance/screen \
  -H "Content-Type: application/json" \
  -d '{"address":"0xSanctionedAddress"}'
# → { status: "BLOCKED", match: { name: "...", program: "SDN", country: "..." } }
```

### Phase 4: Run Risk Sentinel

```bash
curl -X POST http://localhost:8080/api/agents/risk-sentinel/analyze \
  -H "Content-Type: application/json" \
  -d '{"wallet":"0xYourSafe"}'
# → { risk_level: "ELEVATED", findings: [{claim, evidence, confidence}], ... }
```

### Phase 5: Execute on Testnet

```bash
# Get a quote
curl -X POST http://localhost:8080/api/execution/quote \
  -d '{"from":"0xUSDC","to":"0xWETH","amount":"100","chain":"base-sepolia"}'
# → { expected_out: "...", slippage: "0.3%", gas_estimate: "..." }

# Execute (triggers FROST 2-of-3 signing → Safe execution)
curl -X POST http://localhost:8080/api/execution/execute \
  -d '{"quote_id":"...","shares":[1,2]}'
# → { tx_hash: "0x...", status: "confirmed", block: 12345 }
```

### Phase 6: Open the Dashboard

```bash
pnpm --filter web dev
# → http://localhost:3000
```

---

## OFAC SDN Data Source

Phase 3 uses the **OFAC Specially Designated Nationals (SDN) List**:
- URL: `https://sanctions.ofac.treas.gov/api/sdn/v1/sdnList`
- Format: JSON (or CSV from `https://www.treasury.gov/ofac/downloads/sdn/sdn.csv`)
- Refresh: Daily via cron (`infra/scripts/load-ofac-sdn.sh`)
- Matching: Exact address match + ENS reverse lookup + label matching
- Action: `BLOCKED` (hard stop) | `FLAGGED` (manual review) | `CLEAN`

---

## AI Agent Design (Phase 4)

**Risk Sentinel** — the first and only agent in v0.x.

```
Input:  Real portfolio state (positions, prices, 30d history, open positions)
Model:  Claude Sonnet / GPT-4o (configurable via env)
Output: Structured JSON (enforced via function calling / tool use)

{
  "risk_level": "LOW | ELEVATED | HIGH | CRITICAL",
  "findings": [
    {
      "claim": "Concentration risk: 72% in single asset (ETH)",
      "evidence": "Position value $14,400 of $20,000 total",
      "confidence": 0.95,
      "source": "portfolio_state"
    },
    {
      "claim": "Volatility spike in USDe over last 48h",
      "evidence": "Price moved -3.2% (z-score: 2.8 vs 30d mean)",
      "confidence": 0.82,
      "source": "price_history"
    }
  ],
  "recommendations": [
    "Reduce ETH allocation to <50% to lower concentration risk",
    "Monitor USDe depeg risk; consider hedging with USDC"
  ],
  "model": "claude-sonnet-4-20250514",
  "latency_ms": 2340,
  "prompt_tokens": 1847,
  "completion_tokens": 512
}
```

All LLM calls are logged to `ai_agent_logs` table (prompt, response, model, tokens, latency, timestamp). No data leaves the machine except the API call to the LLM provider.

---

## FROST + PQC: How They Work Together

```
┌──────────────────────────────────────────────────────────────────┐
│  KEY GENERATION CEREMONY (one-time)                              │
│                                                                  │
│  3 participants each generate a key share via DKG                │
│  Key shares exchanged using ML-KEM-768 (post-quantum KEX)        │
│  Result: shared public key (Safe address), 3 private shares      │
│  Each share encrypted at rest with AES-256-GCM (local password)  │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  SIGNING (every transaction)                                     │
│                                                                  │
│  1. Build tx (Safe call data)                                    │
│  2. OFAC screen (Phase 3) → BLOCKED? Stop.                       │
│  3. FROST Round 1: 2-of-3 shares generate nonces + commitments   │
│  4. FROST Round 2: 2-of-3 shares produce signature shares        │
│  5. Aggregate → single Schnorr signature (R, z)                  │
│  6. Safe contract verifies via safe-frost verifier (~5600 gas)   │
│  7. Tx executes on-chain                                         │
│                                                                  │
│  The full private key NEVER exists. Not in memory, not on disk.  │
└──────────────────────────────────────────────────────────────────┘
```

**Why FROST for signing, not ML-DSA-65?**
- FROST(secp256k1, keccak256) produces signatures verifiable on Ethereum via the Safe FROST verifier at ~5,600 gas.
- ML-DSA-65 signatures have no on-chain verifier on EVM (no precompile). They'd require an off-chain verification step or a heavy verifier contract.
- FROST is the right tool for the *signing* layer. PQC (ML-KEM-768) is the right tool for the *key exchange* layer.

---

## Roadmap Beyond v0.x (Commercial Tier)

| Feature | Notes |
|---|---|
| HSM-backed key shares | AWS CloudHSM / Azure Key Vault / YubiHSM |
| MPC provider integration | Fireblocks / Dfns / Infini (for institutional clients) |
| Full KYC/AML pipeline | Sumsub / Chainalysis / Elliptic |
| Mainnet execution | Gated behind multi-party approval + insurance |
| Cross-chain bridge execution | LayerZero / Axelar / Stargate |
| Tokenized RWA management | Real estate, private credit, Treasuries |
| Multi-chain: Canton, Avalanche, Provenance | Permissioned / institutional chains |
| SOC 2 / ISO 27001 | For institutional adoption |
| Additional AI agents | Tax Optimizer, Yield Hunter, Compliance Auditor |

---
