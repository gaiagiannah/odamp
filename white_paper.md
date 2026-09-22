```markdown
# ODAMP: Open Digital Asset Management Platform

**A Whitepaper on Threshold-Cryptographic Custody, Post-Quantum Key Exchange, and AI-Driven Portfolio Intelligence for Native and Tokenized Real-World Assets**

---

## Abstract

ODAMP is an open-core, self-hosted platform for managing digital asset portfolios across EVM and Solana chains. It combines FROST 2-of-3 threshold signing (secp256k1, keccak256) with post-quantum key exchange (ML-KEM-768), real-time multi-chain portfolio indexing, OFAC sanctions screening, LLM-backed risk analysis with logged explainable reasoning, and policy-based automated execution on testnet environments. The platform is designed for individual professionals and small teams who require institutional-grade security primitives without institutional-grade vendor lock-in. All components are MIT-licensed; a commercial tier will add HSM-backed custody, mainnet execution, and managed infrastructure.

---

## 1. The Problem

### 1.1 The Custody Gap

The digital asset landscape presents a binary choice for individual professionals and small teams:

| Option | Security | Cost | Autonomy |
|---|---|---|---|
| Exchange (Coinbase, Kraken) | Custodial — you don't hold keys | Low | None. KYC, withdrawal limits, platform risk |
| Self-custody (MetaMask, Phantom) | Non-custodial — you hold keys | Low | Full, but single-key = single point of failure |
| Institutional (Fireblocks, Dfns, Copper) | MPC/HSM, multi-sig, compliance | $50K–$500K+/yr | Full, but enterprise-only pricing and contracts |

**No option exists** for a professional managing $100K–$5M in digital assets who wants:
- No single point of key failure
- Post-quantum key exchange
- Real-time portfolio visibility across chains
- Automated compliance screening
- AI-assisted risk analysis
- Policy-based execution without a $200K/yr vendor contract

### 1.2 The Security Problem

Standard self-custody wallets use a single private key. If that key is compromised — via malware, phishing, or physical theft — the entire portfolio is lost. There is no recovery, no multi-party approval, no audit trail.

Post-quantum cryptography (PQC) has been finalized by NIST (FIPS 203: ML-KEM, FIPS 204: ML-DSA) but has not been integrated into any mainstream wallet product. The key *exchange* layer of wallet operations remains vulnerable to a future quantum adversary who could intercept and decrypt classical key exchange traffic.

### 1.3 The Intelligence Gap

Existing portfolio tools (Zerion, DeBank, Revert Finance) provide:
- Balance aggregation
- P&L tracking
- Basic allocation views

They do **not** provide:
- Anomaly detection on wallet activity
- Risk scoring with explainable reasoning
- Automated compliance screening (OFAC, sanctions)
- Policy-based execution (if X then Y)
- LLM-backed analysis that logs its reasoning for audit

### 1.4 The Tokenized RWA Problem

Tokenized real-world assets (Treasuries, money market funds, private credit, real estate) are growing from ~$36B (2025) to an estimated $5.5T–$11T by 2030. But there is no self-hosted tool that:
- Tracks tokenized RWA positions alongside native tokens
- Screens RWA counterparties against sanctions lists
- Provides unified risk metrics across both asset classes
- Automates rebalancing between native and tokenized positions

---

## 2. The Solution

### 2.1 What ODAMP Is

ODAMP is a **self-hosted, open-core digital asset management platform** that provides:

1. **Threshold cryptographic custody** — FROST 2-of-3 signing on secp256k1 (keccak256). No full private key ever exists. The on-chain wallet is a Safe smart account verified via the `safe-frost` EIP-1271 verifier (~5,600 gas).

2. **Post-quantum key exchange** — ML-KEM-768 (NIST FIPS 204) for the key generation ceremony (DKG). Hybrid with X25519 for forward compatibility. Key shares encrypted at rest with AES-256-GCM.

3. **Multi-chain portfolio engine** — Real-time indexing of native tokens, ERC-20s, SPL tokens, and tokenized RWAs across Ethereum, Base, Arbitrum, BSC, and Solana. Live pricing via CoinGecko (fallback: Chainlink on-chain). Risk metrics: Sharpe, Sortino, VaR(95), max drawdown, Herfindahl-Hirschman Index (concentration).

4. **OFAC sanctions screening** — Pre-flight screening of all transaction counterparties against the OFAC SDN list (public, auto-refreshed daily). Hard block on match. Full audit log.

5. **AI risk agent (Risk Sentinel)** — LLM-backed analysis of real portfolio state. Produces structured, explainable output: each finding includes a claim, evidence, confidence score, and source. All LLM calls logged (prompt, response, model, tokens, latency).

6. **Policy-based execution** — Deterministic trigger→action engine. Pre-trade simulation (slippage, gas, impact). DEX aggregator integration (1inch testnet). FROST 2-of-3 signing → Safe execution. All execution on testnet in v0.x.

### 2.2 What ODAMP Is Not

- Not an exchange. It does not hold user funds on behalf of users.
- Not a custodian. It does not require a custodian license.
- Not a DeFi protocol. It does not issue tokens or create liquidity.
- Not a compliance vendor. OFAC SDN screening is included; full KYC/AML/MiCA reporting is a commercial-tier concern.
- Not a mainnet execution platform in v0.x. All execution is on testnet.

---

## 3. Architecture

### 3.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│  Phase 6: Unified Dashboard (Next.js 15 + Tailwind + Recharts)     │
├─────────────────────────────────────────────────────────────────────┤
│  API Gateway (Hono / TypeScript / Node 22)                         │
│  REST + WebSocket + JWT AuthN + Rate Limiting                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  Phase 1: Security Core (Rust)                                │ │
│  │  • FROST 2-of-3 threshold signing (frost-secp256k1-evm)      │ │
│  │  • DKG ceremony (ML-KEM-768 key exchange)                     │ │
│  │  • Key share encryption (AES-256-GCM)                         │ │
│  │  • Safe smart account (EIP-1271)                              │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  Phase 2: Portfolio Engine (TypeScript)                       │ │
│  │  • Multi-chain indexer (viem EVM + @solana/web3.js)          │ │
│  │  • Market pricing (CoinGecko / Chainlink)                     │ │
│  │  • Risk metrics (Sharpe, VaR, HHI, drawdown)                 │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  Phase 3: Compliance Layer (TypeScript)                       │ │
│  │  • OFAC SDN ingestion (daily auto-refresh)                    │ │
│  │  • Address matching (exact + ENS reverse)                     │ │
│  │  • Pre-flight screening (BLOCK / FLAG / CLEAN)               │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  Phase 4: AI Agent Layer (Python 3.12 / FastAPI)             │ │
│  │  • Risk Sentinel (LLM-backed, structured output)             │ │
│  │  • Logged reasoning (claim → evidence → confidence)          │ │
│  │  • Portfolio state ingestion (real positions + prices)       │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  Phase 5: Execution Engine (TypeScript)                       │ │
│  │  • Pre-trade simulation (slippage, gas, impact)              │ │
│  │  • DEX aggregator (1inch testnet)                             │ │
│  │  • FROST signing → Safe execution → confirmation             │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  Data: PostgreSQL 16 + TimescaleDB + pgvector + Redis 7            │
├─────────────────────────────────────────────────────────────────────┤
│  Blockchain: viem (EVM) | @solana/web3.js (Solana)                 │
│  Contracts: Safe + safe-frost verifier (deployed to testnet)       │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Design Principles

| Principle | Rationale |
|---|---|
| **Self-hosted by default** | No data leaves the machine except LLM API calls (Phase 4) and RPC reads. The user controls the blast radius. |
| **Threshold over single-key** | No full private key ever exists. 2-of-3 FROST means no single device, file, or process can move funds. |
| **Post-quantum where it matters** | ML-KEM-768 for key exchange (the layer a quantum adversary would target). FROST secp256k1 for signing (on-chain verifiable, no PQC precompile needed). |
| **Explainable AI, not black-box** | Every AI finding includes claim, evidence, confidence, and source. All LLM calls logged. No opaque "the AI says sell." |
| **Compliance as a gate, not an afterthought** | OFAC screening happens *before* signing. A blocked transaction never reaches the FROST ceremony. |
| **Testnet-first** | v0.x executes only on testnet. Mainnet gating is a deliberate, separate decision requiring additional safeguards. |
| **Open-core** | Full platform is MIT. Commercial tier adds HSM, mainnet, managed infra. No feature is locked behind a login wall in the open version. |

### 3.3 Why FROST, Not ML-DSA-65, for Signing

This is a deliberate architectural choice:

| Property | FROST (secp256k1, keccak256) | ML-DSA-65 |
|---|---|---|
| On-chain verifier (EVM) | ✅ `safe-frost` contract, ~5,600 gas | ❌ No precompile, no efficient verifier |
| Signature size | 65 bytes (R, z) | ~2,420 bytes |
| Gas cost to verify | ~5,600 | Est. 500K+ (if a verifier existed) |
| Threshold capability | ✅ 2-of-3, 3-of-5, etc. | ❌ No threshold variant |
| Post-quantum | ❌ (secp256k1 is classical) | ✅ |
| Standard | ZCash FROST spec + Safe Research EVM variant | NIST FIPS 204 |

**Resolution**: Use FROST for the *signing* layer (where on-chain verifiability is non-negotiable) and ML-KEM-768 for the *key exchange* layer (where post-quantum resistance is the primary threat model). This is a hybrid approach that matches the actual threat: a quantum adversary targets key exchange, not on-chain signature verification (which is public and doesn't need to remain secret).

---

## 4. Security Model

### 4.1 Threat Model

| Threat | Mitigation |
|---|---|
| Single device compromise | 2-of-3 FROST: one compromised share is insufficient to sign |
| Key share theft (file) | AES-256-GCM encryption at rest. Local password. No key in DB. |
| Quantum adversary intercepting DKG traffic | ML-KEM-768 (FIPS 204) for key exchange during ceremony |
| Malicious counterparty | OFAC SDN pre-flight screening. Hard block on match |
| Phishing / social engineering | Safe smart account: transactions are structured calls, not raw EOA transfers. Multi-sig approval required |
| API compromise | JWT auth, rate limiting, read-only default. Write ops require FROST ceremony |
| LLM prompt injection (Phase 4) | Structured output enforcement (function calling). LLM has no execution capability — it produces *signals*, not actions. Human or policy engine must approve |
| Supply chain (dependencies) | `pnpm audit`, `cargo audit`, `pip audit` in CI. Lock files committed. |

### 4.2 Key Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│  1. DKG CEREMONY (one-time, requires 2-of-3 participants)      │
│                                                                  │
│  Each participant generates a local key pair.                   │
│  Public shares exchanged via ML-KEM-768 encapsulation.          │
│  Result: shared public key → Safe contract address.             │
│  3 private shares distributed. Each encrypted with AES-256-GCM  │
│  (password-derived key via Argon2id). Stored in ~/.odamp/keys/  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  2. SIGNING (every transaction)                                 │
│                                                                  │
│  1. Build Safe tx (call data, to, value, nonce)                 │
│  2. OFAC screen counterparty → BLOCKED? Abort.                 │
│  3. FROST Round 1: 2 shares → nonces + commitments (broadcast)  │
│  4. FROST Round 2: 2 shares → signature shares                 │
│  5. Aggregate → (R, z) Schnorr signature                        │
│  6. Submit to Safe contract (safe-frost verifier, ~5600 gas)   │
│  7. Safe executes on-chain                                      │
│                                                                  │
│  The full private key NEVER exists. Not in memory, not on disk, │
│  not in any single process.                                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  3. KEY ROTATION (optional, periodic)                           │
│                                                                  │
│  Re-run DKG with same or new participants.                      │
│  Deploy new Safe contract (or upgrade via Safe governance).     │
│  Old key shares invalidated (encrypted files deleted).          │
└─────────────────────────────────────────────────────────────────┘
```

