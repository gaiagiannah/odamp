-- AI agent call log (full audit trail)
CREATE TABLE IF NOT EXISTS ai_agent_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent VARCHAR(64) NOT NULL,       -- "risk_sentinel"
    model VARCHAR(128) NOT NULL,      -- "claude-sonnet-4-20250514"
    prompt_hash TEXT NOT NULL,        -- SHA-256 of full prompt
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    latency_ms INTEGER,
    response JSONB NOT NULL,          -- full structured output
    findings_count INTEGER DEFAULT 0,
    max_severity VARCHAR(16),         -- "LOW", "ELEVATED", "HIGH", "CRITICAL"
    wallet_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_logs_wallet ON ai_agent_logs(wallet_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_logs_agent ON ai_agent_logs(agent, created_at DESC);   