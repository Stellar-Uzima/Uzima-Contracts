#!/usr/bin/env bash
# Diff-based CI gates (#1871, #1872, #1873). Usage: ci_gates.sh <stubs|topics|initguard> [base-ref]
# Only lines/functions ADDED relative to the base ref can fail a gate, so existing
# violations on main act as the baseline. Waive a single line with `ci-gate-allow`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
GATE="${1:?usage: ci_gates.sh <stubs|topics|initguard> [base-ref]}"
BASE="${2:-origin/main}"

# Prints "file:line: text" for every added line in Rust sources.
added() {
  git diff -U0 --diff-filter=AM "$BASE...HEAD" -- 'contracts/*.rs' 'libs/*.rs' |
    awk '/^\+\+\+ b\//{f=substr($0,7); next}
         /^@@/{match($0,/\+[0-9]+/); n=substr($0,RSTART+1,RLENGTH-1)+0; next}
         /^\+/{print f":"n": "substr($0,2); n++}'
}

fail=0
case "$GATE" in
  stubs)
    # TODO/FIXME/placeholder markers and todo!/unimplemented! introduced by the diff.
    hits="$(added | grep -E 'todo!\(|unimplemented!\(|TODO|FIXME|placeholder' | grep -v 'ci-gate-allow' || true)"
    [ -z "$hits" ] || { echo "New stub markers:"; echo "$hits"; fail=1; }
    # Empty #[test] fns whose #[test] attribute line was added by the diff.
    for f in $(added | cut -d: -f1 | sort -u); do
      [ -f "$f" ] || continue
      awk -v f="$f" '
        /#\[test\]/ {t=NR; next}
        t && /fn [a-zA-Z0-9_]+\(\)[^{]*\{[[:space:]]*\}/ {print f":"t": empty test"; t=0; next}
        t && /fn [a-zA-Z0-9_]+\(\)/ {fn=NR; t2=t; t=0; next}
        fn && /^[[:space:]]*\}[[:space:]]*$/ && NR==fn+1 {print f":"t2": empty test"; fn=0; next}
        fn && NR>fn {fn=0}' "$f"
    done | while IFS= read -r l; do
      loc="${l%%: *}"
      if added | grep -q "^${loc}:"; then echo "$l"; fi
    done > /tmp/empty_tests.$$ || true
    if [ -s /tmp/empty_tests.$$ ]; then echo "New empty tests:"; cat /tmp/empty_tests.$$; fail=1; fi
    rm -f /tmp/empty_tests.$$
    ;;
  topics)
    # Event topics in added publish lines must be snake_case.
    bad="$(added | grep -E 'publish\(' | grep -v 'ci-gate-allow' |
      grep -oE '^[^ ]+ .*(symbol_short!\("[^"]*"\)|Symbol::new\([^,]+, *"[^"]*"\))' |
      awk '{ s=$0; while (match(s, /"[^"]*"/)) { t=substr(s,RSTART+1,RLENGTH-2);
             if (t !~ /^[a-z][a-z0-9_]*$/) print $1 " " t; s=substr(s,RSTART+RLENGTH) } }' || true)"
    [ -z "$bad" ] || { echo "Non-snake_case event topics (add ci-gate-allow to waive):"; echo "$bad"; fail=1; }
    ;;
  initguard)
    changed="$(added | cut -d: -f1 | sort -u)"
    echo "initialize functions: file:line status"
    for f in $(grep -rlE 'fn initialize\b' contracts libs --include=*.rs 2>/dev/null | sort); do
      awk -v f="$f" '
        {l[NR]=$0}
        END{for(i=1;i<=NR;i++) if (l[i] ~ /fn initialize[^_a-zA-Z0-9]/) {
          s="UNANNOTATED"
          for(j=i-2;j<=i+25&&j<=NR;j++) if (j>0) {
            if (l[j] ~ /try_init_guard/) s="guarded"
            else if (l[j] ~ /init-guard-waiver/ && s!="guarded") s="waived"
          }
          print f":"i" "s}}' "$f"
    done | tee /tmp/init_audit.$$
    while read -r loc status; do
      f="${loc%%:*}"
      if [ "$status" = UNANNOTATED ] && echo "$changed" | grep -qx "$f"; then
        echo "::error::$loc has no try_init_guard or 'init-guard-waiver' comment"; fail=1
      fi
    done < /tmp/init_audit.$$
    rm -f /tmp/init_audit.$$
    ;;
  *) echo "unknown gate $GATE"; exit 2 ;;
esac
exit "$fail"
