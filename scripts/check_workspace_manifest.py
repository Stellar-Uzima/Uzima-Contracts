#!/usr/bin/env python3
"""Reject duplicate entries in the root manifest's workspace path lists.

Cargo accepts a repeated `workspace.members` / `workspace.exclude` entry
without complaint, so duplicates accumulate silently and the list stops being
reviewable (#1632 kept five crates twice). This check parses the manifest the
way Cargo does and fails when either list names the same path more than once,
reporting the line numbers so the offending entry is easy to delete.

Requires Python 3.11+ for `tomllib`.
"""

from collections import Counter
from pathlib import Path
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "Cargo.toml"
KEYS = ("members", "exclude")


def line_numbers(text, key, value):
    """Return the 1-based lines where `"value"` appears inside `key = [...]`."""
    quoted = f'"{value}"'
    found = []
    inside = False
    for number, line in enumerate(text.splitlines(), start=1):
        stripped = line.split("#", 1)[0]
        if not inside:
            inside = stripped.strip().startswith(f"{key} = [")
            continue
        if stripped.strip().startswith("]"):
            break
        if quoted in stripped:
            found.append(number)
    return found


def main():
    try:
        text = MANIFEST.read_text(encoding="utf-8")
        manifest = tomllib.loads(text)
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"Manifest error: {error}", file=sys.stderr)
        return 2

    workspace = manifest.get("workspace", {})
    problems = []
    for key in KEYS:
        entries = workspace.get(key)
        if not isinstance(entries, list):
            print(f"Manifest error: workspace.{key} is missing or not a list", file=sys.stderr)
            return 2
        for value, count in Counter(entries).items():
            if count > 1:
                lines = ", ".join(str(n) for n in line_numbers(text, key, value))
                problems.append(f"workspace.{key}: {value!r} listed {count}x (lines {lines})")

    if problems:
        print(f"{MANIFEST.relative_to(ROOT)} has duplicate workspace entries:", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        print("Remove the repeated entries; each path belongs in the list once.", file=sys.stderr)
        return 1

    total = sum(len(workspace.get(key, [])) for key in KEYS)
    print(f"Workspace manifest lists {total} unique paths across {', '.join(KEYS)}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
