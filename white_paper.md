ODAMP: Open Digital Asset Management Platform
A Technical and Policy Whitepaper on Institutional-Grade Security, AI Automation, and Multi-Asset Tokenization for the Individual Investor

Abstract
Digital finance has reproduced, rather than dissolved, the access asymmetries of traditional finance. Institutional actors operate with multi-party cryptographic custody, smart order routing across hundreds of venues, machine-readable risk intelligence, and forthcoming quantum-safe signatures. Individual investors, by contrast, typically hold a single private key in a software wallet, trade through rate-limited exchange interfaces, and have no unified view of their exposure across chains, protocols, or asset classes. This paper specifies ODAMP (Open Digital Asset Management Platform): an open-core system that applies threshold cryptography, post-quantum signatures, explainable AI agents, and on-chain compliance tooling to the management of both native digital assets and tokenized real-world assets. The paper distinguishes clearly between what is technically achievable by an independent development effort today, and what depends on regulatory licensure or institutional partnership that lies outside a single project's control. It is written as a foundation for actual implementation, not a pitch deck — claims are cited to primary standards documents, peer-reviewed and preprint cryptography literature, and named regulatory texts, and areas of genuine uncertainty (market-size projections, timelines) are presented as ranges with their sources, not single invented numbers.

1. Introduction and Motivation
1.1 The Access Gap Is Measurable, Not Rhetorical
The case for democratized access to sophisticated financial infrastructure is often made rhetorically. It can also be made with reference to specific, citable facts.

In the United States, eligibility to invest in private securities offerings under Regulation D is gated by the SEC's "accredited investor" definition: a natural person must have an individual income above $200,000 (or $300,000 jointly with a spouse) in each of the prior two years, or a net worth exceeding $1 million excluding primary residence.[^1] This threshold was set in 1982 and has never been adjusted for inflation; the SEC's own 2015 staff report calculated that an inflation-adjusted net-worth threshold would be roughly $2.16–2.45 million.[^2] Because the rule uses wealth as a proxy for financial sophistication, both SEC commissioners and outside researchers have argued that it excludes many sophisticated but non-wealthy investors while admitting wealthy but unsophisticated ones.[^3] This is not a crypto-specific critique — it is a structural feature of how access to certain investment classes is currently gated in the U.S. market.

Globally, the picture is one of large but shrinking exclusion. The World Bank's Global Findex 2025 Database — a survey of roughly 148,000 adults across 141 economies — found that 79% of adults worldwide now hold a formal financial account, up from 74% in 2021, but 1.3 billion adults remain outside the formal financial system entirely, with more than half concentrated in eight countries (Bangladesh, China, Egypt, India, Indonesia, Mexico, Nigeria, and Pakistan).[^4] Roughly 40% of adults in developing economies report being unable to reliably access emergency funds.[^5] These are the populations for whom mobile-first, low-bandwidth, stablecoin-capable financial tooling is not a convenience but a substitute for infrastructure that does not otherwise reach them.

1.2 The Cost of Insecurity Is Also Measurable
The counter-argument to "just let anyone use DeFi tools" is that self-custodied digital assets have a well-documented loss profile. According to Chainalysis's 2026 Crypto Crime Report, approximately $3.4 billion in digital assets was stolen in service-side hacks between January and early December 2025 — the second-worst year on record after 2022's $3.7–3.8 billion.[^6] A single incident, the February 2025 Bybit exchange hack, accounted for roughly $1.5 billion in one day and was attributed by the FBI to North Korea's Lazarus Group / TraderTraitor cluster.[^7] North Korea-affiliated actors alone were responsible for an estimated $2.02 billion of 2025's stolen funds, a 51% year-over-year increase, bringing their cumulative confirmed total since tracking began to roughly $6.75 billion.[^7] In earlier years, cross-chain bridges alone accounted for a majority of DeFi-specific losses (64% of DeFi losses in 2022).[^8]

