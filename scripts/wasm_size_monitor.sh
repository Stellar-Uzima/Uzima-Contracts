#!/usr/bin/env bash

# WASM Size Monitoring Script for Stellar Contracts
# Checks built .wasm artifacts against committed baselines and Stellar size limits.

set -euo pipefail

DIST_DIR="dist"
BASELINE_FILE="scripts/wasm_size_baselines.json"
REPORT_FILE="reports/wasm_size_report.md"
UPDATE_BASELINE=0
EXPORT_TRENDS=0
EXCLUDED_CONTRACTS=("load_testing")

show_help() {
    cat <<'EOF'
Usage: scripts/wasm_size_monitor.sh [OPTIONS]

Options:
  --dist DIR          Directory containing built .wasm files (default: dist)
  --baseline FILE    Baseline JSON file (default: scripts/wasm_size_baselines.json)
  --report FILE      Markdown report path (default: reports/wasm_size_report.md)
  --update           Regenerate the baseline JSON from the current dist directory
  --export-trends    Backward-compatible alias: print the JSON summary path
  --help, -h         Show this help message

The gate fails when a non-excluded contract exceeds its baseline by more than
10% or crosses the 60 KiB warning threshold. The load_testing contract is
excluded because it is a known non-deployable test contract.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dist)
            DIST_DIR="${2:?missing value for --dist}"
            shift 2
            ;;
        --baseline)
            BASELINE_FILE="${2:?missing value for --baseline}"
            shift 2
            ;;
        --report)
            REPORT_FILE="${2:?missing value for --report}"
            shift 2
            ;;
        --update)
            UPDATE_BASELINE=1
            shift
            ;;
        --export-trends)
            EXPORT_TRENDS=1
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            show_help >&2
            exit 2
            ;;
    esac
done

if ! command -v python3 >/dev/null 2>&1; then
    echo "Error: python3 is required for WASM size monitoring" >&2
    exit 1
fi

mkdir -p "$(dirname "$REPORT_FILE")" "$(dirname "$BASELINE_FILE")"

EXCLUDED_JOINED=$(IFS=,; echo "${EXCLUDED_CONTRACTS[*]}")
export DIST_DIR BASELINE_FILE REPORT_FILE UPDATE_BASELINE EXPORT_TRENDS EXCLUDED_JOINED

python3 <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

DIST_DIR = Path(os.environ["DIST_DIR"])
BASELINE_FILE = Path(os.environ["BASELINE_FILE"])
REPORT_FILE = Path(os.environ["REPORT_FILE"])
UPDATE_BASELINE = os.environ["UPDATE_BASELINE"] == "1"
EXPORT_TRENDS = os.environ["EXPORT_TRENDS"] == "1"
EXCLUDED = {name for name in os.environ["EXCLUDED_JOINED"].split(",") if name}

MAX_BYTES = 65_536
WARNING_BYTES = 60 * 1024
REGRESSION_PERCENT = 10.0
SCHEMA_VERSION = 1


def human_size(size: int) -> str:
    prefix = "-" if size < 0 else ""
    value = abs(size)
    if value >= 1024:
        return f"{prefix}{value / 1024:.1f} KiB"
    return f"{prefix}{value} B"


def load_baseline() -> dict:
    if not BASELINE_FILE.exists():
        return {
            "schemaVersion": SCHEMA_VERSION,
            "limits": {
                "maxBytes": MAX_BYTES,
                "warningBytes": WARNING_BYTES,
                "regressionPercent": REGRESSION_PERCENT,
            },
            "excludedContracts": sorted(EXCLUDED),
            "contracts": {},
        }
    with BASELINE_FILE.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    data.setdefault("contracts", {})
    data.setdefault("excludedContracts", sorted(EXCLUDED))
    data.setdefault("limits", {})
    return data


def discover_wasm_files() -> list[Path]:
    if not DIST_DIR.exists():
        return []
    return sorted(path for path in DIST_DIR.glob("*.wasm") if path.is_file())


def write_baseline(sizes: dict[str, int]) -> None:
    payload = {
        "schemaVersion": SCHEMA_VERSION,
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "generatedBy": "scripts/wasm_size_monitor.sh --update",
        "limits": {
            "maxBytes": MAX_BYTES,
            "warningBytes": WARNING_BYTES,
            "regressionPercent": REGRESSION_PERCENT,
        },
        "excludedContracts": sorted(EXCLUDED),
        "contracts": dict(sorted(sizes.items())),
    }
    BASELINE_FILE.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def format_delta(current: int, baseline: int | None) -> str:
    if baseline is None:
        return "new"
    delta = current - baseline
    if baseline == 0:
        pct = 100.0 if current else 0.0
    else:
        pct = (delta / baseline) * 100
    sign = "+" if delta >= 0 else ""
    return f"{sign}{human_size(delta)} ({sign}{pct:.1f}%)"


