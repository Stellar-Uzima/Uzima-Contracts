import { readFile, writeFile } from "node:fs/promises";
import { readdir } from "node:fs/promises";
import path from "node:path";

const REPO_ROOT = process.cwd();
const CONTRACTS_DIR = path.join(REPO_ROOT, "contracts");
const OUT_REGISTRY = path.join(REPO_ROOT, "schemas", "events", "registry.json");
const OUT_DOCS = path.join(REPO_ROOT, "docs", "EVENTS.md");

/**
 * Very small, deterministic generator intended for CI enforcement.
 * We intentionally keep extraction conservative: we only extract string topics
 * present in `symbol_short!("...")` or string literals inside the topics argument.
 */

function isDirectoryEntry(ent) {
  return ent && typeof ent.isDirectory === "function" && ent.isDirectory();
}

async function listContractNames() {
  const entries = await readdir(CONTRACTS_DIR, { withFileTypes: true });
  return entries.filter(isDirectoryEntry).map((e) => e.name).sort();
}

async function listRustFiles(dir) {
  const out = [];
  const entries = await readdir(dir, { withFileTypes: true });
  for (const ent of entries) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      if (ent.name === "target") continue; // skip build artifacts
      out.push(...(await listRustFiles(p)));
    } else if (ent.isFile() && ent.name.endsWith(".rs")) {
      out.push(p);
    }
  }
  return out;
}

function findAllPublishCalls(text) {
  // We parse by scanning for "env.events().publish(" then extracting balanced parentheses.
  const needle = "env.events().publish(";
  const calls = [];
  let idx = 0;
  while (true) {
    const start = text.indexOf(needle, idx);
    if (start === -1) break;
    const argsStart = start + needle.length;
    let i = argsStart;
    let depth = 1;
    while (i < text.length && depth > 0) {
      const ch = text[i];
      if (ch === "(") depth++;
      else if (ch === ")") depth--;
      i++;
    }
    const argsEnd = i - 1; // index of closing ')'
    const argsText = text.slice(argsStart, argsEnd);
    calls.push({ start, argsStart, argsEnd, argsText });
    idx = argsEnd + 1;
  }
  return calls;
}

function splitTopLevelCommaPair(argsText) {
  // Split into [topicsExpr, dataExpr] at the first top-level comma.
  // We track (), {}, [], <> (very roughly for generics) and string literals.
  let depthParen = 0;
  let depthBrack = 0;
  let depthBrace = 0;
  let depthAngle = 0;
  let inString = false;
  let stringQuote = null;
  let escape = false;
  for (let i = 0; i < argsText.length; i++) {
    const ch = argsText[i];
    if (inString) {
      if (escape) {
        escape = false;
      } else if (ch === "\\") {
        escape = true;
      } else if (ch === stringQuote) {
        inString = false;
        stringQuote = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'") {
      inString = true;
      stringQuote = ch;
      continue;
    }
    if (ch === "(") depthParen++;
    else if (ch === ")") depthParen = Math.max(0, depthParen - 1);
    else if (ch === "[") depthBrack++;
    else if (ch === "]") depthBrack = Math.max(0, depthBrack - 1);
    else if (ch === "{") depthBrace++;
    else if (ch === "}") depthBrace = Math.max(0, depthBrace - 1);
    else if (ch === "<") depthAngle++;
    else if (ch === ">") depthAngle = Math.max(0, depthAngle - 1);

    const topLevel = depthParen === 0 && depthBrack === 0 && depthBrace === 0 && depthAngle === 0;
    if (topLevel && ch === ",") {
      const left = argsText.slice(0, i).trim();
      const right = argsText.slice(i + 1).trim();
      return [left, right];
    }
  }
  return [argsText.trim(), ""];
}

function extractStringTopics(topicsExpr) {
  const topics = [];

  // symbol_short!("ABC")
  const symRe = /symbol_short!\(\s*"([^"]+)"\s*\)/g;
  for (const m of topicsExpr.matchAll(symRe)) topics.push(m[1]);

  // Also capture plain string literals inside tuple topics like ("LOG", topic)
  const litRe = /"([^"]+)"/g;
  for (const m of topicsExpr.matchAll(litRe)) {
    const s = m[1];
    if (!topics.includes(s)) topics.push(s);
  }

  // Deduplicate while preserving order
  return topics.filter((t, i) => topics.indexOf(t) === i);
}

function resolveIdentifierTopics(fileTextBeforeCall, identifier) {
  // Try to resolve `let <identifier> = <expr>;` (including `let mut`).
  // We pick the last match before the publish call for best locality.
  const re = new RegExp(`\\blet\\s+(?:mut\\s+)?${identifier}\\s*=\\s*([^;]+);`, "g");
  let last = null;
  for (const m of fileTextBeforeCall.matchAll(re)) last = m[1];
  if (!last) return [];
  return extractStringTopics(last);
}

