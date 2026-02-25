#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { computeProofHash, computePublicInputsHash } from "./utils.mjs";

function arg(name, fallback = undefined) {
  const idx = process.argv.indexOf(name);
  if (idx === -1 || idx + 1 >= process.argv.length) return fallback;
  return process.argv[idx + 1];
}

async function loadJson(filePath) {
  const raw = await fs.readFile(filePath, "utf8");
  return JSON.parse(raw);
}

async function main() {
  const proofPath = arg("--proof");
  const attestationPath = arg("--attestation");

  if (!proofPath) {
    throw new Error("Missing --proof <artifact.json>");
  }

  const proofArtifact = await loadJson(proofPath);
  const attestation =
    attestationPath == null
      ? proofArtifact.attestation
      : await loadJson(attestationPath);

  if (!proofArtifact.public_inputs || !proofArtifact.proof_hex) {
    throw new Error("Malformed proof artifact");
  }
  if (!attestation) {
    throw new Error("Missing attestation");
  }

  const computedPublicInputsHash = computePublicInputsHash(proofArtifact.public_inputs);
  const computedProofHash = computeProofHash(proofArtifact.proof_hex);

  const verdict =
    computedPublicInputsHash === attestation.public_inputs_hash &&
    computedProofHash === attestation.proof_hash &&
    attestation.verified === true;

  const output = {
    schema: "uzima.zk.verify.result.v1",
    proof_file: path.resolve(proofPath),
    valid: verdict,
    computed_public_inputs_hash: computedPublicInputsHash,
    computed_proof_hash: computedProofHash,
    expected_public_inputs_hash: attestation.public_inputs_hash,
    expected_proof_hash: attestation.proof_hash,
  };

  process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
  process.exit(verdict ? 0 : 2);
}

main().catch((error) => {
  process.stderr.write(`${error.stack ?? String(error)}\n`);
  process.exit(1);
});