### 4.3 Audit Trail

Every operation produces a log entry:

| Event | Logged Fields |
|---|---|
| Key generation | Timestamp, participants, ML-KEM parameters, resulting Safe address |
| Signing ceremony | Timestamp, shares used (IDs, not contents), tx hash, gas |
| OFAC screen | Timestamp, address screened, result (BLOCK/FLAG/CLEAN), matched entity (if any), SDN list version |
| AI analysis | Timestamp, model, prompt hash, response, tokens, latency, findings |
| Execution | Timestamp, quote ID, slippage, gas, tx hash, block, status |
| Policy trigger | Timestamp, rule ID, trigger condition met, action taken |

All logs are append-only in PostgreSQL. No deletion. Exportable to CSV/JSON for external audit.

---

## 5. Technical Differentiation

### 5.1 vs. Existing Self-Custody Wallets

| Feature | MetaMask / Phantom | Safe (Gnosis) | ODAMP |
|---|---|---|---|
| Single key | ✅ (vulnerability) | ❌ (multi-sig) | ❌ (FROST 2-of-3) |
| Threshold crypto | ❌ | ❌ (ECDSA multi-sig) | ✅ (FROST) |
| Post-quantum key exchange | ❌ | ❌ | ✅ (ML-KEM-768) |
| Multi-chain | Limited | EVM only | EVM + Solana |
| OFAC screening | ❌ | ❌ | ✅ |
| AI risk analysis | ❌ | ❌ | ✅ |
| Policy-based automation | ❌ | Limited (modules) | ✅ (deterministic engine) |
| Self-hosted | ✅ | ✅ (Safe{Core} Data) | ✅ |
| Cost | Free | Free (open) + optional paid | Free (MIT) |

