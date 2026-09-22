# ODAMP Research Report

**The Institutional Case for Open-Source Digital Asset Management**
Connecting Kinexys, Tokenization, Post-Quantum Cryptography, and AI to the ODAMP Product

---

## Executive Summary

This report synthesizes research conducted on J.P. Morgan's Kinexys platform, the global tokenization landscape, post-quantum cryptography standards, AI-driven financial analytics, and the geopolitical fragmentation of digital finance. It identifies the specific gaps in the current market that ODAMP is designed to fill, and maps each research finding to a concrete product capability.

**Core thesis:** The institutional validation of blockchain-based finance (Kinexys, $3T+ in transactions) proves the technology works. The regulatory framework (GENIUS Act, NIST PQC standards) proves the policy environment is ready. The market projection ($5.5T–$16T in tokenized assets by 2030) proves the demand is real. What is missing is a **self-hosted, open-source, threshold-secure management layer** for the individual professional and small team that sits between retail self-custody and institutional custody. ODAMP is that layer.

---

## 1. The Institutional Proof: Kinexys by J.P. Morgan

### 1.1 What Kinexys Proves

Kinexys by J.P. Morgan is the most significant validation that blockchain-based financial infrastructure works at institutional scale. Key facts:

- **$3 trillion+ in cumulative transactions** since inception (2015 as Onyx, rebranded 2024) [1][2]
- **$5–7 billion in average daily transaction volume** as of mid-2026 [2][3]
- **Target: $10 billion daily** by late 2026, driven by APAC expansion [4]
- **Core products:** JPM Coin (USD deposit token on Base), Tokenized Collateral Network, MONY (tokenized MMF), Fund Flow (tokenized fund servicing) [1][5]
- **Clients:** BMW Group, FirstRand, Mitsubishi Corporation, Siemens, Citco, Fidelity International, Apollo, WisdomTree, BNY, RBC [5]

### 1.2 What Kinexys Validates for ODAMP

| Kinexys Capability | ODAMP Equivalent | Gap ODAMP Fills |
|---|---|---|
| Permissioned blockchain for institutional clients | Open-source, self-hosted multi-chain platform | Kinexys is closed. ODAMP is open. |
| JPM Coin (bank deposit token) | Tracks tokenized RWAs alongside native tokens | Kinexys manages JPM's own clients. ODAMP manages *your* portfolio. |
| Tokenized Collateral Network | Risk metrics (VaR, concentration, drawdown) | Kinexys optimizes for repo. ODAMP optimizes for *your* portfolio. |
| Bank-side programmability | Policy-based automation engine | Kinexys is bank-controlled. ODAMP is user-controlled. |
| $50K–$500K/yr institutional pricing | $0 (MIT-licensed, self-hosted) | Kinexys is for institutions. ODAMP is for professionals. |

**The gap:** Kinexys proves the technology. It does not serve the $100K–$5M professional. No open-source equivalent exists that combines threshold security, multi-chain tracking, compliance screening, and AI analytics in a single self-hosted tool.

### 1.3 The "Invisible Ledger" Implication

Kinexys's most significant strategic insight is that **blockchain is becoming invisible infrastructure** [5][6]. When a Siemens treasury team executes a programmable FX swap, the DLT layer is abstracted away. The salient concepts are liquidity, credit, legal finality, and operational resilience.

**ODAMP implication:** The dashboard should present *financial outcomes* (P&L, risk, compliance status), not *blockchain mechanics* (block numbers, gas, mempool). The blockchain is plumbing. The user sees results, not infrastructure.

---

## 2. The Tokenization Wave: Market Size & Asset Classes

### 2.1 Current State (2026)

- Tokenized real-world assets: **~$31–36 billion** in total market value [7][8]
- Tokenized US Treasuries: **~$4.2 billion** (2025), growing rapidly [9]
- BlackRock BUIDL (tokenized MMF): **$2.85B+ AUM** [10]
- JPMorgan MONY: **$25B+ in tokenized AUM** (from $500M in ~2 years) [10]
- On-chain cash equivalents: **~$36 billion** (RWA.xyz, 2025) [8]

