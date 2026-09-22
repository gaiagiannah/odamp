cat > services/ai-agent/prompts/risk_sentinel.md << 'EOF'
# Risk Sentinel — Prompt Design Reference

## System Prompt (embedded in agents/risk_sentinel.py)

Role: Portfolio risk analyst for a digital asset management platform.

## Design Constraints

1. **No execution authority.** Signals only. The policy engine acts on signals.
2. **Explainable output.** Every finding: claim + evidence + confidence + source.
3. **Data-grounded.** Must reference specific numbers from input.
4. **Honest about gaps.** Missing data → `data_gaps` field.
5. **Temperature 0.** Deterministic for same input.
6. **Structured JSON only.** No prose. No markdown. No preamble.

## Output Schema

```json
{
  "risk_level": "LOW | ELEVATED | HIGH | CRITICAL",
  "findings": [
    {
      "claim": "string (specific, numeric)",
      "evidence": "string (data points referenced)",
      "confidence": 0.0,
      "source": "portfolio_state | price_history | calculation",
      "severity": "LOW | ELEVATED | HIGH | CRITICAL"
    }
  ],
  "recommendations": ["string (risk management actions, not trade calls)"],
  "data_gaps": ["string (what data would improve this analysis)"]
}   

Future Agents (not in v0.x)
Agent	Purpose
Tax Optimizer	FIFO/HIFO lot analysis, wash sale detection
Yield Hunter	On-chain yield comparison, APY tracking
Compliance Auditor	Regulatory change monitoring
EOF


---

## `docs/RESEARCH-REPORT.md`

```bash
cat > docs/RESEARCH-REPORT.md << 'EOF'
# ODAMP Research Report

**The Institutional Case for Open-Source Digital Asset Management**

Version 1.0 — September 18, 2026

## Executive Summary

This report synthesizes research on J.P. Morgan's Kinexys platform,
the global tokenization landscape, post-quantum cryptography standards,
AI-driven financial analytics, and the geopolitical fragmentation of
digital finance. It maps each research finding to a concrete ODAMP
product capability.

**Core thesis:** Kinexys ($3T+ transactions) proves the technology.
The GENIUS Act + NIST PQC standards prove the policy environment.
$5.5T–$16T in tokenized assets by 2030 proves the demand. What's
missing is a self-hosted, open-source, threshold-secure management
layer for the individual professional. ODAMP is that layer.

## Key Findings → Product Mapping

| Research Finding | ODAMP Capability |
|-----------------|-----------------|
| Kinexys: $3T+ transactions, $5B daily volume | Multi-chain portfolio tracking (Phase 2) |
| Kinexys: bank-side programmability (MIT DCI) | Policy-based automation (Phase 5) |
| NIST FIPS 203/204: ML-KEM, ML-DSA finalized | PQC key exchange in DKG (Phase 1) |
| Safe Research: safe-frost verifier, ~5600 gas | FROST 2-of-3 → Safe smart account (Phase 1) |
| frost-secp256k1 v2.2: production Rust crate | Threshold signing library (Phase 1) |
| OFAC SDN: public data, ~18,700 entities | Pre-flight sanctions screening (Phase 3) |
| GENIUS Act (2025): stablecoin framework | Asset categorization in registry (Phase 2) |
| Citi: $5.5T tokenized assets by 2030 | Tokenized RWA tracking (Phase 2) |
| BCG: AI for compliance + risk in digital assets | Risk Sentinel agent (Phase 4) |
| EY: institutional DeFi engagement 24%→75% | Multi-chain support (Phase 2) |
| IMF: tokenization loosens issuer/infra link | Chain-agnostic design (all phases) |

## The Custody Gap

| Option | Security | Cost | Target |
|--------|----------|------|--------|
| Exchange | Custodial | Low | Retail |
| MetaMask/Phantom | Single-key | Low | Prosumer |
| **ODAMP** | **FROST 2-of-3 + PQC** | **$0** | **Professional** |
| Fireblocks/Dfns | MPC/HSM | $50K–$500K/yr | Institution |

No product exists in the ODAMP column.

## References

1. Kinexys by J.P. Morgan — $3T+ cumulative transactions (2026)
2. NIST FIPS 203 (ML-KEM), FIPS 204 (ML-DSA) — Final, Aug 2024
3. Safe Research — safe-frost Solidity verifier, 2025
4. ZCash Foundation — frost-secp256k1 v2.2.0, 2026
5. OFAC SDN List — sanctions.ofac.treas.gov
6. GENIUS Act (P.L. 119-27) — July 2025
7. Citi "Tokenization 2030" — $5.5T base case
8. Ark Invest — $11T tokenized assets by 2030
9. BCG 2026 Flagship Report — AI in digital asset operations
10. MIT Digital Currency Institute — bank-side programmability (2024)
11. IMF — tokenization and the issuer/infrastructure link (2025)
12. NSA CNSA 2.0 — PQC migration mandate, 2035 deadline
EOF