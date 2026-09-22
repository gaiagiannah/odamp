#!/usr/bin/env bash
set -euo pipefail

# Fetch OFAC SDN list and load into Postgres.
# Run daily via cron: 0 6 * * * /path/to/load-ofac-sdn.sh

OFAC_URL="${OFAC_SDN_URL:-https://sanctions.ofac.treas.gov/api/sdn/v1/sdnList}"
DATABASE_URL="${DATABASE_URL:-postgresql://odamp:odamp_dev@localhost:5432/odamp}"

echo "[$(date -Iseconds)] Fetching OFAC SDN list..."

# Fetch JSON
TMP_FILE=$(mktemp)
trap "rm -f $TMP_FILE" EXIT

curl -sS -o "$TMP_FILE" "$OFAC_URL"

if [ ! -s "$TMP_FILE" ]; then
    echo "ERROR: OFAC API returned empty response" >&2
    exit 1
fi

# Parse and load (uses Python for JSON parsing)
python3 - "$TMP_FILE" "$DATABASE_URL" <<'EOF'
import sys, json, hashlib
from datetime import datetime, timezone

import psycopg2

tmp_file, db_url = sys.argv[1], sys.argv[2]

with open(tmp_file) as f:
    data = json.load(f)

entries = data.get("results", data if isinstance(data, list) else [])
if not entries:
    print("WARNING: No entries found in OFAC response", file=sys.stderr)
    sys.exit(1)

# Compute list version
addr_count = sum(len(e.get("addresses", [])) for e in entries)
version = f"{len(entries)} entries, {addr_count} addresses, {datetime.now(timezone.utc).isoformat()}"
version_hash = hashlib.sha256(json.dumps(entries, sort_keys=True).encode()).hexdigest()[:16]

conn = psycopg2.connect(db_url)
cur = conn.cursor()

# Truncate and reload (simple approach for v0.x)
cur.execute("TRUNCATE sdn_entries RESTART IDENTITY")

for e in entries:
    cur.execute("""
        INSERT INTO sdn_entries (uid, name, aliases, addresses, program, country, date_listed, date_delisted)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
    """, (
        e.get("uid", ""),
        e.get("name", ""),
        e.get("aliases", []),
        e.get("addresses", []),
        e.get("program"),
        e.get("country"),
        e.get("dateListed"),
        e.get("dateDelisted"),
    ))

# Update metadata
cur.execute("""
    INSERT INTO sdn_list_meta (version, entry_count, address_count)
    VALUES (%s, %s, %s)
""", (f"{version} (hash: {version_hash})", len(entries), addr_count))

conn.commit()
cur.close()
conn.close()

print(f"[$(date -Iseconds)] Loaded {len(entries)} SDN entries ({addr_count} addresses). Version: {version_hash}")
EOF   