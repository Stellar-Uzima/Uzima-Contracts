#!/usr/bin/env node
/**
 * test_normalize_config.mjs - Unit tests for config normalization and validation.
 */
import assert from 'assert';

function isObject(item) { return item !== null && typeof item === 'object' && !Array.isArray(item); }
function deepClone(obj) { return JSON.parse(JSON.stringify(obj)); }
function deepMerge(target, source) {
  let output = { ...target };
  if (isObject(target) && isObject(source)) {
    Object.keys(source).forEach((key) => {
      if (isObject(source[key])) {
        if (!(key in target)) Object.assign(output, { [key]: source[key] });
        else output[key] = deepMerge(target[key], source[key]);
      } else Object.assign(output, { [key]: source[key] });
    });
  }
  return output;
}
function canonicalSort(obj) {
  if (!isObject(obj)) return obj;
  const sorted = {};
  Object.keys(obj).sort().forEach((key) => { sorted[key] = canonicalSort(obj[key]); });
  return sorted;
}
function normalizeConfig(config) {
  const n = deepClone(config);
  delete n.extends;
  if (n.contracts && isObject(n.contracts)) {
    for (const [name, contract] of Object.entries(n.contracts)) {
      if (typeof contract.enabled !== 'boolean') n.contracts[name].enabled = true;
    }
  }
  return n;
}

const VALID_ENVIRONMENTS = ['development', 'testing', 'staging', 'production'];
const VALID_SAFETY_LEVELS = ['low', 'medium', 'high'];
const VALID_NETWORK_PROFILES = ['local', 'testnet', 'futurenet', 'mainnet'];
const NETWORK_REQUIRED_FIELDS = [
  'name', 'description', 'rpc-url', 'network-passphrase', 'horizon-url',
  'environment', 'requires-funding', 'gas-configuration', 'safety-level', 'confirmation-required',
];
const DEFAULT_IDENTITY_OVERRIDES = {
  development: { 'dry-run': false, simulation: false },
  testing: { 'dry-run': false, simulation: true },
  production: { 'dry-run': true, simulation: true },
};

function validateNetworkProfile(name, profile) {
  const errors = [];
  for (const field of NETWORK_REQUIRED_FIELDS) { if (!(field in profile)) errors.push(`missing "${field}"`); }
  if (profile.environment && !VALID_ENVIRONMENTS.includes(profile.environment)) errors.push('invalid environment');
  if (profile['safety-level'] && !VALID_SAFETY_LEVELS.includes(profile['safety-level'])) errors.push('invalid safety-level');
  return errors;
}
function normalizeIdentityDefaults(defaults) {
  const n = { ...defaults };
  if (!n.network) n.network = 'testnet';
  if (!n.identity) n.identity = 'default';
  if (typeof n['dry-run'] !== 'boolean') n['dry-run'] = DEFAULT_IDENTITY_OVERRIDES[n.identity]?.['dry-run'] ?? false;
  if (typeof n.simulation !== 'boolean') n.simulation = DEFAULT_IDENTITY_OVERRIDES[n.identity]?.simulation ?? true;
  return n;
}
function validateIdentityConfig(name, identity) {
  const errors = [];
  if (!identity.network) errors.push('missing "network"');
  else if (!VALID_NETWORK_PROFILES.includes(identity.network)) errors.push('invalid network');
  if (!identity.identity) errors.push('missing "identity"');
  return errors;
}
function validateNetworkGroups(groups, knownNetworks) {
  const errors = [];
  for (const [groupName, group] of Object.entries(groups)) {
    if (!group.networks || !Array.isArray(group.networks)) { errors.push('"networks" must be array'); continue; }
    for (const net of group.networks) { if (!knownNetworks.includes(net)) errors.push(`unknown "${net}"`); }
  }
  return errors;
}

let passed = 0, failed = 0;
function test(name, fn) { try { fn(); passed++; console.log(`  PASS: ${name}`); } catch (err) { failed++; console.log(`  FAIL: ${name}: ${err.message}`); } }

console.log('Running config normalization tests...\n');

