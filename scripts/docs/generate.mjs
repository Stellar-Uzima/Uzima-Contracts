import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.resolve(__dirname, '../../');

const CONTRACTS_DIR = path.join(PROJECT_ROOT, 'contracts');
const OUTPUT_JSON = path.join(PROJECT_ROOT, 'schemas/docs/api.json');
const OUTPUT_MD = path.join(PROJECT_ROOT, 'docs/API_REFERENCE.md');
const OUTPUT_HTML = path.join(PROJECT_ROOT, 'docs/portal/index.html');

function parseContract(contractPath) {
  const name = path.basename(contractPath);
  const libPath = path.join(contractPath, 'src/lib.rs');
  const errorsPath = path.join(contractPath, 'src/errors.rs');
  const testPath = path.join(contractPath, 'src/test.rs');

  if (!fs.existsSync(libPath)) return null;

  const libContent = fs.readFileSync(libPath, 'utf8');
  const contractData = {
    name,
    functions: [],
    types: [],
    errorCodes: [],
    examples: []
  };

  // Extract functions
  const lines = libContent.split('\n');
  let currentDocs = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.startsWith('///')) {
      currentDocs.push(line.replace('///', '').trim());
    } else if (line.startsWith('pub fn ')) {
      const match = line.match(/pub fn (\w+)\(([^)]*)\)(?: -> ([^{]+))?/);
      if (match) {
        contractData.functions.push({
          name: match[1],
          params: match[2].trim(),
          returns: (match[3] || '()').trim(),
          description: currentDocs.join(' ') || '—'
        });
      }
      currentDocs = [];
    } else if (line.length > 0 && !line.startsWith('#')) {
      currentDocs = [];
    }
  }

  // Extract types (simplified)
  const typeMatches = libContent.matchAll(/pub (struct|enum) (\w+) \{([\s\S]*?)\}/g);
  for (const match of typeMatches) {
    const kind = match[1];
    const typeName = match[2];
    const body = match[3];
    const fields = [];

    if (kind === 'struct') {
      const fieldMatches = body.matchAll(/pub (\w+): ([^,]+)/g);
      for (const f of fieldMatches) {
        fields.push({ name: f[1], type: f[2].trim(), description: '—' });
      }
    } else {
      const variantMatches = body.matchAll(/(\w+)(?: = (\d+))?/g);
      for (const v of variantMatches) {
        fields.push({ variant: v[1], value: v[2] || '—', description: '—' });
      }
    }

    contractData.types.push({ kind, name: typeName, fields });
  }

  // Extract errors
  if (fs.existsSync(errorsPath)) {
    const errorsContent = fs.readFileSync(errorsPath, 'utf8');
    const errorMatches = errorsContent.matchAll(/pub enum (\w+) \{([\s\S]*?)\}/g);
    for (const match of errorMatches) {
      const variants = match[2].matchAll(/(\w+) = (\d+)/g);
      for (const v of variants) {
        contractData.errorCodes.push({ name: v[1], code: v[2] });
      }
    }
  }

  // Extract examples from test.rs
  if (fs.existsSync(testPath)) {
    const testContent = fs.readFileSync(testPath, 'utf8');
    const testMatches = testContent.matchAll(/#\[test\]\s+fn (\w+)\(\) \{([\s\S]*?)\n\}/g);
    for (const match of testMatches) {
      const testName = match[1];
      const testBody = match[2].split('\n').slice(0, 10).join('\n').trim(); // Take first 10 lines as example
      contractData.examples.push({ name: testName, code: testBody });
    }
  }

  return contractData;
}

