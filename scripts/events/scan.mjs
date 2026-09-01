import { readFile, readdir } from "node:fs/promises";
import path from "node:path";

function isDirectoryEntry(ent) {
  return ent && typeof ent.isDirectory === "function" && ent.isDirectory();
}

export async function listContractNames(contractsDir) {
  const entries = await readdir(contractsDir, { withFileTypes: true });
  return entries
    .filter(isDirectoryEntry)
    .map((e) => e.name)
    .sort();
}

export async function listRustFiles(dir) {
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

export function findMatching(text, openIdx, openChar, closeChar) {
  let depth = 0;
  let inString = false;
  let stringQuote = null;
  let escape = false;
  for (let i = openIdx; i < text.length; i++) {
    const ch = text[i];
    if (inString) {
      if (escape) escape = false;
      else if (ch === "\\") escape = true;
      else if (ch === stringQuote) inString = false;
      continue;
    }
    if (ch === '"' || ch === "'") {
      inString = true;
      stringQuote = ch;
      continue;
    }
    if (ch === openChar) depth++;
    else if (ch === closeChar) {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

export function findAllPublishCalls(text) {
  const needle = "env.events().publish";
  const calls = [];
  let idx = 0;
  while (true) {
    const start = text.indexOf(needle, idx);
    if (start === -1) break;
    const parenOpen = text.indexOf("(", start + needle.length);
    if (parenOpen === -1) {
      idx = start + needle.length;
      continue;
    }
    const parenClose = findMatching(text, parenOpen, "(", ")");
    if (parenClose === -1) {
      idx = parenOpen + 1;
      continue;
    }
    const argsText = text.slice(parenOpen + 1, parenClose).trim();
    calls.push({ start, argsStart: parenOpen + 1, argsEnd: parenClose, argsText });
    idx = parenClose + 1;
  }
  return calls;
}

export function splitTopLevelCommaPair(argsText) {
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

    const topLevel =
      depthParen === 0 && depthBrack === 0 && depthBrace === 0 && depthAngle === 0;
    if (topLevel && ch === ",") {
      const left = argsText.slice(0, i).trim();
      const right = argsText.slice(i + 1).trim();
      return [left, right];
    }
  }
  return [argsText.trim(), ""];
}

export function extractStringTopics(topicsExpr) {
  const topics = [];

  // symbol_short!("ABC")
  const symRe = /symbol_short!\(\s*"([^"]+)"\s*\)/g;
  for (const m of topicsExpr.matchAll(symRe)) topics.push(m[1]);

  // Symbol::new(&env, "ABC") or Symbol::new(env, "ABC")
  const symNewRe = /Symbol::new\s*\(\s*(?:&?\w+)?\s*,\s*"([^"]+)"\s*\)/g;
  for (const m of topicsExpr.matchAll(symNewRe)) topics.push(m[1]);

  // Plain string literals inside tuple topics like ("LOG", topic)
  const litRe = /"([^"]+)"/g;
  for (const m of topicsExpr.matchAll(litRe)) {
    const s = m[1];
    if (!topics.includes(s)) topics.push(s);
  }

  // Deduplicate while preserving order
  return topics.filter((t, i) => topics.indexOf(t) === i);
}

export function resolveIdentifierTopics(fileTextBeforeCall, identifier) {
  // Try to resolve `let <identifier> = <expr>;` (including `let mut`).
  // We pick the last match before the publish call for best locality.
  const re = new RegExp(
    `\\blet\\s+(?:mut\\s+)?${identifier}\\s*=\\s*([^;]+);`,
    "g"
  );
  let last = null;
  for (const m of fileTextBeforeCall.matchAll(re)) last = m[1];
  if (!last) return [];
  return extractStringTopics(last);
}

