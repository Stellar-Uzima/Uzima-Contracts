#!/usr/bin/env node
/**
 * Contract Resource Budget Runner
 *
 * Reads budgets from resource-budgets/budgets.json, measures actual WASM sizes
 * from the build output, and compares against budgets. Produces:
 *   - reports/budget_violations.json  — structured violations for CI
 *   - reports/budget_report.md        — human-readable PR comment
 *   - Exit code 1 if any budget is violated
 *
 * Usage:
 *   node scripts/measure_budgets.mjs                    # measure all contracts
 *   node scripts/measure_budgets.mjs medical_records    # measure one contract
 *   node scripts/measure_budgets.mjs --update-baselines # update baseline file
 *   node scripts/measure_budgets.mjs --update-instruction-budgets # record CPU baselines
 *
 * Instruction-count regressions (> 15% and > 50,000 instructions over the
 * committed baseline) fail the run and are reported in
 * reports/instruction_count_deltas.json (issue #1958).
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync, statSync } from 'fs';
import { join, basename } from 'path';
import { execSync } from 'child_process';

const ROOT = process.cwd();
const BUDGET_FILE = join(ROOT, 'resource-budgets', 'budgets.json');
const WASM_DIR = join(ROOT, 'target', 'wasm32-unknown-unknown', 'release');
const REPORT_DIR = join(ROOT, 'reports');
const VIOLATIONS_FILE = join(REPORT_DIR, 'budget_violations.json');
const REPORT_FILE = join(REPORT_DIR, 'budget_report.md');
const BASELINE_FILE = join(ROOT, 'resource-budgets', 'baselines.json');
const INSTRUCTION_FILE = join(ROOT, 'resource-budgets', 'instruction-budgets.json');
const DELTAS_FILE = join(REPORT_DIR, 'instruction_count_deltas.json');

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function loadJSON(path) {
  if (!existsSync(path)) {
    console.error(`[budget] File not found: ${path}`);
    process.exit(1);
  }
  return JSON.parse(readFileSync(path, 'utf8'));
}

function saveJSON(path, data) {
  writeFileSync(path, JSON.stringify(data, null, 2) + '\n');
}

function getWasmSize(contractName) {
  const wasmPath = join(WASM_DIR, `${contractName}.wasm`);
  if (!existsSync(wasmPath)) return null;
  return statSync(wasmPath).size;
}

/**
 * Collect per-function CPU instruction counts from a contract's benchmark
 * tests (issue #1958). Accepts both `[BENCH] name cpu=N insns` and the
 * storage harness' `[STORAGE-BENCH] name ... after=N` / `cpu_cost=N` forms.
 */
function collectInstructionCounts(contractName) {
  try {
    const output = execSync(
      `cargo test -p ${contractName} bench_ -- --nocapture 2>&1`,
      { encoding: 'utf8', timeout: 300000, cwd: ROOT }
    );
    const counts = {};
    const pattern = /\[(?:BENCH|STORAGE-BENCH)\]\s+(\S+).*?(?:cpu|after|cpu_cost)=(\d+)/g;
    let match;
    while ((match = pattern.exec(output)) !== null) {
      counts[match[1]] = parseInt(match[2], 10);
    }
    return counts;
  } catch (err) {
    console.error(`[budget] benchmark collection failed for ${contractName}: ${err.message}`);
    return {};
  }
}

/** Compare collected counts against the committed per-function baselines. */
function checkInstructionRegressions(results, filterContract) {
  const budgets = existsSync(INSTRUCTION_FILE) ? loadJSON(INSTRUCTION_FILE) : {};
  const functions = budgets.functions || {};
  const tolerance = budgets.regression_tolerance_pct ?? 15;
  const minDelta = budgets.min_delta_instructions ?? 50000;
  const deltas = {};
  let regressions = 0;

  for (const result of results) {
    if (filterContract && result.contract !== filterContract) continue;
    for (const [fn, actual] of Object.entries(result.instruction_counts || {})) {
      const key = `${result.contract}::${fn}`;
      const baseline = functions[key];
      const hasBaseline = typeof baseline === 'number' && baseline > 0;
      const deltaPct = hasBaseline ? ((actual - baseline) / baseline) * 100 : 0;
      deltas[key] = {
        actual,
        baseline: hasBaseline ? baseline : null,
        delta_pct: Number(deltaPct.toFixed(1)),
      };
      if (hasBaseline && deltaPct > tolerance && actual - baseline > minDelta) {
        regressions += 1;
        result.violations.push({
          metric: 'cpu_instruction_regression',
          function: fn,
          actual,
          baseline,
          over_pct: deltaPct.toFixed(1),
          severity: deltaPct > 30 ? 'critical' : 'warning',
        });
      }
    }
  }

  if (existsSync(REPORT_DIR)) {
    saveJSON(DELTAS_FILE, { checked_at: new Date().toISOString(), deltas });
  }
  return regressions;
}

