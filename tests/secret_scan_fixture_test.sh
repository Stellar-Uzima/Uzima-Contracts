#!/usr/bin/env bash
# Regression test for issue #1965.
# 1. The canary fixture must be detected as a secret by TruffleHog when the CLI
#    is available (proves the scanner is actually wired up).
# 2. scripts/deploy_forensics.ts must require FORENSICS_SECRET and contain no
#    embedded StrKey secret literal.
set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE="$ROOT_DIR/tests/fixtures/secret_scan/fake_stellar_secret.txt"
TARGET="$ROOT_DIR/scripts/deploy_forensics.ts"
PASS=0
FAIL=0

ok() { echo "  ok   - $1"; PASS=$((PASS + 1)); }
nope() { echo "  FAIL - $1"; FAIL=$((FAIL + 1)); }

echo "secret-scan fixture checks"

[[ -f "$FIXTURE" ]] && ok "canary fixture exists" || nope "canary fixture exists"
if grep -qF 'tests/fixtures/secret_scan/fake_stellar_secret' "$ROOT_DIR/.trufflehog-excludes.txt"; then
  ok "canary fixture is excluded from the repo-wide scan"
else
  nope "canary fixture is excluded from the repo-wide scan"
fi
if grep -qF 'process.env.FORENSICS_SECRET' "$TARGET"; then
  ok "FORENSICS_SECRET is required from env"
else
  nope "FORENSICS_SECRET is required from env"
fi
if grep -qE "const secret = ['\"]S[A-Z2-7]{55}['\"]" "$TARGET"; then
  nope "no embedded StrKey literal in deploy_forensics.ts"
else
  ok "no embedded StrKey literal in deploy_forensics.ts"
fi

if command -v trufflehog >/dev/null 2>&1; then
  if trufflehog filesystem --no-verification --json "$FIXTURE" 2>/dev/null \
       | grep -qiE 'DetectorName|detector_name|stellar'; then
    ok "TruffleHog detects the canary fixture"
  else
    nope "TruffleHog detects the canary fixture"
  fi
else
  echo "  skip - trufflehog CLI not installed"
fi

echo
echo "passed: $PASS, failed: $FAIL"
[[ "$FAIL" -eq 0 ]]
