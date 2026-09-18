-- ODAMP initial schema (Phase 0/2 scope)
-- Deliberately trimmed relative to the original research report:
-- only tables needed for a genuinely working portfolio + security +
-- compliance flow. Tax, governance, and RWA-marketplace tables are
-- added in later phases once the features they support exist.

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    jurisdiction VARCHAR(10) NOT NULL,
    metadata JSONB
);

CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    threshold SMALLINT NOT NULL,
    total_shares SMALLINT NOT NULL,
    group_public_key_hex TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (threshold >= 1 AND threshold <= total_shares)
);

-- Individual key shares are NOT stored here in plaintext. This table
-- tracks *where* each share lives (custody domain metadata only) so
-- the system can prompt the right devices/services during a signing
-- ceremony. The actual key material stays on-device / in the HSM.
CREATE TABLE IF NOT EXISTS key_share_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    share_identifier TEXT NOT NULL,
    custody_domain VARCHAR(50) NOT NULL, -- 'user_device' | 'encrypted_backup' | 'hsm'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (wallet_id, share_identifier)
);

-- NOTE: amount/avg_cost_usd/current_price_usd use DOUBLE PRECISION
-- rather than NUMERIC deliberately, so they bind cleanly to Rust f64
-- via sqlx (sqlx maps Postgres NUMERIC to a Decimal type, not f64).
-- This is a Phase 1 pragmatic choice. Floating point is NOT
-- appropriate for real financial ledger entries at production scale
-- (rounding errors compound) — a real money-handling system should
-- use fixed-point/Decimal arithmetic throughout (both in Postgres
-- and in Rust, e.g. the `rust_decimal` crate) rather than f64. That
-- migration is flagged as a pre-launch hardening task, not done here.
CREATE TABLE IF NOT EXISTS positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_id UUID REFERENCES wallets(id),
    asset_symbol VARCHAR(50) NOT NULL,
    asset_type VARCHAR(50) NOT NULL, -- 'crypto' | 'tokenized_equity' | 'tokenized_commodity' | 'stablecoin'
    amount DOUBLE PRECISION NOT NULL,
    avg_cost_usd DOUBLE PRECISION NOT NULL,
    current_price_usd DOUBLE PRECISION NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_id UUID REFERENCES wallets(id),
    tx_hash VARCHAR(256),
    chain VARCHAR(50) NOT NULL,
    from_address VARCHAR(256) NOT NULL,
    to_address VARCHAR(256) NOT NULL,
    from_asset VARCHAR(50),
    from_amount NUMERIC(38, 18),
    to_asset VARCHAR(50),
    to_amount NUMERIC(38, 18),
    sanctions_screened BOOLEAN NOT NULL DEFAULT FALSE,
    sanctions_blocked BOOLEAN NOT NULL DEFAULT FALSE,
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Append-only log of every automated agent action, with the
-- mandatory plain-language rationale required by the whitepaper's
-- explainability principle (Section 5.1). No agent action should
-- ever be written elsewhere without a corresponding row here.
CREATE TABLE IF NOT EXISTS agent_actions (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type VARCHAR(50) NOT NULL,
    action_type VARCHAR(50) NOT NULL,
    reasoning TEXT NOT NULL,
    parameters JSONB,
    result VARCHAR(20) NOT NULL, -- 'executed' | 'recommended' | 'blocked'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_positions_user_id ON positions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_actions_user_id ON agent_actions(user_id);
