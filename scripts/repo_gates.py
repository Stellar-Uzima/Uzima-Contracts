#!/usr/bin/env python3
"""Small CI gates: plans | artifacts BASE | links BASE | labels."""

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Known baseline: this plan still carries placeholders (see its warning note).
# Remove the entry once the plan holds real values.
PLAN_ALLOWLIST = {"deployments/testnet/medical_records/plan.json"}
PLACEHOLDER = re.compile(
    r"REPLACE_WITH|PLACEHOLDER|CHANGE_?ME|\bTODO\b|<[A-Z_]{3,}>|(?:0123456789abcdef){2,}|0{32,}",
    re.IGNORECASE,
)
ARTIFACT = re.compile(r"(^|/)target/|\.(rlib|rmeta|wasm)$")
LINK = re.compile(r'\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)')


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).splitlines()


def plans():
    bad = []
    for p in sorted(ROOT.glob("deployments/**/plan*.json")):
        rel = p.relative_to(ROOT).as_posix()
        if rel in PLAN_ALLOWLIST:
            continue
        for n, line in enumerate(p.read_text().splitlines(), 1):
            if PLACEHOLDER.search(line):
                bad.append(f"{rel}:{n}: placeholder value: {line.strip()}")
    return bad


def added(base):
    return git("diff", "--diff-filter=AM", "--name-only", f"origin/{base}...HEAD")


def artifacts(base):
    return [f"{f}: build artifact must not be committed" for f in added(base) if ARTIFACT.search(f)]


def links(base):
    bad = []
    for f in added(base):
        if not (f == "README.md" or (f.startswith("docs/") and f.endswith(".md"))):
            continue
        src = ROOT / f
        if not src.is_file():
            continue
        fenced = False
        for n, line in enumerate(src.read_text(errors="replace").splitlines(), 1):
            if line.lstrip().startswith("```"):
                fenced = not fenced
            if fenced:
                continue
            for target in LINK.findall(line):
                if re.match(r"^([a-z][a-z0-9+.-]*:|#|//)", target, re.I):
                    continue
                rel = target.split("#")[0].split("?")[0]
                if not rel:
                    continue
                dest = ROOT / rel.lstrip("/") if rel.startswith("/") else src.parent / rel
                if not dest.exists():
                    bad.append(f"{f}:{n}: broken relative link: {target}")
    return bad


def labels():
    policy = json.loads((ROOT / ".github/issue-label-policy.json").read_text())
    for name in policy["labels"]:
        subprocess.run(["gh", "label", "create", name, "--force"], cwd=ROOT, check=True)
    print(f"Synced {len(policy['labels'])} canonical labels")
    return []


def main():
    cmd, arg = (sys.argv[1:] + [None])[:2]
    gates = {"plans": plans, "artifacts": artifacts, "links": links, "labels": labels}
    if cmd not in gates:
        print(__doc__, file=sys.stderr)
        return 2
    bad = gates[cmd](arg) if arg else gates[cmd]()
    print("\n".join(bad) if bad else f"{cmd}: ok")
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
