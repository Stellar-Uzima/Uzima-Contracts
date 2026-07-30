#!/usr/bin/env node
/**
 * generate_budget_dashboard.mjs - Generate per-contract resource budget dashboard HTML
 *
 * Reads budgets from resource-budgets/budgets.json and baselines from baselines.json,
 * generates an HTML dashboard with per-contract budget visualization.
 *
 * Usage:
 *   node scripts/generate_budget_dashboard.mjs
 *   node scripts/generate_budget_dashboard.mjs --output dashboard/resource-budgets.html
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';

const ROOT = process.cwd();
const BUDGET_FILE = join(ROOT, 'resource-budgets', 'budgets.json');
const BASELINE_FILE = join(ROOT, 'resource-budgets', 'baselines.json');
const DEFAULT_OUTPUT = join(ROOT, 'dashboard', 'resource-budgets.html');

function loadJSON(path) {
  if (!existsSync(path)) return null;
  return JSON.parse(readFileSync(path, 'utf8'));
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function formatNumber(num) {
  if (num >= 1e9) return (num / 1e9).toFixed(1) + 'B';
  if (num >= 1e6) return (num / 1e6).toFixed(1) + 'M';
  if (num >= 1e3) return (num / 1e3).toFixed(1) + 'K';
  return num.toString();
}

function getUsagePercent(actual, max) {
  if (!actual || !max) return 0;
  return Math.min(100, Math.round((actual / max) * 100));
}

function getStatusClass(percent) {
  if (percent >= 95) return 'critical';
  if (percent >= 80) return 'warning';
  return 'healthy';
}

function generateContractRows(contracts, baselines) {
  const defaults = contracts.defaults || {};
  let rows = '';

  for (const [name, budget] of Object.entries(contracts.contracts || {})) {
    const maxWasm = budget.max_wasm_bytes || defaults.max_wasm_bytes || 65536;
    const maxStorage = budget.max_storage_entries || defaults.max_storage_entries || 500;
    const maxCpu = budget.max_cpu_instructions || defaults.max_cpu_instructions || 10000000;
    const tolerance = budget.regression_tolerance_pct || defaults.regression_tolerance_pct || 10;

    const baseline = baselines?.[name];
    const actualWasm = baseline?.wasm_bytes || 0;
    const wasmPercent = getUsagePercent(actualWasm, maxWasm);
    const statusClass = getStatusClass(wasmPercent);

    rows += `
      <tr class="contract-row">
        <td class="contract-name">${name}</td>
        <td>${formatBytes(actualWasm)}</td>
        <td>${formatBytes(maxWasm)}</td>
        <td>
          <div class="progress-bar">
            <div class="progress-fill ${statusClass}" style="width: ${wasmPercent}%"></div>
          </div>
          <span class="progress-label ${statusClass}">${wasmPercent}%</span>
        </td>
        <td>${formatNumber(maxStorage)}</td>
        <td>${formatNumber(maxCpu)}</td>
        <td>${tolerance}%</td>
        <td><span class="status-badge ${statusClass}">${statusClass === 'healthy' ? 'OK' : statusClass === 'warning' ? 'WARN' : 'CRIT'}</span></td>
      </tr>`;
  }
  return rows;
}

function generateHTML(budgets, baselines) {
  const contractRows = generateContractRows(budgets, baselines);
  const contractCount = Object.keys(budgets.contracts || {}).length;
  const defaults = budgets.defaults || {};

  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Uzima Resource Budget Dashboard</title>
    <style>
        :root {
            --bg-primary: #0a0b10;
            --bg-secondary: #161821;
            --accent-primary: #6366f1;
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --success: #10b981;
            --warning: #f59e0b;
            --danger: #ef4444;
        }
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { background: var(--bg-primary); color: var(--text-primary); font-family: 'Inter', sans-serif; padding: 2rem; }
        .dashboard { max-width: 1400px; margin: 0 auto; }
        h1 { font-size: 2rem; margin-bottom: 0.5rem; }
        .subtitle { color: var(--text-secondary); margin-bottom: 2rem; }
        .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 2rem; }
        .stat-card { background: var(--bg-secondary); padding: 1.5rem; border-radius: 12px; border: 1px solid rgba(255,255,255,0.05); }
        .stat-card h3 { color: var(--text-secondary); font-size: 0.85rem; margin-bottom: 0.5rem; }
        .stat-value { font-size: 1.8rem; font-weight: 700; }
        .table-container { background: var(--bg-secondary); padding: 1.5rem; border-radius: 12px; border: 1px solid rgba(255,255,255,0.05); overflow-x: auto; }
        table { width: 100%; border-collapse: collapse; }
        th { text-align: left; color: var(--text-secondary); font-size: 0.8rem; padding: 0.75rem 0.5rem; border-bottom: 1px solid rgba(255,255,255,0.1); }
        td { padding: 0.75rem 0.5rem; font-size: 0.9rem; border-bottom: 1px solid rgba(255,255,255,0.03); }
        .contract-name { font-weight: 600; }
        .progress-bar { display: inline-block; width: 100px; height: 8px; background: rgba(255,255,255,0.1); border-radius: 4px; overflow: hidden; margin-right: 8px; vertical-align: middle; }
        .progress-fill { height: 100%; border-radius: 4px; }
        .progress-fill.healthy { background: var(--success); }
        .progress-fill.warning { background: var(--warning); }
        .progress-fill.critical { background: var(--danger); }
        .progress-label { font-size: 0.75rem; vertical-align: middle; }
        .progress-label.healthy { color: var(--success); }
        .progress-label.warning { color: var(--warning); }
        .progress-label.critical { color: var(--danger); }
        .status-badge { padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.7rem; font-weight: 600; }
        .status-badge.healthy { background: rgba(16,185,129,0.15); color: var(--success); }
        .status-badge.warning { background: rgba(245,158,11,0.15); color: var(--warning); }
        .status-badge.critical { background: rgba(239,68,68,0.15); color: var(--danger); }
        .footer { margin-top: 2rem; color: var(--text-secondary); font-size: 0.8rem; text-align: center; }
    </style>
</head>
<body>
    <div class="dashboard">
        <h1>Resource Budget Dashboard</h1>
        <p class="subtitle">Per-contract resource budgets and utilization tracking</p>
        <div class="stats-grid">
            <div class="stat-card"><h3>Total Contracts</h3><p class="stat-value">${contractCount}</p></div>
            <div class="stat-card"><h3>Default WASM Limit</h3><p class="stat-value">${formatBytes(defaults.max_wasm_bytes || 65536)}</p></div>
            <div class="stat-card"><h3>Default Storage Limit</h3><p class="stat-value">${formatNumber(defaults.max_storage_entries || 500)}</p></div>
            <div class="stat-card"><h3>Default CPU Limit</h3><p class="stat-value">${formatNumber(defaults.max_cpu_instructions || 10000000)}</p></div>
        </div>
        <div class="table-container">
            <h2 style="margin-bottom: 1rem; font-size: 1.2rem;">Contract Budgets</h2>
            <table>
                <thead><tr><th>Contract</th><th>WASM Size</th><th>WASM Limit</th><th>Utilization</th><th>Storage Limit</th><th>CPU Limit</th><th>Tolerance</th><th>Status</th></tr></thead>
                <tbody>${contractRows}</tbody>
            </table>
        </div>
        <div class="footer">Generated: ${new Date().toISOString()} | Budgets: resource-budgets/budgets.json</div>
    </div>
</body>
</html>`;
}

function main() {
  const args = process.argv.slice(2);
  const outputIdx = args.indexOf('--output');
  const outputPath = outputIdx >= 0 ? args[outputIdx + 1] : DEFAULT_OUTPUT;

  if (!existsSync(BUDGET_FILE)) {
    console.error(`[budget-dashboard] Budget file not found: ${BUDGET_FILE}`);
    process.exit(1);
  }

  const budgets = loadJSON(BUDGET_FILE);
  const baselines = loadJSON(BASELINE_FILE);
  const html = generateHTML(budgets, baselines);

  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, html);

  console.log(`[budget-dashboard] Dashboard generated: ${outputPath}`);
  console.log(`[budget-dashboard] ${Object.keys(budgets.contracts || {}).length} contracts included`);
}

main();
