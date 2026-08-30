#!/bin/bash

# capture_trace.sh - Durable invocation-artifact capture for Soroban scripts.
#
# Soroban CLI's `contract invoke` streams its output to the terminal and, by
# default, keeps nothing machine-readable afterwards -- so invocation traces
# are unrecoverable post-hoc. This library (sourced, never executed) lets
# scripts:
#
#   1. run an invocation exactly as before (stdout/stderr still stream to the
#      terminal), while teeing the streams into per-invocation files, and
#   2. harvest the raw transaction result XDR the CLI persists verbatim in its
#      on-disk RPC-action log (the RPC `resultMetaXdr` field, a
#      `TransactionMeta` whose protocol-20+ `V3` arm nests the
#      `SorobanTransactionMeta` the decoder in `libs/xdr_trace` consumes).
#
# Artifacts land under `reports/traces/<network>/<contract_id>/` (git-ignored,
# see .gitignore) and are named with contract id + network + a unique sequence
# so successive invocations never overwrite each other. Optional CLI output
# flags (`--output xdr`/`--output json`) differ across CLI versions; they are
# only used when the installed CLI advertises them, so capture degrades
# gracefully instead of breaking every deploy.

# Root under which all invocation artifacts are persisted.
TRACE_ROOT="${TRACE_ROOT:-reports/traces}"

# The on-disk directory where the Soroban CLI caches every RPC action it runs
# (one ULID-named JSON blob per action). Mirrors the CLI's own ProjectDirs
# resolution for org.stellar.stellar-cli.
trace_cli_actions_dir() {
    if [ -n "${XDG_DATA_HOME:-}" ]; then
        printf '%s/stellar-cli/actions\n' "$XDG_DATA_HOME"
    elif [ -d "$HOME/Library/Application Support/stellar-cli/actions" ]; then
        printf '%s/Library/Application Support/stellar-cli/actions\n' "$HOME"
    elif [ -n "${LOCALAPPDATA:-}" ]; then
        printf '%s/stellar/stellar-cli/actions\n' "$LOCALAPPDATA"
    else
        printf '%s/.local/share/stellar-cli/actions\n' "$HOME"
    fi
}

# Creates (if needed) and echoes the artifact directory for one contract on one
# network: reports/traces/<network>/<contract_id>/.
trace_artifact_dir() {
    local network="$1" contract_id="$2" dir
    dir="$TRACE_ROOT/${network}/${contract_id}"
    mkdir -p "$dir"
    printf '%s\n' "$dir"
}

# A unique artifact stem for one invocation: contract id, network, label, and
# a time+pid+rng sequence that successive calls never collide on.
trace_stem() {
    local network="$1" contract_id="$2" label="$3"
    local ts nano seq
    ts="$(date -u +%Y%m%dT%H%M%SZ)"
    nano="$(date -u +%s%N 2>/dev/null || true)"
    case "$nano" in
        *[!0-9]* | '')
            seq="$(date -u +%s)$$$RANDOM"
            ;;
        *)
            seq="$nano$$"
            ;;
    esac
    label="$(printf '%s' "$label" | tr -cs '[:alnum:]' '_')"
    printf '%s-%s-%s-%s-%s\n' "$contract_id" "$network" "$label" "$ts" "$seq"
}

# Echoes an invocation `--output <mode>` argument when the installed CLI
# supports one (flag and possible values differ across CLI versions), and
# nothing otherwise. Callers must not depend on it: the durable capture below
# works without any output flag.
cli_invoke_output_arg() {
    local help
    help="$(soroban contract invoke --help 2>/dev/null || true)"
    if [ "$help" != "${help/--output/}" ]; then
        printf '%s\n' '--output xdr'
    fi
}

# Runs a command, streaming stdout and stderr to the terminal exactly like a
# plain run while also teeing each stream into its artifact file. Echoes
# nothing; returns the command's exit status unchanged.
trace_run_tee() {
    local out_file="$1" err_file="$2"
    shift 2
    : > "$out_file"
    : > "$err_file"
    "$@" > >(tee "$out_file") 2> >(tee "$err_file" >&2)
}

# Recursively finds the first JSON value for a key without depending on `jq`.
# Handles both the CLI's Send and Simulate action shapes.
trace_json_field() {
    local file="$1" key="$2" value
    if ! command -v python3 >/dev/null 2>&1; then
        return 1
    fi
    value="$(python3 - "$file" "$key" <<'PY' 2>/dev/null || true
import json
import sys


def find_first(obj, key):
    if isinstance(obj, dict):
        if key in obj:
            return obj[key]
        for value in obj.values():
            found = find_first(value, key)
            if found is not None:
                return found
    elif isinstance(obj, list):
        for value in obj:
            found = find_first(value, key)
            if found is not None:
                return found
    return None


with open(sys.argv[1], encoding="utf-8") as handle:
    root = json.load(handle)

result = find_first(root, sys.argv[2])
if result is not None:
    print(result)
PY
)"
    if [ -n "$value" ]; then
        printf '%s\n' "$value"
    fi
}