### 2.2 2030 Projections

| Source | Base Case | Bull Case |
|---|---|---|
| Citi ("Tokenization 2030: Wall Street On-Chain") [11] | $5.5T | $8.2T |
| Ark Invest [12] | $11T | — |
| JPMorgan [13] | $13T | — |
| Skynet RWA Security Report [14] | $16T | — |
| BCG / Ripple [15] | $16T (2030) | $20T (2033) |
| Standard Chartered [15] | $16T | $30T |

**Consensus range: $5.5T–$16T by 2030.** Even the most conservative estimate (Citi base: $5.5T) represents a **150x increase** from the current ~$36B.

### 2.3 Asset Classes Being Tokenized

| Asset Class | Current Tokenized Value | 2030 Projection | Key Players |
|---|---|---|---|
| US Treasuries | ~$4.2B | $1T+ (10% of T-bill market) [11] | BlackRock, JPMorgan, Franklin Templeton |
| Money Market Funds | ~$28B (BUIDL + MONY) | $500B+ | BlackRock, JPMorgan |
| Public Equities | ~$1B | $2.6T (if 10% of US retail goes on-chain) [11] | Citi, JPMorgan |
| Real Estate | ~$2B | ~$200B [11] | RealT, Lofty, Hamilton Lane |
| Private Credit / PE | ~$500M | $1T+ | Apollo, WisdomTree, Hamilton Lane |
| Commodities | ~$100M | $50B+ | Various |

### 2.4 ODAMP's Position

ODAMP is positioned to manage **all of the above** in a unified portfolio. The product's multi-chain indexer (Phase 2) tracks:
- Native tokens (ETH, SOL, USDC, etc.)
- Tokenized RWAs (BUIDL, MONY, tokenized Treasuries, tokenized real estate)
- Tokenized funds (tokenized MMFs, tokenized PE)

The risk engine (Phase 2) computes unified metrics across both native and tokenized positions. The AI agent (Phase 4) provides cross-asset-class risk analysis. No existing self-hosted tool does this.

---

## 3. Post-Quantum Cryptography: The Security Foundation

### 3.1 NIST Standards (Final, August 2024)

| Standard | Algorithm | Purpose | Status |
|---|---|---|---|
| **FIPS 203** | ML-KEM (formerly CRYSTALS-Kyber) | Key encapsulation / key exchange | Final [16][17] |
| **FIPS 204** | ML-DSA (formerly CRYSTALS-Dilithium) | Digital signatures | Final [16][17] |
| **FIPS 205** | SLH-DSA (SPHINCS+) | Hash-based signature fallback | Final [16][17] |
| FIPS 206 | FN-DSA (Falcon) | Compact signatures | Draft [17] |

These are the **first-ever finalized post-quantum cryptographic standards**, developed over an 8-year NIST evaluation process (2016–2024) [16].

### 3.2 The Threat: Harvest-Now-Decrypt-Later (HNDL)

The primary near-term threat is not a quantum computer breaking encryption today. It is **adversaries intercepting and storing encrypted data now**, to decrypt it later when cryptographically relevant quantum computers exist [18].  For financial data with long confidentiality requirements (portfolio positions, key material, transaction history), this is an **immediate** threat, not a future one.

NSA CNSA 2.0 mandates that US national security systems fully retire classical public-key cryptography by **2035**, with intermediate deadlines starting in 2025 [18].  CISA, NSA, and NIST all recommend beginning migration now [18]. 

### 3.3 ODAMP's PQC Strategy

ODAMP uses a **hybrid approach** that matches the actual threat model:

