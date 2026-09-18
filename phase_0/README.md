# ODAMP — Phase 0

This is the first working scaffold of ODAMP, built to match `ODAMP_Whitepaper.md`.
Nothing in here is a stub that returns fake data — every module either does
real work (threshold signing, portfolio math, sanctions matching) or is
explicitly commented as a placeholder to be replaced (e.g. the in-memory
wallet store, the tiny hardcoded sanctions list).

## What's here

```
odamp/
├── Cargo.toml                  # workspace root
├── core/
│   ├── security/                # real FROST threshold signing (RFC 9591)
│   ├── portfolio/                # real Sharpe ratio / max drawdown / allocation math
│   └── compliance/               # real OFAC-style address screening logic
├── api/                          # Axum HTTP server wiring the three crates together
├── migrations/0001_init.sql      # Postgres schema (not wired to the API yet — Phase 2)
├── docker-compose.yml            # local Postgres for when Phase 2 wires up persistence
└── .gitignore
```

## Merging this into your existing repo

You already have `Cargo.toml`, `core/`, and `frontend/web/` at the root of
`odamp`. To merge:

1. Copy `core/security/`, `core/portfolio/`, `core/compliance/`, and `api/`
   into your existing repo, next to (or replacing, if empty) what's already
   in `core/`.
2. Either replace your root `Cargo.toml` with this one, or — if you already
   have content in yours — merge the `[workspace]` members list and
   `[workspace.dependencies]` table in.
3. Copy `migrations/`, `docker-compose.yml`, and `.gitignore` in as-is.
4. Your `frontend/web/` is untouched — we'll scaffold that in the next step.

## Running it in VS Code

```bash
cd odamp
cargo build
```

This is the first command that will actually hit the network (to pull
`frost-secp256k1`, `axum`, etc. from crates.io) — I couldn't run this in my
sandbox, so **this is the first real test**. If you get a compile error,
paste it back to me exactly and I'll fix it — don't try to guess-fix FROST
API mismatches yourself, that crate's API does shift between versions.

Once it builds:

```bash
cargo test          # runs the real unit tests in security/ and portfolio/ and compliance/
cargo run --bin odamp-api
```

Then in a second terminal:

```bash
curl http://localhost:8080/health
# -> ok

curl -X POST http://localhost:8080/api/v1/security/wallet \
  -H "Content-Type: application/json" \
  -d '{"threshold": 2, "total_shares": 3}'
# -> a real threshold wallet, with a real secp256k1 group public key

curl -X POST http://localhost:8080/api/v1/portfolio/analyze \
  -H "Content-Type: application/json" \
  -d '{"positions": [
        {"id":"00000000-0000-0000-0000-000000000001","asset_symbol":"BTC","asset_type":"crypto","amount":0.5,"avg_cost_usd":40000,"current_price_usd":60000}
      ]}'
# -> real total value / P&L / allocation, computed from the numbers you sent
```

## What Phase 0 deliberately does NOT do yet

- No database wiring (the schema exists in `migrations/`, the API doesn't
  use it yet — that's Phase 2, on-chain indexer + portfolio persistence).
- No real on-chain data (portfolio positions are supplied by the caller,
  not pulled from a real chain — that's also Phase 2).
- No live OFAC feed (the sanctions list is a two-address placeholder, not
  the real published SDN list — Phase 3).
- No DKG (distributed key generation) — trusted-dealer keygen only, which
  is explicitly flagged as dev-only in `core/security/src/lib.rs`.

## Next steps

Tell me once `cargo build` and `cargo test` succeed (or paste the errors),
and we'll move to either (a) wiring the Postgres schema into the API, or
(b) scaffolding `frontend/web` as a Next.js app that talks to this API —
your call.
