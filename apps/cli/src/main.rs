use clap::{Parser, Subcommand};
use colored::Colorize;

mod wallet;
mod sign;
mod balance;
mod screen;

#[derive(Parser)]
#[command(name = "odamp", version, about = "ODAMP CLI — Digital Asset Management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        action: WalletCommands,
    },
    /// Sign a message or transaction
    Sign {
        /// Hex-encoded message to sign
        #[arg(long)]
        message: String,
        /// Comma-separated share IDs (e.g., "1,2")
        #[arg(long)]
        shares: String,
    },
    /// Check balance on-chain
    Balance {
        /// Wallet address
        #[arg(long)]
        address: String,
        /// Chain name (ethereum, base, sepolia, etc.)
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    /// Screen an address against OFAC SDN list
    Screen {
        /// Address to screen
        #[arg(long)]
        address: String,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Create a new threshold wallet (DKG ceremony)
    Create {
        /// Number of key shares
        #[arg(long, default_value_t = 3)]
        shares: u32,
        /// Signing threshold
        #[arg(long, default_value_t = 2)]
        threshold: u32,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    /// List all known wallets
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Wallet { action } => match action {
            WalletCommands::Create { shares, threshold, chain } => {
                wallet::create(shares, threshold, &chain)?;
            }
            WalletCommands::List => {
                wallet::list()?;
            }
        },
        Commands::Sign { message, shares } => {
            sign::sign(&message, &shares)?;
        }
        Commands::Balance { address, chain } => {
            balance::check(&address, &chain).await?;
        }
        Commands::Screen { address } => {
            screen::screen(&address)?;
        }
    }

    Ok(())
}   

                Ok(result) => {
                    println!("{}", "✓ Signature produced".green().bold());
                    println!("{}", format!("  Sig:  {}", result.signature).bold());
                    println!("{}", format!("  Hash: {}", result.message_hash).dimmed());
                    println!("{}", format!("  Shares: {:?}", result.shares_used).dimmed());
                }
                Err(e) => {
                    println!("{}", format!("  ✗ {}", e).red());
                }
            }
        }
        Commands::Balance { address, chain } => {
            let chain_config = odamp_shared::get_chain(&chain)
                .ok_or_else(|| anyhow::anyhow!("Unknown chain: {}", chain))?;
            let rpc_url = std::env::var(chain_config.rpc_env_var)
                .map_err(|_| anyhow::anyhow!("Set {} in .env", chain_config.rpc_env_var))?;

            println!("{}", "ODAMP — Balance Check".bold());
            println!("{}", format!("  Address: {}  Chain: {}", address, chain).dimmed());

            let client = odamp_indexer::EvmClient::new(&rpc_url);
            let native = client.get_native_balance(&address).await?;
            let native_f64: f64 = native.parse::<u128>().unwrap_or(0) as f64 / 1e18;
            println!("{}", format!("  Native:  {:.6} {}", native_f64, chain_config.name).green());

            let tokens = [
                odamp_shared::USDC_ETHEREUM,
                odamp_shared::USDT_ETHEREUM,
                odamp_shared::WBTC_ETHEREUM,
                odamp_shared::DAI_ETHEREUM,
            ];
            for token in &tokens {
                if let Ok(bal) = client.get_erc20_balance(token.address, &address).await {
                    let bal_f64: f64 = bal.parse::<u128>().unwrap_or(0) as f64 / 10f64.powi(token.decimals as i32);
                    if bal_f64 > 0.0 {
                        println!("{}", format!("  {:<10} {:.4}", token.symbol, bal_f64).green());
                    }
                }
            }
        }
        Commands::Screen { address } => {
            println!("{}", "ODAMP — OFAC SDN Screening".bold());
            println!("{}", format!("  Address: {}", address).dimmed());

            let mut engine = odamp_compliance::ComplianceEngine::new();
            let cache = std::path::Path::new(std::env::var("HOME").as_deref().unwrap_or("/root"))
                .join(".odamp").join("sdn_cache.json");

            if cache.exists() {
                let json = std::fs::read_to_string(&cache)?;
                let entries: Vec<odamp_compliance::ofac::SdnEntry> = serde_json::from_str(&json)?;
                let version = odamp_compliance::ofac::list_version(&entries);
                engine.load_sdn_list(entries, version);
            } else {
                println!("{}", "  ⚠ No local SDN cache. Result will be FLAGGED (fail-closed).".yellow());
            }

            let result = engine.screen(&address);
            match result.status {
                odamp_shared::ScreenResult::Clean => println!("{}", "  ✓ CLEAN".green().bold()),
                odamp_shared::ScreenResult::Flagged => {
                    println!("{}", "  ⚠ FLAGGED".yellow().bold());
                    if let Some(e) = &result.matched_entity { println!("{}", format!("  Reason: {}", e).yellow()); }
                }
                odamp_shared::ScreenResult::Blocked => {
                    println!("{}", "  ✗ BLOCKED".red().bold());
                    if let Some(e) = &result.matched_entity { println!("{}", format!("  Entity: {}", e).red()); }
                }
            }
            println!("{}", format!("  List: {}", result.list_version).dimmed());
        }
    }

    Ok(())
}
EOF

