# ODAMP — Phase 0 + 1 + 2 + 3

Four unverified phases stacked now. When you do sit down to build, go
bottom-up: `cargo build -p odamp-security`, then `-p odamp-portfolio`, then
`-p odamp-compliance`, then `-p odamp-indexer`, then finally the full `api`
binary. That way an error in an early crate doesn't show up disguised as a
confusing error in a later one.

## Phase 3: the OFAC sanctions feed — read this before trusting it

This is the phase I'm least confident is fully correct, and I want to be
specific about why, because it's compliance code and overclaiming here is
worse than overclaiming almost anywhere else in the project.

OFAC's real published sanctions data uses an XML schema where a sanctioned
party's digital-currency address is represented through a layer of
cross-referenced lookup tables (a `Feature` node references a `FeatureType`
by ID, resolved against a separate table to mean "Digital Currency Address -
XBT", etc.). Implementing that schema correctly, from memory, with no live
file to test against, is a real risk of silently getting it wrong.

So `core/compliance/src/ofac.rs` does something more modest instead: it
makes a real HTTP fetch of OFAC's published data, then runs a **regex-based
heuristic** — look for Bitcoin/Ethereum-shaped strings near the literal text
"Digital Currency Address" — rather than claiming to correctly parse the
schema. This will have both false positives and false negatives. **Do not
treat this as a reliable compliance control until you've validated its
output against OFAC's own published list of known sanctioned addresses** —
the module doc comment in `ofac.rs` spells out exactly what to check.

### What's wired up

- App startup now attempts a real fetch from OFAC's sanctions list service
  (`DEFAULT_OFAC_URL` in `ofac.rs`) and falls back to the old one-address
  placeholder if it fails or extracts nothing — logged loudly either way,
  because running on the placeholder means screening isn't actually
  protecting anything.
- `POST /api/v1/compliance/refresh` — manually re-fetch without restarting
  the process. Not scheduled yet (no background cron/interval task) — that's
  a reasonable next addition once the extraction itself is validated.
- `/api/v1/compliance/screen` now reads through a `RwLock` instead of a
  fixed list, so a refresh actually takes effect for subsequent screens.

### The URL itself may be wrong

`DEFAULT_OFAC_URL` is my best knowledge of OFAC's current data-service
endpoint (they migrated to a new "Sanctions List Service" platform in 2024).
Government data endpoints move. If startup logs a fetch failure, check
https://ofac.treasury.gov/sanctions-list-service for the current published
location before assuming the parsing logic is broken — it might just be the
URL.

### First thing to actually test

```bash
cargo test -p odamp-compliance
```

This runs the heuristic extractor against small inline fixtures (not the
real OFAC file) so you can confirm the regex logic itself works before
worrying about whether the live fetch and schema-guessing are right.

## Everything from Phases 0–2 still applies

See `core/security/src/lib.rs`, `api/src/db.rs`, `core/indexer/src/lib.rs`
doc comments for the running list of what's real vs. simplified vs.
flagged-for-later in each layer.

## Next steps

- Validate the OFAC extraction against known sanctioned addresses (see
  above) — arguably higher priority than new features at this point
- Scaffold `frontend/web` as a Next.js dashboard against this API
- Add scheduled (not just manual) sanctions refresh
