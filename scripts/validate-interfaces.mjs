#!/usr/bin/env node
/**
 * Contract Interface Registry Validator
 *
 * Validates that contract interfaces in the registry match the actual
 * contract implementations. Produces:
 *   - reports/interface_violations.json — structured violations
 *   - reports/interface_report.md — human-readable PR comment
 *   - Exit code 1 if any breaking changes detected
 *
 * Usage:
 *   node scripts/validate-interfaces.mjs                    # validate all
 *   node scripts/validate-interfaces.mjs medical_records    # validate one
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { join } from 'path';
import { execSync } from 'child_process';

const ROOT = process.cwd();
const REGISTRY_FILE = join(ROOT, 'schemas', 'interface-registry', 'registry.json');
const REPORT_DIR = join(ROOT, 'reports');
const VIOLATIONS_FILE = join(REPORT_DIR, 'interface_violations.json');
const REPORT_FILE = join(REPORT_DIR, 'interface_report.md');

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function loadJSON(path) {
  if (!existsSync(path)) {
    console.error(`[interface] File not found: ${path}`);
    process.exit(1);
  }
  return JSON.parse(readFileSync(path, 'utf8'));
}

function saveJSON(path, data) {
  writeFileSync(path, JSON.stringify(data, null, 2) + '\n');
}

// ---------------------------------------------------------------------------
// Interface extraction from source
// ---------------------------------------------------------------------------

function extractContractFunctions(contractName) {
  const libPath = join(ROOT, 'contracts', contractName, 'src', 'lib.rs');
  if (!existsSync(libPath)) return null;

  const content = readFileSync(libPath, 'utf8');
  const functions = [];

  // Match pub fn declarations in #[contractimpl] blocks
  const fnRegex = /pub\s+fn\s+(\w+)\s*\(/g;
  let match;
  while ((match = fnRegex.exec(content)) !== null) {
    const name = match[1];
    // Skip internal/private functions
    if (name.startsWith('_') || name === 'migrate' || name === 'verify_integrity') continue;

    // Extract args (simplified)
    const argsStart = content.indexOf('(', match.index);
    const argsEnd = content.indexOf(')', argsStart);
    const argsStr = content.substring(argsStart + 1, argsEnd);
    const args = argsStr.split(',')
      .map(a => a.trim())
      .filter(a => a.length > 0 && !a.startsWith('env'))
      .map(a => {
        const parts = a.split(':');
        return parts.length >= 2 ? parts[0].trim() : null;
      })
      .filter(a => a !== null);

    // Determine if state mutation
    const fnBody = content.substring(argsEnd, content.indexOf('\n}\n', argsEnd) || argsEnd + 500);
    const stateMutation = fnBody.includes('env.storage()') || fnBody.includes('.set(');

    // Determine return type
    const returnMatch = content.substring(match.index, argsEnd + 50).match(/->\s*(\w+)/);
    const returns = returnMatch ? returnMatch[1] : 'void';

    functions.push({
      name,
      args,
      returns,
      state_mutation: stateMutation
    });
  }

  return functions;
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

function validateInterfaces(registry, filterContract) {
  const results = [];

  for (const [contractName, contract] of Object.entries(registry.contracts)) {
    if (filterContract && contractName !== filterContract) continue;

    const actualFunctions = extractContractFunctions(contractName);
    if (!actualFunctions) {
      results.push({
        contract: contractName,
        status: 'not_found',
        violations: [{
          type: 'contract_not_found',
          severity: 'error',
          message: `Contract source not found at contracts/${contractName}/src/lib.rs`
        }]
      });
      continue;
    }

    const violations = [];
    const actualNames = actualFunctions.map(f => f.name);

    // Check registered interfaces exist in source
    for (const [ifaceName, iface] of Object.entries(contract.interfaces)) {
      if (!actualNames.includes(ifaceName)) {
        violations.push({
          type: 'interface_missing',
          severity: 'error',
          interface: ifaceName,
          message: `Registered interface '${ifaceName}' not found in contract source`
        });
      }
    }

    // Check for new public functions not in registry (informational)
    for (const fn of actualFunctions) {
      if (!contract.interfaces[fn.name]) {
        violations.push({
          type: 'unregistered_function',
          severity: 'warning',
          interface: fn.name,
          message: `Public function '${fn.name}' not in registry — consider adding`
        });
      }
    }

    // Check argument count consistency
    for (const [ifaceName, iface] of Object.entries(contract.interfaces)) {
      const actual = actualFunctions.find(f => f.name === ifaceName);
      if (actual && actual.args.length !== iface.args.length) {
        violations.push({
          type: 'arg_count_mismatch',
          severity: 'error',
          interface: ifaceName,
          expected: iface.args.length,
          actual: actual.args.length,
          message: `Argument count mismatch for '${ifaceName}': expected ${iface.args.length}, got ${actual.args.length}`
        });
      }
    }

    const hasErrors = violations.some(v => v.severity === 'error');

    results.push({
      contract: contractName,
      registry_version: contract.version,
      status: hasErrors ? 'violations' : 'ok',
      violations
    });
  }

  return results;
}

// ---------------------------------------------------------------------------
// Report generation
// ---------------------------------------------------------------------------

function generateReport(results) {
  const hasViolations = results.some(r => r.violations.some(v => v.severity === 'error'));
  const lines = [];

  lines.push('## Contract Interface Registry Report');
  lines.push('');

  if (!hasViolations) {
    lines.push('> All contract interfaces match the registry.');
    lines.push('');
  } else {
    lines.push('> **Interface violations detected** — see details below.');
    lines.push('');
  }

  // Summary table
  lines.push('| Contract | Status | Errors | Warnings |');
  lines.push('|----------|--------|--------|----------|');

  for (const r of results) {
    const errors = r.violations.filter(v => v.severity === 'error').length;
    const warnings = r.violations.filter(v => v.severity === 'warning').length;
    const status = r.status === 'ok' ? ':white_check_mark: OK' :
                   r.status === 'not_found' ? ':x: NOT FOUND' :
                   ':warning: VIOLATIONS';
    lines.push(`| ${r.contract} | ${status} | ${errors} | ${warnings} |`);
  }

  lines.push('');

  // Violation details
  const violated = results.filter(r => r.violations.length > 0);
  if (violated.length > 0) {
    lines.push('### Violation Details');
    lines.push('');
    for (const r of violated) {
      lines.push(`#### ${r.contract}`);
      for (const v of r.violations) {
        const icon = v.severity === 'error' ? ':x:' : ':warning:';
        lines.push(`- ${icon} **${v.type}**: ${v.message}`);
      }
      lines.push('');
    }

    lines.push('### Remediation');
    lines.push('');
    lines.push('To fix interface violations:');
    lines.push('1. Update the contract source to match the registry, OR');
    lines.push('2. Update the registry in `schemas/interface-registry/registry.json`');
    lines.push('3. If this is a breaking change, document the migration path');
    lines.push('');
  }

  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main() {
  const args = process.argv.slice(2);
  const filterContract = args.find(a => !a.startsWith('--')) || null;

  if (!existsSync(REGISTRY_FILE)) {
    console.error(`[interface] Registry not found: ${REGISTRY_FILE}`);
    process.exit(1);
  }

  mkdirSync(REPORT_DIR, { recursive: true });

  const registry = loadJSON(REGISTRY_FILE);
  const results = validateInterfaces(registry, filterContract);

  // Write violations JSON
  const errors = results.reduce((sum, r) => sum + r.violations.filter(v => v.severity === 'error').length, 0);
  saveJSON(VIOLATIONS_FILE, {
    checked_at: new Date().toISOString(),
    total_contracts: results.length,
    contracts_with_violations: results.filter(r => r.violations.some(v => v.severity === 'error')).length,
    total_errors: errors,
    results
  });

  // Write markdown report
  const report = generateReport(results);
  writeFileSync(REPORT_FILE, report);

  // Console output
  console.log(report);

  if (errors > 0) {
    console.error(`[interface] ${errors} error(s) found.`);
    process.exit(1);
  } else {
    console.log('[interface] All interfaces valid.');
  }
}

main();
