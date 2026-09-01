import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import {
  scanContractEvents,
  scanStructSchemas,
  toPascalCase,
} from "./scan.mjs";

const REPO_ROOT = process.cwd();
const CONTRACTS_DIR = path.join(REPO_ROOT, "contracts");
const REGISTRY_PATH = path.join(
  REPO_ROOT,
  "schemas",
  "events",
  "event-schema-registry.json"
);
const DOCS_PATH = path.join(REPO_ROOT, "docs", "EVENTS.md");
const SCHEMAS_DIR = path.join(REPO_ROOT, "schemas", "events");

export function renderDocs(registryContracts, version, generatedAt) {
  const lines = [];
  lines.push("# Contract Events");
  lines.push("");
  lines.push(
    "This document is auto-generated from on-chain event emissions found in `contracts/**/src/**/*.rs`."
  );
  lines.push("");
  lines.push(`- Registry format version: \`${version}\``);
  lines.push(`- Generated at: \`${generatedAt}\``);
  lines.push("");

  for (const c of registryContracts) {
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

export async function generateArtifacts(options = {}) {
  const generatedAt = options.generatedAt || new Date().toISOString();

  // 1. Scan contract event emissions
  const { contracts, dynamicTopicEmissions } = await scanContractEvents(
    REPO_ROOT,
    CONTRACTS_DIR
  );

  if (dynamicTopicEmissions.length > 0) {
    const summary = dynamicTopicEmissions
      .slice(0, 20)
      .map((d) => `- ${d.contract}: ${d.file}:${d.line}`)
      .join("\n");
    throw new Error(
      [
        "Found event emissions with fully-dynamic topics that cannot be validated deterministically.",
        'Please refactor to include at least one stable string topic (e.g. symbol_short!("...") or "...") in the topics tuple.',
        "",
        "Examples:",
        summary,
        dynamicTopicEmissions.length > 20
          ? `\n(and ${dynamicTopicEmissions.length - 20} more...)`
          : "",
      ].join("\n")
    );
  }

  // 2. Load existing canonical event-schema-registry.json
  let existingRegistry = {};
  try {
    const raw = await readFile(REGISTRY_PATH, "utf8");
    existingRegistry = JSON.parse(raw);
  } catch {
    existingRegistry = {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      title: "Uzima Contract Event Schema Registry",
      description:
        "Central registry mapping every contract event to its typed schema, version, and validation rules",
      version: "2.0.0",
      events: {},
      validation_rules: {
        require_ledger_field: true,
        require_actor_field_for_mutations: true,
        require_additional_properties_false: true,
        max_event_data_bytes: 8192,
      },
    };
  }

  const registry = {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    title: "Uzima Contract Event Schema Registry",
    description:
      "Central registry mapping every contract event to its typed schema, version, and validation rules",
    version: existingRegistry.version || "2.0.0",
    generated_at: generatedAt,
    events: { ...existingRegistry.events },
    validation_rules: existingRegistry.validation_rules || {
      require_ledger_field: true,
      require_actor_field_for_mutations: true,
      require_additional_properties_false: true,
      max_event_data_bytes: 8192,
    },
  };

  // 3. Scan struct schemas from Rust contract types
  const structSchemas = await scanStructSchemas(CONTRACTS_DIR);

  // 4. Build individual event schema files
  const schemaFiles = new Map();

  // Add struct-generated schemas
  for (const [filename, schema] of structSchemas.entries()) {
    schemaFiles.set(filename, schema);
  }

  // Add/ensure schemas for every event in event-schema-registry.json
  for (const [key, eventDef] of Object.entries(registry.events)) {
    const eventName = eventDef.name;
    const filename = `${eventName}_event.schema.json`;
    const title = `${toPascalCase(eventName)}Event`;

    const eventSchema = {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      title,
      ...eventDef.schema,
    };
    schemaFiles.set(filename, eventSchema);
  }

  // 5. Generate docs/EVENTS.md
  const docs = renderDocs(contracts, "1.0.0", generatedAt);

  return {
    registry,
    docs,
    schemaFiles,
  };
}

export async function writeGeneratedArtifacts(artifacts) {
  await writeFile(
    REGISTRY_PATH,
    JSON.stringify(artifacts.registry, null, 2) + "\n",
    "utf8"
  );
  await writeFile(DOCS_PATH, artifacts.docs, "utf8");

  for (const [filename, schema] of artifacts.schemaFiles.entries()) {
    const filePath = path.join(SCHEMAS_DIR, filename);
    await writeFile(
      filePath,
      JSON.stringify(schema, null, 2) + "\n",
      "utf8"
    );
  }
}

// If executed directly:
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname)
) {
  const artifacts = await generateArtifacts();
  await writeGeneratedArtifacts(artifacts);
  process.stdout.write("✅ Event schema registry and schemas generated.\n");
}