### 5.2 vs. Institutional Custody (Fireblocks, Dfns, Copper)

| Feature | Fireblocks / Dfns | ODAMP |
|---|---|---|
| Threshold crypto | ✅ (MPC) | ✅ (FROST) |
| HSM-backed | ✅ | ❌ (v0.x: file-encrypted) |
| Mainnet execution | ✅ | ❌ (v0.x: testnet only) |
| Compliance (KYC/AML) | ✅ | Partial (OFAC only) |
| Cost | $50K–$500K+/yr | $0 (self-hosted) |
| Vendor lock-in | High | None (MIT, self-hosted) |
| On-prem option | Limited (enterprise) | Full (Docker Compose) |
| AI risk analysis | ❌ (not a feature) | ✅ |
| Explainable reasoning | ❌ | ✅ (logged) |

### 5.3 The Core Innovation

ODAMP's innovation is not any single component. It is the **integration of threshold cryptography, post-quantum key exchange, compliance screening, and explainable AI into a single self-hosted, open-source tool** that a professional can run on their own hardware with zero vendor dependency.

No existing product combines all five:
1. FROST threshold signing (on-chain verifiable)
2. PQC key exchange (ML-KEM-768)
3. OFAC pre-flight screening
4. LLM-backed explainable risk analysis
5. Policy-based automated execution

