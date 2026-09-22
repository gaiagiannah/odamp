-- OFAC SDN entries (loaded from public feed)
CREATE TABLE IF NOT EXISTS sdn_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uid VARCHAR(64) UNIQUE NOT NULL,
    name TEXT NOT NULL,
    aliases TEXT[] DEFAULT '{}',
    addresses TEXT[] DEFAULT '{}',
    program VARCHAR(32),
    country VARCHAR(64),
    date_listed DATE,
    date_delisted DATE,
    loaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sdn_addresses ON sdn_entries USING GIN(addresses);
CREATE INDEX IF NOT EXISTS idx_sdn_name ON sdn_entries (name);

-- Screening audit log (append-only)
CREATE TABLE IF NOT EXISTS compliance_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address TEXT NOT NULL,
    status VARCHAR(16) NOT NULL,  -- CLEAN, FLAGGED, BLOCKED
    matched_entity TEXT,
    program VARCHAR(32),
    list_version TEXT NOT NULL,
    tx_context JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_created ON compliance_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_status ON compliance_logs(status);

-- SDN list metadata
CREATE TABLE IF NOT EXISTS sdn_list_meta (
    id SERIAL PRIMARY KEY,
    version TEXT NOT NULL,
    entry_count INTEGER NOT NULL,
    address_count INTEGER NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);   