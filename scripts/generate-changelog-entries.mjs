#!/usr/bin/env node
/**
 * generate-changelog-entries.mjs
 *
 * Generates changelog entries from contract ABI and interface registry changes.
 *
 * Compares the current `schemas/interface-registry/registry.json` against the
 * last committed version and produces structured entries for CHANGELOG.md.
 *
 * Usage:
 *   node scripts/generate-changelog-entries.mjs                  # output to stdout
 *   node scripts/generate-changelog-entries.mjs --append         # append to CHANGELOG.md
 *   node scripts/generate-changelog-entries.mjs --format json    # machine-readable output
 */

import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.resolve(__dirname, '..');

const REGISTRY_PATH = 'schemas/interface-registry/registry.json';
const CHANGELOG_PATH = 'CHANGELOG.md';

// ── Helpers ──────────────────────────────────────────────────────────────────

function loadRegistry(filePath) {
  const fullPath = path.join(PROJECT_ROOT, filePath);
  if (!fs.existsSync(fullPath)) return null;
  return JSON.parse(fs.readFileSync(fullPath, 'utf8'));
}

function getCommittedRegistry() {
  try {
    const content = execSync(`git show HEAD:${REGISTRY_PATH}`, {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    return JSON.parse(content);
  } catch {
    return null;
  }
}

function getGitLog(since = null) {
  try {
    const range = since ? `${since}..HEAD` : 'HEAD';
    const log = execSync(`git log ${range} --pretty=format:"%h|%s|%an" --no-merges`, {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
    });
    return log
      .split('\n')
      .filter(Boolean)
      .map((line) => {
        const [hash, subject, author] = line.replace(/^"|"$/g, '').split('|');
        return { hash, subject, author };
      });
  } catch {
    return [];
  }
}

// ── Diff detection ───────────────────────────────────────────────────────────

function diffRegistry(oldReg, newReg) {
  const changes = {
    addedContracts: [],
    removedContracts: [],
    addedFunctions: [],
    removedFunctions: [],
    changedFunctions: [],
  };

  const oldContracts = oldReg?.contracts || {};
  const newContracts = newReg?.contracts || {};

  const oldNames = new Set(Object.keys(oldContracts));
  const newNames = new Set(Object.keys(newContracts));

  // Added contracts
  for (const name of newNames) {
    if (!oldNames.has(name)) {
      const fnCount = Object.keys(newContracts[name].interfaces || {}).length;
      changes.addedContracts.push({ name, version: newContracts[name].version, functionCount: fnCount });
    }
  }

  // Removed contracts
  for (const name of oldNames) {
    if (!newNames.has(name)) {
      changes.removedContracts.push({ name, version: oldContracts[name].version });
    }
  }

  // Contract-level changes for contracts present in both
  for (const name of newNames) {
    if (!oldNames.has(name)) continue;

    const oldFns = oldContracts[name].interfaces || {};
    const newFns = newContracts[name].interfaces || {};

    const oldFnNames = new Set(Object.keys(oldFns));
    const newFnNames = new Set(Object.keys(newFns));

    // Added functions
    for (const fn of newFnNames) {
      if (!oldFnNames.has(fn)) {
        changes.addedFunctions.push({
          contract: name,
          function: fn,
          args: newFns[fn].args?.map((a) => `${a.name}: ${a.type}`).join(', ') || '',
          returns: newFns[fn].returns || 'void',
        });
      }
    }

    // Removed functions
    for (const fn of oldFnNames) {
      if (!newFnNames.has(fn)) {
        changes.removedFunctions.push({
          contract: name,
          function: fn,
          args: oldFns[fn].args?.map((a) => `${a.name}: ${a.type}`).join(', ') || '',
          returns: oldFns[fn].returns || 'void',
        });
      }
    }

    // Changed functions (args or returns changed)
    for (const fn of newFnNames) {
      if (!oldFnNames.has(fn)) continue;
      const oldFn = oldFns[fn];
      const newFn = newFns[fn];

      const oldArgSig = (oldFn.args || []).map((a) => `${a.name}:${a.type}`).join(',');
      const newArgSig = (newFn.args || []).map((a) => `${a.name}:${a.type}`).join(',');

      if (oldArgSig !== newArgSig || (oldFn.returns || 'void') !== (newFn.returns || 'void')) {
        changes.changedFunctions.push({
          contract: name,
          function: fn,
          oldArgs: oldFn.args?.map((a) => `${a.name}: ${a.type}`).join(', ') || '',
          newArgs: newFn.args?.map((a) => `${a.name}: ${a.type}`).join(', ') || '',
          oldReturns: oldFn.returns || 'void',
          newReturns: newFn.returns || 'void',
        });
      }
    }
  }

  return changes;
}

// ── Changelog generation ─────────────────────────────────────────────────────

function generateChangelogEntries(changes, commits) {
  const lines = [];
  const date = new Date().toISOString().split('T')[0];

  const hasBreaking =
    changes.removedContracts.length > 0 ||
    changes.removedFunctions.length > 0 ||
    changes.changedFunctions.length > 0;

  const hasNonBreaking = changes.addedContracts.length > 0 || changes.addedFunctions.length > 0;

  // Security-related commits
  const securityCommits = commits.filter((c) =>
    /security|cve|vulnerability|auth|encrypt/i.test(c.subject),
  );

  if (!hasBreaking && !hasNonBreaking && securityCommits.length === 0) {
    return null;
  }

  // Added
  if (hasNonBreaking) {
    lines.push('### Added');
    lines.push('');

    for (const c of changes.addedContracts) {
      lines.push(
        `- **New contract**: \`${c.name}\` (v${c.version}) — ${c.functionCount} public functions`,
      );
    }

    for (const c of changes.addedFunctions) {
      lines.push(
        `- \`${c.contract}.${c.function}\`(${c.args}) -> ${c.returns}`,
      );
    }
    lines.push('');
  }

  // Changed
  if (changes.changedFunctions.length > 0) {
    lines.push('### Changed');
    lines.push('');

    for (const c of changes.changedFunctions) {
      lines.push(
        `- \`${c.contract}.${c.function}\` — args changed from \`(${c.oldArgs})\` to \`(${c.newArgs})\`, returns \`${c.newReturns}\` (was \`${c.oldReturns}\`)`,
      );
    }
    lines.push('');
  }

  // Removed (breaking)
  if (hasBreaking) {
    lines.push('### Removed');
    lines.push('');

    for (const c of changes.removedContracts) {
      lines.push(`- **BREAKING**: Removed contract \`${c.name}\` (was v${c.version})`);
    }

    for (const c of changes.removedFunctions) {
      lines.push(
        `- **BREAKING**: Removed \`${c.contract}.${c.function}\`(${c.args}) -> ${c.returns}`,
      );
    }
    lines.push('');
  }

  // Security
  if (securityCommits.length > 0) {
    lines.push('### Security');
    lines.push('');

    for (const c of securityCommits) {
      lines.push(`- \`${c.hash}\` ${c.subject}`);
    }
    lines.push('');
  }

  // Metadata
  lines.push('---');

  return {
    date,
    hasBreaking,
    content: lines.join('\n'),
  };
}

function appendToChangelog(entry) {
  const changelogPath = path.join(PROJECT_ROOT, CHANGELOG_PATH);
  if (!fs.existsSync(changelogPath)) {
    console.error(`CHANGELOG.md not found at ${changelogPath}`);
    process.exit(1);
  }

  let content = fs.readFileSync(changelogPath, 'utf8');

  // Find the [Unreleased] section and insert after it
  const unreleasedHeader = '## [Unreleased]';
  const idx = content.indexOf(unreleasedHeader);
  if (idx === -1) {
    // Insert at the top after the header comment
    const headerEnd = content.indexOf('\n---', 5);
    const insertAt = headerEnd !== -1 ? headerEnd + 4 : content.indexOf('\n\n', 1);
    content =
      content.slice(0, insertAt) +
      '\n\n' +
      entry.content +
      '\n' +
      content.slice(insertAt);
  } else {
    // Insert after the [Unreleased] header
    const lineEnd = content.indexOf('\n', idx);
    const insertAt = lineEnd !== -1 ? lineEnd + 1 : idx + unreleasedHeader.length;
    content =
      content.slice(0, insertAt) +
      '\n' +
      entry.content +
      '\n' +
      content.slice(insertAt);
  }

  fs.writeFileSync(changelogPath, content);
  console.log(`Appended changelog entry to ${CHANGELOG_PATH}`);
}

// ── Main ─────────────────────────────────────────────────────────────────────

function main() {
  const args = process.argv.slice(2);
  const appendMode = args.includes('--append');
  const jsonMode = args.includes('--format') && args.includes('json');

  // Load registries
  const current = loadRegistry(REGISTRY_PATH);
  const baseline = getCommittedRegistry();

  if (!current) {
    console.error('No current registry found. Run `node scripts/abi-compat.mjs` first.');
    process.exit(1);
  }

  const changes = diffRegistry(baseline, current);
  const commits = getGitLog();

  const totalChanges =
    changes.addedContracts.length +
    changes.removedContracts.length +
    changes.addedFunctions.length +
    changes.removedFunctions.length +
    changes.changedFunctions.length;

  if (totalChanges === 0) {
    console.log('No interface changes detected. No changelog entry generated.');
    process.exit(0);
  }

  const entry = generateChangelogEntries(changes, commits);

  if (!entry) {
    console.log('No meaningful changelog entry to generate.');
    process.exit(0);
  }

  if (jsonMode) {
    console.log(
      JSON.stringify(
        { date: entry.date, hasBreaking: entry.hasBreaking, changes },
        null,
        2,
      ),
    );
  } else if (appendMode) {
    appendToChangelog(entry);
  } else {
    console.log(entry.content);
  }
}

main();
