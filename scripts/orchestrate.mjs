#!/usr/bin/env node
// orchestrate.mjs — Typed deployment orchestration layer (Issue #1187)
//
// Replaces ad-hoc shell deployment scripts with a single, typed entry-point
// that reads the deployment manifest, validates state, and delegates to
// existing shell helpers for the actual soroban CLI calls.
//
// Usage:
//   node scripts/orchestrate.mjs plan    <network> [--output <file>]
//   node scripts/orchestrate.mjs deploy  <network> [--dry-run] [--group core|domain|all]
//   node scripts/orchestrate.mjs verify  <network>
//   node scripts/orchestrate.mjs status  <network>
//   node scripts/orchestrate.mjs rollback <network> <contract>

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { execSync } from "child_process";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT = join(__dirname, "..");

// ─── Types (JSDoc for runtime clarity) ─────────────────────────────────────

/**
 * @typedef {Object} NetworkConfig
 * @property {string} name
 * @property {string} passphrase
 * @property {string} rpc_url
 * @property {string} environment
 *
 * @typedef {Object} ContractEntry
 * @property {string} name
 * @property {string} tier
 * @property {string} wasm_path
 * @property {number} deploy_order
 * @property {Record<string, {contract_id: string|null, init_required: boolean}>} networks
 * @property {string[]} dependencies
 *
 * @typedef {Object} DeploymentManifest
 * @property {Record<string, NetworkConfig>} networks
 * @property {ContractEntry[]} contracts
 * @property {Record<string, {description: string, contracts: string[], parallel: boolean}>} deploy_groups
 */

// ─── Helpers ────────────────────────────────────────────────────────────────

function loadManifest() {
  const p = join(ROOT, "deployments", "deployment-manifest.json");
  return JSON.parse(readFileSync(p, "utf8"));
}

function loadNetworkManifest() {
  const p = join(ROOT, "deployments", "network-manifests.json");
  return JSON.parse(readFileSync(p, "utf8"));
}

function log(level, msg) {
  const ts = new Date().toISOString();
  const prefix = { INFO: "\x1b[34m[INFO]\x1b[0m", OK: "\x1b[32m[OK]\x1b[0m", WARN: "\x1b[33m[WARN]\x1b[0m", ERROR: "\x1b[31m[ERROR]\x1b[0m" };
  console.log(`${ts} ${prefix[level] || level} ${msg}`);
}

function run(cmd) {
  try {
    return execSync(cmd, { cwd: ROOT, encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] }).trim();
  } catch (e) {
    return null;
  }
}

function wasmPath(name) {
  return join(ROOT, "target", "wasm32-unknown-unknown", "release", `${name}.wasm`);
}

function wasmSize(name) {
  const p = wasmPath(name);
  if (!existsSync(p)) return null;
  try {
    const out = run(`wc -c < "${p}"`);
    return out ? parseInt(out, 10) : 0;
  } catch {
    return 0;
  }
}

// ─── Commands ───────────────────────────────────────────────────────────────

function cmdPlan(network, outputFile) {
  const manifest = loadManifest();
  const netCfg = manifest.networks[network];
  if (!netCfg) {
    log("ERROR", `Unknown network: ${network}. Available: ${Object.keys(manifest.networks).join(", ")}`);
    process.exit(1);
  }

  const contracts = [...manifest.contracts].sort((a, b) => a.deploy_order - b.deploy_order);
  const plan = {
    plan_version: "2.0.0",
    generated_at: new Date().toISOString(),
    network: { name: network, passphrase: netCfg.passphrase, rpc_url: netCfg.rpc_url },
    orchestration: "typed-layer",
    groups: {},
    contracts: [],
    summary: { total: 0, core: 0, domain: 0, wasm_available: 0, wasm_missing: 0 },
  };

  for (const c of contracts) {
    const netState = c.networks[network];
    if (!netState) continue;
    const size = wasmSize(c.name);
    const entry = {
      order: c.deploy_order,
      name: c.name,
      tier: c.tier,
      wasm_path: c.wasm_path,
      wasm_available: size !== null,
      wasm_size_bytes: size,
      init_required: netState.init_required,
      dependencies: c.dependencies,
      contract_id: netState.contract_id,
    };
    plan.contracts.push(entry);
    plan.summary.total++;
    plan.summary[c.tier] = (plan.summary[c.tier] || 0) + 1;
    if (size !== null) plan.summary.wasm_available++;
    else plan.summary.wasm_missing++;
  }

  // Group contracts by deploy_groups
  for (const [groupName, groupDef] of Object.entries(manifest.deploy_groups || {})) {
    plan.groups[groupName] = {
      description: groupDef.description,
      parallel: groupDef.parallel,
      contracts: groupDef.contracts.filter((n) => plan.contracts.some((c) => c.name === n)),
    };
  }

  const json = JSON.stringify(plan, null, 2);
  if (outputFile) {
    mkdirSync(dirname(outputFile), { recursive: true });
    writeFileSync(outputFile, json);
    log("OK", `Plan written to ${outputFile}`);
  } else {
    console.log(json);
  }
}

