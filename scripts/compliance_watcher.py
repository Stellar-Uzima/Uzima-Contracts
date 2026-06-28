#!/usr/bin/env python3
"""
A simple polling watcher that invokes contract methods periodically to detect new breaches/audit events.
This is intentionally simple and uses subprocess to call `soroban contract invoke` for portability.

Usage:
  ./scripts/compliance_watcher.py <network> <contract_id> [poll_seconds]

Note: This script is a minimal example for testing and demonstration only.
"""
import sys
import time
import subprocess
import json
from pathlib import Path

if len(sys.argv) < 3:
    print("Usage: compliance_watcher.py <network> <contract_id> [poll_seconds]")
    sys.exit(1)

network = sys.argv[1]
contract = sys.argv[2]
poll = int(sys.argv[3]) if len(sys.argv) > 3 else 10

seen_events = set()

while True:
    try:
        # Invoke a small helper or query (here we fetch metrics as a proxy for activity)
        cmd = ["soroban", "contract", "invoke", "--network", network, "--id", contract, "--fn", "get_compliance_metrics"]
        res = subprocess.run(cmd, capture_output=True, text=True)
        out = res.stdout.strip() or res.stderr.strip()
        # write output for debugging
        Path("/tmp/compliance_metrics.json").write_text(out)

        # In real usage parse events or compare audit logs to detect new breaches
        print("[watcher] fetched metrics at", time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))

    except Exception as e:
        print("Watcher error:", e)

    time.sleep(poll)
