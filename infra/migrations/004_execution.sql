-- Execution log (quotes + fills)
CREATE TABLE IF NOT EXISTS execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    quote_id UUID,
    chain VARCHAR(32) NOT NULL,
    from_token VARCHAR(128) NOT NULL,
    to_token VARCHAR(128) NOT NULL,
    amount_in NUMERIC(78, 18) NOT NULL,
    expected_out NUMERIC(78, 18),
    slippage_bps SMALLINT,
    gas_estimate BIGINT,
    gas_cost_usd NUMERIC(18, 6),
    route TEXT[],
    tx_hash VARCHAR(128),
    block_number BIGINT,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',  -- pending, confirmed, failed
    frost_shares_used SMALLINT[],
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_exec_wallet ON execution_logs(wallet_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_exec_status ON execution_logs(status);   