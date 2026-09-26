#!/usr/bin/env python3
"""
Fail if the committed dashboard data no longer matches its generators.

dashboard/contract_inventory.json and dashboard/risk_matrix.json are generated
artifacts, but nothing regenerated or checked them, so both sat stale:
risk_matrix.json shipped `"contracts": []` and contract_inventory.json shipped
five hand-written sample entries with a hardcoded timestamp, for a repository
of 114 contracts.

Run ./scripts/generate_dashboard_data.sh to fix a failure.

Volatile fields are ignored when comparing. `generated_at` is a wall-clock
timestamp, so byte-comparing it would fail on every run and make the guard
useless; the rest of the document is compared exactly.
"""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

from risk_scoring_engine import build_risk_matrix  # noqa: E402

# Fields that legitimately differ between two runs of the same generator.
VOLATILE = {"generated_at"}


def strip_volatile(document):
    return {k: v for k, v in document.items() if k not in VOLATILE}


def load_committed(relative):
    path = ROOT / relative
    if not path.exists():
        return None
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def expected_inventory():
    """contract_inventory.sh writes to stdout when given no path."""
    result = subprocess.run(
        ["bash", str(ROOT / "scripts" / "contract_inventory.sh")],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(result.stdout)


def summarise(label, expected, actual):
    """A short, actionable description of the difference."""
    lines = [f"  {label}: committed is {type(actual).__name__}"
             f", generated is {type(expected).__name__}"]
    if isinstance(expected, dict) and isinstance(actual, dict):
        lines.append(f"  top-level keys: committed {sorted(actual)}")
        lines.append(f"                   generated {sorted(expected)}")
    for field in ("contracts",):
        if isinstance(expected.get(field), list) and isinstance(actual.get(field), list):
            lines.append(
                f"  {field}: committed {len(actual[field])} entr"
                f"{'y' if len(actual[field]) == 1 else 'ies'}, "
                f"generated {len(expected[field])}"
            )
            expected_by_name = {
                c.get("contract") or c.get("name"): c for c in expected[field]
            }
            actual_by_name = {
                c.get("contract") or c.get("name"): c for c in actual[field]
            }
            for name in sorted(set(expected_by_name) - set(actual_by_name))[:5]:
                lines.append(f"    missing from committed: {name}")
            for name in sorted(set(actual_by_name) - set(expected_by_name))[:5]:
                lines.append(f"    stale in committed:     {name}")
            # Same contracts, different numbers: name diffing cannot explain it.
            changed = [
                name
                for name in sorted(set(expected_by_name) & set(actual_by_name))
                if expected_by_name[name] != actual_by_name[name]
            ]
            for name in changed[:5]:
                lines.append(
                    f"    {name}: committed score "
                    f"{actual_by_name[name].get('composite_score')} "
                    f"({actual_by_name[name].get('risk_tier')}), generated "
                    f"{expected_by_name[name].get('composite_score')} "
                    f"({expected_by_name[name].get('risk_tier')})"
                )
            if len(changed) > 5:
                lines.append(f"    ...and {len(changed) - 5} more")
    return "\n".join(lines)


def check(relative, expected):
    actual = load_committed(relative)
    if actual is None:
        print(f"[dashboard] {relative} is missing.")
        return False
    if strip_volatile(expected) != strip_volatile(actual):
        print(f"[dashboard] {relative} is stale:")
        print(summarise(relative, strip_volatile(expected), strip_volatile(actual)))
        return False
    count = len(expected.get("contracts", []))
    print(f"[dashboard] {relative} is up to date ({count} contracts).")
    return True


def main():
    ok = check("dashboard/contract_inventory.json", expected_inventory())
    ok = check("dashboard/risk_matrix.json", build_risk_matrix(str(ROOT / "contracts"))) and ok

    if not ok:
        print("\nRun ./scripts/generate_dashboard_data.sh and commit the result.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
