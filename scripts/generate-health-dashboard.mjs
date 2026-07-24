#!/usr/bin/env node
/**
 * generate-health-dashboard.mjs
 *
 * Generates a contract health dashboard with build, test, and performance metrics.
 *
 * Reads:
 *   - WASM artifacts from target/wasm32-unknown-unknown/release/
 *   - Resource budgets from resource-budgets/budgets.json
 *   - Test results (if available)
 *
 * Outputs:
 *   - docs/portal/health-dashboard.html — interactive dashboard
 *   - schemas/docs/health_metrics.json — machine-readable metrics
 *
 * Usage:
 *   node scripts/generate-health-dashboard.mjs
 */

import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.resolve(__dirname, '..');

const WASM_DIR = path.join(PROJECT_ROOT, 'target/wasm32-unknown-unknown/release');
const BUDGETS_PATH = path.join(PROJECT_ROOT, 'resource-budgets/budgets.json');
const OUTPUT_HTML = path.join(PROJECT_ROOT, 'docs/portal/health-dashboard.html');
const OUTPUT_JSON = path.join(PROJECT_ROOT, 'schemas/docs/health_metrics.json');

// ── Metrics collection ──────────────────────────────────────────────────────

function collectMetrics() {
  const contracts = [];
  const contractsDir = path.join(PROJECT_ROOT, 'contracts');

  // Load budgets if available
  let budgets = {};
  if (fs.existsSync(BUDGETS_PATH)) {
    try {
      budgets = JSON.parse(fs.readFileSync(BUDGETS_PATH, 'utf8'));
    } catch { /* ignore */ }
  }

  const entries = fs.readdirSync(contractsDir).filter((name) => {
    const dir = path.join(contractsDir, name);
    return fs.statSync(dir).isDirectory() && fs.existsSync(path.join(dir, 'src/lib.rs'));
  });

  for (const name of entries.sort()) {
    const wasmPath = path.join(WASM_DIR, `${name}.wasm`);
    const contractDir = path.join(contractsDir, name);
    const libPath = path.join(contractDir, 'src/lib.rs');
    const testPath = path.join(contractDir, 'src/test.rs');
    const errorsPath = path.join(contractDir, 'src/errors.rs');

    const metric = {
      name,
      has_wasm: fs.existsSync(wasmPath),
      wasm_size_bytes: 0,
      wasm_size_kb: 0,
      has_tests: fs.existsSync(testPath),
      test_count: 0,
      has_errors: fs.existsSync(errorsPath),
      has_doc_comments: false,
      function_count: 0,
      source_lines: 0,
      budget: budgets[name] || null,
      status: 'unknown',
    };

    // WASM size
    if (metric.has_wasm) {
      const stat = fs.statSync(wasmPath);
      metric.wasm_size_bytes = stat.size;
      metric.wasm_size_kb = Math.round((stat.size / 1024) * 10) / 10;
    }

    // Source metrics
    if (fs.existsSync(libPath)) {
      const src = fs.readFileSync(libPath, 'utf8');
      metric.source_lines = src.split('\n').length;
      metric.function_count = (src.match(/pub fn /g) || []).length;
      metric.has_doc_comments = src.startsWith('//!');
    }

    // Test count
    if (metric.has_tests) {
      const testSrc = fs.readFileSync(testPath, 'utf8');
      metric.test_count = (testSrc.match(/#\[test\]/g) || []).length;
    }

    // Status determination
    if (!metric.has_wasm) {
      metric.status = 'excluded';
    } else if (metric.wasm_size_kb > 60) {
      metric.status = 'critical';
    } else if (metric.wasm_size_kb > 51.2) {
      metric.status = 'warning';
    } else if (metric.test_count === 0) {
      metric.status = 'warning';
    } else {
      metric.status = 'healthy';
    }

    contracts.push(metric);
  }

  return contracts;
}

// ── HTML generation ─────────────────────────────────────────────────────────

function generateDashboard(contracts) {
  const timestamp = new Date().toISOString();
  const total = contracts.length;
  const healthy = contracts.filter((c) => c.status === 'healthy').length;
  const warning = contracts.filter((c) => c.status === 'warning').length;
  const critical = contracts.filter((c) => c.status === 'critical').length;
  const excluded = contracts.filter((c) => c.status === 'excluded').length;
  const totalWasm = contracts.reduce((s, c) => s + c.wasm_size_bytes, 0);
  const totalTests = contracts.reduce((s, c) => s + c.test_count, 0);
  const totalFns = contracts.reduce((s, c) => s + c.function_count, 0);

  const statusColor = (s) => {
    switch (s) {
      case 'healthy': return '#3fb950';
      case 'warning': return '#d29922';
      case 'critical': return '#f85149';
      case 'excluded': return '#8b949e';
      default: return '#8b949e';
    }
  };

  const rows = contracts.map((c) => `
    <tr>
      <td><code>${c.name}</code></td>
      <td><span style="color:${statusColor(c.status)};font-weight:bold">${c.status}</span></td>
      <td>${c.has_wasm ? c.wasm_size_kb + ' KB' : '—'}</td>
      <td>${c.test_count}</td>
      <td>${c.function_count}</td>
      <td>${c.source_lines}</td>
      <td>${c.has_doc_comments ? 'Yes' : 'No'}</td>
      <td>${c.has_errors ? 'Yes' : 'No'}</td>
    </tr>`).join('');

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Uzima Contracts — Health Dashboard</title>
  <style>
    :root {
      --bg: #0d1117; --bg2: #161b22; --bg3: #21262d; --border: #30363d;
      --text: #e6edf3; --text2: #8b949e; --accent: #58a6ff;
    }
    body { font-family: -apple-system, sans-serif; background: var(--bg); color: var(--text); margin: 0; padding: 24px; }
    h1 { color: var(--accent); }
    .summary { display: flex; gap: 16px; margin-bottom: 24px; flex-wrap: wrap; }
    .card { background: var(--bg2); border: 1px solid var(--border); border-radius: 8px; padding: 16px 24px; min-width: 120px; }
    .card .value { font-size: 28px; font-weight: bold; }
    .card .label { font-size: 12px; color: var(--text2); margin-top: 4px; }
    table { width: 100%; border-collapse: collapse; background: var(--bg2); border: 1px solid var(--border); border-radius: 8px; }
    th, td { text-align: left; padding: 10px 12px; border-bottom: 1px solid var(--border); font-size: 13px; }
    th { background: var(--bg3); font-weight: 600; }
    code { background: var(--bg3); padding: 2px 6px; border-radius: 4px; }
    .timestamp { color: var(--text2); font-size: 12px; margin-bottom: 24px; }
  </style>
</head>
<body>
  <h1>Contract Health Dashboard</h1>
  <p class="timestamp">Generated: ${timestamp}</p>

  <div class="summary">
    <div class="card"><div class="value">${total}</div><div class="label">Total Contracts</div></div>
    <div class="card"><div class="value" style="color:#3fb950">${healthy}</div><div class="label">Healthy</div></div>
    <div class="card"><div class="value" style="color:#d29922">${warning}</div><div class="label">Warning</div></div>
    <div class="card"><div class="value" style="color:#f85149">${critical}</div><div class="label">Critical</div></div>
    <div class="card"><div class="value">${totalTests}</div><div class="label">Total Tests</div></div>
    <div class="card"><div class="value">${totalFns}</div><div class="label">Public Functions</div></div>
    <div class="card"><div class="value">${(totalWasm / 1024).toFixed(1)} KB</div><div class="label">Total WASM</div></div>
  </div>

  <table>
    <thead>
      <tr>
        <th>Contract</th>
        <th>Status</th>
        <th>WASM Size</th>
        <th>Tests</th>
        <th>Functions</th>
        <th>Source Lines</th>
        <th>Doc Comments</th>
        <th>Error Types</th>
      </tr>
    </thead>
    <tbody>
      ${rows}
    </tbody>
  </table>
</body>
</html>`;
}

// ── Main ─────────────────────────────────────────────────────────────────────

function main() {
  console.log('Collecting contract health metrics...');
  const contracts = collectMetrics();

  console.log(`Found ${contracts.length} contracts.`);

  // Write JSON
  fs.mkdirSync(path.dirname(OUTPUT_JSON), { recursive: true });
  const jsonData = {
    generated_at: new Date().toISOString(),
    total_contracts: contracts.length,
    healthy: contracts.filter((c) => c.status === 'healthy').length,
    warning: contracts.filter((c) => c.status === 'warning').length,
    critical: contracts.filter((c) => c.status === 'critical').length,
    contracts,
  };
  fs.writeFileSync(OUTPUT_JSON, JSON.stringify(jsonData, null, 2));
  console.log(`Saved JSON to ${OUTPUT_JSON}`);

  // Write HTML
  fs.mkdirSync(path.dirname(OUTPUT_HTML), { recursive: true });
  fs.writeFileSync(OUTPUT_HTML, generateDashboard(contracts));
  console.log(`Saved dashboard to ${OUTPUT_HTML}`);

  // Summary
  const healthy = jsonData.healthy;
  const warning = jsonData.warning;
  const critical = jsonData.critical;
  console.log(`\nHealth: ${healthy} healthy, ${warning} warning, ${critical} critical`);
}

main();
