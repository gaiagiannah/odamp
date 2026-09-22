-- Indexer checkpoints (resume from last block)
CREATE TABLE IF NOT EXISTS indexer_checkpoints (
    chain VARCHAR(32) PRIMARY KEY,
    last_block BIGINT NOT NULL,
    last_token_sync TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tracked tokens (which tokens to index for each chain)
CREATE TABLE IF NOT EXISTS tracked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chain VARCHAR(32) NOT NULL,
    contract_address VARCHAR(128),  -- NULL for native
    symbol VARCHAR(32) NOT NULL,
    name VARCHAR(128) NOT NULL,
    decimals SMALLINT NOT NULL DEFAULT 18,
    coingecko_id VARCHAR(64) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(chain, contract_address)
);   