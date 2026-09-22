"""Risk Sentinel — ODAMP's first AI agent.

Analyzes real portfolio state and produces structured, explainable
risk findings. No execution authority — signals only.
"""

import json
from typing import dict, list

from .base_agent import BaseAgent


class RiskSentinel(BaseAgent):
    """LLM-backed portfolio risk analyst with explainable output."""

    SYSTEM_PROMPT = """You are Risk Sentinel, a portfolio risk analyst for a digital asset management platform.

Your job:
1. Analyze the provided portfolio state (positions, prices, values)
2. Identify risk factors: concentration, volatility, correlation, illiquidity, counterparty
3. Produce structured findings with evidence
4. Provide actionable recommendations

Rules:
- Every finding MUST include: claim, evidence (specific numbers), confidence (0-1), source
- Be specific. "High concentration" is not a finding. "72% in ETH, HHI=0.58" is a finding.
- Confidence must reflect data quality, not your certainty in the conclusion
- If data is insufficient to make a finding, say so explicitly
- Do NOT recommend specific trades. Recommend risk management actions.
- Output MUST be valid JSON matching the schema below.

Output schema:
{
  "risk_level": "LOW" | "ELEVATED" | "HIGH" | "CRITICAL",
  "findings": [
    {
      "claim": "string",
      "evidence": "string with specific numbers",
      "confidence": 0.0-1.0,
      "source": "portfolio_state" | "price_history" | "calculation",
      "severity": "LOW" | "ELEVATED" | "HIGH" | "CRITICAL"
    }
  ],
  "recommendations": ["string"],
  "data_gaps": ["string"]  // what data you'd need but don't have
}"""

    def build_prompt(self, data: dict) -> tuple[str, str]:
        user_prompt = f"""Analyze this portfolio state:

```json
{json.dumps(data, indent=2)}   

Produce your risk assessment as structured JSON."""
return self.SYSTEM_PROMPT, user_prompt

def parse_response(self, raw: str) -> dict:
    """Parse LLM response into structured RiskReport."""
    # Try to extract JSON from response (LLM may wrap in markdown)
    text = raw.strip()
    if text.startswith("```"):
        # Remove markdown code fences
        lines = text.split("\n")
        json_lines = [l for l in lines if not l.startswith("```")]
        text = "\n".join(json_lines)

    try:
        result = json.loads(text)
    except json.JSONDecodeError:
        # Fallback: return raw text as a single finding
        return {
            "risk_level": "ELEVATED",
            "findings": [{
                "claim": "Agent response was not valid JSON",
                "evidence": raw[:200],
                "confidence": 0.0,
                "source": "parse_error",
                "severity": "LOW"
            }],
            "recommendations": ["Review agent output manually"],
            "data_gaps": [],
        }

    # Validate required fields
    result.setdefault("risk_level", "ELEVATED")
    result.setdefault("findings", [])
    result.setdefault("recommendations", [])
    result.setdefault("data_gaps", [])

    return result   


### `services/ai-agent/prompts/risk_sentinel.md`

```markdown
# Risk Sentinel — System Prompt Reference

This file documents the system prompt for version control and review.
The actual prompt is embedded in `agents/risk_sentinel.py`.

## Design Constraints

1. **No execution authority.** The agent produces signals, not actions.
2. **Explainable output.** Every finding has claim + evidence + confidence + source.
3. **Data-grounded.** Findings must reference specific numbers from the input.
4. **Honest about gaps.** If data is missing, the agent says so (`data_gaps`).
5. **Temperature 0.** Deterministic output for the same input.

## Future Agents (not in v0.x)

- Tax Optimizer: FIFO/HIFO lot analysis, wash sale detection
- Yield Hunter: On-chain yield opportunities, APY comparison
- Compliance Auditor: Regulatory change monitoring, exposure alerts   