#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import {
  computeAddressCommitment,
  computeCredentialCommitment,
  computeRecordCommitment,
} from "./utils.mjs";

function arg(name, fallback = undefined) {
  const idx = process.argv.indexOf(name);
  if (idx === -1 || idx + 1 >= process.argv.length) return fallback;
  return process.argv[idx + 1];
}

async function loadJson(filePath, fallback) {
  if (!filePath) return fallback;
  const raw = await fs.readFile(filePath, "utf8");
  return JSON.parse(raw);
}

async function main() {
  const outDir = arg("--out-dir", "scripts/zk/artifacts");
  const outFile = arg("--out", "commitments.json");
  const recordPath = arg("--record");
  const credentialPath = arg("--credential");
  const requester = arg("--requester", "requester-default");
  const provider = arg("--provider", "provider-default");

  const record = await loadJson(recordPath, {
    record_id: 1,
    patient_ref: "patient-1",
    provider_ref: provider,
    timestamp: 1700000000,
    category: "Modern",
    data_ref: "ipfs://record",
  });
  const credential = await loadJson(credentialPath, {
    issuer: provider,
    holder: requester,
    claims: ["access:medical-records"],
    issued_at: 1700000000,
    expires_at: 1710000000,
  });

  const artifact = {
    schema: "uzima.zk.commitments.v1",
    generated_at: new Date().toISOString(),
    record_commitment: computeRecordCommitment(record),
    credential_commitment: computeCredentialCommitment(credential),
    requester_commitment: computeAddressCommitment(requester),
    provider_commitment: computeAddressCommitment(provider),
  };

  await fs.mkdir(outDir, { recursive: true });
  const fullPath = path.join(outDir, outFile);
  await fs.writeFile(fullPath, `${JSON.stringify(artifact, null, 2)}\n`, "utf8");
  process.stdout.write(`${fullPath}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error.stack ?? String(error)}\n`);
  process.exit(1);
});