function cmdDeploy(network, { dryRun = false, group = "all" } = {}) {
  const manifest = loadManifest();
  const netCfg = manifest.networks[network];
  if (!netCfg) {
    log("ERROR", `Unknown network: ${network}`);
    process.exit(1);
  }

  // Safety gate: mainnet requires explicit --no-dry-run
  if (network === "mainnet" && !dryRun) {
    log("ERROR", "Mainnet deployments require --dry-run first. Re-run with --dry-run=false after review.");
    process.exit(1);
  }

  const groupDef = manifest.deploy_groups[group];
  let contractsToDeploy;
  if (group === "all") {
    contractsToDeploy = [...manifest.contracts].sort((a, b) => a.deploy_order - b.deploy_order);
  } else if (groupDef) {
    contractsToDeploy = manifest.contracts
      .filter((c) => groupDef.contracts.includes(c.name))
      .sort((a, b) => a.deploy_order - b.deploy_order);
  } else {
    log("ERROR", `Unknown group: ${group}. Available: ${Object.keys(manifest.deploy_groups).join(", ")}, all`);
    process.exit(1);
  }

  log("INFO", `Deploying ${contractsToDeploy.length} contract(s) to ${network} (dry_run=${dryRun}, group=${group})`);

  const results = [];
  for (const c of contractsToDeploy) {
    const netState = c.networks[network];
    if (!netState) {
      log("WARN", `Skipping ${c.name} — not configured for ${network}`);
      continue;
    }

    // Check dependencies are already deployed
    const unmetDeps = c.dependencies.filter((dep) => {
      const depEntry = manifest.contracts.find((mc) => mc.name === dep);
      return depEntry && !depEntry.networks[network]?.contract_id;
    });
    if (unmetDeps.length > 0) {
      log("ERROR", `${c.name}: unmet dependencies: ${unmetDeps.join(", ")}`);
      results.push({ name: c.name, status: "blocked", reason: `unmet dependencies: ${unmetDeps.join(", ")}` });
      continue;
    }

    // Check WASM exists
    if (!existsSync(wasmPath(c.name))) {
      log("WARN", `${c.name}: WASM not found, build required`);
      if (!dryRun) {
        log("INFO", `Building ${c.name}...`);
        const buildResult = run(`cargo build -p ${c.name} --target wasm32-unknown-unknown --release`);
        if (buildResult === null) {
          log("ERROR", `${c.name}: build failed`);
          results.push({ name: c.name, status: "build_failed" });
          continue;
        }
      }
    }

    if (dryRun) {
      log("OK", `${c.name}: ready (order=${c.deploy_order}, init=${netState.init_required}, deps=${c.dependencies.length})`);
      results.push({ name: c.name, status: "dry_run_ok" });
    } else {
      log("INFO", `Deploying ${c.name}...`);
      const deployScript = join(ROOT, "scripts", "deploy.sh");
      const result = run(`bash "${deployScript}" "${c.name}" "${network}"`);
      if (result === null) {
        log("ERROR", `${c.name}: deployment failed`);
        results.push({ name: c.name, status: "deploy_failed" });
        if (groupDef?.parallel !== true) {
          log("ERROR", "Sequential group — aborting remaining deployments");
          break;
        }
      } else {
        log("OK", `${c.name}: deployed`);
        results.push({ name: c.name, status: "deployed" });
      }
    }
  }

  // Summary
  const summary = { deployed: 0, dry_run_ok: 0, failed: 0, skipped: 0 };
  for (const r of results) {
    if (r.status === "deployed") summary.deployed++;
    else if (r.status === "dry_run_ok") summary.dry_run_ok++;
    else summary.failed++;
  }

  log("INFO", `Deployment summary: ${JSON.stringify(summary)}`);
  return results;
}