# ─── infra/docker-compose.yml ──────────────────────────────────
cat > infra/docker-compose.yml << 'EOF'
services:
  postgres:
    image: timescale/timescaledb:latest-pg16
    environment:
      POSTGRES_DB: odamp
      POSTGRES_USER: odamp
      POSTGRES_PASSWORD: odamp_dev
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U odamp"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  ai-agent:
    build:
      context: ../services/ai-agent
      dockerfile: Dockerfile
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://odamp:odamp_dev@postgres:5432/odamp
      LLM_PROVIDER: ${LLM_PROVIDER:-anthropic}
      LLM_API_KEY: ${LLM_API_KEY}
      LLM_MODEL: ${LLM_MODEL:-claude-sonnet-4-20250514}
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  pgdata:
EOF

# ─── infra/migrations ──────────────────────────────────────────
cat > infra/migrations/001_init.sql << 'EOF'
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    jurisdiction VARCHAR(10) NOT NULL,
    metadata JSONB DEFAULT '{}'
);

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
    UNIQUE(wallet_id, chain, COALESCE(token_address, 'native'))
);

CREATE INDEX IF NOT EXISTS idx_positions_wallet ON positions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_positions_updated ON positions(updated_at DESC);

CREATE TABLE IF NOT EXISTS signing_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    message_hash TEXT NOT NULL,
    signature TEXT NOT NULL,
    shares_used SMALLINT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_signing_wallet ON signing_logs(wallet_id, created_at DESC);
EOF

cat > infra/migrations/002_compliance.sql << 'EOF'
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

CREATE TABLE IF NOT EXISTS compliance_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address TEXT NOT NULL,
    status VARCHAR(16) NOT NULL,
    matched_entity TEXT,
    program VARCHAR(32),
    list_version TEXT NOT NULL,
    tx_context JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_compliance_created ON compliance_logs(created_at DESC);