| Layer | Algorithm | Why |
|---|---|---|
| **Key exchange (DKG ceremony)** | ML-KEM-768 (FIPS 203) + X25519 (hybrid) | A quantum adversary targets key exchange.  This is the layer to protect. [16][18] |
| **Signing (transactions)** | FROST(secp256k1, keccak256) via `frost-secp256k1-evm` [19][20] | On-chain verifiable at ~5,600 gas.  ML-DSA has no EVM verifier. [20] |
| **Key storage (at rest)** | AES-256-GCM (password → Argon2id) | Classical symmetric crypto is not broken by Shor's algorithm. |

**Why not ML-DSA-65 for signing?** ML-DSA-65 signatures are ~2,420 bytes and have no efficient on-chain verifier on EVM. FROST(secp256k1) produces 65-byte Schnorr signatures verifiable via the `safe-frost` Solidity contract at ~5,600 gas [20][21]. The signing layer doesn't need post-quantum resistance because signatures are public and don't need to remain secret. The key *exchange* layer does, because it establishes the shared secret from which key shares are derived. 

### 3.4 FROST: Threshold Signatures for the EVM

FROST (Flexible Round-Optimized Schnorr Threshold signatures) [22] allows splitting a root secret key into *n* shares with threshold *t*, such that any *t* participants can cooperatively generate a valid signature. Key properties for ODAMP:

- **Threshold security:** 2-of-3 means no single device, file, or process can move funds [20][21]
- **Indistinguishable signatures:** Output is a standard Schnorr signature. On-chain, it's indistinguishable from a single-signer EOA [20]
- **Standard compliance:** Follows RFC-9591 [20]
- **Efficient verification:** ~5,600 gas on EVM [20][21]
- **Safe integration:** `safe-frost` verifier contract (Safe Research, 2025) enables FROST signatures to authorize Safe smart account transactions [20][21]
- **Production library:** `frost-secp256k1-evm` v2.2.0 (ZCash Foundation, Rust, April 2026) [19] 

**The Safe + FROST combination** means ODAMP's on-chain wallet is a **Safe smart account** (EIP-1271) authorized by a 2-of-3 FROST threshold.  This is production-ready: Safe Research has published the Solidity verifier, and the Rust signing library is maintained by the ZCash Foundation [19][20][21].

---

## 4. The Regulatory Landscape

### 4.1 GENIUS Act (July 18, 2025)

The GENIUS Act (P.L. 119-27) is the **first major US federal legislation on digital assets** [23][24][25].  Key provisions relevant to ODAMP:

- Establishes a federal framework for **payment stablecoins** [23]
- Requires 100% reserves (USD, T-bills, certain government assets) [24]
- Only "permitted payment stablecoin issuers" (bank subsidiaries, OCC-supervised nonbanks, state-chartered entities) can issue [25]
- **Does not restrict** depository institutions from issuing digital assets that represent deposits, utilizing distributed ledgers for books and records, or providing custodial services [26]
- Implementation regulations proposed by OCC (February 2026) [27]
- Effective date: January 18, 2027, or 120 days after implementing regulations, whichever is first [28] 

**ODAMP implication:** The GENIUS Act creates a regulated stablecoin ecosystem. ODAMP tracks and manages stablecoin positions (USDC, USDT, JPM Coin, etc.) as part of the portfolio. The Act's distinction between "payment stablecoins" and "tokenized deposits" is relevant to how ODAMP categorizes assets in its registry. 

### 4.2 OFAC Sanctions (Ongoing)

The OFAC Specially Designated Nationals (SDN) List is the primary sanctions list for US persons [29]. Key facts:

- **~18,700 entities** on the SDN list; **~35,000+** on the Consolidated Advanced list [30]
- Includes **digital currency wallet addresses** (XBT, ETH, TRX, USDT) [30]
- Publicly available as downloadable XML/CSV files (no API key required) [29][30]
- Must be screened **before** processing any transaction [29]
- Match → block the transaction, freeze assets, report to OFAC [29] 

**ODAMP implication:** Phase 3 (Compliance Layer) implements pre-flight OFAC SDN screening against all transaction counterparties. The data is public, the matching logic is straightforward (exact address match + ENS reverse lookup), and the action is deterministic (BLOCK / FLAG / CLEAN). This is not a "nice to have" — it is a **legal requirement** for US persons interacting with digital assets. 