Each component exists in isolation (FROST in ZCash, ML-KEM in OpenSSL, OFAC screening in Chainalysis, AI analysis in various DeFi tools, execution in 1inch). ODAMP is the **integration layer** that makes them work together in a coherent, auditable, self-hosted system.

---

## 6. The AI Agent: Risk Sentinel

### 6.1 Design Philosophy

The AI agent does **not** execute. It **advises**. This is a deliberate boundary:

```
AI Agent (Phase 4)          Policy Engine (Phase 5)         Execution
─────────────────           ─────────────────────           ─────────
Analyzes portfolio  ──→     Evaluates trigger rules  ──→    FROST signs
Produces signals          If signal matches rule              Safe executes
Logs reasoning            Action: alert / rebalance / freeze  On-chain tx
No execution authority     Deterministic, auditable           Confirmed
```

The AI can say "concentration risk is high, consider reducing ETH." The policy engine decides whether that signal triggers an alert, a rebalancing trade, or nothing. A human can override at any step.

### 6.2 Explainability Requirement

Every finding must include:

```json
{
  "claim": "Concentration risk: 72% in single asset (ETH)",
  "evidence": "Position value $14,400 of $20,000 total. HHI = 0.58 (threshold: 0.40)",
  "confidence": 0.95,
  "source": "portfolio_state + hhi_calculation",
  "recommendation": "Reduce ETH to <50% allocation",
  "severity": "ELEVATED"
}
```

The `evidence` field must reference specific data points from the portfolio state. The `source` field identifies which computation or data feed produced the finding. This makes the AI's reasoning **auditable** — a user (or external auditor) can verify the claim against the raw data.

### 6.3 Logging

Every LLM call is logged:

| Field | Purpose |
|---|---|
| `timestamp` | When |
| `model` | Which model (claude-sonnet-4, gpt-4o, etc.) |
| `prompt_hash` | SHA-256 of the full prompt (reproducibility) |
| `prompt_tokens` | Cost tracking |
| `completion_tokens` | Cost tracking |
| `latency_ms` | Performance |
| `response` | Full structured output |
| `findings_count` | How many findings produced |
| `max_severity` | Highest severity level in response |

