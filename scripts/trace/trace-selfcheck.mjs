// Self-check for the ContractTrace NDJSON validator (issue #1515).
// Runs the validator over the checked-in samples and asserts the expected exit
// codes: the valid extractor sample is accepted, and each deliberately broken
// sample is rejected. Exit 0 when every expectation holds, 1 otherwise.
//
// Usage: node scripts/trace/trace-selfcheck.mjs

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const VALIDATOR = "scripts/trace/validate-trace-ndjson.mjs";
const SAMPLES = "schemas/trace/samples";

function run(file) {
  return spawnSync(process.execPath, [VALIDATOR, "--file", file], {
    encoding: "utf8",
    cwd: ROOT,
  });
}

const cases = [
  {
    file: `${SAMPLES}/valid_extractor.ndjson`,
    expectOk: true,
    why: "a valid extractor record must validate",
  },
  {
    file: `${SAMPLES}/invalid_traceid.ndjson`,
    expectOk: false,
    why: "a non-hex trace_id must fail",
  },
  {
    file: `${SAMPLES}/invalid_topic.ndjson`,
    expectOk: false,
    why: "an event topic absent from the registry must fail",
  },
  {
    file: `${SAMPLES}/invalid_envelope.ndjson`,
    expectOk: false,
    why: "an event missing its envelope body must fail",
  },
];

let failed = 0;
for (const { file, expectOk, why } of cases) {
  const result = run(file);
  const ok = expectOk ? result.status === 0 : result.status !== 0;
  if (!ok) {
    failed += 1;
    process.stderr.write(
      `✗ ${file} (expected ${expectOk ? "pass" : "reject"}): ${why}\n` +
        (result.stdout || result.stderr)
    );
  } else {
    process.stdout.write(`✓ ${file}: ${expectOk ? "accepted" : "rejected"}\n`);
  }
}

if (failed > 0) {
  process.stderr.write(`${failed} trace self-check case(s) failed.\n`);
  process.exit(1);
}
process.stdout.write("✅ Trace schema self-check passed.\n");