These are not edge cases. They are a structural consequence of a security model — a single private key, or a custom-built bridge contract, controlling large pooled value — that institutional finance abandoned decades ago in favor of multi-party authorization, hardware security modules, and continuous auditing. ODAMP's starting premise is that the same underlying cryptographic tools that make institutional custody safe are now mature and open enough to be deployed at individual scale, and that doing so is a security intervention, not just an accessibility one.

1.3 What This Paper Is and Is Not
This paper is a technical and policy specification for a project under active development. It is explicitly not:

A claim that the described system is regulated, licensed, or ready for public deposit-taking.
A promise of specific investment returns or risk elimination.
A claim that every institutional integration named here (bank rails, HSM vendors, custody licenses) is contracted or in place. Where a component requires regulatory status the project does not yet hold, this paper says so.
It is:

A specification of a real, buildable architecture using standardized, publicly available cryptographic primitives (Sections 3–4).
A mapping of the compliance obligations that apply to this category of product, cited to their source texts (Section 6).
An honest accounting of what remains aspirational versus implemented (Section 8).
2. Design Principles
ODAMP is organized around six commitments, each of which has a direct architectural consequence:

Principle	Architectural Consequence
Keys are user-held, not custodial	No ODAMP-operated entity ever possesses a complete private key. Threshold cryptography (Section 3.1) is used so that no single party — including the platform operator — can unilaterally move funds.
Security scales to exposure, not to price	A user with $200 in holdings should not need to perform a hardware-backed key ceremony; a user with $200,000 should not be protected by the same single-device setup as the former.
Every automated decision is explainable	AI agents (Section 5) must produce a plain-language, auditable rationale for every action or recommendation. An agent that cannot explain itself cannot execute.
Compliance obligations are met, not evaded	Sanctions screening and Travel Rule obligations that apply to virtual asset service providers are treated as engineering requirements, not obstacles to architect around (Section 6).
The system degrades gracefully	If the network, the AI backend, or a cloud provider is unavailable, balance viewing and local signing must continue to function.
What is not yet real is labeled as such	Any integration standing in for a regulated institutional service (e.g., a testnet execution venue standing in for a licensed broker-dealer relationship) is visibly marked as such in the product, not silently presented as production-grade.
3. Security Architecture
3.1 Threshold Signatures: The Cryptographic Foundation
The mathematical basis for splitting a secret among multiple parties dates to Adi Shamir's 1979 paper "How to Share a Secret," which showed that a secret can be divided into n shares such that any k of them reconstruct it, while fewer than k reveal nothing.[^9] Modern threshold-signature schemes extend this idea so that a group of parties can jointly produce a valid digital signature without ever reconstructing the private key in one place — meaning no single device, server, or person is ever a single point of failure or a single point of compromise.

For the elliptic-curve signature scheme used by most existing blockchains (ECDSA), Gennaro and Goldfeder's 2018 paper "Fast Multiparty Threshold ECDSA with Fast Trustless Setup" (ACM CCS '18) is a foundational construction still cited as a reference implementation basis by production custody systems.[^10] For Schnorr-family signatures (used by Bitcoin Taproot and increasingly by newer chains), the relevant standard is FROST (Flexible Round-Optimized Schnorr Threshold signatures), introduced by Chelsea Komlo and Ian Goldberg at SAC 2020.[^11] FROST reduces the number of network communication rounds required to produce a threshold signature relative to earlier schemes, which matters directly for mobile and low-bandwidth deployments. FROST has since been formalized by the IETF as RFC 9591 (2024), giving it the status of an interoperable internet standard rather than a single vendor's proprietary protocol.[^12]

ODAMP's key-management layer is specified to use these standardized constructions — not a bespoke, unaudited scheme — precisely because threshold cryptography is an area where novel, unreviewed designs have historically been where implementations fail even when the underlying signature algorithm is sound.

Practical tiering. Following the "no single point of failure" principle, key shares are distributed across independent custody domains (e.g., user device, encrypted backup, and — for larger holdings — a hardware-backed share), with the threshold (e.g., 2-of-3) chosen so that any one compromised or lost share does not, by itself, either freeze funds or leak the key.

