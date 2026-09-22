-- ODAMP initial schema
-- Deliberately trimmed: only tables needed for a working
-- portfolio + security + compliance flow.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    jurisdiction VARCHAR(10) NOT NULL,
    metadata JSONB DEFAULT '{}'
);

-- Wallets (public info only — key shares NEVER stored here)
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    chain VARCHAR(32) NOT NULL,
    address VARCHAR(128) NOT NULL,
    group_public_key TEXT NOT NULL,
    threshold SMALLINT NOT NULL,
    total_shares SMALLINT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(chain, address)
);

-- Positions (portfolio holdings)
CREATE TABLE IF NOT EXISTS positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    chain VARCHAR(32) NOT NULL,
    token_address VARCHAR(128),
    symbol VARCHAR(32) NOT NULL,
    balance NUMERIC(78, 18) NOT NULL,
    price_usd NUMERIC(18, 8) NOT NULL,
    value_usd NUMERIC(18, 2) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(wallet_id, chain, token_address)
);

CREATE INDEX IF NOT EXISTS idx_positions_wallet ON positions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_positions_updated ON positions(updated_at DESC);

-- Signing audit log
CREATE TABLE IF NOT EXISTS signing_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    message_hash TEXT NOT NULL,
    signature TEXT NOT NULL,
    shares_used SMALLINT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_signing_wallet ON signing_logs(wallet_id, created_at DESC);   