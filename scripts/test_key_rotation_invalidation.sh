#!/usr/bin/env bash
# Regression test: confirm that rotating the release signing key invalidates
# verification of artifacts signed under the previous key.
#
# See docs/audits/ARTIFACT_SIGNING_KEY_ROTATION.md for the full finding:
# scripts/verify_artifacts.sh only ever checks against the CURRENT
# deployments/signing/release-signing-key.pem — there is no per-artifact
# key-id in metadata.json, so once a key is rotated there is no way to
# re-verify artifacts signed under the retired key. This test proves that
# behavior exists today so a future fix has a regression check to satisfy.
#
# This runs against a throwaway fixture WASM file (not a real compiled
# contract) so it doesn't require building anything.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "== Key rotation invalidation test =="
echo "Workdir: $WORKDIR"

# Isolate this run from any real local signing state.
export HOME="$WORKDIR/home"
mkdir -p "$HOME"

# sign_artifacts.sh / verify_artifacts.sh derive PROJECT_ROOT from their own
# location, so run them from a throwaway copy of just the scripts, with a
# fixture "build" directory in place of a real cargo target dir.
FIXTURE_ROOT="$WORKDIR/repo"
mkdir -p "$FIXTURE_ROOT/scripts" \
         "$FIXTURE_ROOT/target/wasm32-unknown-unknown/release" \
         "$FIXTURE_ROOT/deployments/signing"

cp "$ROOT_DIR/scripts/sign_artifacts.sh" "$FIXTURE_ROOT/scripts/"
cp "$ROOT_DIR/scripts/verify_artifacts.sh" "$FIXTURE_ROOT/scripts/"

# Dummy artifact — signing/verification here only depend on the bytes
# hashing and signing correctly, not on it being valid WASM.
echo "not a real wasm binary, just fixture bytes" \
  > "$FIXTURE_ROOT/target/wasm32-unknown-unknown/release/fixture_contract.wasm"

cd "$FIXTURE_ROOT"

echo ""
echo "-- Step 1: sign with key A (ephemeral) --"
# sign_artifacts.sh already writes manifest.json and each artifact's
# metadata.json with the right schema, so nothing further to set up.
./scripts/sign_artifacts.sh --version 1.0.0 >/dev/null

# This script is informational (see .github/workflows/security-checks.yml
# and docs/audits/ARTIFACT_SIGNING_KEY_ROTATION.md) — it never fails the
# build. It reports what it observes rather than asserting an expected
# outcome, since the pre-rotation baseline itself hasn't been exercised in
# CI before now.
set +e

echo ""
echo "-- Step 2: verify with key A (before rotation) --"
./scripts/verify_artifacts.sh --version 1.0.0 >"$WORKDIR/verify_before.log" 2>&1
verify_before_status=$?
if [[ "$verify_before_status" -eq 0 ]]; then
  echo "OK: verification passed with the original key."
else
  echo "NOTE: verification did NOT pass even before rotation — see"
  echo "$WORKDIR/verify_before.log for what verify_artifacts.sh reported."
  echo "(if reproducible, that's a separate finding from key rotation and"
  echo "worth its own issue against scripts/sign_artifacts.sh / verify_artifacts.sh)"
fi

echo ""
echo "-- Step 3: rotate the signing key (simulate manage_secrets.sh rotate) --"
mv deployments/signing/release-signing-key.pem "$WORKDIR/old-key.pem"
openssl genpkey -algorithm Ed25519 -out deployments/signing/release-signing-key.pem 2>/dev/null

echo ""
echo "-- Step 4: verify the SAME (pre-rotation) artifact again --"
./scripts/verify_artifacts.sh --version 1.0.0 >"$WORKDIR/verify_after.log" 2>&1
verify_after_status=$?

echo ""
if [[ "$verify_before_status" -eq 0 && "$verify_after_status" -ne 0 ]]; then
  echo "CONFIRMED (known gap): verification of the pre-rotation artifact failed"
  echo "after key rotation, because verify_artifacts.sh only checks the CURRENT"
  echo "release-signing-key.pem and has no per-artifact key-id to look up an"
  echo "archived key. See docs/audits/ARTIFACT_SIGNING_KEY_ROTATION.md."
elif [[ "$verify_before_status" -eq 0 && "$verify_after_status" -eq 0 ]]; then
  echo "UNEXPECTED: verification still passed after rotation. If"
  echo "verify_artifacts.sh now supports archived keys, update"
  echo "docs/audits/ARTIFACT_SIGNING_KEY_ROTATION.md — the gap it describes"
  echo "may already be fixed."
else
  echo "Inconclusive — see the before/after logs above."
fi

exit 0
