#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ALLOWLIST_FILE="$ROOT_DIR/scripts/allowlists/production_unwraps.txt"

matches="$(rg -n --no-heading 'unwrap\(' "$ROOT_DIR/contracts" \
    --glob '!**/test.rs' \
    --glob '!**/tests/**' \
    --glob '!**/*test*.rs' || true)"

if [[ -z "$matches" ]]; then
    echo "OK: no production unwrap() calls found."
    exit 0
fi

violations=0
while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if [[ -f "$ALLOWLIST_FILE" ]]; then
        while IFS= read -r allow; do
            [[ -z "$allow" ]] && continue
            if [[ "$line" == *"$allow"* ]]; then
                continue 2
            fi
        done < "$ALLOWLIST_FILE"
    fi
    echo "FAIL: unallowlisted production unwrap: $line"
    violations=$((violations + 1))
done <<< "$matches"

if (( violations > 0 )); then
    echo
    echo "FAIL: $violations production unwrap() occurrence(s) need migration or allowlisting."
    exit 1
fi

echo "OK: only allowlisted production unwrap() calls remain."