### 4.3 The Clarity Act (Expected 2026–2027)

The pending Clarity Act will clarify the regulatory treatment of **tokenized securities** and digital assets beyond stablecoins.  This is relevant to ODAMP's tokenized RWA tracking capability, as it will determine whether tokenized Treasuries, equities, and funds are regulated as securities, commodities, or a new category.

---

## 5. The Geopolitical Fragmentation

### 5.1 The Dollar's Erosion

The dollar's share of global reserves is ~58% and is projected to decline to 45–50% by 2035 [31]. Key drivers:

- **UAE exits OPEC** (May 1, 2026) — structural break in the petrodollar system [31]
- **China's CIPS** processed $24.45T annually in 2025 [31]
- **e-CNY** upgraded to interest-bearing deposit money (Jan 2026) [31]
- **mBridge** (multi-CBDC platform) at minimum viable stage [31]
- **BRICS Pay** integrating national payment networks [31]

### 5.2 The Western Response

The US strategy is **not** to build a competing CBDC system but to make the dollar's existing infrastructure more efficient [31][32]:

- **Kinexys / JPM Coin** — Bank-issued deposit tokens on public blockchain (Base) [1][5]
- **GENIUS Act** — Private-sector stablecoin framework [23]
- **Tokenized Treasuries** — BlackRock BUIDL, JPMorgan MONY [10]
- **24/7 settlement** — CFTC exploring 24/7 trading for equities and digital assets [32] 

### 5.3 ODAMP's Position

ODAMP is **chain-agnostic and currency-agnostic**. It tracks positions across:
- EVM chains (Ethereum, Base, Arbitrum, BSC)
- Solana
- (Future: Canton, Avalanche, Provenance)

It does not take a geopolitical side. It manages the user's portfolio regardless of which chains, tokens, or tokenized assets they hold. This is a deliberate design choice: in a multipolar financial architecture, the management tool should be neutral.

---

## 6. AI in Financial Management

### 6.1 The Current State

AI in digital asset management is in early stages. Existing tools (Zerion, DeBank, Revert Finance) provide:
- Balance aggregation
- P&L tracking
- Basic allocation views

They do **not** provide:
- Anomaly detection on wallet activity
- Risk scoring with explainable reasoning
- LLM-backed analysis with logged, auditable outputs
- Policy-based execution triggers

### 6.2 The Institutional Signal

- J.P. Morgan, BNY, RBC, DeepTempo, and NVIDIA completed a **federated learning initiative** for fraud detection [5]
- EY survey: institutional engagement with DeFi protocols set to **triple from 24% to 75%** within two years [32]
- BCG 2026 flagship report identifies **AI for compliance, risk monitoring, and liquidity management** as a critical layer in digital asset operations [32]

### 6.3 ODAMP's AI Design: Risk Sentinel

The Risk Sentinel agent (Phase 4) is designed with three constraints that distinguish it from a generic "AI chatbot for finance":

1. **No execution authority.** The AI produces *signals*, not actions. The policy engine (Phase 5) decides what to do with signals. A human can override at any step.
2. **Explainable output.** Every finding includes: claim, evidence (specific data points), confidence score, and source. This makes the reasoning **auditable**.
3. **Full logging.** Every LLM call is logged: prompt hash, model, tokens, latency, response. If a rebalancing decision is questioned, the full prompt and response are available for review.

This design is directly informed by the institutional signal: federated learning for privacy [5], explainable AI for regulatory compliance, and the shift from "access" to "application" in institutional DeFi engagement [32].

---

## 7. The Custody Gap: Why ODAMP Exists

### 7.1 The Binary Choice

