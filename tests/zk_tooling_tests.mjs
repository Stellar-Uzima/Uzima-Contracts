#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";

const ROOT = process.cwd();
const TMP = path.join(ROOT, "tests", ".tmp-zk");

function runNode(script, args = []) {
  const proc = spawnSync(process.execPath, [script, ...args], {
    cwd: ROOT,
    encoding: "utf8",
  });
  return proc;
}

async function main() {
  await fs.mkdir(TMP, { recursive: true });

  const vkRun = runNode("scripts/zk/generate_vk.mjs", [
    "--out-dir",
    TMP,
    "--out",
    "vk.json",
    "--circuit",
    "uzima_access_v1",
  ]);
  assert.equal(vkRun.status, 0, vkRun.stderr);

  const commitmentsRun = runNode("scripts/zk/commitments.mjs", [
    "--out-dir",
    TMP,
    "--out",
    "commitments.json",
    "--requester",
    "requester-A",
    "--provider",
    "provider-X",
  ]);
  assert.equal(commitmentsRun.status, 0, commitmentsRun.stderr);

  const proofRun = runNode("scripts/zk/prove_access.mjs", [
    "--out-dir",
    TMP,
    "--out",
    "proof.json",
    "--record-id",
    "5",
    "--issuer",
    "issuer-Y",
    "--requester",
    "requester-A",
    "--provider",
    "provider-X",
    "--vk-version",
    "2",
    "--nullifier-seed",
    "seed-fixed-001",
  ]);
  assert.equal(proofRun.status, 0, proofRun.stderr);

  const verifyOk = runNode("scripts/zk/verify.mjs", [
    "--proof",
    path.join(TMP, "proof.json"),
  ]);
  assert.equal(verifyOk.status, 0, verifyOk.stderr);

  const parsed = JSON.parse(await fs.readFile(path.join(TMP, "proof.json"), "utf8"));
  parsed.attestation.proof_hash = "0xdeadbeef";
  await fs.writeFile(path.join(TMP, "proof_bad.json"), JSON.stringify(parsed, null, 2), "utf8");

  const verifyBad = runNode("scripts/zk/verify.mjs", [
    "--proof",
    path.join(TMP, "proof_bad.json"),
  ]);
  assert.equal(verifyBad.status, 2, verifyBad.stderr);

  process.stdout.write("zk_tooling_tests: ok\n");
}

main().catch((error) => {
  process.stderr.write(`${error.stack ?? String(error)}\n`);
  process.exit(1);
});

