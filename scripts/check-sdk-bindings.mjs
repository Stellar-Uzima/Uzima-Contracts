#!/usr/bin/env node

/**
 * Detects new public contract interfaces that have no SDK binding coverage.
 *
 * schemas/interface-registry/registry.json is the source of truth for the
 * public interfaces that generate-sdk-types.mjs turns into TypeScript, Python,
 * Kotlin and Swift client bindings. A new `#[contractimpl] pub fn` must be
 * added to that registry (and the bindings regenerated) in the same change.
 * Existing functions are the baseline: only functions added relative to the
 * base ref are checked.
 *
 * Usage:
 *   node scripts/check-sdk-bindings.mjs [--base <git-ref>]
 *
 * The base ref defaults to $SDK_BINDINGS_BASE, otherwise HEAD (so local runs
 * inspect uncommitted work). CI sets the base to the PR/push base commit.
 */

import fs from 'fs';
import path from 'path';
import { execFileSync } from 'child_process';
import { fileURLToPath } from 'url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REGISTRY_PATH = path.join(ROOT, 'schemas/interface-registry/registry.json');

const args = process.argv.slice(2);
const baseIndex = args.indexOf('--base');
const explicitBase = baseIndex !== -1 ? args[baseIndex + 1] : process.env.SDK_BINDINGS_BASE;

function loadRegistry() {
  try {
    return JSON.parse(fs.readFileSync(REGISTRY_PATH, 'utf8')).contracts || {};
  } catch (error) {
    throw new Error(`Unable to read ${REGISTRY_PATH}: ${error.message}`);
  }
}

function refExists(ref) {
  try {
    execFileSync('git', ['rev-parse', '--verify', '--quiet', ref], {
      cwd: ROOT,
      stdio: 'ignore',
    });
    return true;
  } catch {
    return false;
  }
}

function addedPublicFunctions(base) {
  const output = execFileSync('git', [
    'diff',
    '-U0',
    '--no-color',
    '--diff-filter=AM',
    base,
    '--',
    'contracts/*/src/lib.rs',
  ], { cwd: ROOT, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });

  const added = [];
  let file = null;
  let line = 0;
  for (const raw of output.split('\n')) {
    if (raw.startsWith('+++ b/')) {
      file = raw.slice(6).trim();
      continue;
    }
    const hunk = raw.match(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
    if (hunk) {
      line = Number(hunk[1]);
      continue;
    }
    if (!raw.startsWith('+') || raw.startsWith('+++') || !file) continue;
    const text = raw.slice(1);
    const match = text.match(/\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/);
    if (match) added.push({ file, line, name: match[1] });
    line += 1;
  }
  return added;
}

function contractImplLines(content) {
  const lines = content.split('\n');
  const ranges = [];
  for (let i = 0; i < lines.length; i++) {
    if (!lines[i].includes('#[contractimpl]')) continue;
    let start = -1;
    for (let j = i; j < lines.length; j++) {
      if (lines[j].includes('impl ') && lines[j].includes('{')) {
        start = j;
        break;
      }
    }
    if (start === -1) {
      ranges.push([i + 1, lines.length]);
      continue;
    }
    let depth = 0;
    let end = lines.length;
    for (let j = start; j < lines.length; j++) {
      for (const char of lines[j]) {
        if (char === '{') depth += 1;
        if (char === '}') depth -= 1;
      }
      if (depth === 0 && j > start) {
        end = j + 1;
        break;
      }
    }
    ranges.push([start + 1, end]);
  }
  return ranges;
}

function main() {
  const registry = loadRegistry();
  const BASE = explicitBase || 'HEAD';

  if (!refExists(BASE)) {
    console.error(`[sdk-bindings] base ref not found: ${BASE}`);
    process.exit(1);
  }

  const added = addedPublicFunctions(BASE).filter((fn) =>
    fn.name !== 'migrate' && fn.name !== 'verify_integrity' && !fn.name.startsWith('_'),
  );
  const gaps = [];
  const rangesByFile = new Map();

  for (const fn of added) {
    const contract = fn.file.startsWith('contracts/')
      ? fn.file.split('/')[1]
      : null;
    if (!contract) continue;

    if (!rangesByFile.has(fn.file)) {
      const fullPath = path.join(ROOT, fn.file);
      rangesByFile.set(
        fn.file,
        fs.existsSync(fullPath) ? contractImplLines(fs.readFileSync(fullPath, 'utf8')) : [],
      );
    }
    const inContractImpl = rangesByFile.get(fn.file)
      .some(([start, end]) => fn.line >= start && fn.line <= end);
    if (!inContractImpl) continue;

    const interfaces = registry[contract]?.interfaces || {};
    if (!registry[contract]) {
      gaps.push(`${contract}: new public contract has no registry entry or SDK bindings`);
    } else if (!interfaces[fn.name]) {
      gaps.push(`${contract}.${fn.name}: new public interface has no SDK binding`);
    }
  }

  if (gaps.length === 0) {
    console.log('[sdk-bindings] no unbound public interfaces added.');
    return;
  }

  console.error(`[sdk-bindings] ${gaps.length} new public interface(s) without bindings:`);
  for (const gap of [...new Set(gaps)].sort()) {
    console.error(`  - ${gap}`);
  }
  console.error('\nAdd the interface to schemas/interface-registry/registry.json and run');
  console.error('`npm run sdk:generate` to update the TypeScript, Python, Kotlin and Swift bindings.');
  process.exit(1);
}

main();
