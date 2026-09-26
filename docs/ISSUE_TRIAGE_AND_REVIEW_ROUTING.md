# Issue Triage and Review Routing

This guide defines a compact label vocabulary, response targets, and ownership coverage for incoming work. The machine-readable label policy is [.github/issue-label-policy.json](../.github/issue-label-policy.json).

## Issue labels

Use labels from these five groups; an issue can carry one label from each group:

| Group | Canonical values | Purpose |
|---|---|---|
| `kind/*` | `bug`, `feature`, `docs`, `question`, `maintenance` | What kind of work is requested |
| `priority/*` | `p0`, `p1`, `p2`, `p3` | Urgency and response target |
| `area/*` | `contracts`, `deployment`, `ci`, `sdk`, `docs`, `security`, `governance` | Responsible part of the project |
| `state/*` | `needs-triage`, `accepted`, `blocked` | Current triage state |

Apply `state/needs-triage` to each new issue. During triage, add one kind, one area, and one priority; remove `state/needs-triage` once an owner has assessed it. Older labels remain searchable during migration, but new labels must use the canonical names. The issue-label workflow checks every newly applied label. The taxonomy deliberately does not duplicate GitHub's assignee, milestone, or project fields.

## Response targets

The response clock starts when the issue is opened. It measures an initial maintainer assessment, not completion of the work.

| Priority | Examples | First assessment |
|---|---|---|
| `priority/p0` | Active exploit, critical data loss, or safety issue | 24 hours |
| `priority/p1` | Security or correctness regression with major impact | 3 calendar days |
| `priority/p2` | Normal bug, enhancement, or documentation issue | 7 calendar days |
| `priority/p3` | Low impact cleanup or future work | Next weekly backlog review |

The daily [triage SLA workflow](../.github/workflows/triage-sla.yml) reports issues still marked `state/needs-triage` after their P0–P2 target and flags issues without a priority. P3 is reviewed at the weekly backlog review. To acknowledge an issue, assign an owner, record the next step, and remove `state/needs-triage`; keep the priority and area labels for search and planning.

## Ownership and routing

Every first-level directory under `contracts/` must match an owner in [.github/CODEOWNERS](../.github/CODEOWNERS) and a review tier in [.github/review-routing.json](../.github/review-routing.json). Specific paths describe high-risk review requirements; `contracts/*/` is a one-approval domain default for newly added contract directories. More-specific entries carry the intended risk level. Keep the fallback owner in place for files outside explicit paths.

The [governance policy workflow](../.github/workflows/governance-policy.yml) runs `scripts/check_codeowners_coverage.py` when contract directories or routing policy change. To add a contract, add its path to CODEOWNERS and assign the right review tier in the JSON. For example, a new payment contract should receive an explicit payment owner and the critical tier; a low-risk analytics module may use the domain default. The check fails if a contract directory is not covered by both policies.

## Maintainer examples

1. A newly reported consent bypass gets `kind/bug`, `area/security`, `priority/p0`, and `state/needs-triage`. The on-call maintainer assesses it within 24 hours, assigns an owner, and removes the waiting label.
2. A request to improve generated SDK documentation gets `kind/docs`, `area/sdk`, `priority/p2`, and `state/needs-triage`. If it is accepted, mark it `state/accepted` and assign it to a milestone or project.
3. A new contract folder needs an owner entry and a routing tier before the coverage workflow passes. Use a more-specific path for security-sensitive modules; otherwise, the domain default applies.