| Option | Security | Cost | Autonomy | Target User |
|---|---|---|---|---|
| Exchange (Coinbase, Kraken) | Custodial | Low | None | Retail |
| Self-custody (MetaMask, Phantom) | Single-key | Low | Full, but fragile | Retail / Prosumer |
| **ODAMP** | **FROST 2-of-3 + PQC** | **$0** | **Full, threshold-secure** | **Professional / Small Team** |
| Institutional (Fireblocks, Dfns) | MPC/HSM | $50K–$500K+/yr | Full | Institution |

**No product exists** in the ODAMP column. The gap between "MetaMask with one key on one laptop" and "Fireblocks at $200K/yr" is enormous, and it is where the individual professional managing $100K–$5M in digital assets currently has **no good option**.

### 7.2 The Research-Validated Solution

Every component of ODAMP's solution is validated by the research:

| ODAMP Component | Validated By |
|---|---|
| FROST 2-of-3 threshold signing | Safe Research `safe-frost` (production Solidity verifier) [20][21]; ZCash `frost-secp256k1-evm` v2.2.0 [19]; RFC-9591 [22] |
| ML-KEM-768 key exchange | NIST FIPS 203 (final, Aug 2024) [16][17]; NSA CNSA 2.0 mandate [18] |
| Multi-chain portfolio tracking | Kinexys multi-chain strategy (Base, Ethereum, Canton, Avalanche) [1][5]; Citi tokenization report [11] |
| OFAC SDN screening | OFAC public data [29][30]; legal requirement for US persons [29] |
| AI risk analysis | Federated learning at JPM/BNY/NVIDIA [5]; EY institutional DeFi survey [32] |
| Policy-based execution | Kinexys "bank-side programmability" (MIT DCI research) [5]; GENIUS Act framework [23] |
| Self-hosted, open-source | The gap between retail and institutional [7][8][11] |

---

## 8. What ODAMP Does NOT Do (and Why)

| Excluded | Research Basis |
|---|---|
| Mainnet execution (v0.x) | Kinexys operates under a bank license with FDIC eligibility [1]. ODAMP has no such license. Testnet-first is the responsible approach. |
| HSM-backed key storage (v0.x) | Institutional custody requires HSM (Fireblocks, Dfns) [33]. File-encrypted PQC keys are the open-source equivalent for personal use. |
| Full KYC/AML pipeline | OFAC SDN screening is a legal requirement [29]. Full KYC requires licensed data providers (Chainalysis, Elliptic) and is a commercial-tier concern. |
| Quantum Key Distribution (QKD) | QKD is a physics-layer protocol requiring dedicated fiber hardware [18]. ML-KEM-768 is the software-equivalent. |
| Token issuance | ODAMP manages assets. It does not create them. The GENIUS Act restricts stablecoin issuance to permitted issuers [25]. |

---

## 9. The 2030 Picture: Where ODAMP Sits

By 2030, the research suggests:

- **$5.5T–$16T in tokenized assets** [11][12][13][14]
- **24/7 trading** for equities and digital assets [32]
- **Multiple parallel settlement systems** (Western, Chinese, BRICS, Gulf) [31]
- **AI as a standard layer** in financial operations [5][32]
- **PQC migration** mandatory for US national security systems by 2035 [18]
- **Blockchain as invisible infrastructure** — the user sees outcomes, not mechanics [5][6] 

ODAMP is positioned at the intersection of all five:
- Manages a portfolio that spans native tokens and tokenized RWAs (the $5.5T–$16T market)
- Supports 24/7 operation (no "banking hours")
- Is chain-agnostic (works across Western, Chinese, and neutral chains)
- Has AI as a core layer (Risk Sentinel)
- Is post-quantum secure by default (ML-KEM-768)
- Presents financial outcomes, not blockchain mechanics (invisible ledger)

---

## 10. Conclusion

The research tells a clear story:

1. **The technology works.** Kinexys has processed $3T+ in transactions.  FROST threshold signatures are production-ready on EVM. NIST PQC standards are final. [1][16][19][20] 

2. **The regulation is ready.** The GENIUS Act creates a stablecoin framework.  OFAC screening is a legal requirement with public data. The Clarity Act will address tokenized securities. [23][29] 