def evaluate_contract(name: str, size: int, baseline: int | None) -> dict:
    excluded = name in EXCLUDED
    over_warning = size > WARNING_BYTES
    over_max = size > MAX_BYTES
    regression = False
    regression_pct = 0.0
    if baseline is not None and baseline > 0:
        regression_pct = ((size - baseline) / baseline) * 100
        regression = regression_pct > REGRESSION_PERCENT
    elif baseline == 0:
        regression_pct = 100.0 if size else 0.0
        regression = size > 0

    failures = []
    if not excluded and over_warning:
        failures.append("over 60 KiB warning threshold")
    if not excluded and regression:
        failures.append(f">{REGRESSION_PERCENT:.0f}% baseline regression")

    if excluded:
        status = "EXCLUDED"
    elif failures:
        status = "FAIL"
    elif baseline is None:
        status = "NO_BASELINE"
    else:
        status = "OK"

    return {
        "name": name,
        "size": size,
        "baseline": baseline,
        "delta": format_delta(size, baseline),
        "status": status,
        "excluded": excluded,
        "over_warning": over_warning,
        "over_max": over_max,
        "regression_pct": regression_pct,
        "failures": failures,
    }


def render_report(results: list[dict], baseline_path: Path, dist_path: Path) -> str:
    failing = [item for item in results if item["failures"]]
    missing_baseline = [item for item in results if item["status"] == "NO_BASELINE"]
    lines = [
        "# WASM Size Report",
        "",
        f"WASM_SIZE_STATUS={'failed' if failing else 'passed'}",
        f"Baseline: `{baseline_path.as_posix()}`",
        f"Artifact directory: `{dist_path.as_posix()}`",
        f"Regression gate: > {REGRESSION_PERCENT:.0f}% over baseline",
        f"Warning gate: > {human_size(WARNING_BYTES)}",
        f"Excluded contracts: {', '.join(sorted(EXCLUDED)) or 'none'}",
        "",
        "| Contract | Size | Baseline | Delta vs baseline | Status |",
        "| --- | ---: | ---: | ---: | --- |",
    ]
    if results:
        for item in results:
            baseline = "new" if item["baseline"] is None else human_size(item["baseline"])
            lines.append(
                f"| `{item['name']}` | {human_size(item['size'])} | {baseline} | {item['delta']} | {item['status']} |"
            )
    else:
        lines.append("| _none_ | - | - | - | NO_WASM |")

    lines.append("")
    if failing:
        lines.append("## Regression Failures")
        lines.append("")
        for item in failing:
            lines.append(f"- `{item['name']}`: {', '.join(item['failures'])}; size {human_size(item['size'])}, delta {item['delta']}.")
        lines.append("")
    if missing_baseline:
        lines.append("## Missing Baselines")
        lines.append("")
        lines.append("These contracts were checked against the 60 KiB warning gate but have no committed baseline yet.")
        for item in missing_baseline:
            lines.append(f"- `{item['name']}`: {human_size(item['size'])}")
        lines.append("")
    lines.append("Regenerate baselines locally with:")
    lines.append("")
    lines.append("```bash")
    lines.append("./scripts/wasm_size_monitor.sh --update")
    lines.append("```")
    lines.append("")
    return "\n".join(lines)


baseline = load_baseline()
limits = baseline.get("limits", {})
MAX_BYTES = int(limits.get("maxBytes", MAX_BYTES))
WARNING_BYTES = int(limits.get("warningBytes", WARNING_BYTES))
REGRESSION_PERCENT = float(limits.get("regressionPercent", REGRESSION_PERCENT))
EXCLUDED.update(str(name) for name in baseline.get("excludedContracts", []) if name)
wasm_files = discover_wasm_files()
sizes = {path.stem: path.stat().st_size for path in wasm_files if path.stem not in EXCLUDED}

if UPDATE_BASELINE:
    if not sizes:
        print(f"No non-excluded WASM files found in {DIST_DIR}; refusing to write an empty baseline.", file=sys.stderr)
        sys.exit(1)
    write_baseline(sizes)
    print(f"Updated {BASELINE_FILE} with {len(sizes)} contract baseline(s).")
    sys.exit(0)

results = []
contracts = baseline.get("contracts", {})
for path in wasm_files:
    name = path.stem
    baseline_size = contracts.get(name)
    results.append(evaluate_contract(name, path.stat().st_size, baseline_size))

report = render_report(results, BASELINE_FILE, DIST_DIR)
REPORT_FILE.write_text(report, encoding="utf-8")
print(report)

if EXPORT_TRENDS:
    print(f"\nJSON baseline: {BASELINE_FILE}")

if not wasm_files:
    print(f"No WASM files found in {DIST_DIR}; run the wasm build and collect artifacts first.", file=sys.stderr)
    sys.exit(1)

if any(item["failures"] for item in results):
    sys.exit(1)
PY
