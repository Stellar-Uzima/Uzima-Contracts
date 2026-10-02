#!/usr/bin/env python3
"""Require an ADR for every breaking ABI or storage-layout change (issue #1651).

The ADR process (docs/adr/ADR-PROCESS.md) already said when a decision *should*
be recorded. Nothing enforced it, so a contract's exported functions or its
`DataKey` storage layout could change in a pull request with no decision record
anywhere in the diff. This script closes that gap and also keeps the ADR
directory itself tidy.

What it checks, in order:

1. **ADR hygiene** — every `docs/adr/ADR-NNN-*.md` has a matching title
   (`# ADR-NNN: ...`), a `**Status:**` whose value is one of the lifecycle
   states, and an ISO `**Date:**`.
2. **Index sync** — the "Existing ADRs" table in `docs/adr/ADR-PROCESS.md`
   lists exactly the numbered ADR files on disk.
3. **ADR required for breaking changes** — when the change set removes or
   re-signs a contract entry point, or drops a `DataKey` variant or
   `#[contracttype]` struct field, at least one numbered ADR must have been
   added or modified in the same change set. A maintainer can waive this with
   `--allow-breaking` when the change is provably not breaking; the waiver is
   printed so it is visible in CI logs rather than silent.

Change detection is conservative by design: a false positive costs a paragraph
of documentation, while a false negative ships an undocumented breaking change
to every integrator. Interface and storage differences are computed by
comparing the file at the merge base with the file at the head (via `git show`),
which is precise. When a diff is supplied out of band (`--diff-file`) and the
old/new content is unavailable, the script falls back to diff heuristics and
says so.

Usage:
    python3 scripts/check_adr_required.py                 # check HEAD vs origin/main
    python3 scripts/check_adr_required.py --base SHA --head SHA
    python3 scripts/check_adr_required.py --diff-file d.patch
    python3 scripts/check_adr_required.py --allow-breaking  # waive rule 3
    python3 scripts/check_adr_required.py --json

Exit codes: 0 clean, 1 violations, 2 environment/usage error.
"""

import argparse
import datetime
import json
import re
import subprocess
import sys
from pathlib import Path

# `docs/adr/ADR-001-foo.md` -> number 001.
ADR_FILE_RE = re.compile(r"^ADR-(\d{3})-.*\.md$")
# A numbered ADR referenced by the index table, e.g. `| ADR-001 | ...`.
ADR_INDEX_RE = re.compile(r"\|\s*ADR-(\d{3})\s*\|")
ADR_TITLE_RE = re.compile(r"^#\s+ADR-(\d{3})\s*:\s*\S", re.M)
ADR_STATUS_RE = re.compile(r"^\*\*Status:\*\*\s*(.+?)\s*$", re.M)
ADR_DATE_RE = re.compile(r"^\*\*Date:\*\*\s*(\S+)\s*$", re.M)

VALID_STATUSES = ("Proposed", "Accepted", "Deprecated", "Superseded")

# Contract source that can carry an external interface or a persisted schema.
CONTRACT_SRC_RE = re.compile(r"^contracts/([^/]+)/src/.*\.rs$")
# `pub fn name(` on one line.
PUB_FN_RE = re.compile(r"\bpub\s+fn\s+\w+\s*\(")
# The declaration of the storage-key enum: `enum DataKey {`.
DATAKEY_DECL_RE = re.compile(r"\benum\s+DataKey\s*\{")
# The body of the storage-key enum, DOTALL so it can span lines.
DATAKEY_BODY_RE = re.compile(r"\benum\s+DataKey\s*\{(.*?)\}", re.S)
# A `#[contracttype]` struct and its body.
CONTRACTTYPE_STRUCT_RE = re.compile(
    r"#\[contracttype\][^;]*?\bstruct\s+\w+\s*\{(.*?)\}", re.S
)
# A bare enum variant, e.g. `PatientRecords,` / `Admin(Address),`.
VARIANT_RE = re.compile(r"^\s*([A-Z][A-Za-z0-9_]*)\s*(\(.*\))?\s*,?\s*$")
# A struct field, e.g. `pub owner: Address,`. The `(?!:)` keeps the `std` in
# `x: std::string::String` from being read as a field name.
FIELD_RE = re.compile(r"(?:pub\s+)?([a-z_][A-Za-z0-9_]*)\s*:(?!:)")


