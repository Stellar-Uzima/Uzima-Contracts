#!/usr/bin/env python3
"""Validate one newly applied issue label against the canonical taxonomy."""

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / ".github/issue-label-policy.json"


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {Path(sys.argv[0]).name} LABEL", file=sys.stderr)
        return 2
    try:
        policy = json.loads(POLICY.read_text())
        allowed = set(policy["labels"])
    except (OSError, ValueError, KeyError) as error:
        print(f"Label policy error: {error}", file=sys.stderr)
        return 2

    label = sys.argv[1]
    if label not in allowed:
        print(f"Unsupported issue label: {label!r}", file=sys.stderr)
        print("Use a canonical label from .github/issue-label-policy.json.", file=sys.stderr)
        return 1
    print(f"Issue label is canonical: {label}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