test('deepMerge: merges flat objects', () => assert.deepStrictEqual(deepMerge({ a: 1 }, { b: 2 }), { a: 1, b: 2 }));
test('deepMerge: source overrides', () => assert.strictEqual(deepMerge({ a: 1 }, { a: 2 }).a, 2));
test('deepMerge: deep nested', () => assert.deepStrictEqual(deepMerge({ a: { b: 1, c: 2 } }, { a: { b: 3 } }), { a: { b: 3, c: 2 } }));
test('canonicalSort: sorts keys', () => assert.deepStrictEqual(Object.keys(canonicalSort({ z: 1, a: 2 })), ['a', 'z']));
test('canonicalSort: sorts nested', () => assert.deepStrictEqual(Object.keys(canonicalSort({ z: { c: 1, a: 2 } }).z), ['a', 'c']));
test('canonicalSort: primitives', () => { assert.strictEqual(canonicalSort(42), 42); assert.strictEqual(canonicalSort('hi'), 'hi'); });
test('normalizeConfig: removes extends', () => assert.ok(!('extends' in normalizeConfig({ extends: './x', contracts: {} }))));
test('normalizeConfig: injects enabled', () => assert.strictEqual(normalizeConfig({ contracts: { f: {} } }).contracts.f.enabled, true));
test('normalizeConfig: preserves enabled', () => assert.strictEqual(normalizeConfig({ contracts: { f: { enabled: false } } }).contracts.f.enabled, false));
test('validateNetworkProfile: valid', () => assert.strictEqual(validateNetworkProfile('testnet', {
  name: 'T', description: 'd', 'rpc-url': 'https://rpc.test', 'network-passphrase': 'p',
  'horizon-url': 'https://h.test', environment: 'testing', 'requires-funding': true,
  'gas-configuration': { 'max-instructions': 1, 'tx-resource-fee': 1 },
  'safety-level': 'medium', 'confirmation-required': false }).length, 0));
test('validateNetworkProfile: missing fields', () => assert.ok(validateNetworkProfile('bad', { name: 'B' }).length > 0));
test('validateNetworkProfile: invalid env', () => assert.ok(validateNetworkProfile('x', {
  name: 'X', description: 'd', 'rpc-url': 'http://x', 'network-passphrase': 'p',
  'horizon-url': 'http://x', environment: 'NOPE', 'requires-funding': false,
  'gas-configuration': { 'max-instructions': 1, 'tx-resource-fee': 1 },
  'safety-level': 'low', 'confirmation-required': false }).some((e) => e.includes('invalid env'))));
test('validateIdentityConfig: valid', () => assert.strictEqual(validateIdentityConfig('d', { network: 'local', identity: 'd' }).length, 0));
test('validateIdentityConfig: missing network', () => assert.ok(validateIdentityConfig('x', { identity: 'x' }).some((e) => e.includes('missing "network"'))));
test('validateIdentityConfig: invalid network', () => assert.ok(validateIdentityConfig('x', { network: 'bad', identity: 'x' }).some((e) => e.includes('invalid network'))));
test('normalizeIdentityDefaults: injects defaults', () => { const r = normalizeIdentityDefaults({ network: 'testnet', identity: 'dev' }); assert.strictEqual(r['dry-run'], false); assert.strictEqual(r.simulation, true); });
test('normalizeIdentityDefaults: development', () => { const r = normalizeIdentityDefaults({ network: 'local', identity: 'development' }); assert.strictEqual(r['dry-run'], false); assert.strictEqual(r.simulation, false); });
test('normalizeIdentityDefaults: production', () => { const r = normalizeIdentityDefaults({ network: 'mainnet', identity: 'production' }); assert.strictEqual(r['dry-run'], true); assert.strictEqual(r.simulation, true); });
test('normalizeIdentityDefaults: preserves booleans', () => { const r = normalizeIdentityDefaults({ network: 'testnet', identity: 'test', 'dry-run': true, simulation: false }); assert.strictEqual(r['dry-run'], true); assert.strictEqual(r.simulation, false); });
test('validateNetworkGroups: valid', () => assert.strictEqual(validateNetworkGroups({ dev: { networks: ['local'], description: 'd' } }, ['local']).length, 0));
test('validateNetworkGroups: unknown net', () => assert.ok(validateNetworkGroups({ bad: { networks: ['none'], description: 'd' } }, ['local']).length > 0));

console.log(`\n${passed + failed} tests: ${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