function payloadShapeAndArity(dataExpr) {
  const t = dataExpr.trim();
  if (!t) return { shape: "unknown", arity: 0 };
  if (t.startsWith("(")) {
    // count top-level comma-separated items inside the outermost tuple
    let inner = t;
    // find matching ')'
    let depth = 0;
    let end = -1;
    for (let i = 0; i < t.length; i++) {
      if (t[i] === "(") depth++;
      else if (t[i] === ")") {
        depth--;
        if (depth === 0) {
          end = i;
          break;
        }
      }
    }
    if (end > 0) inner = t.slice(1, end).trim();
    if (!inner) return { shape: "tuple", arity: 0 };

    let depthParen = 0;
    let depthBrack = 0;
    let depthBrace = 0;
    let inString = false;
    let escape = false;
    let count = 1;
    for (let i = 0; i < inner.length; i++) {
      const ch = inner[i];
      if (inString) {
        if (escape) escape = false;
        else if (ch === "\\") escape = true;
        else if (ch === '"') inString = false;
        continue;
      }
      if (ch === '"') {
        inString = true;
        continue;
      }
      if (ch === "(") depthParen++;
      else if (ch === ")") depthParen = Math.max(0, depthParen - 1);
      else if (ch === "[") depthBrack++;
      else if (ch === "]") depthBrack = Math.max(0, depthBrack - 1);
      else if (ch === "{") depthBrace++;
      else if (ch === "}") depthBrace = Math.max(0, depthBrace - 1);

      const topLevel = depthParen === 0 && depthBrack === 0 && depthBrace === 0;
      if (topLevel && ch === ",") count++;
    }
    return { shape: "tuple", arity: count };
  }
  return { shape: "single", arity: 1 };
}

function lineNumberFromIndex(text, index) {
  // 1-indexed line number
  let line = 1;
  for (let i = 0; i < index && i < text.length; i++) {
    if (text[i] === "\n") line++;
  }
  return line;
}

async function generate() {
  const contracts = [];
  const dynamicTopicEmissions = [];
  const contractNames = await listContractNames();
  for (const name of contractNames) {
    const contractRoot = path.join(CONTRACTS_DIR, name);
    const rustFiles = await listRustFiles(contractRoot);
    const events = [];

    for (const filePath of rustFiles) {
      const rel = path.relative(REPO_ROOT, filePath).replaceAll("\\", "/");
      const text = await readFile(filePath, "utf8");
      const calls = findAllPublishCalls(text);
      for (const call of calls) {
        const [topicsExpr, dataExpr] = splitTopLevelCommaPair(call.argsText);
        let topics = extractStringTopics(topicsExpr);
        if (topics.length === 0) {
          const ident = topicsExpr.trim();
          if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(ident)) {
            topics = resolveIdentifierTopics(text.slice(0, call.start), ident);
          }
        }
        if (topics.length === 0) {
          const line = lineNumberFromIndex(text, call.start);
          dynamicTopicEmissions.push({
            contract: name,
            file: rel,
            line,
            topicsExpr: topicsExpr.slice(0, 200)
          });
          continue;
        }

        const payload = payloadShapeAndArity(dataExpr);
        const line = lineNumberFromIndex(text, call.start);

        // Stable deterministic ID: contract + topics + source position
        const id = `${name}:${topics.join(".")}:${rel}:${line}`;
        events.push({
          id,
          contract: name,
          topics,
          payload,
          source: { file: rel, line }
        });
      }
    }

    if (events.length > 0) {
      // deterministic ordering
      events.sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
      contracts.push({ name, events });
    }
  }

  const registry = {
    registry_version: "1.0.0",
    generated_at: new Date().toISOString(),
    generator: {
      name: "scripts/events/generate-registry.mjs",
      source_glob: "contracts/**/src/**/*.rs"
    },
    contracts
  };

  if (dynamicTopicEmissions.length > 0) {
    const summary = dynamicTopicEmissions
      .slice(0, 20)
      .map((d) => `- ${d.contract}: ${d.file}:${d.line}`)
      .join("\n");
    throw new Error(
      [
        "Found event emissions with fully-dynamic topics that cannot be validated deterministically.",
        "Please refactor to include at least one stable string topic (e.g. symbol_short!(\"...\") or \"...\") in the topics tuple.",
        "",
        "Examples:",
        summary,
        dynamicTopicEmissions.length > 20 ? `\n(and ${dynamicTopicEmissions.length - 20} more...)` : ""
      ].join("\n")
    );
  }

  const md = renderDocs(registry);

  await writeFile(OUT_REGISTRY, JSON.stringify(registry, null, 2) + "\n", "utf8");
  await writeFile(OUT_DOCS, md, "utf8");
}

function renderDocs(registry) {
  const lines = [];
  lines.push("# Contract Events");
  lines.push("");
  lines.push("This document is auto-generated from on-chain event emissions found in `contracts/**/src/**/*.rs`.");
  lines.push("");
  lines.push(`- Registry format version: \`${registry.registry_version}\``);
  lines.push(`- Generated at: \`${registry.generated_at}\``);
  lines.push("");

  for (const c of registry.contracts) {
    lines.push(`## ${c.name}`);
    lines.push("");
    lines.push("| Topics | Payload | Source |");
    lines.push("|---|---:|---|");
    for (const e of c.events) {
      const topics = e.topics.map((t) => `\`${t}\``).join(" · ");
      const payload = `${e.payload.shape} (${e.payload.arity})`;
      const source = `\`${e.source.file}:${e.source.line}\``;
      lines.push(`| ${topics} | ${payload} | ${source} |`);
    }
    lines.push("");
  }
  return lines.join("\n") + "\n";
}

await generate();