function cmdVerify(network) {
  const manifest = loadManifest();
  const netCfg = manifest.networks[network];
  if (!netCfg) {
    log("ERROR", `Unknown network: ${network}`);
    process.exit(1);
  }

  log("INFO", `Verifying deployments on ${network}...`);
  let healthy = 0;
  let unhealthy = 0;

  for (const c of manifest.contracts) {
    const netState = c.networks[network];
    if (!netState?.contract_id) continue;

    const verifyScript = join(ROOT, "scripts", "verify_deployment.sh");
    const result = run(`bash "${verifyScript}" "${netState.contract_id}" "${network}" default "${c.name}"`);
    if (result !== null) {
      log("OK", `${c.name}: healthy`);
      healthy++;
    } else {
      log("ERROR", `${c.name}: unhealthy`);
      unhealthy++;
    }
  }

  log("INFO", `Verification complete: ${healthy} healthy, ${unhealthy} unhealthy`);
  return { healthy, unhealthy };
}

function cmdStatus(network) {
  const manifest = loadNetworkManifest();
  const net = manifest.networks[network];
  if (!net) {
    log("ERROR", `Unknown network: ${network}`);
    process.exit(1);
  }

  log("INFO", `Deployment status for ${network}:`);
  const rows = [];
  for (const [name, contractId] of Object.entries(net.contract_ids || {})) {
    rows.push({ contract: name, contract_id: contractId || "(not deployed)" });
  }

  console.table(rows);
}

function cmdRollback(network, contractName) {
  log("INFO", `Rolling back ${contractName} on ${network}...`);
  const rollbackScript = join(ROOT, "scripts", "rollback_deployment.sh");
  const result = run(`bash "${rollbackScript}" "${contractName}" "${network}"`);
  if (result === null) {
    log("ERROR", "Rollback failed");
    process.exit(1);
  }
  log("OK", `Rollback complete for ${contractName}`);
}

// ─── CLI ────────────────────────────────────────────────────────────────────

function printUsage() {
  console.log(`
Usage: node scripts/orchestrate.mjs <command> <network> [options]

Commands:
  plan    <network> [--output <file>]          Generate typed deployment plan
  deploy  <network> [--dry-run] [--group G]   Deploy contracts (dry-run or live)
  verify  <network>                            Verify deployed contracts
  status  <network>                            Show deployment status
  rollback <network> <contract>                Rollback a contract deployment

Options:
  --output <file>   Write output to file (plan command)
  --dry-run         Simulate deployment without executing (deploy command)
  --group <name>    Deploy group: core, domain, or all (default: all)

Examples:
  node scripts/orchestrate.mjs plan testnet --output plan.json
  node scripts/orchestrate.mjs deploy testnet --dry-run
  node scripts/orchestrate.mjs deploy testnet --group core
  node scripts/orchestrate.mjs verify testnet
  node scripts/orchestrate.mjs status mainnet
  node scripts/orchestrate.mjs rollback testnet governor
`);
}

function main() {
  const args = process.argv.slice(2);
  if (args.length < 2) {
    printUsage();
    process.exit(1);
  }

  const [command, network, ...rest] = args;

  // Parse options
  const opts = {};
  for (let i = 0; i < rest.length; i++) {
    if (rest[i] === "--output" && rest[i + 1]) {
      opts.output = rest[++i];
    } else if (rest[i] === "--dry-run") {
      opts.dryRun = true;
    } else if (rest[i] === "--group" && rest[i + 1]) {
      opts.group = rest[++i];
    }
  }

  switch (command) {
    case "plan":
      cmdPlan(network, opts.output);
      break;
    case "deploy":
      cmdDeploy(network, opts);
      break;
    case "verify":
      cmdVerify(network);
      break;
    case "status":
      cmdStatus(network);
      break;
    case "rollback":
      if (rest.length === 0) {
        log("ERROR", "rollback requires a contract name");
        process.exit(1);
      }
      cmdRollback(network, rest[0]);
      break;
    default:
      log("ERROR", `Unknown command: ${command}`);
      printUsage();
      process.exit(1);
  }
}

main();
