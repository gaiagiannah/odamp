# ODAMP — Phase 0 + Phase 1

## What changed in Phase 1

Phase 0 proved the crypto/math/compliance logic worked in isolation, with an
in-memory API. Phase 1 does two things:

1. **Wires real Postgres persistence** — wallets (public info only),
   positions, and a sanctions-screening audit trail now survive a restart.
2. **Fixes a real architectural bug from Phase 0**: the old API kept every
   generated key share in server memory so `/wallet/sign` had something to
   sign with. A process holding all N shares can use the key regardless of
   "threshold" — that defeats the point. Now, `generate_wallet_trusted_dealer`
   returns shares to the caller exactly once and the library keeps no copy.
   Signing requires the caller to supply the shares back in the request.
   This is more annoying to test by hand — that friction is intentional,
   it's what real self-custody looks like at the API boundary.

## Running it

### 1. Start Postgres

```bash
docker compose up -d
```

### 2. Set your DATABASE_URL

```bash
cp .env.example .env
# defaults already match docker-compose.yml, edit if you changed anything
```

### 3. Build and run

```bash
cargo build
cargo test
cargo run --bin odamp-api
```

The API applies migrations automatically at startup (via `sqlx::migrate!`
against `../migrations`) — you should see `connected to Postgres and ran
migrations` in the logs.

## Trying the full flow

```bash
# 1. Create a user
curl -s -X POST http://localhost:8080/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"jurisdiction": "US"}'
# -> {"user_id": "..."}  — save this

USER_ID="<paste the user_id above>"

# 2. Create a 2-of-3 threshold wallet
curl -s -X POST http://localhost:8080/api/v1/security/wallet \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$USER_ID\", \"threshold\": 2, \"total_shares\": 3}"
# -> {"wallet_id": "...", "group_public_key_hex": "...", "shares": [3 shares]}
# SAVE all of this — the server does not keep the shares.

# 3. Sign something using 2 of the 3 shares from step 2
curl -s -X POST http://localhost:8080/api/v1/security/wallet/sign \
  -H "Content-Type: application/json" \
  -d '{
        "wallet_id": "<wallet_id from step 2>",
        "message": "move 0.1 BTC",
        "shares": [ <share[0] from step 2>, <share[1] from step 2> ]
      }'
# -> {"signature_hex": "..."}

# 4. Add a position
curl -s -X POST "http://localhost:8080/api/v1/portfolio/$USER_ID/positions" \
  -H "Content-Type: application/json" \
  -d '{
        "id": "11111111-1111-1111-1111-111111111111",
        "asset_symbol": "BTC",
        "asset_type": "crypto",
        "amount": 0.5,
        "avg_cost_usd": 40000,
        "current_price_usd": 60000
      }'

# 5. Read the portfolio back — this now persists across restarts
curl -s "http://localhost:8080/api/v1/portfolio/$USER_ID"
# -> {"total_value_usd": 30000, "total_unrealized_pnl_usd": 10000, ...}

# 6. Screen a transaction (also writes an audit-trail row)
curl -s -X POST http://localhost:8080/api/v1/compliance/screen \
  -H "Content-Type: application/json" \
  -d "{
        \"user_id\": \"$USER_ID\",
        \"wallet_id\": null,
        \"chain\": \"ethereum\",
        \"from_address\": \"0xabc...\",
        \"to_address\": \"0x0000000000000000000000000000000000dEaD\"
      }"
# -> blocked: true (that address is the Phase 1 placeholder sanctions entry)
```

## Known simplifications in this phase (read before treating anything as final)

- **Money fields are `f64`/`DOUBLE PRECISION`, not fixed-point.** Fine for a
  dashboard prototype; wrong for a real ledger. Flagged in
  `migrations/0001_init.sql` — a pre-launch task is migrating to
  `rust_decimal`/Postgres `NUMERIC` throughout.
- **PublicKeyPackage reconstruction in `sign_threshold`** rebuilds the
  group's public verification data from the supplied shares rather than
  loading a persisted `PublicKeyPackage` — noted in `core/security/src/lib.rs`
  as a Phase 2 cleanup once that's stored as its own column.
- **`sqlx::query` (runtime-checked), not `sqlx::query!` (compile-time
  checked)** — deliberate, so this compiles without a live DB connection
  available at `cargo build` time. Worth migrating once you have Postgres
  running locally during development.
- **Still no DKG** (distributed key generation) — trusted-dealer keygen
  only. Flagged in `core/security/src/lib.rs`.
- **Sanctions list is still a one-address placeholder**, not the real OFAC
  feed. That's Phase 3, unchanged from the Phase 0 plan.

## Next steps

Same fork in the road as before — tell me which:
- Wire the real OFAC SDN feed into the compliance crate (Phase 3 pulled
  forward, since the plumbing to use it is now in place)
- Build the on-chain indexer (real wallet balances from a real chain,
  replacing the manual `POST /positions` calls above)
- Scaffold `frontend/web` as a Next.js dashboard against this API
