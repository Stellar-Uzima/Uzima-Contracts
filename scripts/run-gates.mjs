#!/usr/bin/env node
/**
 * Run the repository gates in a defined order.
 *
 * The order lives in scripts/gates.json so that the make targets and the npm
 * scripts cannot drift apart. This runner only executes that manifest; it adds
 * preflight checks and reporting.
 *
 * Usage:
 *   node scripts/run-gates.mjs --tier fast     # no cargo build required
 *   node scripts/run-gates.mjs --tier full     # everything, incl. budgets
 *   node scripts/run-gates.mjs --list          # print the order, run nothing
 *   node scripts/run-gates.mjs --only budgets  # run a single gate
 *
 * Gates are executed in manifest order and the run stops at the first failure.
 */

import { spawnSync } from 'child_process';
import { existsSync, readdirSync, readFileSync } from 'fs';
import { dirname, join, resolve } from 'path';
import { fileURLToPath } from 'url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const MANIFEST = join(ROOT, 'scripts', 'gates.json');
const WASM_DIR = join(ROOT, 'target', 'wasm32-unknown-unknown', 'release');

// Tiers that do not need a release build, i.e. everything cheap enough to run
// on every save.
const FAST_TIERS = ['structure', 'schema', 'artifacts'];

const REMEDIATION = {
  node_modules: "node_modules is missing. Run 'npm ci'.",
  release_wasm:
    "No release .wasm in target/wasm32-unknown-unknown/release/. " +
    "Budgets measure real artifacts, so run 'make build-opt' first.",
  bash: 'bash was not found on PATH.',
  python3: 'python3 was not found on PATH.',
  cargo: "cargo was not found on PATH. Run 'make install-deps'.",
};

function fail(message) {
  console.error(`\n[gates] ${message}`);
  process.exit(1);
}

function loadManifest() {
  let manifest;
  try {
    manifest = JSON.parse(readFileSync(MANIFEST, 'utf8'));
  } catch (err) {
    fail(`could not read ${MANIFEST}: ${err.message}`);
  }

  const gates = manifest.gates || [];
  if (gates.length === 0) fail('manifest declares no gates');

  const known = new Set((manifest.tiers || []).map((t) => t.id));
  for (const gate of gates) {
    if (!gate.id) fail('every gate needs an id');
    // Exactly one entrypoint, so the runner never has to guess which wins.
    if (Boolean(gate.npm) === Boolean(gate.command)) {
      fail(`gate "${gate.id}" must declare exactly one of npm / command`);
    }
    if (!known.has(gate.tier)) {
      fail(`gate "${gate.id}" has unknown tier "${gate.tier}"`);
    }
  }
  return manifest;
}

function parseArgs(argv) {
  const opts = { tier: 'full', only: null, list: false };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--list') opts.list = true;
    else if (arg === '--tier') opts.tier = argv[++i];
    else if (arg === '--only') opts.only = argv[++i];
    else if (arg === '--tiers') opts.tier = argv[++i];
    else fail(`unknown argument "${arg}"`);
  }
  if (opts.tier !== 'fast' && opts.tier !== 'full') {
    fail(`--tier must be "fast" or "full", got "${opts.tier}"`);
  }
  return opts;
}

function preflight(requirements = []) {
  for (const requirement of requirements) {
    if (requirement === 'node_modules') {
      if (!existsSync(join(ROOT, 'node_modules'))) return 'node_modules';
    } else if (requirement === 'release_wasm') {
      const built =
        existsSync(WASM_DIR) &&
        readdirSync(WASM_DIR).some((f) => f.endsWith('.wasm'));
      if (!built) return 'release_wasm';
    } else if (which(requirement) === null) {
      return requirement;
    }
  }
  return null;
}

function which(binary) {
  // `command -v` is a shell builtin, so this needs a shell. The name comes
  // from the reviewed manifest, but validate it anyway rather than trusting it
  // into a command string.
  if (!/^[A-Za-z0-9_.-]+$/.test(binary)) return null;
  const probe = spawnSync(`command -v ${binary}`, { shell: true, encoding: 'utf8' });
  return probe.status === 0 ? probe.stdout.trim() : null;
}

function commandFor(gate) {
  return gate.npm ? `npm run --silent ${gate.npm}` : gate.command;
}

function selectGates(manifest, opts) {
  if (opts.only) {
    const gate = manifest.gates.find((g) => g.id === opts.only);
    if (!gate) {
      fail(
        `no gate "${opts.only}". Known gates: ${manifest.gates.map((g) => g.id).join(', ')}`
      );
    }
    return [gate];
  }
  if (opts.tier === 'fast') {
    return manifest.gates.filter((g) => FAST_TIERS.includes(g.tier));
  }
  return manifest.gates;
}

function describe(manifest, gates) {
  console.log('[gates] defined order:');
  let tier = null;
  for (const gate of gates) {
    if (gate.tier !== tier) {
      tier = gate.tier;
      const summary = (manifest.tiers.find((t) => t.id === tier) || {}).summary;
      console.log(`\n  ${tier}${summary ? ` - ${summary}` : ''}`);
    }
    console.log(`    ${gate.id.padEnd(10)} ${commandFor(gate)}`);
  }
  console.log('');
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const manifest = loadManifest();
  const gates = selectGates(manifest, opts);

  if (opts.list) {
    describe(manifest, gates);
    console.log(`[gates] ${gates.length} gate(s) in tier "${opts.tier}".`);
    return;
  }

  describe(manifest, gates);
  const started = Date.now();

  for (const gate of gates) {
    const missing = preflight(gate.requires);
    if (missing) {
      console.error(`\n[gates] ${gate.id}: ${REMEDIATION[missing]}`);
      console.error('[gates] nothing was skipped silently: fix the above and re-run.');
      process.exit(1);
    }

    const at = Date.now();
    process.stdout.write(`[gates] running ${gate.id}... `);
    const run = spawnSync(commandFor(gate), {
      cwd: ROOT,
      shell: true,
      stdio: 'inherit',
    });

    if (run.error) {
      console.log('failed');
      fail(`${gate.id} could not start: ${run.error.message}`);
    }
    if (run.status !== 0) {
      console.log(`FAILED (exit ${run.status})`);
      fail(
        `${gate.id} failed. Later gates were not run.\n` +
          `       Fix ${gate.id} and re-run, or inspect a single gate with:\n` +
          `       node scripts/run-gates.mjs --only ${gate.id}`
      );
    }
    console.log(`ok (${((Date.now() - at) / 1000).toFixed(1)}s)`);
  }

  console.log(
    `\n[gates] all ${gates.length} gate(s) passed in ${((Date.now() - started) / 1000).toFixed(1)}s.`
  );
}

main();