function generateMarkdown(data) {
  let md = `# Uzima Contracts — API Reference\n\n`;
  md += `> Auto-generated from contract source code. Do not edit manually.\n\n`;
  md += `- **API version**: \`1.0.0\`\n`;
  md += `- **Generated**: \`${new Date().toISOString()}\`\n`;
  md += `- **Contracts documented**: ${data.length}\n\n`;

  md += `## Table of Contents\n\n`;
  data.forEach(c => {
    md += `- [${c.name}](#${c.name.replace(/_/g, '-')})\n`;
  });
  md += `\n---\n\n`;

  data.forEach(c => {
    md += `## ${c.name}\n\n`;
    
    md += `### Functions\n\n`;
    md += `| Function | Parameters | Returns | Description |\n`;
    md += `|---|---|---|---|\n`;
    c.functions.forEach(f => {
      md += `| \`${f.name}\` | \`${f.params.replace(/\|/g, '\\|')}\` | \`${f.returns.replace(/\|/g, '\\|')}\` | ${f.description} |\n`;
    });
    md += `\n`;

    if (c.types.length > 0) {
      md += `### Types\n\n`;
      c.types.forEach(t => {
        md += `#### \`${t.kind} ${t.name}\`\n\n`;
        if (t.kind === 'struct') {
          md += `| Field | Type | Description |\n`;
          md += `|---|---|---|\n`;
          t.fields.forEach(f => {
            md += `| \`${f.name}\` | \`${f.type}\` | ${f.description} |\n`;
          });
        } else {
          md += `| Variant | Value | Description |\n`;
          md += `|---|---|---|\n`;
          t.fields.forEach(f => {
            md += `| \`${f.variant}\` | ${f.value} | ${f.description} |\n`;
          });
        }
        md += `\n`;
      });
    }

    if (c.errorCodes.length > 0) {
      md += `### Error Codes\n\n`;
      md += `| Variant | Code | Description |\n`;
      md += `|---|---|---|\n`;
      c.errorCodes.forEach(e => {
        md += `| \`${e.name}\` | ${e.code} | — |\n`;
      });
      md += `\n`;
    }

    if (c.examples.length > 0) {
      md += `### Examples\n\n`;
      c.examples.slice(0, 3).forEach(ex => {
        md += `#### \`${ex.name}\`\n\n`;
        md += `\`\`\`rust\n${ex.code}\n\`\`\`\n\n`;
      });
    }

    md += `---\n\n`;
  });

  return md;
}

function generateHTML(data) {
  const timestamp = new Date().toISOString();
  const totalFunctions = data.reduce((acc, c) => acc + c.functions.length, 0);
  const totalTypes = data.reduce((acc, c) => acc + c.types.length, 0);
  const totalExamples = data.reduce((acc, c) => acc + c.examples.length, 0);

  let html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Uzima Contracts — API Documentation Portal</title>
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
    pre { background: var(--code-bg); padding: 16px; border-radius: 6px; overflow-x: auto; }
  </style>
</head>
<body>
  <header>
    <div style="font-weight:bold;color:var(--accent)">⚕ Uzima Contracts</div>
    <div style="font-size:12px;color:var(--text2)">Generated: ${timestamp}</div>
  </header>
  <nav>
    <div style="padding:10px 20px;font-size:11px;color:var(--text2);text-transform:uppercase">Contracts</div>
    ${data.map(c => `<a href="#${c.name}">${c.name}</a>`).join('')}
  </nav>
  <main>
    <div class="hero">
      <h1>API Documentation Portal</h1>
      <p>Auto-generated reference for ${data.length} smart contracts.</p>
      <div>
        <span class="badge">${totalFunctions} Functions</span>
        <span class="badge">${totalTypes} Types</span>
        <span class="badge">${totalExamples} Examples</span>
      </div>
    </div>
    ${data.map(c => `
      <article id="${c.name}">
        <h2 style="color:var(--accent)">${c.name}</h2>
        <section>
          <h3>Functions</h3>
          <table>
            <thead><tr><th>Function</th><th>Params</th><th>Returns</th></tr></thead>
            <tbody>
              ${c.functions.map(f => `<tr><td><code>${f.name}</code></td><td><code>${f.params}</code></td><td><code>${f.returns}</code></td></tr>`).join('')}
            </tbody>
          </table>
        </section>
        ${c.types.length ? `<section><h3>Types</h3>${c.types.map(t => `<div><h4>${t.kind} ${t.name}</h4></div>`).join('')}</section>` : ''}
        ${c.examples.length ? `<section><h3>Examples</h3>${c.examples.slice(0,1).map(ex => `<pre><code>${ex.code}</code></pre>`).join('')}</section>` : ''}
      </article>
    `).join('')}
  </main>
</body>
</html>`;
  return html;
}

async function main() {
  console.log('Starting documentation generation...');
  const contracts = fs.readdirSync(CONTRACTS_DIR)
    .map(name => path.join(CONTRACTS_DIR, name))
    .filter(p => fs.statSync(p).isDirectory());

  const data = contracts
    .map(p => parseContract(p))
    .filter(d => d !== null)
    .sort((a, b) => a.name.localeCompare(b.name));

  console.log(`Found ${data.length} contracts.`);

  fs.mkdirSync(path.dirname(OUTPUT_JSON), { recursive: true });
  fs.writeFileSync(OUTPUT_JSON, JSON.stringify(data, null, 2));
  console.log(`Saved JSON to ${OUTPUT_JSON}`);

  fs.writeFileSync(OUTPUT_MD, generateMarkdown(data));
  console.log(`Saved Markdown to ${OUTPUT_MD}`);

  fs.mkdirSync(path.dirname(OUTPUT_HTML), { recursive: true });
  fs.writeFileSync(OUTPUT_HTML, generateHTML(data));
  console.log(`Saved HTML to ${OUTPUT_HTML}`);

  console.log('Documentation generation complete! 🚀');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