3.2 Post-Quantum Cryptography
In August 2024, the U.S. National Institute of Standards and Technology finalized three post-quantum cryptography standards after an eight-year, multi-round public evaluation process:[^13]

FIPS 203 (ML-KEM), a module-lattice key-encapsulation mechanism derived from CRYSTALS-Kyber, providing quantum-resistant key exchange with three parameter sets (ML-KEM-512/768/1024).[^14]
FIPS 204 (ML-DSA), a module-lattice digital signature scheme derived from CRYSTALS-Dilithium, with security levels ML-DSA-44/65/87 corresponding roughly to AES-128/192/256 equivalent strength.[^13][^14]
FIPS 205 (SLH-DSA), a stateless hash-based signature scheme derived from SPHINCS+, whose security rests only on the collision-resistance of hash functions rather than lattice-hardness assumptions — providing algorithmic diversity as a hedge in case lattice-based schemes are ever weakened.[^13]
A fourth algorithm, FN-DSA (Falcon), is in final standardization as FIPS 206.[^15] NIST additionally selected HQC as a fifth, code-based backup key-encapsulation mechanism in March 2025, providing a mathematically independent alternative to ML-KEM.[^16]

These are not abstractions with distant deadlines. The NSA's Commercial National Security Algorithm Suite 2.0 (CNSA 2.0) sets 2030 as the mandatory PQC migration deadline for National Security Systems.[^17] The practical concern motivating this timeline is "harvest now, decrypt later": an adversary can record encrypted traffic or signed transactions today and decrypt or forge them once a cryptographically relevant quantum computer exists, meaning long-lived financial records and custody signatures are exposed now even though the quantum threat itself is not yet realized.

Implementation reality check. Post-quantum signatures are substantially larger and, in some cases, slower than classical ones: ML-DSA-65 signatures run 2,420–4,595 bytes versus roughly 64–72 bytes for an ECDSA/Schnorr signature, and SLH-DSA signatures are 7,856–49,856 bytes.[^13][^18] Efficient threshold (multi-party) versions of these PQC signature schemes are an active, unsettled research area as of 2026 — for example, the "TALUS" and "Trilithium" constructions for threshold ML-DSA are 2025–2026 preprints, not yet standardized or battle-tested in production.[^19] ODAMP's honest position is therefore a hybrid strategy: classical threshold signatures (FROST/ECDSA-threshold) in production now, with ML-DSA/ML-KEM layered in dual-signature mode as the threshold-PQC research matures — consistent with NIST's own guidance to migrate via hybrid classical+PQC schemes rather than a single flag-day cutover.

3.3 Transaction-Level Protections
Independent of key management, the execution layer applies defense-in-depth controls that do not depend on any single vendor: pre-signing transaction simulation (so the user sees the exact state change before authorizing it), address allowlisting, configurable spending limits, and static/dynamic analysis of any smart contract before interaction. These are standard practice among security-conscious wallets and are included here as baseline requirements rather than differentiators.

4. Blockchain Intelligence and Data Architecture
A unified portfolio view requires aggregating on-chain state across multiple chains and asset standards (native tokens, ERC-20/721/1155, SPL tokens, DeFi LP positions) into a single, real-time model of what a user actually owns and what it is worth. This is a data-engineering problem, not a cryptographic one, and ODAMP treats it as such: an indexing layer that reads public chain state, a pricing layer that aggregates market data from multiple sources to avoid single-oracle dependency, and a risk-analytics layer that computes standard risk-adjusted metrics (Sharpe ratio, maximum drawdown, concentration risk) rather than presenting only raw balances. Public sanctions-list data (Section 6) and open smart-contract-risk datasets are treated as additional data feeds into this same layer, not a separate bolt-on.

