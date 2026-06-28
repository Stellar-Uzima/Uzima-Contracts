#!/usr/bin/env node
/**
 * generate_usage_report.js
 *
 * Automated report generator for Uzima Contract Usage Analytics.
 * Reads aggregated usage data and produces a Markdown report in the
 * /reports directory. In production this script would call the
 * ContractUsageAnalytics contract via Stellar Horizon / RPC APIs.
 */

'use strict';

const fs   = require('fs');
const path = require('path');

// ─── Mock data (replace with real Stellar RPC calls) ─────────────────────────

const MOCK_FUNCTIONS = [
  { name: 'submit_clinical_trial', callCount: 1250, totalCpu: 56_500_000, totalRam: 1_310_720, errorCount: 6,  avgLatencyMs: 120 },
  { name: 'authorize_access',       callCount: 3420, totalCpu: 43_776_000, totalRam:   409_600, errorCount: 3,  avgLatencyMs:  45 },
  { name: 'register_patient',       callCount:  890, totalCpu: 76_006_000, totalRam: 2_621_440, errorCount: 11, avgLatencyMs: 210 },
  { name: 'mint_consent_nft',       callCount:  450, totalCpu: 54_045_000, totalRam: 5_033_164, errorCount: 11, avgLatencyMs: 350 },
  { name: 'sync_medical_record',    callCount: 2100, totalCpu: 136_920_000, totalRam: 3_250_000, errorCount: 17, avgLatencyMs: 180 },
];

const MOCK_SNAPSHOTS = Array.from({ length: 7 }, (_, i) => ({
  timestamp:      1_745_500_000 + i * 86_400,
  totalCalls:     [450, 620, 580, 890, 750, 1100, 1250][i],
  activeUsers:    [120, 150, 140, 210, 180, 250,  280][i],
  errorRateBps:   [80,  50,  90,  120, 100, 70,   85][i],
}));

// ─── Helpers ─────────────────────────────────────────────────────────────────

function bpsToPercent(bps) {
  return (bps / 100).toFixed(2) + '%';
}

function fmtBytes(bytes) {
  if (bytes < 1024)         return `${bytes} B`;
  if (bytes < 1_048_576)    return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1_048_576).toFixed(2)} MB`;
}

function fmtUnits(n) {
  if (n < 1_000)        return String(n);
  if (n < 1_000_000)    return `${(n / 1_000).toFixed(1)} K`;
  return `${(n / 1_000_000).toFixed(2)} M`;
}

function isoDate(ts) {
  return new Date(ts * 1000).toISOString().slice(0, 10);
}

function pad(str, len) {
  return String(str).padEnd(len);
}

// ─── Report builder ──────────────────────────────────────────────────────────

function buildReport(functions, snapshots) {
  const totalCalls    = functions.reduce((s, f) => s + f.callCount, 0);
  const totalErrors   = functions.reduce((s, f) => s + f.errorCount, 0);
  const overallErrPct = totalCalls ? ((totalErrors / totalCalls) * 100).toFixed(2) : '0.00';
  const avgLatency    = Math.round(functions.reduce((s, f) => s + f.avgLatencyMs, 0) / functions.length);
  const latestSnap    = snapshots[snapshots.length - 1];
  const reportDate    = new Date().toISOString().slice(0, 10);

  // Function table rows
  const fnRows = functions.map(f => {
    const errPct = ((f.errorCount / (f.callCount || 1)) * 100).toFixed(2) + '%';
    return `| ${pad(f.name, 30)} | ${pad(f.callCount.toLocaleString(), 10)} | ${pad(fmtUnits(f.totalCpu), 10)} | ${pad(fmtBytes(f.totalRam), 12)} | ${pad(errPct, 10)} | ${pad(f.avgLatencyMs + 'ms', 10)} |`;
  }).join('\n');

  // Snapshot table rows
  const snapRows = snapshots.map(s =>
    `| ${isoDate(s.timestamp)} | ${s.totalCalls.toLocaleString()} | ${s.activeUsers} | ${bpsToPercent(s.errorRateBps)} |`
  ).join('\n');

  return `# Uzima Contract Usage Analytics Report
**Generated:** ${reportDate}

---

## Summary

| Metric               | Value          |
|----------------------|----------------|
| Total Function Calls | ${totalCalls.toLocaleString()} |
| Total Errors         | ${totalErrors}  |
| Overall Error Rate   | ${overallErrPct}% |
| Avg Function Latency | ${avgLatency} ms |
| Active Users (today) | ${latestSnap.activeUsers} |

---

## Function Call Frequencies

| Function Name                   | Calls      | CPU Usage  | RAM Usage    | Error Rate | Avg Latency |
|---------------------------------|------------|------------|--------------|------------|-------------|
${fnRows}

---

## Daily Snapshot History

| Date       | Total Calls | Active Users | Error Rate |
|------------|-------------|--------------|------------|
${snapRows}

---

## Gas Usage Trends

The table above captures cumulative CPU instruction units and RAM byte-seconds
consumed per function.  High \`totalCpu\` coupled with low \`errorCount\` indicates
efficient, stable functions.  Functions with \`avgLatencyMs > 200\` should be
reviewed for optimisation opportunities.

---

## Error Rate Analysis

Overall contract error rate is **${overallErrPct}%**.
${parseFloat(overallErrPct) > 1
    ? '⚠️  Error rate exceeds the 1 % threshold — investigation recommended.'
    : '✅  Error rate is within acceptable bounds.'}

---

## Performance Metrics

| Metric               | Value      |
|----------------------|------------|
| Fastest function     | ${functions.reduce((a, b) => a.avgLatencyMs < b.avgLatencyMs ? a : b).name} |
| Slowest function     | ${functions.reduce((a, b) => a.avgLatencyMs > b.avgLatencyMs ? a : b).name} |
| Most called function | ${functions.reduce((a, b) => a.callCount > b.callCount ? a : b).name} |

---

*Report auto-generated by \`scripts/generate_usage_report.js\` — Uzima Contracts Analytics Framework*
`;
}

// ─── Main ─────────────────────────────────────────────────────────────────────

function main() {
  const reportsDir = path.resolve(__dirname, '..', 'reports');
  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true });
  }

  const report   = buildReport(MOCK_FUNCTIONS, MOCK_SNAPSHOTS);
  const fileName = `usage_report_${new Date().toISOString().slice(0, 10)}.md`;
  const outPath  = path.join(reportsDir, fileName);

  fs.writeFileSync(outPath, report, 'utf8');
  console.log(`✅  Report generated: ${outPath}`);
  console.log('\n--- Preview ---\n');
  console.log(report.slice(0, 600) + '\n...');
}

main();