CREATE TABLE IF NOT EXISTS sdn_list_meta (
    id SERIAL PRIMARY KEY,
    version TEXT NOT NULL,
    entry_count INTEGER NOT NULL,
    address_count INTEGER NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
EOF

cat > infra/migrations/003_ai_agents.sql << 'EOF'
CREATE TABLE IF NOT EXISTS ai_agent_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent VARCHAR(64) NOT NULL,
    model VARCHAR(128) NOT NULL,
    prompt_hash TEXT NOT NULL,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    latency_ms INTEGER,
    response JSONB NOT NULL,
    findings_count INTEGER DEFAULT 0,
    max_severity VARCHAR(16),
    wallet_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_logs_wallet ON ai_agent_logs(wallet_id, created_at DESC);
EOF

cat > infra/migrations/004_execution.sql << 'EOF'
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
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    frost_shares_used SMALLINT[],
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_exec_wallet ON execution_logs(wallet_id, created_at DESC);
EOF

cat > infra/migrations/005_indexer.sql << 'EOF'
CREATE TABLE IF NOT EXISTS indexer_checkpoints (
    chain VARCHAR(32) PRIMARY KEY,
    last_block BIGINT NOT NULL,
    last_token_sync TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tracked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chain VARCHAR(32) NOT NULL,
    contract_address VARCHAR(128),
    symbol VARCHAR(32) NOT NULL,
    name VARCHAR(128) NOT NULL,
    decimals SMALLINT NOT NULL DEFAULT 18,
    coingecko_id VARCHAR(64) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(chain, COALESCE(contract_address, 'native'))
);
EOF

cat > infra/migrations/006_automation.sql << 'EOF'
CREATE TABLE IF NOT EXISTS automation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    trigger_type VARCHAR(32) NOT NULL,
    trigger_config JSONB NOT NULL,
    action_type VARCHAR(32) NOT NULL,
    action_config JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS automation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES automation_rules(id),
    trigger_data JSONB NOT NULL,
    action_taken VARCHAR(32) NOT NULL,
    result JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
EOF

# ─── infra/scripts/load-ofac-sdn.sh ────────────────────────────
cat > infra/scripts/load-ofac-sdn.sh << 'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

OFAC_URL="${OFAC_SDN_URL:-https://sanctions.ofac.treas.gov/api/sdn/v1/sdnList}"
DATABASE_URL="${DATABASE_URL:-postgresql://odamp:odamp_dev@localhost:5432/odamp}"

echo "[$(date -Iseconds)] Fetching OFAC SDN list..."
TMP_FILE=$(mktemp)
trap "rm -f $TMP_FILE" EXIT

curl -sS -o "$TMP_FILE" "$OFAC_URL"

if [ ! -s "$TMP_FILE" ]; then
    echo "ERROR: OFAC API returned empty response" >&2
    exit 1
fi

python3 - "$TMP_FILE" "$DATABASE_URL" << 'PYEOF'
import sys, json, hashlib
from datetime import datetime, timezone
import psycopg2

tmp_file, db_url = sys.argv[1], sys.argv[2]

with open(tmp_file) as f:
    data = json.load(f)

entries = data.get("results", data if isinstance(data, list) else [])
if not entries:
    print("WARNING: No entries found", file=sys.stderr)
    sys.exit(1)

addr_count = sum(len(e.get("addresses", [])) for e in entries)
version_hash = hashlib.sha256(json.dumps(entries, sort_keys=True).encode()).hexdigest()[:16]
version = f"{len(entries)} entries, {addr_count} addresses, {datetime.now(timezone.utc).isoformat()} (hash: {version_hash})"

conn = psycopg2.connect(db_url)
cur = conn.cursor()
cur.execute("TRUNCATE sdn_entries RESTART IDENTITY")

for e in entries:
    cur.execute("""
        INSERT INTO sdn_entries (uid, name, aliases, addresses, program, country, date_listed, date_delisted)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
    """, (
        e.get("uid", ""), e.get("name", ""),
        e.get("aliases", []), e.get("addresses", []),
        e.get("program"), e.get("country"),
        e.get("dateListed"), e.get("dateDelisted"),
    ))

cur.execute("INSERT INTO sdn_list_meta (version, entry_count, address_count) VALUES (%s, %s, %s)",
            (version, len(entries), addr_count))

conn.commit()
cur.close()
conn.close()
print(f"[$(date -Iseconds)] Loaded {len(entries)} entries ({addr_count} addresses). Hash: {version_hash}")
PYEOF
SCRIPT
chmod +x infra/scripts/load-ofac-sdn.sh

# ─── services/ai-agent ─────────────────────────────────────────
cat > services/ai-agent/requirements.txt << 'EOF'
fastapi==0.115.*
uvicorn[standard]==0.34.*
anthropic==0.42.*
pydantic==2.*
psycopg2-binary==2.9.*
numpy==2.*
httpx==0.28.*
python-dotenv==1.*
pytest==8.*
EOF

cat > services/ai-agent/Dockerfile << 'EOF'
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
EOF

cat > services/ai-agent/config.py << 'EOF'
import os
from dotenv import load_dotenv

load_dotenv()

class Settings:
    def __init__(self):
        self.database_url = os.getenv("DATABASE_URL", "postgresql://odamp:odamp_dev@localhost:5432/odamp")
        self.llm_provider = os.getenv("LLM_PROVIDER", "anthropic")
        self.llm_api_key = os.getenv("LLM_API_KEY", "")
        self.llm_model = os.getenv("LLM_MODEL", "claude-sonnet-4-20250514")
        self.llm_max_tokens = int(os.getenv("LLM_MAX_TOKENS", "2048"))
        self.llm_temperature = float(os.getenv("LLM_TEMPERATURE", "0"))

settings = Settings()
EOF

cat > services/ai-agent/app.py << 'EOF'
"""ODAMP AI Agent — FastAPI entry point."""
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional
from agents.risk_sentinel import RiskSentinel
from config import settings
import logging

logging.basicConfig(level=logging.INFO)
app = FastAPI(title="ODAMP AI Agent", version="0.1.0")

agent = RiskSentinel(
    provider=settings.llm_provider,
    model=settings.llm_model,
    api_key=settings.llm_api_key,
    max_tokens=settings.llm_max_tokens,
    temperature=settings.llm_temperature,
)

class AnalyzeRequest(BaseModel):
    wallet_id: str
    portfolio_state: Optional[dict] = None

@app.get("/health")
async def health():
    return {"status": "ok", "agent": "risk_sentinel", "model": settings.llm_model}

@app.post("/analyze")
async def analyze(req: AnalyzeRequest):
    try:
        portfolio_state = req.portfolio_state
        if portfolio_state is None:
            portfolio_state = _fetch_portfolio_state(req.wallet_id)
        result = await agent.analyze(portfolio_state)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

def _fetch_portfolio_state(wallet_id: str) -> dict:
    try:
        import psycopg2
        conn = psycopg2.connect(settings.database_url)
        cur = conn.cursor()
        cur.execute("""
            SELECT symbol, chain, balance::TEXT, price_usd, value_usd
            FROM positions WHERE wallet_id = %s ORDER BY value_usd DESC
        """, (wallet_id,))
        rows = cur.fetchall()
        conn.close()
        return {
            "wallet_id": wallet_id,
            "positions": [
                {"symbol": r[0], "chain": r[1], "balance": r[2], "price_usd": float(r[3]), "value_usd": float(r[4])}
                for r in rows
            ],
            "total_value_usd": sum(float(r[4]) for r in rows),
        }
    except Exception as e:
        return {"wallet_id": wallet_id, "positions": [], "total_value_usd": 0.0, "error": str(e)}
EOF

cat > services/ai-agent/agents/__init__.py << 'EOF'
from .base_agent import BaseAgent
from .risk_sentinel import RiskSentinel
EOF

cat > services/ai-agent/agents/base_agent.py << 'EOF'
"""Base class for ODAMP AI agents."""
import hashlib, json, time
from abc import ABC, abstractmethod

class BaseAgent(ABC):
    def __init__(self, provider, model, api_key, max_tokens=2048, temperature=0.0):
        self.provider = provider
        self.model = model
        self.max_tokens = max_tokens
        self.temperature = temperature
        if provider == "anthropic":
            import anthropic
            self.client = anthropic.Anthropic(api_key=api_key)
        elif provider == "openai":
            import openai
            self.client = openai.OpenAI(api_key=api_key)

    @abstractmethod
    def build_prompt(self, data: dict) -> tuple: ...

    @abstractmethod
    def parse_response(self, raw: str) -> dict: ...

    async def analyze(self, data: dict) -> dict:
        system, user = self.build_prompt(data)
        prompt_hash = hashlib.sha256((system + user).encode()).hexdigest()
        start = time.time()
        raw = self._call_llm(system, user)
        latency_ms = int((time.time() - start) * 1000)
        result = self.parse_response(raw)
        result["model"] = self.model
        result["latency_ms"] = latency_ms
        result["prompt_hash"] = prompt_hash
        import logging
        logging.info(f"Agent call: model={self.model} latency={latency_ms}ms findings={len(result.get('findings', []))}")
        return result

    def _call_llm(self, system: str, user: str) -> str:
        if self.provider == "anthropic":
            resp = self.client.messages.create(
                model=self.model, max_tokens=self.max_tokens,
                temperature=self.temperature, system=system,
                messages=[{"role": "user", "content": user}],
            )
            return resp.content[0].text
        else:
            resp = self.client.chat.completions.create(
                model=self.model, max_tokens=self.max_tokens,
                temperature=self.temperature,
                messages=[{"role": "system", "content": system}, {"role": "user", "content": user}],
            )
            return resp.choices[0].message.content
EOF

cat > services/ai-agent/agents/risk_sentinel.py << 'EOF'
"""Risk Sentinel — LLM-backed portfolio risk analyst."""
import json
from .base_agent import BaseAgent

class RiskSentinel(BaseAgent):
    SYSTEM_PROMPT = """You are Risk Sentinel, a portfolio risk analyst for a digital asset management platform.

Rules:
- Every finding MUST include: claim, evidence (specific numbers), confidence (0-1), source
- Be specific. "High concentration" is not a finding. "72% in ETH, HHI=0.58" is.
- If data is insufficient, say so in data_gaps.
- Do NOT recommend specific trades. Recommend risk management actions.
- Output MUST be valid JSON matching the schema below.

Output schema:
{
  "risk_level": "LOW" | "ELEVATED" | "HIGH" | "CRITICAL",
  "findings": [{"claim": "", "evidence": "", "confidence": 0.0, "source": "", "severity": ""}],
  "recommendations": [""],
  "data_gaps": [""]
}"""

    def build_prompt(self, data: dict) -> tuple:
        user = f"Analyze this portfolio state:\n\n```json\n{json.dumps(data, indent=2)}\n```\n\nProduce your risk assessment as structured JSON."
        return self.SYSTEM_PROMPT, user

    def parse_response(self, raw: str) -> dict:
        text = raw.strip()
        if text.startswith("```"):
            lines = text.split("\n")
            text = "\n".join(l for l in lines if not l.startswith("```"))
        try:
            result = json.loads(text)
        except json.JSONDecodeError:
            return {
                "risk_level": "ELEVATED",
                "findings": [{"claim": "Agent response was not valid JSON", "evidence": raw[:200], "confidence": 0.0, "source": "parse_error", "severity": "LOW"}],
                "recommendations": ["Review agent output manually"],
                "data_gaps": [],
            }
        result.setdefault("risk_level", "ELEVATED")
        result.setdefault("findings", [])
        result.setdefault("recommendations", [])
        result.setdefault("data_gaps", [])
        return result
EOF

cat > services/ai-agent/tests/__init__.py << 'EOF'
EOF

cat > services/ai-agent/tests/test_risk_sentinel.py << 'EOF'
import json, pytest
from agents.risk_sentinel import RiskSentinel

@pytest.fixture
def agent():
    return RiskSentinel(provider="anthropic", model="claude-sonnet-4-20250514", api_key="test", max_tokens=2048, temperature=0)

def test_parse_valid_json(agent):
    raw = json.dumps({"risk_level": "ELEVATED", "findings": [{"claim": "48% in ETH", "evidence": "$14k of $29k", "confidence": 0.95, "source": "portfolio_state", "severity": "ELEVATED"}], "recommendations": ["Diversify"], "data_gaps": []})
    result = agent.parse_response(raw)
    assert result["risk_level"] == "ELEVATED"
    assert len(result["findings"]) == 1

def test_parse_markdown(agent):
    raw = '```json\n{"risk_level": "LOW", "findings": [], "recommendations": [], "data_gaps": []}\n```'
    result = agent.parse_response(raw)
    assert result["risk_level"] == "LOW"

def test_parse_invalid(agent):
    result = agent.parse_response("I cannot analyze this.")
    assert result["findings"][0]["source"] == "parse_error"
EOF

# ─── contracts ─────────────────────────────────────────────────
cat > contracts/safe-frost/README.md << 'EOF'
# Safe FROST Verifier

Solidity contract for on-chain FROST threshold signature verification.
Based on Safe Research's safe-frost. Deploy to Safe as EIP-1271 module.

## Deploy
```bash
forge script script/Deploy.s.sol --rpc-url $SEPOLIA_RPC_URL --private-key $PK --broadcast   