5. The AI Agent Layer
5.1 Why Explainability Is a Hard Requirement, Not a Feature
Autonomous financial agents are not a hypothetical risk category. Multiple 2025–2026 incidents — including exploits targeting AI-driven trading agents — have demonstrated that unconstrained autonomous execution in DeFi can produce losses with no clear post-hoc accountability trail. Regulators evaluating AI in financial contexts (for example, under the EU AI Act's treatment of high-risk AI systems, and existing U.S. suitability/best-execution obligations for automated advice) consistently return to the same requirement: a system that makes or influences financial decisions must be able to produce a record of why it acted.

ODAMP's agent architecture therefore enforces, as non-negotiable constraints:

Bounded authority: every agent operates within user-set spending limits and asset-class restrictions that the agent cannot itself modify.
Mandatory reasoning trace: every agent action is logged with a plain-language rationale before execution, not generated retroactively.
Human-in-the-loop above a threshold: any single action above a user-configurable dollar threshold requires explicit confirmation.
Circuit breakers: an agent whose actions produce losses beyond a set threshold is automatically suspended pending review.
5.2 Interoperability
Agents are built to communicate with tools and data sources through the Model Context Protocol (MCP), an open specification for connecting AI systems to external tools and data, rather than a closed, single-vendor agent framework — so that the agent layer is not permanently locked to one AI provider's proprietary tool-calling format.

6. Regulatory and Compliance Architecture
This is the section where the gap between "aspirational institutional-grade" and "actually compliant" matters most, and where this paper is deliberately conservative.

6.1 Sanctions Screening
The U.S. Treasury's Office of Foreign Assets Control (OFAC) maintains the Specially Designated Nationals (SDN) list, a public dataset. Real-time screening of transaction counterparties against this list — and equivalent EU/UN lists — is a genuinely implementable feature for an independent project, because the underlying data is public. This is treated as a first-class, buildable compliance feature rather than aspirational infrastructure.

6.2 The Travel Rule
FATF Recommendation 16 (the "Travel Rule"), extended to virtual asset service providers (VASPs) in FATF's 2019 guidance, requires VASPs to collect, verify, and transmit originator and beneficiary information — name, account number, and identifying details — for virtual-asset transfers above a threshold (historically USD/EUR 1,000), including transfers involving self-hosted wallets.[^20] The EU's implementation is the Transfer of Funds Regulation (TFR).[^20] Critically, Travel Rule compliance is an obligation of regulated VASPs transmitting funds on behalf of customers — it presumes the existence of a licensed entity. A self-custodial tool where the user alone holds signing authority sits in a different regulatory posture than a custodial exchange, but the architecture is specified to support Travel Rule data exchange for any regulated on/off-ramp integration, because that is the direction global policy is unambiguously moving.

6.3 Broader Regulatory Landscape
Relevant frameworks that any real deployment must track jurisdiction-by-jurisdiction include the EU's Markets in Crypto-Assets Regulation (MiCA, applicable since December 2024) and its associated DAC8 tax-reporting rules; U.S. state money-transmitter licensing; and, for stablecoin-related settlement rails, the U.S. GENIUS Act framework for payment stablecoins. None of these are optional "nice to haves" for a product operating in this space — they define whether specific features (holding customer funds, transmitting third-party payments, issuing a token) are legally available at all in a given jurisdiction, and a from-scratch project does not have these licenses on day one. The architecture is built so that regulated capabilities (custody, transmission) are pluggable behind an interface that a licensed partner can fill in, rather than assumed.

7. Tokenization of Real-World Assets: Scope and Honest Market Sizing
Tokenized real-world assets are a genuine, growing category — but public projections of its eventual size vary by an order of magnitude depending on scope and methodology, and a credible whitepaper should show that range rather than quote the single largest number available.

McKinsey (2024), using a conservative scope that explicitly excludes stablecoins, tokenized deposits, and CBDCs, projected roughly $2 trillion in tokenized financial assets by 2030 in its base case, $4 trillion in a bullish case, and noted that "broad adoption of tokenization is still far away."[^21]
Boston Consulting Group, using a broader "10% of global GDP" heuristic across illiquid asset classes, has published figures ranging from an early $16 trillion by 2030 estimate to a more recent BCG/Ripple joint estimate of roughly $9.4 trillion by 2030 and ~$19 trillion by 2033.[^22]
Standard Chartered has published a more bullish $30.1 trillion by 2034 estimate.[^22]
As of early 2026, independent tracking suggests the actual tokenized RWA market (excluding stablecoins) is roughly $12 billion, dominated by tokenized private credit ($8–9B) and tokenized U.S. Treasuries ($3B) — several orders of magnitude below any of the 2030 projections.[^23]
The honest reading: tokenization of traditional financial instruments (money-market funds, Treasuries, private credit) is real and growing today; tokenization of physical/illiquid assets (real estate, commodities, infrastructure) is, per McKinsey's own analysis, held back by "marginal benefits, feasibility concerns, complex compliance requirements, or lack of incentive for key industry players."[^21] ODAMP's asset-coverage roadmap is therefore sequenced to start with the categories that are actually liquid and tokenized today — tokenized Treasuries, tokenized gold (e.g., PAXG, XAUT, which together represent the large majority of tokenized gold value), and major equities/ETF tokenization efforts — before extending toward less mature categories like tokenized real estate or commodities, rather than presenting all categories as equally available now.

8. Honest Assessment of Current Limitations
A whitepaper that omits its own weaknesses is not institutional-grade; it is marketing. The following are acknowledged, specific limitations of the architecture as of this writing:

No custody license. ODAMP as an independent project does not hold money-transmitter, broker-dealer, or trust-company status in any jurisdiction. Self-custodial architecture (Section 3) is chosen specifically because it reduces — but does not eliminate — the regulatory perimeter the project must clear to operate lawfully.
Threshold post-quantum signatures are not production-ready. As discussed in Section 3.2, efficient threshold ML-DSA is 2025–2026 research, not a deployed standard. Any near-term PQC claim must describe hybrid classical+PQC signing, not pure threshold-PQC.
Tokenized RWA liquidity is thin. Per Section 7, most tokenization categories described in earlier drafts of this project's research are not yet liquid, tradable markets — they are pilots.
AI agent explainability is a design requirement, not a solved problem. Producing reliable, faithful natural-language rationales for automated financial decisions — rather than plausible-sounding post-hoc justifications — remains an open challenge across the industry, and the guardrails in Section 5.1 (bounded authority, human-in-the-loop, circuit breakers) are treated as necessary precisely because explainability alone cannot be fully guaranteed.
Compliance infrastructure requires ongoing jurisdiction-specific legal review that a technical specification cannot substitute for.
9. Implementation Roadmap (Scoped to Buildable Reality)
Rather than the multi-year, 20+-person roadmap of earlier drafts of this research, the roadmap below is scoped to what a focused development effort can build and demonstrate, phase by phase, with each phase producing a genuinely working artifact:

Phase	Deliverable	What Makes It Real (not simulated)
0 — Foundation	Repo structure, database schema, API skeleton	Runs locally; no placeholder services
1 — Security Core	Working 2-of-3 FROST threshold wallet; key generation and signing ceremony	Real cryptographic library (e.g., frost-secp256k1), real testnet transactions
2 — Portfolio Engine	Multi-chain balance indexer + real market pricing + real risk metrics	Real on-chain reads (public RPC/indexer APIs), not mock data
3 — Compliance Layer	OFAC SDN screening against transaction addresses	Real public sanctions dataset, real matching logic
4 — AI Agent Layer	One agent (e.g., Risk Sentinel) with logged, explainable reasoning	Real LLM-backed analysis of real portfolio state
5 — Execution (testnet)	Simulated + testnet order execution with pre-trade simulation	Real DEX aggregator APIs against testnets
6 — Frontend	Unified dashboard	Reflects real data from phases 1–5
Each phase explicitly ends with something that can be run and shown — not a folder of stubs.

10. Conclusion
The individual components this paper describes are real: threshold signatures are a standardized, IETF-ratified construction; post-quantum cryptography has been finalized by NIST; sanctions data is public; on-chain data is public; explainable-AI guardrails are an active, serious design discipline. What does not yet exist, and what this project sets out to build, is the integration of these components into a single, honestly-labeled system that gives an individual meaningfully better security and transparency than a single-key software wallet — without overstating its regulatory status or its market. That is the standard this whitepaper holds itself to, and the standard the resulting code should be held to as well.

References
[^1]: U.S. Securities and Exchange Commission, Regulation D, Rule 501, Securities Act of 1933, definition of "accredited investor." 
[^2]: U.S. SEC, Report on the Review of the Definition of "Accredited Investor," December 2015. 
[^3]: Cato Institute, "Let Investors Decide, Part 1" (commentary on SEC accredited investor rulemaking, citing SEC Commissioners Elad Roisman and Hester Peirce). 
[^4]: World Bank, The Global Findex Database 2025, based on surveys of ~148,000 adults across 141 economies conducted in 2024. 
[^5]: World Bank, Global Findex 2025 Executive Summary — emergency-funds access indicator. 
[^6]: Chainalysis, 2026 Crypto Crime Report (introduction and stolen-funds update), covering January–early December 2025. 
[^7]: Chainalysis 2026 Crypto Crime Report; FBI Internet Crime Complaint Center (IC3) public service announcement attributing the February 21, 2025 Bybit incident to North Korea's TraderTraitor/Lazarus Group cluster. 
[^8]: Chainalysis, 2023 Crypto Crime Report (2022 annual data on DeFi and cross-chain bridge losses). 
[^9]: A. Shamir, "How to Share a Secret," Communications of the ACM, 1979. 
[^10]: R. Gennaro and S. Goldfeder, "Fast Multiparty Threshold ECDSA with Fast Trustless Setup," Proceedings of the 2018 ACM SIGSAC Conference on Computer and Communications Security (CCS '18), pp. 1179–1194. 
[^11]: C. Komlo and I. Goldberg, "FROST: Flexible Round-Optimized Schnorr Threshold Signatures," Selected Areas in Cryptography (SAC) 2020, LNCS vol. 12804, pp. 34–65, Springer, 2021. 
[^12]: D. Connolly, C. Komlo, I. Goldberg, C. A. Wood, "The Flexible Round-Optimized Schnorr Threshold (FROST) Protocol for Two-Round Schnorr Signatures," IETF RFC 9591, 2024. 
[^13]: NIST, FIPS 203 (ML-KEM), FIPS 204 (ML-DSA), FIPS 205 (SLH-DSA), finalized August 13, 2024. 
[^14]: NIST FIPS 203/204 parameter specifications, as summarized in DigiCert, "An In-Depth Look At The NIST PQC Algorithms," and academic surveys citing the original FIPS documents. 
[^15]: NIST FIPS 206 (FN-DSA / Falcon), in final standardization. 
[^16]: NIST selection of HQC as a fifth PQC algorithm (code-based KEM backup), March 2025. 
[^17]: U.S. National Security Agency, Commercial National Security Algorithm Suite 2.0 (CNSA 2.0), 2030 migration deadline for National Security Systems. 
[^18]: Comparative signature-size analysis, arXiv preprint "Post-Quantum Cryptography Migration in Australian Real-Time Payment Infrastructure" (2026) and related 2025–2026 PQC-in-finance preprints. 
[^19]: S. Celi, R. del Pino, T. Espitau, G. Niot, T. Prest, "Efficient Threshold ML-DSA," IACR ePrint 2026/013 (to appear, USENIX Security '26); "TALUS: Threshold ML-DSA with One-Round Online Signing," arXiv 2603.22109, 2026. 
[^20]: Financial Action Task Force, Recommendation 16 ("the Travel Rule"), 2019 Updated Guidance for a Risk-Based Approach to Virtual Assets and VASPs, with 2021 revised guidance; EU Transfer of Funds Regulation (TFR) as the EU implementation. 
[^21]: McKinsey & Company, "From Ripples to Waves: The Transformational Power of Tokenizing Assets," June 2024. 
[^22]: Boston Consulting Group, "Relevance of On-Chain Asset Tokenization in 'Crypto Winter'" (2022 estimate); BCG/Ripple, "New Value in Motion" (2025 joint estimate); Standard Chartered tokenization market research. 
[^23]: Independent 2026 market tracking of tokenized RWA categories (tokenized private credit and tokenized U.S. Treasuries), as aggregated in industry analyses of RWA.xyz-type datasets.