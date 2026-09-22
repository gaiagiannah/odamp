"""ODAMP AI Agent — FastAPI entry point."""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional
import os

from agents.risk_sentinel import RiskSentinel
from config import settings
import logging

app = FastAPI(title="ODAMP AI Agent", version="0.1.0")

# Initialize agent
agent = RiskSentinel(
    provider=settings.llm_provider,
    model=settings.llm_model,
    api_key=settings.llm_api_key,
    max_tokens=settings.llm_max_tokens,
    temperature=settings.llm_temperature,
)


class AnalyzeRequest(BaseModel):
    wallet_id: str
    portfolio_state: Optional[dict] = None  # If not provided, fetch from DB


class AnalyzeResponse(BaseModel):
    risk_level: str
    findings: list
    recommendations: list
    model: str
    latency_ms: int


@app.get("/health")
async def health():
    return {"status": "ok", "agent": "risk_sentinel", "model": settings.llm_model}


@app.post("/analyze", response_model=AnalyzeResponse)
async def analyze(req: AnalyzeRequest):
    """Run Risk Sentinel analysis on a portfolio."""
    try:
        portfolio_state = req.portfolio_state
        if portfolio_state is None:
            # Fetch from DB
            portfolio_state = _fetch_portfolio_state(req.wallet_id)

        result = await agent.analyze(portfolio_state)
        return AnalyzeResponse(**result)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/logs")
async def get_logs(limit: int = 50):
    """Get recent AI agent logs."""
    # Query ai_agent_logs table
    # TODO: implement DB query
    return {"logs": [], "note": "DB logging not yet wired"}


def _fetch_portfolio_state(wallet_id: str) -> dict:
    """Fetch portfolio state from Postgres."""
    import psycopg2
    import json

    conn = psycopg2.connect(settings.database_url)
    cur = conn.cursor()
    cur.execute("""
        SELECT symbol, chain, balance::TEXT, price_usd, value_usd
        FROM positions
        WHERE wallet_id = %s
        ORDER BY value_usd DESC
    """, (wallet_id,))
    rows = cur.fetchall()
    conn.close()

    return {
        "wallet_id": wallet_id,
        "positions": [
            {"symbol": r[0], "chain": r[1], "balance": r[2], "price_usd": r[3], "value_usd": r[4]}
            for r in rows
        ],
        "total_value_usd": sum(r[4] for r in rows),
    }   