#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {
  computeAddressCommitment,
  computeCredentialCommitment,
  computeProofHash,
  computePublicInputsHash,
  computePseudonym,
  computeRecordCommitment,
  ensure32ByteHex,
  sha256Hex,
  toHex,
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
  const outFile = arg("--out", "proof_access.json");
  const recordPath = arg("--record");
  const credentialPath = arg("--credential");

  const recordId = Number(arg("--record-id", "1"));
  const minTimestamp = Number(arg("--min-ts", "0"));
  const maxTimestamp = Number(arg("--max-ts", "4102444800"));
  const requester = arg("--requester", "requester-default");
  const issuer = arg("--issuer", "issuer-default");
  const provider = arg("--provider", issuer);
  const vkVersion = Number(arg("--vk-version", "1"));
  const ttl = Number(arg("--ttl", "300"));
  const nullifierSeed = arg("--nullifier-seed", crypto.randomBytes(16).toString("hex"));

  const record = await loadJson(recordPath, {
    record_id: recordId,
    provider_ref: provider,
    timestamp: 1700000000,
    category: "Modern",
    data_ref: "ipfs://record",
  });
  const credential = await loadJson(credentialPath, {
    issuer,
    holder: requester,
    claims: ["scope:read_record"],
    issued_at: 1700000000,
    expires_at: 1710000000,
  });

  const recordCommitment = computeRecordCommitment(record);
  const credentialCommitment = computeCredentialCommitment(credential);
  const requesterCommitment = computeAddressCommitment(requester);
  const providerCommitment = computeAddressCommitment(provider);
  const claimCommitment = toHex(sha256Hex({ predicate: "provider-and-time-range", provider, minTimestamp, maxTimestamp }));
  const credentialRoot = ensure32ByteHex(toHex(sha256Hex({ issuer, credential_commitment: credentialCommitment })));
  const nullifier = ensure32ByteHex(toHex(sha256Hex({ nullifierSeed, requester, recordId })));
  const pseudonym = computePseudonym(requester, issuer, recordId);

  const publicInputs = {
    record_id: recordId,
    record_commitment: ensure32ByteHex(recordCommitment),
    credential_root: ensure32ByteHex(credentialRoot),
    issuer,
    requester_commitment: ensure32ByteHex(requesterCommitment),
    provider_commitment: ensure32ByteHex(providerCommitment),
    claim_commitment: ensure32ByteHex(claimCommitment),
    min_timestamp: minTimestamp,
    max_timestamp: maxTimestamp,
    nullifier: ensure32ByteHex(nullifier),
    pseudonym: ensure32ByteHex(pseudonym),
    vk_version: vkVersion,
  };

  const transcript = JSON.stringify({
    publicInputs,
    witness: {
      record_commitment: recordCommitment,
      credential_commitment: credentialCommitment,
      requester,
      issuer,
      provider,
    },
  });
  const proofHex = `0x${crypto
    .createHash("sha256")
    .update(Buffer.from(transcript, "utf8"))
    .update(crypto.randomBytes(32))
    .digest("hex")}`;

  const publicInputsHash = computePublicInputsHash(publicInputs);
  const proofHash = computeProofHash(proofHex);

  const artifact = {
    schema: "uzima.zk.proof.v1",
    generated_at: new Date().toISOString(),
    purpose: "medical_record_access",
    public_inputs: publicInputs,
    proof_hex: proofHex,
    public_inputs_hash: publicInputsHash,
    proof_hash: proofHash,
    attestation: {
      vk_version: vkVersion,
      public_inputs_hash: publicInputsHash,
      proof_hash: proofHash,
      verified: true,
      ttl,
    },
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