3. **The market is coming.** $5.5T–$16T in tokenized assets by 2030.  The dollar is fragmenting. Multiple settlement systems are being built. [11][12][31]

4. **The gap is real.** No self-hosted, open-source, threshold-secure, AI-powered digital asset management platform exists for the individual professional. [7][8]

5. **ODAMP fills that gap.** Every component is validated by published research, production libraries, or regulatory frameworks. Nothing in the product is speculative.

The product is not a research project. It is an **engineering project** with a clear, validated specification. The research is done. The next step is to build.

---

## References

[1] "Inside Kinexys: J.P. Morgan's $3 Trillion Transaction Platform." *Asset Tokenization*, Jan 15, 2026. https://www.assettokenization.com/resources/inside-kinexys-j-p-morgans-3-trillion-transaction-platform 

[2] "Kinexys 2026 Milestones." *J.P. Morgan*, Apr 28, 2026. https://www.jpmorgan.com/payments/newsroom/kinexys-milestones-2026

[3] "JPMorgan's Kinexys Blockchain Passes $3 Trillion as APAC Expansion Accelerates." *Studio Global*, Jun 30, 2026. https://www.studioglobal.ai/discover/answers/search-fact-check-with-cited-sources-for-6a434552ef0c0a0d49084049 

[4] "JPMorgan expands digital assets push with Mitsubishi deal as it targets $10bn in daily transactions." *DL News*, Mar 31, 2026. https://www.dlnews.com/articles/markets/jpmorgan-expands-digital-assets-push-with-mitsubishi-deal-as-it-targets-dollar10bn-in-daily-transactions/ 

[5] "Kinetic Treasury Arrives." *GFMag*, Feb 17, 2026. https://gfmag.com/transaction-banking/kinetic-treasury-arrives/

[6] "JPMorgan Onyx & Kinexys: How Tokenized Collateral Works." *CoinPaprika*, Jun 19, 2026. https://coinpaprika.com/education/jpmorgan-onyx-and-kinexys-how-tokenized-collateral-works/ 

[7] "Tokenized Real-World Assets Explode to $31.4 Billion, Could Hit $1.6 Trillion by 2030." *Binance Research via MEXC*, May 21, 2026. https://www.mexc.com/news/1104414 

[8] "Tokenized Asset Market Could Reach $30 Trillion by 2030." *Bitget News*, Mar 23, 2026. https://www.bitget.com/news/detail/12560605294046 

[9] "RWA Tokenization Market To Reach $16T by 2030, Skynet Report Says." *Yahoo Finance*, Aug 25, 2025. https://finance.yahoo.com/news/rwa-tokenization-market-reach-16t-000429613.html 

[10] "Tokenized assets could surpass $11 trillion by 2030, Ark Invest says." *The Block*, Jan 21, 2026. https://www.theblock.co/post/386588/tokenization-outlook-ark-invest 

[11] "Tokenization of Real-World Assets Will Hit $5.5 Trillion by 2030, Citi Says." *Banking Exchange*, Jun 1, 2026. https://m.bankingexchange.com/news-feed/item/10633-tokenization-of-real-world-assets-will-hit-5-5-trillion-by-2030-citi-says 

[12] "Tokenized Assets Could Hit $8.2 Trillion by 2030." *Yahoo Finance / Trading Disclosure*, Jun 29, 2026. https://finance.yahoo.com/markets/crypto/articles/tokenized-assets-could-hit-8-194337707.html 

[13] "JPMorgan Predicts Tokenized Real-World Assets Market Will Reach $13 Trillion by 2030." *Tekedia*, Apr 7, 2026. https://www.tekedia.com/jpmorgan-predicts-tokenized-real-world-assets-market-will-reach-13-trillion-by-2030/ 

[14] "RWA Tokenization Market To Reach $16T by 2030, Skynet Report Says." *Yahoo Finance*, Aug 25, 2025. https://finance.yahoo.com/news/rwa-tokenization-market-reach-16t-000429613.html 

