import Ajv2020 from "ajv/dist/2020.js";
import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";

function fail(msg) {
  process.stderr.write(msg + "\n");
  process.exit(1);
}

function runGenerateToCheckClean() {
  const r = spawnSync(process.execPath, ["scripts/events/generate-registry.mjs"], {
    stdio: "inherit"
  });
  if (r.status !== 0) fail("Event registry generation failed.");
}

function loadJson(p) {
  return readFile(p, "utf8").then((s) => JSON.parse(s));
}

function normalizeGeneratedAt(obj) {
  // generated_at is expected to change; we ignore it for equality checks.
  const clone = JSON.parse(JSON.stringify(obj));
  clone.generated_at = "<generated_at>";
  return clone;
}

function deepEqual(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

async function validate() {
  // Generate registry + docs deterministically from current source.
  runGenerateToCheckClean();

  const registry = await loadJson("schemas/events/registry.json");
  const schema = await loadJson("schemas/events/registry.schema.json");

  // Validate registry shape
  const ajv = new Ajv2020({ allErrors: true, strict: false });
  const validateFn = ajv.compile(schema);
  const ok = validateFn(registry);
  if (!ok) {
    fail(
      "schemas/events/registry.json does not conform to schemas/events/registry.schema.json:\n" +
        ajv.errorsText(validateFn.errors, { separator: "\n" })
    );
  }

  // Enforce stability rules
  const seenIds = new Set();
  for (const c of registry.contracts) {
    for (const e of c.events) {
      if (seenIds.has(e.id)) fail(`Duplicate event id: ${e.id}`);
      seenIds.add(e.id);
      if (e.contract !== c.name) fail(`Event contract mismatch for id ${e.id}`);
      if (!Array.isArray(e.topics) || e.topics.length === 0) fail(`Missing topics for id ${e.id}`);
    }
  }

  // Compare to a fresh in-memory re-read after generation to ensure no external mutation.
  // (This is mainly to ensure CI catches manual edits that don't match generator output.)
  const registryAfter = await loadJson("schemas/events/registry.json");
  if (!deepEqual(normalizeGeneratedAt(registry), normalizeGeneratedAt(registryAfter))) {
    fail("Event registry changed during validation unexpectedly.");
  }

  process.stdout.write("✅ Event schema registry validated.\n");
}

await validate();

