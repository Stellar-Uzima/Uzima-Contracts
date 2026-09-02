import {
  generateArtifacts,
  writeGeneratedArtifacts,
} from "./generate-registry.mjs";

/**
 * Generates JSON schemas for all the event types in the codebase
 * using the unified event generation pipeline.
 */
async function generate() {
  const artifacts = await generateArtifacts();
  await writeGeneratedArtifacts(artifacts);
  process.stdout.write("✅ Event schemas generated.\n");
}

await generate();