export function payloadShapeAndArity(dataExpr) {
  const t = dataExpr.trim();
  if (!t) return { shape: "unknown", arity: 0 };
  if (t.startsWith("(")) {
    let inner = t;
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

export function lineNumberFromIndex(text, index) {
  let line = 1;
  for (let i = 0; i < index && i < text.length; i++) {
    if (text[i] === "\n") line++;
  }
  return line;
}

export async function scanContractEvents(repoRoot, contractsDir) {
  const contracts = [];
  const dynamicTopicEmissions = [];
  const contractNames = await listContractNames(contractsDir);

  for (const name of contractNames) {
    const contractRoot = path.join(contractsDir, name);
    const rustFiles = await listRustFiles(contractRoot);
    const events = [];

    for (const filePath of rustFiles) {
      const rel = path.relative(repoRoot, filePath).replaceAll("\\", "/");
      const text = await readFile(filePath, "utf8");
      const calls = findAllPublishCalls(text);
      for (const call of calls) {
        const [topicsExpr, dataExpr] = splitTopLevelCommaPair(call.argsText);
        let topics = extractStringTopics(topicsExpr);
        if (topics.length === 0) {
          const idents = Array.from(
            topicsExpr.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\b/g)
          )
            .map((m) => m[1])
            .filter(
              (id) =>
                id !== "symbol_short" &&
                id !== "Symbol" &&
                id !== "new" &&
                id !== "env"
            );
          for (const ident of idents) {
            const resolved = resolveIdentifierTopics(
              text.slice(0, call.start),
              ident
            );
            for (const r of resolved) {
              if (!topics.includes(r)) topics.push(r);
            }
          }
        }

        if (topics.length === 0) {
          const line = lineNumberFromIndex(text, call.start);
          dynamicTopicEmissions.push({
            contract: name,
            file: rel,
            line,
            topicsExpr: topicsExpr.slice(0, 200),
          });
          continue;
        }

        const payload = payloadShapeAndArity(dataExpr);
        const line = lineNumberFromIndex(text, call.start);

        const id = `${name}:${topics.join(".")}:${rel}:${line}`;
        events.push({
          id,
          contract: name,
          topics,
          payload,
          source: { file: rel, line },
        });
      }
    }

    if (events.length > 0) {
      events.sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
      contracts.push({ name, events });
    }
  }

  return { contracts, dynamicTopicEmissions };
}

export function findAllStructDefs(text) {
  const needle = "#[contracttype]";
  const defs = [];
  let idx = 0;
  while (true) {
    const start = text.indexOf(needle, idx);
    if (start === -1) break;
    const structStart = text.indexOf("pub struct", start);
    if (structStart === -1) {
      idx = start + needle.length;
      continue;
    }
    let i = structStart + "pub struct".length;
    while (i < text.length && /\s/.test(text[i])) {
      i++;
    }
    const nameStart = i;
    while (i < text.length && /\w/.test(text[i])) {
      i++;
    }
    const name = text.slice(nameStart, i);
    const braceStart = text.indexOf("{", i);
    if (braceStart === -1) {
      idx = i;
      continue;
    }
    const braceEnd = findMatching(text, braceStart, "{", "}");
    if (braceEnd === -1) {
      idx = braceStart + 1;
      continue;
    }
    const body = text.slice(braceStart + 1, braceEnd);
    defs.push({ name, body });
    idx = braceEnd + 1;
  }
  return defs;
}

export function parseStructBody(body) {
  const fields = [];
  const lines = body.split("\n");
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("//")) continue;
    const parts = trimmed.split(":");
    if (parts.length !== 2) continue;
    const name = parts[0].replace("pub", "").trim();
    const type = parts[1].replace(/,$/, "").trim();
    fields.push({ name, type });
  }
  return fields;
}

export function toCamelCase(str) {
  return str.replace(/_([a-z0-9])/g, (g) => g[1].toUpperCase());
}

export function toPascalCase(str) {
  const camel = toCamelCase(str);
  return camel.charAt(0).toUpperCase() + camel.slice(1);
}

export function toSnakeCase(str) {
  return str
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase();
}

export function rustTypeToJsonType(rustType) {
  if (rustType.startsWith("Option<")) {
    return rustTypeToJsonType(rustType.slice(7, -1).trim());
  }
  switch (rustType) {
    case "String":
    case "Address":
    case "BytesN<32>":
    case "Bytes":
    case "Symbol":
      return "string";
    case "u64":
    case "u32":
    case "u128":
    case "i128":
    case "i64":
    case "i32":
    case "usize":
      return "integer";
    case "bool":
      return "boolean";
    case "Vec<String>":
    case "Vec<u64>":
    case "Vec<u32>":
    case "Vec<Address>":
      return "array";
    default:
      if (rustType.startsWith("Vec<")) return "array";
      return "object";
  }
}

export function generateJsonSchemaForStruct(structName, fields) {
  const schema = {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    title: structName,
    type: "object",
    properties: {},
    required: [],
  };

  for (const field of fields) {
    const jsonType = rustTypeToJsonType(field.type);
    const propName = toCamelCase(field.name);
    schema.properties[propName] = { type: jsonType };
    if (!field.type.startsWith("Option<")) {
      schema.required.push(propName);
    }
  }

  return schema;
}

export async function scanStructSchemas(contractsDir) {
  const schemas = new Map();
  const contractNames = await listContractNames(contractsDir);
  for (const name of contractNames) {
    const contractRoot = path.join(contractsDir, name);
    const rustFiles = await listRustFiles(contractRoot);
    for (const filePath of rustFiles) {
      if (!filePath.endsWith("event_schema.rs")) continue;
      const text = await readFile(filePath, "utf8");
      const defs = findAllStructDefs(text);
      for (const def of defs) {
        const fields = parseStructBody(def.body);
        const schema = generateJsonSchemaForStruct(def.name, fields);
        const filename = `${toSnakeCase(def.name)}.schema.json`;
        schemas.set(filename, schema);
      }
    }
  }
  return schemas;
}