This creates a complete audit trail of AI reasoning. If a rebalancing decision is questioned, the full prompt and response are available.

---

## 7. Compliance: OFAC SDN Screening

### 7.1 Scope

ODAMP v0.x performs **OFAC Specially Designated Nationals (SDN) List** screening only. This is:
- The most critical sanctions list for US persons
- Publicly available (no API key required)
- Updated regularly by the Treasury Department
- Sufficient for individual professionals and small teams

It does **not** include:
- EU consolidated sanctions list
- UN Security Council lists
- UK HMT lists
- Sectoral sanctions (SDP, CAPTA, etc.)
- Full KYC/AML identity verification

These are commercial-tier concerns.

### 7.2 Matching Logic

```
Input: Counterparty address (EVM or Solana)

1. Exact match against SDN address list → BLOCKED
2. ENS reverse lookup (EVM) → name match against SDN names → FLAGGED
3. Label match (if address has a known label in the indexer) → FLAGGED
4. No match → CLEAN

BLOCKED: Transaction aborts. No signing. Logged.
FLAGGED: Transaction pauses. Alert to user. Manual override required.
CLEAN: Proceed to FROST signing.
```

### 7.3 Data Freshness

- SDN list fetched daily via `infra/scripts/load-ofac-sdn.sh`
- List version (date + hash) stored in DB
- Every screening result includes the list version used
- If list fetch fails, screening degrades to **FLAGGED** (fail-closed, not fail-open)

---

## 8. Execution Model (Testnet)

### 8.1 Pre-Trade Simulation

Before any execution, the system produces a simulation:

```json
{
  "from": "0xUSDC (Base Sepolia)",
  "to": "0xWETH (Base Sepolia)",
  "amount": "100 USDC",
  "route": "1inch v5 → Uniswap V3 → Curve",
  "expected_out": "0.0284 WETH",
  "slippage_tolerance": "0.5%",
  "gas_estimate": "142,000 gas",
  "gas_cost_eth": "0.000085 ETH",
  "gas_cost_usd": "$0.28",
  "total_cost_usd": "$0.31",
  "simulated_at": "2026-09-18T14:32:00Z"
}
```

### 8.2 Execution Flow

```
1. User requests trade (or policy engine triggers)
2. Pre-trade simulation produced → shown to user (or auto-approved if within policy bounds)
3. OFAC screen on counterparty (DEX router address) → CLEAN
4. FROST ceremony: 2-of-3 shares sign the Safe tx
5. Safe executes on Base Sepolia
6. Confirmation received → logged
7. Portfolio engine re-indexes → new state reflected in dashboard
```

### 8.3 Why Testnet Only (v0.x)

- No real financial loss possible
- Full execution pipeline can be tested end-to-end
- FROST signing, Safe execution, DEX routing, and portfolio re-indexing all validated
- Mainnet gating will require: additional approval layers, insurance, possibly a timelock, and a separate security review

---

## 9. Open-Core Model

### 9.1 Open (MIT License)

- Full source code for all 6 phases
- FROST wallet, DKG, signing
- Multi-chain indexer
- OFAC screening
- Risk Sentinel agent
- Policy engine
- Testnet execution
- Dashboard
- Docker Compose deployment
- All documentation

### 9.2 Commercial Tier (Future, Separate License)

| Feature | Why Commercial |
|---|---|
| HSM-backed key shares | Requires HSM hardware or cloud HSM subscription |
| Mainnet execution | Requires insurance, additional security review, legal review |
| Managed infrastructure | Hosting, monitoring, backup, patching |
| Full KYC/AML pipeline | Requires licensed data providers (Chainalysis, Elliptic) |
| Multi-tenant SaaS | Requires infrastructure investment |
| Enterprise support | SLA, onboarding, training |
| Additional AI agents | Tax Optimizer, Yield Hunter, Compliance Auditor |
| Cross-chain bridge execution | Requires bridge liquidity and insurance |

The open version is **fully functional** for personal and small-team use on testnet. The commercial tier adds production readiness, mainnet access, and managed services.

---

## 10. Roadmap

