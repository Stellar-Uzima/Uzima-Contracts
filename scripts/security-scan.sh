#!/usr/bin/env bash
set -euo pipefail

REPORT_DIR="${1:-security-reports}"
mkdir -p "$REPORT_DIR"

AUDIT_JSON="$REPORT_DIR/cargo-audit.json"
GEIGER_OUT="$REPORT_DIR/cargo-geiger.txt"
CUSTOM_OUT="$REPORT_DIR/custom-security-lint.txt"
SUMMARY_MD="$REPORT_DIR/security-summary.md"

critical=0
medium=0

echo "Running cargo-audit..."
if ! cargo audit --json > "$AUDIT_JSON"; then
  # cargo-audit exits non-zero when advisories are found; severity-based gating is handled below.
  true
fi

audit_counts="$(python3 - <<'PY' "$AUDIT_JSON"
import json, sys
with open(sys.argv[1], "r", encoding="utf-8") as fh:
    data = json.load(fh)
vulns = data.get("vulnerabilities", {}).get("list", [])
critical = 0
medium = 0
for vuln in vulns:
    advisory = vuln.get("advisory", {})
    sev = str(advisory.get("severity", "")).lower()
    if sev in {"critical", "high"}:
        critical += 1
    elif sev in {"medium", "moderate"}:
        medium += 1
print(f"{critical} {medium} {len(vulns)}")
PY
)"

audit_critical="$(echo "$audit_counts" | awk '{print $1}')"
audit_medium="$(echo "$audit_counts" | awk '{print $2}')"
audit_total="$(echo "$audit_counts" | awk '{print $3}')"

critical=$((critical + audit_critical))
medium=$((medium + audit_medium))

echo "Running cargo-geiger..."
if cargo geiger --workspace --all-features --all-targets > "$GEIGER_OUT" 2>&1; then
  unsafe_total="$(grep -Eo "[0-9]+ unsafe" "$GEIGER_OUT" | awk '{sum+=$1} END {print sum+0}')"
else
  echo "cargo-geiger execution failed; treating as non-blocking informational warning." >> "$GEIGER_OUT"
  unsafe_total="n/a"
fi

echo "Running custom security lint rules..."
{
  echo "Potential hardcoded secrets and weak patterns"
  grep -RInE --exclude-dir=.git --exclude-dir=target --exclude-dir=security-reports \
    "(api[_-]?key|secret[_-]?key|private[_-]?key|BEGIN (RSA|EC|OPENSSH) PRIVATE KEY|password[[:space:]]*=[[:space:]]*['\\\"][^'\\\"]+['\\\"]|http://)" \
    . || true
} > "$CUSTOM_OUT"

custom_findings="$(grep -c ":" "$CUSTOM_OUT" || true)"
if [ "$custom_findings" -gt 0 ]; then
  medium=$((medium + custom_findings))
fi

{
  echo "## Security Scan Report"
  echo
  echo "- Cargo audit advisories: $audit_total"
  echo "- Critical/High advisories: $audit_critical"
  echo "- Medium advisories: $audit_medium"
  echo "- Unsafe usages reported by cargo-geiger: $unsafe_total"
  echo "- Custom lint findings: $custom_findings"
  echo
  if [ "$critical" -gt 0 ]; then
    echo "❌ Blocking: critical/high security findings detected."
  elif [ "$medium" -gt 0 ]; then
    echo "⚠️ Warning: medium security findings detected."
  else
    echo "✅ No blocking security findings detected."
  fi
} > "$SUMMARY_MD"

cat "$SUMMARY_MD"

if [ "$critical" -gt 0 ]; then
  exit 1
fi

exit 0
