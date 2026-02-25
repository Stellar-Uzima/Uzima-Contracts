#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { ensure32ByteHex, sha256Hex, toHex } from "./utils.mjs";

function arg(name, fallback = undefined) {
  const idx = process.argv.indexOf(name);
  if (idx === -1 || idx + 1 >= process.argv.length) return fallback;
  return process.argv[idx + 1];
}

async function main() {
  const outDir = arg("--out-dir", "scripts/zk/artifacts");
  const outFile = arg("--out", "vk.json");
  const circuit = arg("--circuit", "uzima_access_v1");
  const attestor = arg(
    "--attestor",
    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
  );
  const metadata = arg("--metadata", "trusted-attestor-hash-anchored");

  const vkRaw = crypto.randomBytes(128);
  const vkHash = toHex(sha256Hex(vkRaw));
  const circuitId = toHex(sha256Hex(circuit));
  const metadataHash = toHex(sha256Hex(metadata));

  const artifact = {
    schema: "uzima.zk.vk.v1",
    generated_at: new Date().toISOString(),
    circuit,
    vk_bytes_hex: `0x${vkRaw.toString("hex")}`,
    vk_hash: ensure32ByteHex(vkHash),
    circuit_id: ensure32ByteHex(circuitId),
    metadata_hash: ensure32ByteHex(metadataHash),
    attestor,
    vk_version: 1,
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

