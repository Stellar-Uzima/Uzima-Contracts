#!/usr/bin/env python3
"""Reject tracked paths that collide when compared case-insensitively.

`docs/RBAC.MD` and `docs/RBAC.md` were both tracked and byte-identical. On a
case-insensitive checkout (macOS, Windows) they are not two files but one, so a
plain `git rm docs/RBAC.MD` deletes the physical file and leaves the other
index entry dangling — the failure mode is silent, and the next person to
commit from such a checkout loses a file without being told (#1599).

Git preserves case in the index, so the collision is detectable even though it
is invisible on disk. This check reads the index rather than the filesystem for
exactly that reason, and fails with the colliding paths so the fix is obvious.

Scope is every tracked path, not just `docs/`, because the same trap applies to
any directory — and the cheapest moment to catch it is before it is committed.
"""

from collections import defaultdict
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]


def tracked_paths():
    """Return tracked paths with their case preserved, or None on failure."""
    try:
        result = subprocess.run(
            ["git", "ls-files", "-z"],
            cwd=ROOT,
            capture_output=True,
            check=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"Cannot read the git index: {error}", file=sys.stderr)
        return None
    # `-z` separates on NUL and disables git's own path quoting, so paths with
    # spaces or non-ASCII characters survive intact.
    return [entry.decode("utf-8", "surrogateescape") for entry in result.stdout.split(b"\0") if entry]


def main():
    paths = tracked_paths()
    if paths is None:
        return 2

    # Fold case with an explicit lower() rather than os.path.normcase(), which
    # is a no-op on POSIX and would miss the collision entirely.
    grouped = defaultdict(list)
    for path in paths:
        grouped[path.lower()].append(path)

    collisions = {key: group for key, group in grouped.items() if len(group) > 1}

    if collisions:
        print(
            f"{len(collisions)} tracked path(s) collide case-insensitively:",
            file=sys.stderr,
        )
        for group in sorted(collisions.values()):
            print(f"  {group[0]}", file=sys.stderr)
            for extra in group[1:]:
                print(f"  {extra}", file=sys.stderr)
        print("", file=sys.stderr)
        print(
            "These are one file on a case-insensitive filesystem. Keep the "
            "canonical",
            file=sys.stderr,
        )
        print(
            "spelling, delete the other with `git rm`, and re-check any links "
            "to it.",
            file=sys.stderr,
        )
        return 1

    print(f"No case-insensitive path collisions among {len(paths)} tracked files.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
