"""Tests for Risk Sentinel agent (mocked LLM)."""

import json
import pytest
from unittest.mock import patch, MagicMock
from agents.risk_sentinel import RiskSentinel


@pytest.fixture
def agent():
    return RiskSentinel(
        provider="anthropic",
        model="claude-sonnet-4-20250514",
        api_key="test-key",
        max_tokens=2048,
        temperature=0,
    )


@pytest.fixture
def sample_portfolio():
    return {
        "wallet_id": "test-wallet",
        "positions": [
            {"symbol": "ETH", "chain": "ethereum", "balance": "4.0", "price_usd": 3500.0, "value_usd": 14000.0},
            {"symbol": "USDC", "chain": "ethereum", "balance": "5000", "price_usd": 1.0, "value_usd": 5000.0},
            {"symbol": "WBTC", "chain": "ethereum", "balance": "0.1", "price_usd": 100000.0, "value_usd": 10000.0},
        ],
        "total_value_usd": 29000.0,
    }


class TestRiskSentinel:
    def test_parse_valid_json(self, agent):
        raw = json.dumps({
            "risk_level": "ELEVATED",
            "findings": [
                {
                    "claim": "Concentration: 48% in ETH",
                    "evidence": "ETH value $14,000 of $29,000 total",
                    "confidence": 0.95,
                    "source": "portfolio_state",
                    "severity": "ELEVATED"
                }
            ],
            "recommendations": ["Consider diversifying ETH allocation"],
            "data_gaps": ["No 30-day price history available"],
        })
        result = agent.parse_response(raw)
        assert result["risk_level"] == "ELEVATED"
        assert len(result["findings"]) == 1
        assert result["findings"][0]["confidence"] == 0.95

    def test_parse_markdown_wrapped(self, agent):
        raw = """```json
{"risk_level": "LOW", "findings": [], "recommendations": [], "data_gaps": []}
```"""
        result = agent.parse_response(raw)
        assert result["risk_level"] == "LOW"

    def test_parse_invalid_json(self, agent):
        raw = "I cannot analyze this portfolio because..."
        result = agent.parse_response(raw)
        assert result["risk_level"] == "ELEVATED"
        assert result["findings"][0]["source"] == "parse_error"

    def test_build_prompt_includes_data(self, agent, sample_portfolio):
        system, user = agent.build_prompt(sample_portfolio)
        assert "Risk Sentinel" in system
        assert "14000.0" in user  # ETH value present
        assert "29000.0" in user  # Total present   