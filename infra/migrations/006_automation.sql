-- Policy/automation rules
CREATE TABLE IF NOT EXISTS automation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    trigger_type VARCHAR(32) NOT NULL,   -- "threshold", "schedule", "event", "ai_signal"
    trigger_config JSONB NOT NULL,
    action_type VARCHAR(32) NOT NULL,    -- "alert", "rebalance", "freeze", "execute"
    action_config JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Automation execution log
CREATE TABLE IF NOT EXISTS automation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES automation_rules(id),
    trigger_data JSONB NOT NULL,
    action_taken VARCHAR(32) NOT NULL,
    result JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);   