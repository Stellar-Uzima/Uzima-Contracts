#!/usr/bin/env bash
# trace_extractor.sh - Shell wrapper for the Soroban trace extractor.
#
# Decodes Soroban transaction metadata XDR into canonical NDJSON conforming
# to schemas/trace/contract_trace.schema.json.
#
# Usage:
#   1. Artifact / File mode:
#      ./scripts/trace_extractor.sh --xdr <path/to/meta.xdr> [OPTIONS]
#      cat meta.xdr | ./scripts/trace_extractor.sh [OPTIONS]
#
#   2. Live Invocation mode:
#      ./scripts/trace_extractor.sh <contract_id> <network> <function> [args...]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Ensure trace_extractor binary is compiled
find_or_build_extractor() {
    local bin_path=""
    if [ -x "$REPO_ROOT/target/release/trace_extractor" ]; then
        bin_path="$REPO_ROOT/target/release/trace_extractor"
    elif [ -x "$REPO_ROOT/target/debug/trace_extractor" ]; then
        bin_path="$REPO_ROOT/target/debug/trace_extractor"
    else
        echo "Building trace_extractor binary..." >&2
        cargo build --quiet --package xdr_trace --bin trace_extractor --manifest-path "$REPO_ROOT/Cargo.toml"
        bin_path="$REPO_ROOT/target/debug/trace_extractor"
    fi
    printf '%s\n' "$bin_path"
}

EXTRACTOR_BIN="$(find_or_build_extractor)"

# Check invocation mode
if [ "$#" -eq 0 ]; then
    # Stdin mode
    exec "$EXTRACTOR_BIN"
fi

case "$1" in
    --xdr|--hex|--base64|--help|-h|--registry|--raw-xdr|--pretty|--no-validate-registry)
        # Direct CLI flag forwarding
        exec "$EXTRACTOR_BIN" "$@"
        ;;
    *.xdr|*.hex)
        # Positional XDR file
        exec "$EXTRACTOR_BIN" --xdr "$@"
        ;;
    *)
        # Live invocation mode: <contract_id> <network> <function> [args...]
        if [ "$#" -lt 3 ]; then
            echo "Usage:" >&2
            echo "  $0 --xdr <file> [OPTIONS]" >&2
            echo "  $0 <contract_id> <network> <function> [args...]" >&2
            exit 1
        fi

        CONTRACT_ID="$1"
        NETWORK="$2"
        FUNCTION="$3"
        shift 3

        # Sourced from capture_trace.sh
        # shellcheck source=scripts/capture_trace.sh disable=SC1091
        source "$SCRIPT_DIR/capture_trace.sh"

        capture_invoke_result "$NETWORK" "$CONTRACT_ID" "$FUNCTION" \
            soroban contract invoke \
            --id "$CONTRACT_ID" \
            --network "$NETWORK" \
            -- \
            "$FUNCTION" \
            "$@"

        # Find the produced .resultmeta.xdr artifact
        XDR_ARTIFACT="$TRACE_ARTIFACT_DIR/$TRACE_ARTIFACT_STEM.resultmeta.xdr"
        if [ -f "$XDR_ARTIFACT" ]; then
            exec "$EXTRACTOR_BIN" --xdr "$XDR_ARTIFACT" --contract-name "$FUNCTION"
        else
            echo "Error: No .resultmeta.xdr artifact captured for $CONTRACT_ID on $NETWORK" >&2
            exit 1
        fi
        ;;
esac
