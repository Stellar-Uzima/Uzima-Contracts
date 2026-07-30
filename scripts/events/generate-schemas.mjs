import { readFile, writeFile } from "node:fs/promises";
import { readdir } from "node:fs/promises";
import path from "node:path";

const REPO_ROOT = process.cwd();
const CONTRACTS_DIR = path.join(REPO_ROOT, "contracts");
const OUT_SCHEMAS_DIR = path.join(REPO_ROOT, "schemas", "events");

/**
 * Generates JSON schemas for all the event types in the codebase.
 */

function isDirectoryEntry(ent) {
  return ent && typeof ent.isDirectory === "function" && ent.isDirectory();
}

async function listContractNames() {
  const entries = await readdir(CONTRACTS_DIR, { withFileTypes: true });
  return entries
    .filter(isDirectoryEntry)
    .map((e) => e.name)
    .sort();
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

function findAllStructDefs(text) {
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
    let depth = 1;
    i = braceStart + 1;
    while (i < text.length && depth > 0) {
      const ch = text[i];
      if (ch === "{") depth++;
      else if (ch === "}") depth--;
      i++;
    }
    const braceEnd = i - 1;
    const body = text.slice(braceStart + 1, braceEnd);
    defs.push({ name, body });
    idx = braceEnd + 1;
  }
  return defs;
}

function parseStructBody(body) {
  const fields = [];
  const lines = body.split("\n");
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("//")) continue;
    const parts = trimmed.split(":");
    if (parts.length !== 2) continue;
    const name = parts[0].replace("pub", "").trim();
    const type = parts[1].replace(",", "").trim();
    fields.push({ name, type });
  }
  return fields;
}

function toCamelCase(str) {
  return str.replace(/_([a-z])/g, (g) => g[1].toUpperCase());
}

function toPascalCase(str) {
  const camel = toCamelCase(str);
  return camel.charAt(0).toUpperCase() + camel.slice(1);
}

function toSnakeCase(str) {
  return str
    .replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`)
    .replace(/^_/, "");
}

function generateJsonSchema(structName, fields) {
  const schema = {
    $schema: "http://json-schema.org/draft-07/schema#",
    title: structName,
    type: "object",
    properties: {},
    required: [],
  };

  for (const field of fields) {
    const jsonType = rustTypeToJsonType(field.type);
    schema.properties[toCamelCase(field.name)] = { type: jsonType };
    if (!field.type.startsWith("Option<")) {
      schema.required.push(toCamelCase(field.name));
    }
  }

  return schema;
}

function rustTypeToJsonType(rustType) {
  if (rustType.startsWith("Option<")) {
    return rustTypeToJsonType(rustType.slice(7, -1));
  }
  switch (rustType) {
    case "String":
      return "string";
    case "u64":
    case "u32":
    case "i128":
      return "integer";
    case "bool":
      return "boolean";
    case "Address":
      return "string";
    case "BytesN<32>":
      return "string";
    case "Vec<String>":
      return "array";
    case "Vec<u64>":
      return "array";
    default:
      return "object";
  }
}

async function generate() {
  const contractNames = await listContractNames();
  for (const name of contractNames) {
    const contractRoot = path.join(CONTRACTS_DIR, name);
    const rustFiles = await listRustFiles(contractRoot);
    for (const filePath of rustFiles) {
      if (!filePath.endsWith("event_schema.rs")) continue;
      const text = await readFile(filePath, "utf8");
      const defs = findAllStructDefs(text);
      for (const def of defs) {
        const fields = parseStructBody(def.body);
        const schema = generateJsonSchema(def.name, fields);
        const schemaPath = path.join(
          OUT_SCHEMAS_DIR,
          `${toSnakeCase(def.name)}.schema.json`,
        );
        await writeFile(
          schemaPath,
          JSON.stringify(schema, null, 2) + "\n",
          "utf8",
        );
      }
    }
  }
}

await generate();
