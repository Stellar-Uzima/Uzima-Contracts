#!/usr/bin/env python3
"""Report open issues still awaiting triage past their priority response SLA."""

from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / ".github/issue-label-policy.json"


def main():
    try:
        policy = json.loads(POLICY.read_text())
        hours_by_priority = policy["priority_response_hours"]
        issues = json.load(sys.stdin)
    except (OSError, ValueError, KeyError) as error:
        print(f"Unable to read triage data: {error}", file=sys.stderr)
        return 2

    now = datetime.now(timezone.utc)
    overdue = []
    unprioritized = []
    for issue in issues:
        labels = {label["name"] for label in issue.get("labels", [])}
        if "state/needs-triage" not in labels:
            continue
        priorities = labels.intersection(hours_by_priority)
        if not priorities:
            unprioritized.append(issue)
            continue
        priority = min(priorities, key=lambda item: hours_by_priority[item])
        created = datetime.fromisoformat(issue["createdAt"].replace("Z", "+00:00"))
        due = created + timedelta(hours=hours_by_priority[priority])
        if now > due:
            overdue.append((issue, priority, due))

    for issue in unprioritized:
        print(f"Needs priority: #{issue['number']} {issue['url']}")
    for issue, priority, due in overdue:
        print(
            f"Overdue ({priority}, due {due.isoformat()}): "
            f"#{issue['number']} {issue['title']} {issue['url']}"
        )
    if overdue or unprioritized:
        print(f"Triage follow-up required: {len(overdue)} overdue, {len(unprioritized)} unprioritized.")
        return 1
    print("No overdue prioritized issues are awaiting triage.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