# Base64-decodes into a binary file, portable across GNU (`-d`) and BSD (`-D`)
# base64 with a python3 fallback.
b64_decode_to_file() {
    local b64="$1" out="$2"
    if command -v base64 >/dev/null 2>&1; then
        if base64 --version >/dev/null 2>&1; then
            printf '%s' "$b64" | base64 -d > "$out" 2>/dev/null
        else
            printf '%s' "$b64" | base64 -D > "$out" 2>/dev/null
        fi
    elif command -v python3 >/dev/null 2>&1; then
        printf '%s' "$b64" | python3 -c 'import base64, sys; sys.stdout.buffer.write(base64.b64decode(sys.stdin.read().strip()))' > "$out" 2>/dev/null
    else
        return 1
    fi
}

# Persists a harvested CLI RPC-action JSON and, when present, decodes its
# resultMetaXdr into a binary `<stem>.resultmeta.xdr` artifact.
trace_persist_action() {
    local action_file="$1" dir="$2" stem="$3"
    local json_file result_meta
    json_file="$dir/$stem.rpc-action.json"
    cp "$action_file" "$json_file" 2>/dev/null || return 0
    result_meta="$(trace_json_field "$json_file" resultMetaXdr || true)"
    if [ -n "$result_meta" ]; then
        b64_decode_to_file "$result_meta" "$dir/$stem.resultmeta.xdr" \
            || rm -f "$dir/$stem.resultmeta.xdr"
    fi
}

# Echoes (sorted) every artifact file persisted for one invocation stem.
trace_printed_artifacts() {
    local dir="$1" stem="$2"
    find "$dir" -maxdepth 1 -type f -name "$stem.*" 2>/dev/null | sort
}

# The single entry point scripts use.
#
#   capture_invoke_result <network> <contract_id> <label> <cmd...>
#
# Runs `<cmd...>` with live terminal output, persists the per-invocation
# artifacts under `reports/traces/`, and on success harvests the CLI RPC-action
# entry this invocation wrote. Sets for the caller:
#
#   TRACE_ARTIFACT_DIR    the artifact directory
#   TRACE_ARTIFACT_STEM   the unique artifact stem
#   TRACE_ARTIFACT_OUTPUT the captured stdout file
#   TRACE_ARTIFACT_ERROR  the captured stderr file
#   TRACE_ARTIFACTS       newline-separated persisted artifact paths
#
# Returns the invocation's exit status unchanged.
capture_invoke_result() {
    local network="$1" contract_id="$2" label="$3"
    shift 3
    local dir stem out_file err_file actions_dir actions_before actions_after
    local output_arg cmd
    local arg

    dir="$(trace_artifact_dir "$network" "$contract_id")"
    stem="$(trace_stem "$network" "$contract_id" "$label")"
    out_file="$dir/$stem.output.txt"
    err_file="$dir/$stem.stderr.log"

    TRACE_ARTIFACT_DIR="$dir"
    TRACE_ARTIFACT_STEM="$stem"
    TRACE_ARTIFACT_OUTPUT="$out_file"
    TRACE_ARTIFACT_ERROR="$err_file"

    # Remember the newest CLI action file before we run, so the artifact we
    # harvest afterwards is exactly the one this invocation wrote.
    actions_dir="$(trace_cli_actions_dir)"
    actions_before=""
    if [ -d "$actions_dir" ]; then
        actions_before="$(ls -1 "$actions_dir" 2>/dev/null | sort | tail -n 1 || true)"
    fi

    # Optionally add an `--output XDR` mode before the `--` separator when the
    # installed CLI advertises it; the persisted XDR then literally is the
    # invocation result XDR.
    output_arg="$(cli_invoke_output_arg)"
    cmd=()
    for arg in "$@"; do
        if [ "$arg" = "--" ] && [ -n "$output_arg" ]; then
            cmd+=("$output_arg")
        fi
        cmd+=("$arg")
    done

    local rc=0
    if trace_run_tee "$out_file" "$err_file" "${cmd[@]}"; then
        rc=0
    else
        rc=$?
    fi

    actions_after=""
    if [ -d "$actions_dir" ]; then
        actions_after="$(ls -1 "$actions_dir" 2>/dev/null | sort | tail -n 1 || true)"
    fi
    if [ -n "$actions_after" ] && [ "$actions_after" != "$actions_before" ]; then
        trace_persist_action "$actions_dir/$actions_after" "$dir" "$stem"
    fi

    TRACE_ARTIFACTS="$(trace_printed_artifacts "$dir" "$stem")"

    return "$rc"
}