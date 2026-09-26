#!/usr/bin/env python3
"""Ensure each first-level contract directory has an owner and review route."""

import fnmatch
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "contracts"
CODEOWNERS = ROOT / ".github/CODEOWNERS"
ROUTING = ROOT / ".github/review-routing.json"


def codeowner_patterns():
    patterns = []
    for line_number, raw in enumerate(CODEOWNERS.read_text().splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        fields = line.split()
        if len(fields) < 2 or not any(owner.startswith("@") for owner in fields[1:]):
            raise ValueError(f"{CODEOWNERS}:{line_number}: expected a path and @owner")
        patterns.append(fields[0].lstrip("/"))
    return patterns


def routing_patterns():
    data = json.loads(ROUTING.read_text())
    tiers = data.get("review_tiers", {})
    if not tiers:
        raise ValueError("review-routing.json has no review_tiers")
    return [path.lstrip("/") for tier in tiers.values() for path in tier.get("paths", [])]


def covered(directory, patterns):
    # Patterns in these files are repository-relative and contract directory
    # entries end in '/'. Match both that directory and files below it.
    candidates = (directory, directory.rstrip("/"))
    return any(fnmatch.fnmatchcase(candidate, pattern) for pattern in patterns for candidate in candidates)


def main():
    try:
        owners = codeowner_patterns()
        routes = routing_patterns()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"Configuration error: {error}", file=sys.stderr)
        return 2

    directories = sorted(path.name for path in CONTRACTS.iterdir() if path.is_dir())
    missing_owners = [name for name in directories if not covered(f"contracts/{name}/", owners)]
    missing_routes = [name for name in directories if not covered(f"contracts/{name}/", routes)]
    if missing_owners or missing_routes:
        if missing_owners:
            print("Missing CODEOWNERS coverage:", ", ".join(missing_owners), file=sys.stderr)
        if missing_routes:
            print("Missing review-routing coverage:", ", ".join(missing_routes), file=sys.stderr)
        return 1

    print(f"Ownership and review routing cover all {len(directories)} contract directories.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
