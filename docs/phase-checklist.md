---
# ODAMP — Complete Project Inventory

```bash
cat > docs/phase-checklist.md << 'EOF'
# ODAMP Phase Checklist

## Phase 0: Foundation
- [ ] `cargo build --workspace` compiles clean
- [ ] `cargo test --workspace` passes
- [ ] `docker compose up` → Postgres + Redis running
- [ ] Migrations 001–006 applied
- [ ] `curl localhost:3000/health` → 200
- [ ] CI passes on push

## Phase 1: Security Core
- [ ] `generate_key_shares(3, 2, "pw")` → 3 encrypted shares + group pubkey
- [ ] `sign_message(msg, shares[1,2], pws, 2)` → 64-byte hex signature
- [ ] Signature verifies against group pubkey via `VerifyingKey::verify`
- [ ] Wrong password → decryption error
- [ ] 1 share (below threshold) → `InsufficientShares` error
- [ ] `odamp wallet create` CLI works end-to-end
- [ ] `odamp sign` CLI works end-to-end
- [ ] Key files in `~/.odamp/keys/` have 0600 permissions
- [ ] No full private key exists in any file or process

## Phase 2: Portfolio Engine
- [ ] `EvmClient::get_native_balance` returns correct wei from public RPC
- [ ] `EvmClient::get_erc20_balance` returns correct balance for USDC/USDT/WBTC/DAI
- [ ] `PriceClient::get_prices` returns USD prices from CoinGecko
- [ ] `compute_risk_metrics` returns correct Sharpe/VaR/HHI for known inputs
- [ ] `odamp balance --address 0x... --chain ethereum` shows real data
- [ ] Positions upsert into Postgres
- [ ] Portfolio endpoint returns aggregated positions + total value

## Phase 3: Compliance Layer
- [ ] `load-ofac-sdn.sh` fetches and loads SDN list into Postgres
- [ ] `ComplianceEngine::screen` with known SDN address → BLOCKED
- [ ] `ComplianceEngine::screen` with clean address → CLEAN
- [ ] Empty list (not loaded) → FLAGGED (fail-closed)
- [ ] `odamp screen --address 0x...` CLI works
- [ ] Screening results logged to `compliance_logs` table
- [ ] List version tracked in `sdn_list_meta`

## Phase 4: AI Agent Layer
- [ ] `uvicorn app:app --port 8000` starts clean
- [ ] `POST /analyze` with portfolio state → structured JSON response
- [ ] Response includes: risk_level, findings[], recommendations[], data_gaps[]
- [ ] Each finding has: claim, evidence, confidence, source, severity
- [ ] LLM call logged (prompt_hash, model, tokens, latency)
- [ ] Invalid LLM response → graceful fallback (parse_error finding)
- [ ] `pytest tests/ -v` passes (mocked LLM)

## Phase 5: Execution (Testnet)
- [ ] Quote endpoint returns slippage + gas estimate
- [ ] OFAC screen runs before signing
- [ ] FROST 2-of-3 signs the Safe tx
- [ ] Safe executes on Base Sepolia
- [ ] Tx visible on Blockscout
- [ ] Execution logged to `execution_logs`
- [ ] Portfolio re-indexed after execution

## Phase 6: Frontend
- [ ] Next.js app builds and runs
- [ ] Dashboard shows portfolio (Phase 2 data)
- [ ] Security page shows wallet status (Phase 1)
- [ ] Compliance page shows screening history (Phase 3)
- [ ] AI page shows Risk Sentinel report (Phase 4)
- [ ] Execution page shows trade history (Phase 5)
- [ ] No mock data. All real API calls.
EOF   