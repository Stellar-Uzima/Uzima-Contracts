import { readFile, writeFile } from "node:fs/promises";
import { readdir } from "node:fs/promises";
import path from "node:path";

const REPO_ROOT = process.cwd();
const OUT_JSON = path.join(REPO_ROOT, "reports", "testing", "pyramid.json");
const OUT_MD = path.join(REPO_ROOT, "reports", "testing", "pyramid.md");

function pct(n) {
  return `${(n * 100).toFixed(1)}%`;
}

async function listFilesRecursive(dir, pred) {
  const out = [];
  const entries = await readdir(dir, { withFileTypes: true });
  for (const ent of entries) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      if (ent.name === "target" || ent.name === "node_modules") continue;
      out.push(...(await listFilesRecursive(p, pred)));
    } else if (ent.isFile()) {
      if (!pred || pred(p)) out.push(p);
    }
  }
  return out;
}

function rel(p) {
  return path.relative(REPO_ROOT, p).replaceAll("\\", "/");
}

function classify(fileRel) {
  // E2E
  if (fileRel.startsWith("tests/e2e/")) return "e2e";

  // Integration
  if (fileRel.startsWith("tests/integration/")) return "integration";
  if (fileRel.startsWith("tests/") && /integration/i.test(path.posix.basename(fileRel))) return "integration";
  if (/^contracts\/[^/]+\/tests\//.test(fileRel)) return "integration";

  // Unit
  if (fileRel.startsWith("tests/unit/")) return "unit";
  if (/^contracts\/[^/]+\/src\//.test(fileRel)) {
    const base = path.posix.basename(fileRel);
    if (/^test.*\.rs$/i.test(base) || /tests?\.rs$/i.test(base) || /_tests?\.rs$/i.test(base)) return "unit";
    if (fileRel.includes("/test_")) return "unit";
    if (fileRel.endsWith("/test.rs")) return "unit";
  }

  // Default: ignore (not in pyramid scope)
  return null;
}

function countRustTests(text) {
  // Simple heuristic: count test attributes.
  // Covers: #[test], #[tokio::test], #[async_std::test], #[rstest]
  const re = /#\s*\[\s*(?:tokio::|async_std::)?test\s*(?:\([^\)]*\))?\s*\]/g;
  let count = 0;
  for (const _ of text.matchAll(re)) count++;
  return count;
}

function buildMarkdown(summary, perContract) {
  const lines = [];
  lines.push("# Testing pyramid report");
  lines.push("");
  lines.push("Auto-generated from Rust test files using location-based classification.");
  lines.push("");
  lines.push("## Summary");
  lines.push("");
  lines.push(`- Total tests: **${summary.total}**`);
  lines.push(`- Unit: **${summary.counts.unit}** (${pct(summary.ratios.unit)})`);
  lines.push(`- Integration: **${summary.counts.integration}** (${pct(summary.ratios.integration)})`);
  lines.push(`- E2E: **${summary.counts.e2e}** (${pct(summary.ratios.e2e)})`);
  lines.push("");
  lines.push("## Per-contract (unit+integration only)");
  lines.push("");
  lines.push("| Contract | Unit | Integration | Total |");
  lines.push("|---|---:|---:|---:|");
  for (const c of perContract) {
    lines.push(`| \`${c.contract}\` | ${c.unit} | ${c.integration} | ${c.total} |`);
  }
  lines.push("");
  return lines.join("\n") + "\n";
}

function ratios(counts) {
  const total = counts.unit + counts.integration + counts.e2e;
  if (total === 0) return { unit: 0, integration: 0, e2e: 0 };
  return {
    unit: counts.unit / total,
    integration: counts.integration / total,
    e2e: counts.e2e / total
  };
}

async function main() {
  const config = JSON.parse(await readFile(path.join(REPO_ROOT, "config", "testing-pyramid.json"), "utf8"));
  const rustFiles = await listFilesRecursive(REPO_ROOT, (p) => p.endsWith(".rs"));

  const fileCounts = [];
  const totals = { unit: 0, integration: 0, e2e: 0 };

  // per-contract rollup for unit+integration
  const perContractMap = new Map();

  for (const abs of rustFiles) {
    const r = rel(abs);
    const level = classify(r);
    if (!level) continue;
    const text = await readFile(abs, "utf8");
    const count = countRustTests(text);
    if (count === 0) continue;

    fileCounts.push({ file: r, level, tests: count });
    totals[level] += count;

    const m = r.match(/^contracts\/([^/]+)\//);
    if (m && (level === "unit" || level === "integration")) {
      const contract = m[1];
      const cur = perContractMap.get(contract) || { contract, unit: 0, integration: 0, total: 0 };
      cur[level] += count;
      cur.total += count;
      perContractMap.set(contract, cur);
    }
  }

  const total = totals.unit + totals.integration + totals.e2e;
  const summary = { total, counts: totals, ratios: ratios(totals), targets: config.targets, tolerance: config.tolerance };

  const perContract = [...perContractMap.values()].sort((a, b) => (a.contract < b.contract ? -1 : 1));

  const report = {
    generated_at: new Date().toISOString(),
    summary,
    files: fileCounts.sort((a, b) => (a.file < b.file ? -1 : 1)),
    per_contract: perContract
  };

  await writeFile(OUT_JSON, JSON.stringify(report, null, 2) + "\n", "utf8");
  await writeFile(OUT_MD, buildMarkdown(summary, perContract), "utf8");

  // Validation / enforcement
  const errors = [];
  if (total < config.min_total_tests) {
    errors.push(`Total tests ${total} is below min_total_tests (${config.min_total_tests}).`);
  }

  for (const [level, target] of Object.entries(config.targets)) {
    const actual = summary.ratios[level];
    const ok = Math.abs(actual - target) <= config.tolerance;
    if (!ok) {
      errors.push(`Ratio for ${level} is ${pct(actual)}; target ${pct(target)} ± ${pct(config.tolerance)}.`);
    }
    if (config.fail_on_missing_level && summary.counts[level] === 0) {
      errors.push(`Missing test level: ${level} has 0 tests.`);
    }
  }

  if (errors.length) {
    process.stderr.write("❌ Testing pyramid validation failed:\n" + errors.map((e) => `- ${e}`).join("\n") + "\n");
    process.exit(1);
  }

  process.stdout.write("✅ Testing pyramid validation passed.\n");
}

await main();

