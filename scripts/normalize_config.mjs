#!/usr/bin/env node
/**
 * normalize_config.mjs - Validates and normalizes config files for network profiles and identities.
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.resolve(__dirname, '..');
const CONFIG_DIR = path.join(PROJECT_ROOT, 'config');

const VALID_ENVIRONMENTS = ['development', 'testing', 'staging', 'production'];
const VALID_SAFETY_LEVELS = ['low', 'medium', 'high'];
const NETWORK_REQUIRED_FIELDS = [
  'name', 'description', 'rpc-url', 'network-passphrase', 'horizon-url',
  'environment', 'requires-funding', 'gas-configuration', 'safety-level', 'confirmation-required',
];
const GAS_CONFIG_FIELDS = ['max-instructions', 'tx-resource-fee'];
const VALID_NETWORK_PROFILES = ['local', 'testnet', 'futurenet', 'mainnet'];

const DEFAULT_IDENTITY_OVERRIDES = {
  development: { 'dry-run': false, simulation: false },
  testing: { 'dry-run': false, simulation: true },
  production: { 'dry-run': true, simulation: true },
};

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

function parseToml(content) {
  const result = {};
  let currentSection = null;
  for (const line of content.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/);
    if (sectionMatch) {
      currentSection = sectionMatch[1];
      if (!result[currentSection]) result[currentSection] = {};
      continue;
    }
    if (currentSection && trimmed.includes('=')) {
      const eqIndex = trimmed.indexOf('=');
      const key = trimmed.slice(0, eqIndex).trim();
      let value = trimmed.slice(eqIndex + 1).trim();
      if (value.startsWith('"') && value.endsWith('"')) value = value.slice(1, -1);
      else if (value === 'true') value = true;
      else if (value === 'false') value = false;
      else if (/^\d+$/.test(value)) value = parseInt(value, 10);
      else if (value.startsWith('[') && value.endsWith(']')) {
        value = value.slice(1, -1).trim().split(',').map((s) => s.trim().replace(/^"|"$/g, ''));
      } else if (value.startsWith('{') && value.endsWith('}')) {
        const obj = {};
        value.slice(1, -1).trim().split(',').forEach((pair) => {
          const [k, v] = pair.split('=').map((s) => s.trim());
          if (k && v) obj[k] = parseInt(v.replace(/_/g, ''), 10);
        });
        value = obj;
      }
      result[currentSection][key] = value;
    }
  }
  return result;
}

function validateNetworkProfile(name, profile) {
  const errors = [];
  for (const field of NETWORK_REQUIRED_FIELDS) {
    if (!(field in profile)) errors.push(`Network "${name}": missing "${field}"`);
  }
  if (profile.environment && !VALID_ENVIRONMENTS.includes(profile.environment))
    errors.push(`Network "${name}": invalid environment`);
  if (profile['safety-level'] && !VALID_SAFETY_LEVELS.includes(profile['safety-level']))
    errors.push(`Network "${name}": invalid safety-level`);
  if (profile['rpc-url'] && !profile['rpc-url'].startsWith('http'))
    errors.push(`Network "${name}": rpc-url must start with http(s)`);
  if (profile['gas-configuration']) {
    for (const f of GAS_CONFIG_FIELDS) {
      if (!(f in profile['gas-configuration']))
        errors.push(`Network "${name}": gas-configuration missing "${f}"`);
    }
  }
  if (name === 'mainnet') {
    if (profile['requires-funding'] !== false) errors.push('Network "mainnet": requires-funding must be false');
    if (profile['confirmation-required'] !== true) errors.push('Network "mainnet": confirmation-required must be true');
    if (profile['safety-level'] !== 'high') errors.push('Network "mainnet": safety-level must be "high"');
  }
  return errors;
}

function normalizeIdentityDefaults(defaults) {
  const n = { ...defaults };
  if (!n.network) n.network = 'testnet';
  if (!n.identity) n.identity = 'default';
  if (typeof n['dry-run'] !== 'boolean')
    n['dry-run'] = DEFAULT_IDENTITY_OVERRIDES[n.identity]?.['dry-run'] ?? false;
  if (typeof n.simulation !== 'boolean')
    n.simulation = DEFAULT_IDENTITY_OVERRIDES[n.identity]?.simulation ?? true;
  return n;
}

function validateIdentityConfig(name, identity) {
  const errors = [];
  if (!identity.network) errors.push(`Identity "${name}": missing "network"`);
  else if (!VALID_NETWORK_PROFILES.includes(identity.network))
    errors.push(`Identity "${name}": invalid network`);
  if (!identity.identity) errors.push(`Identity "${name}": missing "identity"`);
  return errors;
}

function validateNetworkGroups(groups, knownNetworks) {
  const errors = [];
  for (const [groupName, group] of Object.entries(groups)) {
    if (!group.networks || !Array.isArray(group.networks)) {
      errors.push(`Group "${groupName}": "networks" must be an array`);
      continue;
    }
    for (const net of group.networks) {
      if (!knownNetworks.includes(net)) errors.push(`Group "${groupName}": unknown network "${net}"`);
    }
  }
  return errors;
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

const args = process.argv.slice(2);
const fixMode = args.includes('--fix');
const profileFilter = args.includes('--profile') ? args[args.indexOf('--profile') + 1] : null;
let allErrors = [];
let filesModified = 0;

const jsonFiles = fs.readdirSync(CONFIG_DIR).filter((f) => f.endsWith('.json') && f !== 'schema.json');
for (const file of jsonFiles) {
  const filePath = path.join(CONFIG_DIR, file);
  try {
    const raw = fs.readFileSync(filePath, 'utf8');
    const config = JSON.parse(raw);
    if (config.extends) {
      const parentPath = path.join(CONFIG_DIR, config.extends);
      if (fs.existsSync(parentPath)) {
        deepMerge(JSON.parse(fs.readFileSync(parentPath, 'utf8')), config);
        console.log(`  ${file}: valid (extends ${config.extends})`);
      } else allErrors.push(`${file}: extends non-existent "${config.extends}"`);
    } else console.log(`  ${file}: valid`);
    if (fixMode) {
      const normalized = canonicalSort(normalizeConfig(config));
      const newContent = JSON.stringify(normalized, null, 2) + '\n';
      if (newContent !== raw) { fs.writeFileSync(filePath, newContent, 'utf8'); filesModified++; }
    }
  } catch (err) { allErrors.push(`${file}: ${err.message}`); }
}

const networksTomlPath = path.join(CONFIG_DIR, 'networks.toml');
if (fs.existsSync(networksTomlPath)) {
  const parsed = parseToml(fs.readFileSync(networksTomlPath, 'utf8'));
  const networkProfiles = {}, networkGroups = {}, networkDefaults = {};
  for (const [key, value] of Object.entries(parsed)) {
    if (key.startsWith('networks.')) {
      const name = key.slice('networks.'.length);
      if (!profileFilter || profileFilter === name) networkProfiles[name] = value;
    } else if (key.startsWith('groups.')) networkGroups[key.slice('groups.'.length)] = value;
    else if (key.startsWith('defaults.')) networkDefaults[key.slice('defaults.'.length)] = value;
  }
  console.log('\nNetwork profiles:');
  for (const [name, profile] of Object.entries(networkProfiles)) {
    const errors = validateNetworkProfile(name, profile);
    if (errors.length > 0) allErrors.push(...errors);
    else console.log(`  ${name}: valid`);
  }
  console.log('\nNetwork groups:');
  const groupErrors = validateNetworkGroups(networkGroups, Object.keys(networkProfiles));
  if (groupErrors.length > 0) allErrors.push(...groupErrors);
  else for (const name of Object.keys(networkGroups)) console.log(`  ${name}: valid`);
  console.log('\nIdentity defaults:');
  for (const [name, identity] of Object.entries(networkDefaults)) {
    const errors = validateIdentityConfig(name, identity);
    if (errors.length > 0) allErrors.push(...errors);
    else console.log(`  ${name}: valid`);
  }
}

console.log('\n========================================');
console.log('CONFIG NORMALIZATION REPORT');
console.log('========================================');
if (allErrors.length > 0) {
  console.log(`\n${allErrors.length} error(s) found:\n`);
  allErrors.forEach((err) => console.log(`  - ${err}`));
  process.exit(1);
} else {
  console.log('\nAll config files are valid.');
  if (fixMode && filesModified > 0) console.log(`${filesModified} file(s) normalized.`);
  process.exit(0);
}