// ---------------------------------------------------------------------------
// Budget check
// ---------------------------------------------------------------------------

function checkBudgets(budgets, filterContract) {
  const results = [];
  const defaults = budgets.defaults;

  for (const [name, budget] of Object.entries(budgets.contracts)) {
    if (filterContract && name !== filterContract) continue;

    const wasmSize = getWasmSize(name);
    const maxWasm = budget.max_wasm_bytes || defaults.max_wasm_bytes;
    const maxStorage = budget.max_storage_entries || defaults.max_storage_entries;
    const maxCpu = budget.max_cpu_instructions || defaults.max_cpu_instructions;
    const tolerance = budget.regression_tolerance_pct || defaults.regression_tolerance_pct;

    const violations = [];

    // WASM size check
    if (wasmSize !== null) {
      if (wasmSize > maxWasm) {
        violations.push({
          metric: 'wasm_bytes',
          actual: wasmSize,
          budget: maxWasm,
          over_pct: ((wasmSize - maxWasm) / maxWasm * 100).toFixed(1),
          severity: wasmSize > maxWasm * 0.95 ? 'critical' : 'warning'
        });
      }
    }

    // Baseline regression check
    if (existsSync(BASELINE_FILE)) {
      const baselines = loadJSON(BASELINE_FILE);
      const baseline = baselines[name];
      if (baseline && wasmSize !== null && baseline.wasm_bytes) {
        const regression = ((wasmSize - baseline.wasm_bytes) / baseline.wasm_bytes * 100);
        if (regression > tolerance) {
          violations.push({
            metric: 'wasm_regression',
            actual: wasmSize,
            baseline: baseline.wasm_bytes,
            over_pct: regression.toFixed(1),
            severity: regression > tolerance * 2 ? 'critical' : 'warning'
          });
        }
      }
    }

    results.push({
      contract: name,
      wasm_bytes: wasmSize,
      max_wasm_bytes: maxWasm,
      budget_notes: budget.notes || '',
      instruction_counts: collectInstructionCounts(name),
      violations
    });
  }

  return results;
}

// ---------------------------------------------------------------------------
// Report generation
// ---------------------------------------------------------------------------

