import Ajv2020 from "ajv/dist/2020.js";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { generateArtifacts } from "./generate-registry.mjs";

const REPO_ROOT = process.cwd();
const REGISTRY_PATH = path.join(
  REPO_ROOT,
  "schemas",
  "events",
  "event-schema-registry.json"
);
const DOCS_PATH = path.join(REPO_ROOT, "docs", "EVENTS.md");
const SCHEMAS_DIR = path.join(REPO_ROOT, "schemas", "events");

function fail(msg) {
  process.stderr.write(msg + "\n");
  process.exit(1);
}

function normalizeGeneratedAt(obj) {
  const clone = JSON.parse(JSON.stringify(obj));
  clone.generated_at = "<generated_at>";
  return clone;
}

function normalizeDocs(docsText) {
  return docsText.replace(/- Generated at: `[^`]+`/, "- Generated at: `<generated_at>`");
}

function deepEqual(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

async function validate() {
  const ajv = new Ajv2020({ allErrors: true, strict: false });

  // 1. Regenerate in memory without touching disk
  let artifacts;
  try {
    artifacts = await generateArtifacts();
  } catch (err) {
    fail(`Event generation failed during validation:\n${err.message}`);
  }

  // 2. Diff against committed canonical event-schema-registry.json
  let committedRegistry;
  try {
    const raw = await readFile(REGISTRY_PATH, "utf8");
    committedRegistry = JSON.parse(raw);
  } catch (err) {
    fail(
      `Failed to read canonical registry at ${REGISTRY_PATH}: ${err.message}\n` +
        "Run 'npm run events:generate' to generate it."
    );
  }

  const normCommitted = normalizeGeneratedAt(committedRegistry);
  const normGenerated = normalizeGeneratedAt(artifacts.registry);

  if (!deepEqual(normCommitted, normGenerated)) {
    fail(
      `❌ Drift detected in ${REGISTRY_PATH}: committed registry does not match source contracts.\n` +
        "Run 'npm run events:generate' to synchronize."
    );
  }

  // 3. Diff against committed docs/EVENTS.md
  let committedDocs;
  try {
    committedDocs = await readFile(DOCS_PATH, "utf8");
  } catch (err) {
    fail(
      `Failed to read docs at ${DOCS_PATH}: ${err.message}\n` +
        "Run 'npm run events:generate' to generate it."
    );
  }

  if (normalizeDocs(committedDocs) !== normalizeDocs(artifacts.docs)) {
    fail(
      `❌ Drift detected in ${DOCS_PATH}: committed documentation does not match source contracts.\n` +
        "Run 'npm run events:generate' to synchronize."
    );
  }

  // 4. Validate schema files exist and match
  for (const [filename, expectedSchema] of artifacts.schemaFiles.entries()) {
    const filePath = path.join(SCHEMAS_DIR, filename);
    let committedSchema;
    try {
      const raw = await readFile(filePath, "utf8");
      committedSchema = JSON.parse(raw);
    } catch (err) {
      fail(
        `❌ Missing or unreadable schema file ${filePath}: ${err.message}\n` +
          "Run 'npm run events:generate' to synchronize."
      );
    }

    if (!deepEqual(committedSchema, expectedSchema)) {
      fail(
        `❌ Drift detected in ${filePath}: committed schema does not match registry definition.\n` +
          "Run 'npm run events:generate' to synchronize."
      );
    }
  }

  // 5. Ajv Validation and Stability checks on the canonical registry
  const registry = committedRegistry;
  if (!registry.version) fail("Registry missing 'version' field.");
  if (!registry.events || typeof registry.events !== "object") {
    fail("Registry missing 'events' object.");
  }

  const rules = registry.validation_rules || {};

  const seenKeys = new Set();
  for (const [key, entry] of Object.entries(registry.events)) {
    if (seenKeys.has(key)) fail(`Duplicate event key in registry: ${key}`);
    seenKeys.add(key);

    const expectedKey = `${entry.contract}.${entry.name}`;
    if (key !== expectedKey) {
      fail(
        `Registry key mismatch: expected '${expectedKey}', found '${key}'`
      );
    }

    if (!entry.contract || typeof entry.contract !== "string") {
      fail(`Invalid or missing contract for event '${key}'`);
    }
    if (!entry.name || typeof entry.name !== "string") {
      fail(`Invalid or missing name for event '${key}'`);
    }
    if (!Array.isArray(entry.topics) || entry.topics.length === 0) {
      fail(`Event '${key}' must declare at least one topic.`);
    }

    if (!entry.schema || typeof entry.schema !== "object") {
      fail(`Event '${key}' is missing a schema definition.`);
    }

    // Compile schema with Ajv2020 to verify standard validity
    try {
      ajv.compile(entry.schema);
    } catch (err) {
      fail(`Invalid JSON Schema for event '${key}': ${err.message}`);
    }

    // Check validation rules
    if (rules.require_additional_properties_false && entry.schema.additionalProperties !== false) {
      fail(`Event '${key}' schema must specify "additionalProperties": false`);
    }

    if (rules.require_ledger_field) {
      if (!entry.schema.properties || !entry.schema.properties.ledger) {
        fail(`Event '${key}' schema must include a 'ledger' field.`);
      }
    }
  }

  // 6. Validate all *.schema.json files in schemas/events can be compiled by Ajv
  const entries = await readdir(SCHEMAS_DIR);
  for (const file of entries) {
    if (file.endsWith(".schema.json") && file !== "event-schema-registry.json") {
      const p = path.join(SCHEMAS_DIR, file);
      const content = JSON.parse(await readFile(p, "utf8"));
      try {
        ajv.compile(content);
      } catch (err) {
        fail(`Schema file '${file}' is not a valid JSON schema: ${err.message}`);
      }
    }
  }

  process.stdout.write("✅ Event schema registry and event schemas validated.\n");
}

await validate();
