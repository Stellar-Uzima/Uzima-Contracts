// Validates the M1 extractor's NDJSON/JSONL output against the canonical
// ContractTrace schema, the per-event envelopes, and the authoritative event
// registry. Mirrors the pattern established by scripts/events/validate.mjs.
// Exit code 0 on success, 1 on the first violated record.
//
// Usage:
//   node scripts/trace/validate-trace-ndjson.mjs --file <ndjson>
//   <extractor output> | node scripts/trace/validate-trace-ndjson.mjs
//
// Options:
//   --file      NDJSON/JSONL file to validate (default: read stdin)
//   --schema    trace schema path        (default schemas/trace/contract_trace.schema.json)
//   --registry  event registry path      (default schemas/events/event-schema-registry.json)
//   --envelope  event envelope path      (default schemas/events/event_envelope.schema.json)

import Ajv2020 from "ajv/dist/2020.js";
import Ajv from "ajv";
import { readFile } from "node:fs/promises";

function fail(msg) {
  process.stderr.write(msg + "\n");
  process.exit(1);
}

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  return index === -1 ? fallback : process.argv[index + 1];
}

function loadJson(p) {
  return readFile(p, "utf8").then((s) => JSON.parse(s));
}

function ajv() {
  return new Ajv2020({ allErrors: true, strict: false });
}

function ajvDraft7() {
  return new Ajv({ allErrors: true, strict: false });
}

async function readInputLines(file) {
  if (file) {
    const text = await readFile(file, "utf8");
    return text.split("\n");
  }
  // Read piped NDJSON from the extractor (stdin).
  return new Promise((resolve, reject) => {
    let text = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      text += chunk;
    });
    process.stdin.on("end", () => resolve(text.split("\n")));
    process.stdin.on("error", reject);
  });
}

async function validate() {
  const file = arg("--file", undefined);
  const schemaPath = arg("--schema", "schemas/trace/contract_trace.schema.json");
  const registryPath = arg("--registry", "schemas/events/event-schema-registry.json");
  const envelopePath = arg("--envelope", "schemas/events/event_envelope.schema.json");

  const schema = await loadJson(schemaPath);
  const registry = await loadJson(registryPath);
  const envelope = await loadJson(envelopePath);

  function compileSchema(s) {
    if (s?.$schema && s.$schema.includes("draft-07")) {
      return ajvDraft7().compile(s);
    }
    return ajv().compile(s);
  }

  const validateTrace = compileSchema(schema);
  const validateEnvelope = compileSchema(envelope);

  const registryEntries = registry.events ?? {};
  const bodyValidators = new Map();
  for (const [key, entry] of Object.entries(registryEntries)) {
    if (entry?.schema) {
      try {
        bodyValidators.set(key, compileSchema(entry.schema));
      } catch (error) {
        fail(`Registry event '${key}' has an invalid body schema: ${error.message}`);
      }
    }
  }

  const lines = await readInputLines(file);
  const records = lines.filter((line) => line.trim().length > 0);

  for (let lineNo = 0; lineNo < records.length; lineNo++) {
    const line = records[lineNo];
    let record;
    try {
      record = JSON.parse(line);
    } catch (error) {
      fail(`Invalid JSON at NDJSON line ${lineNo + 1}: ${error.message}`);
    }

    if (!validateTrace(record)) {
      fail(
        `ContractTrace schema violation at NDJSON line ${lineNo + 1}:\n` +
          validateTrace.errorsText(validateTrace.errors, { separator: "\n" })
      );
    }

    if (!Array.isArray(record.events)) {
      fail(`ContractTrace at line ${lineNo + 1} has no events array.`);
    }

    for (const event of record.events) {
      if (!validateEnvelope(event)) {
        fail(
          `Event envelope violation at line ${lineNo + 1} (event '${event.name}'):\n` +
            validateEnvelope.errorsText(validateEnvelope.errors, { separator: "\n" })
        );
      }

      const registryKey = `${record.contract_name}.${event.name}`;
      const entry = registryEntries[registryKey];
      if (!entry) {
        fail(
          `Event '${event.name}' of contract '${record.contract_name}' is absent from ` +
            `the authoritative registry (${registryPath}). A new event topic must be added ` +
            `to the registry before it can appear in a trace.`
        );
      }

      for (const topic of event.topics ?? []) {
        if (!entry.topics.includes(topic)) {
          fail(
            `Topic '${topic}' of event '${event.name}' (line ${lineNo + 1}) is not in the ` +
              `registered topics [${entry.topics.join(", ")}] for that event.`
          );
        }
      }

      const validateBody = bodyValidators.get(registryKey);
      if (validateBody && !validateBody(event.body)) {
        fail(
          `Event body violation at line ${lineNo + 1} (event '${event.name}'):\n` +
            validateBody.errorsText(validateBody.errors, { separator: "\n" })
        );
      }
    }
  }

  process.stdout.write(`✅ ${records.length} trace record(s) validated.\n`);
}

await validate();