function generateReport(results) {
  const hasViolations = results.some(r => r.violations.length > 0);
  const lines = [];

  lines.push('## Resource Budget Report');
  lines.push('');

  if (!hasViolations) {
    lines.push('> All contracts are within their resource budgets.');
    lines.push('');
  } else {
    lines.push('> **Budget violations detected** — see details below.');
    lines.push('');
  }

  // Summary table
  lines.push('| Contract | WASM Size | Budget | Status |');
  lines.push('|----------|-----------|--------|--------|');

  for (const r of results) {
    const measured = r.wasm_bytes !== null;
    const size = measured ? `${(r.wasm_bytes / 1024).toFixed(1)} KB` : 'N/A';
    const budget = `${(r.max_wasm_bytes / 1024).toFixed(0)} KB`;
    const violations = r.violations.filter(v => v.metric === 'wasm_bytes');
    // An unmeasured contract is not a passing contract; say so rather than
    // reporting "OK" for something that was never checked.
    const status = !measured ? 'NOT MEASURED'
      : violations.length === 0 ? 'OK'
      : violations[0].severity === 'critical' ? 'CRITICAL' : 'WARNING';
    const icon = !measured ? ':grey_question:'
      : status === 'OK' ? ':white_check_mark:'
      : status === 'CRITICAL' ? ':x:' : ':warning:';
    lines.push(`| ${r.contract} | ${size} | ${budget} | ${icon} ${status} |`);
  }

  lines.push('');

  // Violation details
  const violated = results.filter(r => r.violations.length > 0);
  if (violated.length > 0) {
    lines.push('### Violation Details');
    lines.push('');
    for (const r of violated) {
      lines.push(`#### ${r.contract}`);
      for (const v of r.violations) {
        const actual = v.metric === 'wasm_regression'
          ? `${v.actual} bytes (baseline: ${v.baseline})`
          : `${v.actual} bytes`;
        lines.push(`- **${v.metric}**: ${actual} — budget: ${v.budget} — over by ${v.over_pct}% (${v.severity})`);
      }
      lines.push('');
    }

    lines.push('### Remediation');
    lines.push('');
    lines.push('To fix budget violations:');
    lines.push('1. Profile the contract to identify hotspots');
    lines.push('2. Reduce code complexity or optimize storage patterns');
    lines.push('3. If the budget needs updating, edit `resource-budgets/budgets.json`');
    lines.push('4. If a baseline needs updating, run `node scripts/measure_budgets.mjs --update-baselines`');
    lines.push('');
  }

  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Baseline update
// ---------------------------------------------------------------------------

function updateBaselines() {
  const budgets = loadJSON(BUDGET_FILE);
  const baselines = existsSync(BASELINE_FILE) ? loadJSON(BASELINE_FILE) : {};

  for (const [name] of Object.entries(budgets.contracts)) {
    const wasmSize = getWasmSize(name);
    if (wasmSize !== null) {
      baselines[name] = {
        wasm_bytes: wasmSize,
        updated_at: new Date().toISOString()
      };
      console.log(`[budget] Baseline updated for ${name}: ${wasmSize} bytes`);
    }
  }

  saveJSON(BASELINE_FILE, baselines);
  console.log(`[budget] Baselines written to ${BASELINE_FILE}`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main() {
  const args = process.argv.slice(2);
  const updateBaselinesFlag = args.includes('--update-baselines');
  const updateInstructionsFlag = args.includes('--update-instruction-budgets');
  const filterContract = args.find(a => !a.startsWith('--')) || null;

  if (!existsSync(BUDGET_FILE)) {
    console.error(`[budget] Budget file not found: ${BUDGET_FILE}`);
    process.exit(1);
  }

  if (updateBaselinesFlag) {
    updateBaselines();
    return;
  }

  if (updateInstructionsFlag) {
    const budgets = loadJSON(BUDGET_FILE);
    const results = checkBudgets(budgets, filterContract);
    const functions = {};
    for (const result of results) {
      for (const [fn, count] of Object.entries(result.instruction_counts || {})) {
        functions[`${result.contract}::${fn}`] = count;
      }
    }
    saveJSON(INSTRUCTION_FILE, {
      version: '1.0.0',
      regression_tolerance_pct: 15,
      min_delta_instructions: 50000,
      functions,
    });
    console.log(`[budget] Updated ${Object.keys(functions).length} instruction baselines.`);
    return;
  }

  mkdirSync(REPORT_DIR, { recursive: true });

  const budgets = loadJSON(BUDGET_FILE);
  const results = checkBudgets(budgets, filterContract);
  const regressions = checkInstructionRegressions(results, filterContract);

  // A contract with no release artifact measures nothing, and an unmeasured
  // contract produces no violations. Without this guard a clean checkout
  // reports "All contracts within budget" having checked zero contracts, so
  // the gate would pass while measuring nothing.
  const measured = results.filter(r => r.wasm_bytes !== null);
  if (measured.length === 0) {
    console.error(`[budget] No contract artifacts found in ${WASM_DIR}`);
    console.error(`[budget] Checked ${results.length} contract(s) and measured 0.`);
    console.error("[budget] Build the contracts first: 'make build-opt'");
    process.exit(1);
  }
  if (measured.length < results.length) {
    console.error(
      `[budget] Warning: ${results.length - measured.length} contract(s) had no ` +
      `artifact in ${WASM_DIR} and were not measured.`
    );
  }

  // Write violations JSON
  const violations = results.filter(r => r.violations.length > 0);
  saveJSON(VIOLATIONS_FILE, {
    checked_at: new Date().toISOString(),
    total_contracts: results.length,
    contracts_measured: measured.length,
    contracts_with_violations: violations.length,
    results
  });

  // Write markdown report
  const report = generateReport(results);
  writeFileSync(REPORT_FILE, report);

  // Console output
  console.log(report);

  if (violations.length > 0 || regressions > 0) {
    console.error(
      `[budget] ${violations.length} contract(s) have budget violations; ` +
      `${regressions} instruction-count regression(s) detected.`
    );
    process.exit(1);
  } else {
    console.log('[budget] All contracts within budget.');
  }
}

main();