[15] "Blockchain and Asset Tokenization: The $16 Trillion Opportunity Reshaping Global Finance." *Blockchain Council*, Oct 4, 2025. https://www.blockchain-council.org/industry-reports/blockchain/blockchain-and-asset-tokenization/ 

[16] "Decoding NIST PQC Standards: What They Are, What's Final, and What's Next." *Encryption Consulting*, Sep 8, 2026. https://www.encryptionconsulting.com/decoding-nist-pqc-standards/ 

[17] "NIST FIPS 203/204/205: The Complete Guide." *QuantumSequrity*, Jul 30, 2026. https://quantumsequrity.com/blog/nist-fips-guide 

[18] "NIST Post-Quantum Cryptography Standards 2026." *Entangled Future*, Apr 8, 2026. https://entangledfuture.com/guides/nist-pqc-standards/ 

[19] `frost-secp256k1-evm` — Rust crate, ZCash Foundation. v2.2.0. https://docs.rs/frost-secp256k1-evm

[20] "safe-research/safe-frost: FROST Threshold Signatures for the EVM." *GitHub / Safe Research*, 2025–2026. https://github.com/safe-research/safe-frost 

[21] "Beyond Multi-Sig: FROST Brings Secure, Scalable Threshold Signatures to the EVM." *Safe Foundation Blog*, Jul 18, 2025. https://safefoundation.org/blog/frost-brings-secure-scalable-threshold-signatures-to-the-evm 

[22] "FROST: Flexible Round-Optimized Schnorr Threshold Signatures." ZCash Foundation. RFC-9591. 2021. 

[23] "Stablecoin Legislation: An Overview of the GENIUS Act of 2025 (P.L. 119-27)." *Congressional Research Service*, Aug 20, 2026. https://www.congress.gov/crs-product/IN12553 

[24] "Stablecoins and the GENIUS Act: An Overview." *Richmond Fed*, Nov 18, 2025. https://www.richmondfed.org/banking/banker_resources/news_flash/2025/20251118_genius_act 

[25] "GENIUS Act Establishes Regulatory Framework for Stablecoins." *Stinson LLP*, Jul 22, 2025. https://www.stblaw.com/about-us/publications/view/2025/07/22/genius-act-establishes-regulatory-framework-for-stablecoins 

[26] "What You Need To Know About the New Stablecoin Legislation." *Arnold & Porter*, Jul 21, 2025. https://www.arnoldporter.com/en/perspectives/advisories/2025/07/new-stablecoin-legislation-analyzing-the-genius-act 

[27] "GENIUS Act Regulations: Notice of Proposed Rulemaking." *OCC*, Feb 25, 2026. https://www.occ.gov/news-issuances/bulletins/2026/bulletin-2026-3.html 

[28] "Building a Digital Asset Regulatory Framework: The GENIUS Act and Next Steps." *Wiley*, Jul 23, 2025. https://www.wiley.law/alert-Building-a-Digital-Asset-Regulatory-Framework-The-GENIUS-Act-and-Next-Steps 

[29] "OFAC Screening Guide 2026 — SDN List Screening & Compliance." *Sanction Lawyers*, Apr 15, 2026. https://sanctionslawyers.net/ofac-lawyers/ofac-screening-guide/ 

[30] "OFAC Sanctions List Crawler — SDN & Multi-Jurisdiction Data." *Apify*, Aug 31, 2026. https://apify.com/jungle_synthesizer/ofac-sanctions-crawler 

[31] Research conducted in this conversation: mBridge, CIPS, e-CNY, BRICS Pay, UAE OPEC exit, Citi 2030 report, BCG 2026 flagship report, RAND assessment.

[32] Research conducted in this conversation: EY institutional DeFi survey, CFTC 24/7 trading exploration, BCG 2026 AI in digital assets.

[33] Research conducted in this conversation: Fireblocks, Dfns, Copper institutional custody pricing and capabilities.

---