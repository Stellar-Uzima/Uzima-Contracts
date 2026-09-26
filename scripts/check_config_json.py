#!/usr/bin/env python3
"""Reject unparseable JSON in config/.

`config/health_alerts.json` was committed as an 11-byte truncated fragment
(`{ "$schem`), which is not valid JSON and was referenced by nothing. Nothing
ran at PR time that would have noticed, so it survived unnoticed (#1635).

This check parses every `config/*.json` and fails with the offending path and
the decoder's message. It is deliberately syntax-only and dependency-free:
full schema validation already lives in `scripts/validate_config.mjs` (Ajv), but
that script needs `npm install` and is not wired into CI, so this is the gate
that always runs.
"""

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
CONFIG_DIR = ROOT / "config"


def main():
    if not CONFIG_DIR.is_dir():
        print(f"Config directory not found: {CONFIG_DIR}", file=sys.stderr)
        return 2

    manifests = sorted(CONFIG_DIR.glob("*.json"))
    if not manifests:
        print(f"No JSON manifests found in {CONFIG_DIR.relative_to(ROOT)}/", file=sys.stderr)
        return 2

    broken = []
    for manifest in manifests:
        relative = manifest.relative_to(ROOT)
        try:
            json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
            broken.append(f"{relative}: {error}")

    if broken:
        print("config/ contains malformed JSON:", file=sys.stderr)
        for entry in broken:
            print(f"  {entry}", file=sys.stderr)
        print("Commit complete, valid JSON — a truncated file is not a config.", file=sys.stderr)
        return 1

    print(f"All {len(manifests)} manifests in config/ parse as valid JSON.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