| Phase | Name | Deliverable |
|---|---|---|---|
| **0** | Foundation | Repo, DB, API skeleton, Docker. `curl /health` → 200 |
| **1** | Security Core | FROST 2-of-3 wallet. DKG. Sign on Sepolia. Safe + safe-frost deployed |
| **2** | Portfolio Engine | Multi-chain indexer. Live pricing. Risk metrics. Dashboard shows real data |
| **3** | Compliance Layer | OFAC SDN loaded. Screen endpoint works. Block/Flag/Clean logic |
| **4** | AI Agent Layer | Risk Sentinel live. Structured output. Full logging. Explainable findings |
| **5** | Execution (testnet) | Pre-trade sim. 1inch testnet. FROST → Safe → confirmed tx on Base Sepolia |
| **6** | Frontend | Unified dashboard. All phases reflected. No mocks |

**Total to v0.x complete: ~32 weeks (8 months) for a single experienced engineer.**

### Post-v0.x

| Milestone | Notes |
|---|---|
| v0.5 | Mainnet execution (gated). Additional chains (Arbitrum, BSC) |
| v1.0 | HSM integration. SOC 2 Type I. First external users |
| v1.5 | Tokenized RWA tracking. Additional AI agents. Cross-chain reads |
| v2.0 | Commercial tier launch. Managed SaaS. Enterprise features |

---

## 11. Risks & Limitations

| Risk | Mitigation |
|---|---|
| FROST is not as battle-tested as ECDSA multi-sig | `frost-secp256k1-evm` is used by Safe (production Solidity verifier). ZCash uses FROST in production. But the EVM-specific variant is newer. |
| ML-KEM-768 is not yet deployed at scale | NIST-finalized (FIPS 204, Aug 2024). OpenSSL 3.5+ supports it. But real-world deployment experience is limited. |
| Single-engineer project | Bus factor = 1. Mitigation: full documentation, CI/CD, clear phase boundaries, open-source community. |
| Testnet ≠ mainnet | Gas dynamics, MEV, slippage, and liquidity differ. Mainnet will require re-testing. |
| LLM non-determinism | Structured output enforcement. Confidence scores. Human-in-the-loop for all actions. |
| OFAC list is US-only | Not a global compliance solution. EU/UK/UN lists are commercial-tier. |
| No insurance | Self-custody means the user bears all risk. No platform to sue. |

---

## 12. References

1. **FROST**: "FROST: Flexible Round-Optimized Schnorr Threshold Signatures" — ZCash Foundation, 2021.
2. **frost-secp256k1-evm**: ZCash Foundation Rust crate, v2.2.0 (2026). EVM-compatible FROST with keccak256.
3. **Safe FROST Verifier**: Safe Research, 2025. Solidity contract for on-chain FROST signature verification (~5,600 gas).
4. **ML-KEM (FIPS 204)**: NIST, August 2024. "Module-Lattice-Based Key-Encapsulation Mechanism."
5. **ML-DSA (FIPS 204)**: NIST, August 2024. "Module-Lattice-Based Digital Signature Algorithm."
6. **OFAC SDN List**: US Treasury Department. `https://sanctions.ofac.treas.gov/api/sdn/v1/sdnList`
7. **Safe (Gnosis Safe)**: `https://github.com/safe-global/safe-smart-account`
8. **1inch Swap API**: `https://docs.1inch.io/`
9. **viem**: `https://viem.sh/` — TypeScript Ethereum library.
10. **TimescaleDB**: `https://timescale.com/` — PostgreSQL time-series extension.
11. **Kinexys by J.P. Morgan**: Reference architecture for institutional blockchain-based settlement.
12. **GENIUS Act (2025)**: US federal stablecoin framework. Context for digital asset regulation.

---

## 13. Conclusion

ODAMP fills a specific gap: **institutional-grade security primitives, accessible to individuals and small teams, without institutional pricing or vendor lock-in.**

It does not reinvent cryptography. It does not create a new consensus mechanism. It does not issue a token. It integrates existing, well-researched components — FROST threshold signing, post-quantum key exchange, OFAC screening, LLM-backed analysis, and policy-based execution — into a single, self-hosted, auditable, open-source tool.

The result is a platform where a professional can:
- Hold keys that no single entity (including themselves, on a single device) can compromise
- Exchange keys using post-quantum cryptography
- See their entire multi-chain portfolio in real time
- Know immediately if a counterparty is sanctioned
- Get AI-assisted risk analysis with full reasoning logged
- Execute trades on testnet with pre-trade simulation and multi-party approval

All of this runs on their own hardware. All of this is MIT-licensed. All of this is auditable.

That is ODAMP.

---

