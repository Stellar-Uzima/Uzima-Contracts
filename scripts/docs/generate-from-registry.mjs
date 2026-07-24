#!/usr/bin/env node
/**
 * Generate API documentation directly from the contract interface registry.
 *
 * Reads `schemas/interface-registry/registry.json` (the source of truth for
 * public contract interfaces) and produces:
 *   - `docs/API_REGISTRY_REFERENCE.md`  – human-readable markdown
 *   - `docs/portal/registry.html`       – browsable HTML portal
 *
 * Usage:
 *   node scripts/docs/generate-from-registry.mjs            # generate
 *   node scripts/docs/generate-from-registry.mjs --check    # fail if stale
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.resolve(__dirname, '../../');

const REGISTRY_PATH = path.join(PROJECT_ROOT, 'schemas/interface-registry/registry.json');
const OUTPUT_MD = path.join(PROJECT_ROOT, 'docs/API_REGISTRY_REFERENCE.md');
const OUTPUT_HTML = path.join(PROJECT_ROOT, 'docs/portal/registry.html');
const OUTPUT_JSON = path.join(PROJECT_ROOT, 'schemas/docs/api_from_registry.json');

// ── helpers ──────────────────────────────────────────────────────────────────

function loadRegistry() {
  if (!fs.existsSync(REGISTRY_PATH)) {
    console.error(`Registry not found at ${REGISTRY_PATH}`);
    process.exit(1);
  }
  return JSON.parse(fs.readFileSync(REGISTRY_PATH, 'utf8'));
}

function escapeMarkdownCell(str) {
  return String(str).replace(/\|/g, '\\|');
}

function formatArgList(args) {
  if (!args || args.length === 0) return '—';
  return args
    .map((a) => {
      const req = a.required ? '' : '?';
      return `${a.name}: ${a.type}${req}`;
    })
    .join(', ');
}

// ── markdown generation ──────────────────────────────────────────────────────

function generateMarkdown(registry) {
  const contractNames = Object.keys(registry.contracts).sort();
  const totalFunctions = contractNames.reduce(
    (sum, name) => sum + Object.keys(registry.contracts[name].interfaces).length,
    0,
  );

  let md = `# Uzima Contracts — API Reference (from Interface Registry)\n\n`;
  md += `> Auto-generated from the contract interface registry. Do not edit manually.\n>\n`;
  md += `> Source: \`schemas/interface-registry/registry.json\`\n\n`;
  md += `- **Registry version**: \`${registry.version}\`\n`;
  md += `- **Generated**: \`${new Date().toISOString()}\`\n`;
  md += `- **Contracts documented**: ${contractNames.length}\n`;
  md += `- **Total functions**: ${totalFunctions}\n\n`;

  md += `## Table of Contents\n\n`;
  contractNames.forEach((name) => {
    const anchor = name.replace(/_/g, '-');
    md += `- [${name}](#${anchor})\n`;
  });
  md += `\n---\n\n`;

  contractNames.forEach((name) => {
    const contract = registry.contracts[name];
    const fnNames = Object.keys(contract.interfaces).sort();

    md += `## ${name}\n\n`;
    if (contract.description) {
      md += `${contract.description}\n\n`;
    }
    md += `**Version**: \`${contract.version}\` | **Functions**: ${fnNames.length}\n\n`;

    // Functions table
    md += `### Functions\n\n`;
    md += `| Function | Parameters | Returns | Mutates State | Description |\n`;
    md += `|---|---|---|---|---|\n`;
    fnNames.forEach((fnName) => {
      const fn = contract.interfaces[fnName];
      const args = formatArgList(fn.args);
      const ret = fn.returns || 'void';
      const mutation = fn.state_mutation ? 'Yes' : 'No';
      const desc = escapeMarkdownCell(fn.description || '—');
      md += `| \`${fnName}\` | \`${escapeMarkdownCell(args)}\` | \`${escapeMarkdownCell(ret)}\` | ${mutation} | ${desc} |\n`;
    });
    md += `\n`;

    // Detailed parameter tables per function
    md += `### Function Details\n\n`;
    fnNames.forEach((fnName) => {
      const fn = contract.interfaces[fnName];
      md += `#### \`${fnName}\`\n\n`;
      if (fn.description) {
        md += `${fn.description}\n\n`;
      }
      md += `- **Returns**: \`${fn.returns || 'void'}\`\n`;
      md += `- **State mutation**: ${fn.state_mutation ? 'Yes' : 'No'}\n\n`;

      if (fn.args && fn.args.length > 0) {
        md += `| Parameter | Type | Required | Description |\n`;
        md += `|---|---|---|---|\n`;
        fn.args.forEach((arg) => {
          md += `| \`${arg.name}\` | \`${arg.type}\` | ${arg.required ? 'Yes' : 'No'} | ${escapeMarkdownCell(arg.description || '—')} |\n`;
        });
        md += `\n`;
      } else {
        md += `_No parameters._\n\n`;
      }
    });

    md += `---\n\n`;
  });

  return md;
}

// ── JSON output ──────────────────────────────────────────────────────────────

function generateStructuredJSON(registry) {
  const contractNames = Object.keys(registry.contracts).sort();
  return {
    generated_at: new Date().toISOString(),
    source: 'interface-registry',
    registry_version: registry.version,
    contracts: contractNames.map((name) => {
      const contract = registry.contracts[name];
      return {
        name,
        version: contract.version,
        description: contract.description || '',
        functions: Object.keys(contract.interfaces)
          .sort()
          .map((fnName) => {
            const fn = contract.interfaces[fnName];
            return {
              name: fnName,
              description: fn.description || '',
              args: fn.args || [],
              returns: fn.returns || 'void',
              state_mutation: fn.state_mutation || false,
            };
          }),
      };
    }),
  };
}

// ── HTML generation ──────────────────────────────────────────────────────────

function generateHTML(registry) {
  const contractNames = Object.keys(registry.contracts).sort();
  const timestamp = new Date().toISOString();
  const totalFunctions = contractNames.reduce(
    (sum, name) => sum + Object.keys(registry.contracts[name].interfaces).length,
    0,
  );

  const navItems = contractNames
    .map((name) => `<a href="#${name}">${name}</a>`)
    .join('\n    ');

  const articles = contractNames
    .map((name) => {
      const contract = registry.contracts[name];
      const fnNames = Object.keys(contract.interfaces).sort();
      const rows = fnNames
        .map((fnName) => {
          const fn = contract.interfaces[fnName];
          const args = formatArgList(fn.args);
          const mutation = fn.state_mutation ? 'Yes' : 'No';
          return `<tr>
            <td><code>${fnName}</code></td>
            <td><code>${args}</code></td>
            <td><code>${fn.returns || 'void'}</code></td>
            <td>${mutation}</td>
            <td>${fn.description || '—'}</td>
          </tr>`;
        })
        .join('\n              ');

      return `
      <article id="${name}">
        <h2 style="color:var(--accent)">${name}</h2>
        <p style="color:var(--text2)">${contract.description || ''}</p>
        <p><span class="badge">v${contract.version}</span> <span class="badge">${fnNames.length} functions</span></p>
        <section>
          <h3>Functions</h3>
          <table>
            <thead><tr><th>Function</th><th>Parameters</th><th>Returns</th><th>Mutates</th><th>Description</th></tr></thead>
            <tbody>
              ${rows}
            </tbody>
          </table>
        </section>
      </article>`;
    })
    .join('\n');

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Uzima Contracts — API Registry Reference</title>
  <style>
    :root {
      --bg: #0d1117; --bg2: #161b22; --bg3: #21262d; --border: #30363d;
      --text: #e6edf3; --text2: #8b949e; --accent: #58a6ff; --code-bg: #1f2937;
      --sidebar-w: 260px; --header-h: 56px;
    }
    body { font-family: sans-serif; background: var(--bg); color: var(--text); margin: 0; }
    header { position: fixed; top: 0; left: 0; right: 0; height: var(--header-h); background: var(--bg2); border-bottom: 1px solid var(--border); display: flex; align-items: center; padding: 0 24px; z-index: 100; justify-content: space-between; }
    nav { position: fixed; top: var(--header-h); left: 0; bottom: 0; width: var(--sidebar-w); background: var(--bg2); border-right: 1px solid var(--border); overflow-y: auto; padding: 12px 0; }
    nav a { display: block; padding: 8px 20px; color: var(--text2); text-decoration: none; font-size: 13px; }
    nav a:hover { background: var(--bg3); color: var(--text); }
    main { margin-left: var(--sidebar-w); margin-top: var(--header-h); padding: 40px; }
    .hero { margin-bottom: 40px; }
    .badge { display: inline-block; background: var(--bg3); padding: 4px 12px; border-radius: 12px; font-size: 12px; margin-right: 8px; }
    article { background: var(--bg2); border: 1px solid var(--border); border-radius: 8px; padding: 24px; margin-bottom: 32px; }
    table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
    th, td { text-align: left; padding: 10px; border-bottom: 1px solid var(--border); }
    code { background: var(--code-bg); padding: 2px 4px; border-radius: 4px; font-family: monospace; }
  </style>
</head>
<body>
  <header>
    <div style="font-weight:bold;color:var(--accent)">Uzima Contracts — API Registry</div>
    <div style="font-size:12px;color:var(--text2)">Registry v${registry.version} | Generated: ${timestamp}</div>
  </header>
  <nav>
    <div style="padding:10px 20px;font-size:11px;color:var(--text2);text-transform:uppercase">Contracts</div>
    ${navItems}
  </nav>
  <main>
    <div class="hero">
      <h1>API Registry Reference</h1>
      <p>Auto-generated from the contract interface registry.</p>
      <div>
        <span class="badge">${contractNames.length} Contracts</span>
        <span class="badge">${totalFunctions} Functions</span>
      </div>
    </div>
    ${articles}
  </main>
</body>
</html>`;
}

// ── check mode ───────────────────────────────────────────────────────────────

function normalizeTimestamps(str) {
  return str
    .replace(/\d{4}-\d{2}-\d{2}T[\d:.]+Z/g, '<TIMESTAMP>')
    .replace(/Generated: [^\n<|)]+/g, 'Generated: <TIMESTAMP>');
}

function stripJsonTimestamp(obj) {
  const copy = { ...obj };
  delete copy.generated_at;
  return copy;
}

// ── main ─────────────────────────────────────────────────────────────────────

function main() {
  const checkMode = process.argv.includes('--check');

  console.log('Loading interface registry...');
  const registry = loadRegistry();

  const contractCount = Object.keys(registry.contracts).length;
  console.log(`Found ${contractCount} contracts in registry.`);

  // Generate structured JSON
  fs.mkdirSync(path.dirname(OUTPUT_JSON), { recursive: true });
  const jsonData = generateStructuredJSON(registry);
  if (checkMode) {
    if (!fs.existsSync(OUTPUT_JSON)) {
      console.error(`FAIL: ${OUTPUT_JSON} does not exist. Run without --check first.`);
      process.exit(1);
    }
    const existing = JSON.parse(fs.readFileSync(OUTPUT_JSON, 'utf8'));
    if (JSON.stringify(stripJsonTimestamp(existing)) !== JSON.stringify(stripJsonTimestamp(jsonData))) {
      console.error(`FAIL: ${OUTPUT_JSON} is stale. Regenerate with: node scripts/docs/generate-from-registry.mjs`);
      process.exit(1);
    }
    console.log('JSON output is up to date.');
  } else {
    fs.writeFileSync(OUTPUT_JSON, JSON.stringify(jsonData, null, 2));
    console.log(`Saved JSON to ${OUTPUT_JSON}`);
  }

  // Generate markdown
  const md = generateMarkdown(registry);
  if (checkMode) {
    if (!fs.existsSync(OUTPUT_MD)) {
      console.error(`FAIL: ${OUTPUT_MD} does not exist. Run without --check first.`);
      process.exit(1);
    }
    const existing = fs.readFileSync(OUTPUT_MD, 'utf8');
    if (normalizeTimestamps(existing) !== normalizeTimestamps(md)) {
      console.error(`FAIL: ${OUTPUT_MD} is stale. Regenerate with: node scripts/docs/generate-from-registry.mjs`);
      process.exit(1);
    }
    console.log('Markdown output is up to date.');
  } else {
    fs.writeFileSync(OUTPUT_MD, md);
    console.log(`Saved Markdown to ${OUTPUT_MD}`);
  }

  // Generate HTML
  const html = generateHTML(registry);
  if (checkMode) {
    if (!fs.existsSync(OUTPUT_HTML)) {
      console.error(`FAIL: ${OUTPUT_HTML} does not exist. Run without --check first.`);
      process.exit(1);
    }
    const existing = fs.readFileSync(OUTPUT_HTML, 'utf8');
    if (normalizeTimestamps(existing) !== normalizeTimestamps(html)) {
      console.error(`FAIL: ${OUTPUT_HTML} is stale. Regenerate with: node scripts/docs/generate-from-registry.mjs`);
      process.exit(1);
    }
    console.log('HTML output is up to date.');
  } else {
    fs.mkdirSync(path.dirname(OUTPUT_HTML), { recursive: true });
    fs.writeFileSync(OUTPUT_HTML, html);
    console.log(`Saved HTML to ${OUTPUT_HTML}`);
  }

  console.log('API registry documentation generation complete!');
}

main();