def git(root, *args):
    """Run git and return (returncode, stdout)."""
    try:
        proc = subprocess.run(
            ["git", "-C", str(root), *args],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        raise SystemExit(f"[adr] could not run git: {error}") from error
    return proc.returncode, proc.stdout


def resolve_refs(root, args):
    """Return (base, head) commit refs, or (None, None) when unavailable.

    Mirrors what `git diff <range>` compares: for `A...B` the base is the
    merge-base of A and B, which is where file content should be read from.
    """
    if args.diff_file and not (args.range or args.base):
        return None, None
    head = args.head or "HEAD"
    if args.base:
        return args.base, head
    if args.range:
        if "..." in args.range:
            left, _, right = args.range.partition("...")
            head = right or head
            code, merge_base = git(root, "merge-base", left, head)
            return (merge_base.strip() if code == 0 else left), head
        left, _, right = args.range.partition("..")
        return left, (right or head)
    for candidate in ("origin/main", "origin/master", "main", "master"):
        code, _ = git(root, "rev-parse", "--verify", "--quiet", candidate)
        if code == 0:
            code, merge_base = git(root, "merge-base", candidate, "HEAD")
            return (merge_base.strip() if code == 0 else candidate), "HEAD"
    return "HEAD~1", "HEAD"


def acquire_diff(root, args, base, head):
    """Return the unified diff text for this change set."""
    if args.diff_file:
        try:
            return Path(args.diff_file).read_text(encoding="utf-8")
        except OSError as error:
            raise SystemExit(f"[adr] cannot read --diff-file: {error}") from error
    if args.range:
        diff_range = args.range
    elif base is not None:
        diff_range = f"{base}..{head}"
    else:
        diff_range = "HEAD~1..HEAD"
    code, out = git(root, "diff", "--unified=0", diff_range, "--")
    if code != 0:
        raise SystemExit(f"[adr] git diff {diff_range} failed")
    return out


def file_at(root, ref, path):
    """Return the text of `path` at `ref`, or None if it is not readable there."""
    if ref is None:
        return None
    code, out = git(root, "show", f"{ref}:{path}")
    return out if code == 0 else None


def parse_diff(diff_text):
    """Split a unified diff into {path: {'added': [], 'removed': [], 'context': []}}.

    Only content lines are kept; file and hunk headers are dropped. Paths come
    from the `+++ b/<path>` header, so a deleted file keeps its old path.
    """
    files = {}
    current = None
    for raw in diff_text.splitlines():
        if raw.startswith("+++ "):
            path = raw[4:].strip()
            if path == "/dev/null":
                current = None
                continue
            if path.startswith("b/"):
                path = path[2:]
            current = files.setdefault(path, {"added": [], "removed": [], "context": []})
            continue
        if raw.startswith("--- ") or raw.startswith("@@") or raw.startswith("diff --git"):
            continue
        if current is None:
            continue
        if raw.startswith("+"):
            current["added"].append(raw[1:])
        elif raw.startswith("-"):
            current["removed"].append(raw[1:])
        elif raw.startswith(" "):
            current["context"].append(raw[1:])
    return files


def is_contract_surface(path, lines, root):
    """True when a file exposes `#[contractimpl]` entry points."""
    if any("#[contractimpl]" in line for line in lines):
        return True
    on_disk = root / path
    if on_disk.is_file():
        try:
            return "#[contractimpl]" in on_disk.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return False
    return False


def normalize_pub_fn(line):
    """Return the whitespace-normalized `pub fn` signature, else None.

    The full text is kept, not just the function name: re-signing
    `fn foo(a: u32)` as `fn foo(a: u64)` keeps the name and only changes the
    arguments, and that is exactly the ABI break we need to catch.
    """
    if not PUB_FN_RE.search(line):
        return None
    return re.sub(r"\s+", " ", line.strip())


def datakey_variants(text):
    """Variant names declared in `enum DataKey { ... }`."""
    match = DATAKEY_BODY_RE.search(text)
    if not match:
        return set()
    variants = set()
    for line in match.group(1).splitlines():
        stripped = line.split("//", 1)[0].strip()
        variant = VARIANT_RE.match(stripped)
        if variant:
            variants.add(variant.group(1))
    return variants


def contracttype_fields(text):
    """Field names declared by `#[contracttype]` structs."""
    fields = set()
    for body in CONTRACTTYPE_STRUCT_RE.findall(text):
        for name in FIELD_RE.findall(body):
            fields.add(name)
    return fields


def storage_diff(contract, old, new):
    """Breaking storage changes between two versions of one contract file."""
    reasons = []
    if old is None or new is None:
        return reasons
    for variant in sorted(datakey_variants(old) - datakey_variants(new)):
        reasons.append(f"{contract}: DataKey::{variant} removed")
    for field in sorted(contracttype_fields(old) - contracttype_fields(new)):
        reasons.append(f"{contract}: #[contracttype] field `{field}` removed")
    return reasons


def storage_from_diff(contract, hunks, surface):
    """Fallback storage detection when old/new content is unavailable.

    Narrow on purpose: a removed `DataKey` variant is only reported when the
    `enum DataKey` declaration is itself visible in the diff, so a bare
    removed line in an ordinary refactor is not mistaken for a storage change.
    """
    reasons = []
    all_lines = hunks["added"] + hunks["removed"] + hunks["context"]
    if not surface and not any(DATAKEY_DECL_RE.search(line) for line in all_lines):
        return reasons
    if not any(DATAKEY_DECL_RE.search(line) for line in all_lines):
        return reasons
    added = {v for line in hunks["added"] if (v := variant_of(line))}
    for line in hunks["removed"]:
        variant = variant_of(line)
        if variant and variant not in added:
            reasons.append(f"{contract}: DataKey::{variant} removed")
    return reasons


def variant_of(line):
    """Return the variant name when a line is a bare enum variant, else None."""
    if "#" in line or "fn " in line or "=" in line or ";" in line:
        return None
    match = VARIANT_RE.match(line)
    return match.group(1) if match else None


def classify(files, root, base, head):
    """Return (breaking_abi, storage_changes) as sorted reason lists.

    Returns (reasons, used_fallback) so the caller can report when precise
    old/new comparison was not possible.
    """
    breaking_abi = []
    storage = []
    fallback_used = False

    for path, hunks in sorted(files.items()):
        match = CONTRACT_SRC_RE.match(path)
        if not match:
            continue
        contract = match.group(1)
        all_lines = hunks["added"] + hunks["removed"] + hunks["context"]
        surface = is_contract_surface(path, all_lines, root)

        if surface:
            added = {n for line in hunks["added"] if (n := normalize_pub_fn(line))}
            for line in hunks["removed"]:
                signature = normalize_pub_fn(line)
                if signature and signature not in added:
                    breaking_abi.append(f"{contract}: {signature}")

        old = file_at(root, base, path)
        new = file_at(root, head, path)
        if old is not None and new is not None:
            storage.extend(storage_diff(contract, old, new))
        else:
            fallback_used = True
            storage.extend(storage_from_diff(contract, hunks, surface))

    return sorted(set(breaking_abi)), sorted(set(storage)), fallback_used


def check_adr_files(adr_dir):
    """Rule 1: validate the numbered ADRs' required header fields."""
    problems = []
    numbered = sorted(p for p in adr_dir.glob("ADR-*.md") if ADR_FILE_RE.match(p.name))
    if not numbered:
        problems.append(f"no numbered ADR files found in {adr_dir}")
    for path in numbered:
        text = path.read_text(encoding="utf-8")
        number = ADR_FILE_RE.match(path.name).group(1)

        title = ADR_TITLE_RE.search(text)
        if not title:
            problems.append(f"{path.name}: missing '# ADR-{number}: <title>' heading")
        elif title.group(1) != number:
            problems.append(
                f"{path.name}: heading says ADR-{title.group(1)} but the file is ADR-{number}"
            )

        status = ADR_STATUS_RE.search(text)
        if not status:
            problems.append(f"{path.name}: missing '**Status:**' line")
        elif not any(status.group(1).startswith(state) for state in VALID_STATUSES):
            problems.append(
                f"{path.name}: status '{status.group(1)}' is not one of "
                + ", ".join(VALID_STATUSES)
            )

        date = ADR_DATE_RE.search(text)
        if not date:
            problems.append(f"{path.name}: missing '**Date:**' line")
        else:
            try:
                datetime.date.fromisoformat(date.group(1))
            except ValueError:
                problems.append(f"{path.name}: date '{date.group(1)}' is not YYYY-MM-DD")
    return numbered, problems


def check_index(adr_dir, adr_process):
    """Rule 2: the ADR-PROCESS.md index must list exactly the files on disk."""
    problems = []
    numbers_on_disk = {
        m.group(1) for p in adr_dir.glob("ADR-*.md") if (m := ADR_FILE_RE.match(p.name))
    }
    if not adr_process.is_file():
        return [f"missing {adr_process.name}; cannot verify the ADR index"]
    text = adr_process.read_text(encoding="utf-8")
    start = text.find("## Existing ADRs")
    if start == -1:
        return [f"{adr_process.name}: no '## Existing ADRs' index section"]
    indexed = set(ADR_INDEX_RE.findall(text[start:]))

    for number in sorted(numbers_on_disk - indexed):
        problems.append(
            f"ADR-{number} exists on disk but is missing from the ADR-PROCESS.md index"
        )
    for number in sorted(indexed - numbers_on_disk):
        problems.append(f"ADR-{number} is indexed in ADR-PROCESS.md but no such file exists")
    return problems


def changed_adrs(files):
    """Numbered ADRs added or modified anywhere in the change set."""
    found = set()
    for path in files:
        if not path.startswith("docs/adr/"):
            continue
        match = ADR_FILE_RE.match(Path(path).name)
        if match:
            found.add(match.group(1))
    return sorted(found)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=None, help="repository root (default: repo)")
    parser.add_argument("--range", dest="range", default=None, help="git diff range")
    parser.add_argument("--base", default=None, help="base SHA/ref (with --head)")
    parser.add_argument("--head", default=None, help="head SHA/ref (with --base)")
    parser.add_argument("--diff-file", default=None, help="read a unified diff from here")
    parser.add_argument(
        "--allow-breaking",
        action="store_true",
        help="waive rule 3 for this run (the waiver is printed)",
    )
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    args = parser.parse_args()

    root = Path(args.root).resolve() if args.root else Path(__file__).resolve().parents[1]
    adr_dir = root / "docs" / "adr"
    adr_process = adr_dir / "ADR-PROCESS.md"

    if not adr_dir.is_dir():
        raise SystemExit(f"[adr] missing ADR directory: {adr_dir}")

    numbered, hygiene = check_adr_files(adr_dir)
    index = check_index(adr_dir, adr_process)

    base, head = resolve_refs(root, args)
    diff_text = acquire_diff(root, args, base, head)
    files = parse_diff(diff_text)
    breaking_abi, storage_changes, fallback = classify(files, root, base, head)
    adrs_in_change = changed_adrs(files)

    violations = []
    if hygiene:
        violations.append(("adr-hygiene", hygiene))
    if index:
        violations.append(("adr-index", index))

    needs_adr = bool(breaking_abi or storage_changes)
    waived = needs_adr and args.allow_breaking and not adrs_in_change
    if needs_adr and not adrs_in_change and not args.allow_breaking:
        violations.append(
            (
                "adr-required",
                ["breaking ABI change: " + r for r in breaking_abi]
                + ["storage-layout change: " + r for r in storage_changes]
                + [
                    "no ADR was added or modified in this change set. Add one "
                    "under docs/adr/ (see docs/adr/ADR-TEMPLATE.md), or re-run "
                    "with --allow-breaking if the change is provably not breaking."
                ],
            )
        )

    report = {
        "adr_count": len(numbered),
        "adrs_in_change": adrs_in_change,
        "breaking_abi": breaking_abi,
        "storage_changes": storage_changes,
        "used_diff_fallback": fallback,
        "waived": waived,
        "violations": [name for name, _ in violations],
    }

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(
            f"[adr] {len(numbered)} ADR file(s); "
            f"{len(breaking_abi)} breaking ABI / {len(storage_changes)} storage "
            f"change(s) detected."
        )
        if adrs_in_change:
            print(
                "[adr] ADRs present in this change set: "
                + ", ".join(f"ADR-{n}" for n in adrs_in_change)
            )
        if fallback:
            print(
                "[adr] note: old/new file content was not available for every "
                "path; storage detection used the narrower diff heuristic."
            )
        if waived:
            print(
                "[adr] WARNING: --allow-breaking waived the ADR requirement for "
                "detected breaking changes: "
                + "; ".join(breaking_abi + storage_changes)
            )
        for name, problems in violations:
            print(f"[adr] {name} FAILED:", file=sys.stderr)
            for problem in problems:
                print(f"  - {problem}", file=sys.stderr)

    if violations:
        print(
            "\n  Process: docs/adr/ADR-PROCESS.md\n"
            "  Template: docs/adr/ADR-TEMPLATE.md",
            file=sys.stderr,
        )
        return 1

    if not args.json:
        print("[adr] all ADR checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
