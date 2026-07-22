#!/usr/bin/env node

/**
 * ABI Compatibility Checker for Uzima contract interfaces.
 *
 * Generates ABI snapshots for all registered contracts, compares them against
 * the committed baseline, classifies changes as breaking or non-breaking, and
 * produces a machine-readable compatibility report for CI.
 *
 * Usage:
 *   node scripts/abi-compat.mjs                   # generate snapshots + update baseline
 *   node scripts/abi-compat.mjs --check            # fail if breaking changes detected
 *   node scripts/abi-compat.mjs --check --report reports/abi_compat.txt
 *   node scripts/abi-comcompat.mjs --check --allow-breaking  # report but don't fail
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const args = process.argv.slice(2);
const CHECK_MODE = args.includes('--check');
const ALLOW_BREAKING = args.includes('--allow-breaking');
const reportFlagIndex = args.indexOf('--report');
const REPORT_PATH =
  reportFlagIndex !== -1 ? args[reportFlagIndex + 1] : null;

const REGISTRY_SCHEMA = path.join(
  projectRoot,
  'schemas/interface-registry/registry.schema.json',
);
const REGISTRY_BASELINE = path.join(
  projectRoot,
  'schemas/interface-registry/registry.json',
);

const GENERATOR_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// Canonical interface definitions (single source of truth)
// Mirrored from generate-sdk-types.mjs – kept in sync via the compatibility
// gate itself.  When a developer adds or changes a contract interface they
// must update the appropriate definition below *and* run the generator.
// ---------------------------------------------------------------------------

const ENUM_DEFINITIONS = {
  RecordType: {
    description: 'Enumeration of medical record types',
    values: [
      { member: 'DIAGNOSIS', value: 'diagnosis', description: 'Diagnosis record' },
      { member: 'PRESCRIPTION', value: 'prescription', description: 'Prescription record' },
      { member: 'LAB_RESULT', value: 'lab_result', description: 'Laboratory test result' },
      { member: 'IMAGING', value: 'imaging', description: 'Medical imaging record' },
      { member: 'CONSULTATION', value: 'consultation', description: 'Consultation notes' },
      { member: 'VITAL_SIGNS', value: 'vital_signs', description: 'Vital signs measurement' },
      { member: 'IMMUNIZATION', value: 'immunization', description: 'Immunization record' },
      { member: 'MEDICATION_HISTORY', value: 'medication_history', description: 'Medication history' },
      { member: 'ALLERGY', value: 'allergy', description: 'Allergy information' },
      { member: 'PROCEDURE', value: 'procedure', description: 'Medical procedure record' },
    ],
  },
  EncryptionAlgorithm: {
    description: 'Encryption algorithm used for medical data',
    values: [
      { member: 'NACL_BOX', value: 'nacl-box', description: 'NaCl box (public-key encryption)' },
      { member: 'NACL_SECRETBOX', value: 'nacl-secretbox', description: 'NaCl secretbox (secret-key encryption)' },
      { member: 'AES_256_GCM', value: 'aes-256-gcm', description: 'AES-256 GCM' },
    ],
  },
  ConsentStatus: {
    description: 'Consent grant status',
    values: [
      { member: 'ACTIVE', value: 'active', description: 'Consent is active and valid' },
      { member: 'REVOKED', value: 'revoked', description: 'Consent has been revoked' },
      { member: 'PENDING', value: 'pending', description: 'Consent is pending approval' },
      { member: 'EXPIRED', value: 'expired', description: 'Consent has expired' },
    ],
  },
  VerificationMethodType: {
    description: 'Verification method types per W3C DID specification',
    values: [
      { member: 'ED25519_VERIFICATION_KEY_2020', value: 'Ed25519VerificationKey2020', description: 'Ed25519 Verification Key (2020)' },
      { member: 'ECDSA_SECP256K1_VERIF_KEY_2019', value: 'EcdsaSecp256k1VerifKey2019', description: 'ECDSA Secp256k1 Verification Key (2019)' },
      { member: 'X25519_KEY_AGREEMENT_KEY_2020', value: 'X25519KeyAgreementKey2020', description: 'X25519 Key Agreement Key (2020)' },
      { member: 'JSON_WEB_KEY_2020', value: 'JsonWebKey2020', description: 'JSON Web Key (2020)' },
      { member: 'FIDO2_ED_DSA_2024', value: 'Fido2EdDsa2024', description: 'FIDO2 EdDSA Key (2024)' },
      { member: 'FIDO2_ES256_2024', value: 'Fido2Es2562024', description: 'FIDO2 ES256 Key (2024)' },
    ],
  },
  VerificationRelationship: {
    description: 'Verification relationship types per W3C DID specification',
    values: [
      { member: 'AUTHENTICATION', value: 'Authentication', description: 'Authentication relationship' },
      { member: 'ASSERTION_METHOD', value: 'AssertionMethod', description: 'Assertion method relationship' },
      { member: 'KEY_AGREEMENT', value: 'KeyAgreement', description: 'Key agreement relationship' },
      { member: 'CAPABILITY_INVOCATION', value: 'CapabilityInvocation', description: 'Capability invocation relationship' },
      { member: 'CAPABILITY_DELEGATION', value: 'CapabilityDelegation', description: 'Capability delegation relationship' },
    ],
  },
  PaymentStatusEnum: {
    description: 'Payment/claim status enumeration',
    values: [
      { member: 'SUBMITTED', value: 'submitted', description: 'Payment claim submitted' },
      { member: 'VERIFIED', value: 'verified', description: 'Payment claim verified' },
      { member: 'APPROVED', value: 'approved', description: 'Payment claim approved' },
      { member: 'REJECTED', value: 'rejected', description: 'Payment claim rejected' },
      { member: 'PAID', value: 'paid', description: 'Payment completed' },
      { member: 'DISPUTED', value: 'disputed', description: 'Payment disputed' },
    ],
  },
  ActionType: {
    description: 'Audit action types',
    values: [
      { member: 'DATA_READ', value: 'DataRead', description: 'Data read operation' },
      { member: 'DATA_WRITE', value: 'DataWrite', description: 'Data write operation' },
      { member: 'DATA_DELETE', value: 'DataDelete', description: 'Data delete operation' },
      { member: 'DATA_EXPORT', value: 'DataExport', description: 'Data export operation' },
      { member: 'PERMISSION_GRANT', value: 'PermissionGrant', description: 'Permission grant' },
      { member: 'PERMISSION_REVOKE', value: 'PermissionRevoke', description: 'Permission revoke' },
      { member: 'ROLE_ASSIGN', value: 'RoleAssign', description: 'Role assignment' },
      { member: 'ROLE_REVOKE', value: 'RoleRevoke', description: 'Role revocation' },
      { member: 'RECORD_CREATE', value: 'RecordCreate', description: 'Record creation' },
      { member: 'RECORD_UPDATE', value: 'RecordUpdate', description: 'Record update' },
      { member: 'RECORD_ARCHIVE', value: 'RecordArchive', description: 'Record archive' },
      { member: 'RECORD_RESTORE', value: 'RecordRestore', description: 'Record restore' },
      { member: 'AUTH_SUCCESS', value: 'AuthSuccess', description: 'Authentication success' },
      { member: 'AUTH_FAILURE', value: 'AuthFailure', description: 'Authentication failure' },
      { member: 'AUTH_LOGOUT', value: 'AuthLogout', description: 'User logout' },
      { member: 'TOKEN_REFRESH', value: 'TokenRefresh', description: 'Token refresh' },
      { member: 'CROSS_CHAIN_TRANSFER_INIT', value: 'CrossChainTransferInit', description: 'Cross-chain transfer initiated' },
      { member: 'CROSS_CHAIN_TRANSFER_COMPLETED', value: 'CrossChainTransferCompleted', description: 'Cross-chain transfer completed' },
    ],
  },
};

const INTERFACE_DEFINITIONS = {
  EncryptedData: {
    description: 'Encrypted data container',
    fields: [
      { name: 'ciphertext', type: 'string', description: 'Base64-encoded encrypted data' },
      { name: 'nonce', type: 'string', description: 'Base64-encoded encryption nonce' },
      { name: 'algorithm', type: 'EncryptionAlgorithm', description: 'The encryption algorithm used' },
    ],
  },
  AccessLog: {
    description: 'Access log entry for audit trail',
    fields: [
      { name: 'accessor', type: 'string', description: 'The address of who accessed the record' },
      { name: 'accessTime', type: 'number', description: 'Timestamp of access (Unix seconds)' },
      { name: 'accessType', type: "'read' | 'write' | 'share'", description: 'Type of access' },
      { name: 'ipAddress', type: 'string', description: 'Optional IP address of the accessor', optional: true },
    ],
  },
  RecordMetadata: {
    description: 'Metadata for medical records',
    fields: [
      { name: 'createdAt', type: 'number', description: 'Creation timestamp (Unix seconds)' },
      { name: 'updatedAt', type: 'number', description: 'Last update timestamp (Unix seconds)' },
      { name: 'accessLog', type: 'AccessLog[]', description: 'Complete access history' },
      { name: 'tags', type: 'string[]', description: 'Optional tags for categorization', optional: true },
      { name: 'isTraditionalHealing', type: 'boolean', description: 'Whether this is traditional healing record', optional: true },
    ],
  },
  MedicalRecord: {
    contractName: 'medical_records',
    description: 'Medical record structure matching contract schema',
    fields: [
      { name: 'id', type: 'string', description: 'Unique record identifier' },
      { name: 'patientId', type: 'string', description: "Patient's Stellar address" },
      { name: 'providerId', type: 'string', description: "Healthcare provider's Stellar address" },
      { name: 'recordType', type: 'RecordType', description: 'Type of medical record' },
      { name: 'data', type: 'EncryptedData', description: 'Encrypted record content' },
      { name: 'metadata', type: 'RecordMetadata', description: 'Record metadata and access log' },
      { name: 'timestamp', type: 'number', description: 'Record creation timestamp (Unix seconds)' },
      { name: 'isEncrypted', type: 'boolean', description: 'Whether the data is encrypted' },
      { name: 'signature', type: 'string', description: 'Optional cryptographic signature', optional: true },
    ],
  },
  ConsentGrant: {
    contractName: 'patient_consent_management',
    description: 'Consent grant from patient to provider',
    fields: [
      { name: 'id', type: 'string', description: 'Unique consent identifier' },
      { name: 'patientId', type: 'string', description: "Patient's Stellar address" },
      { name: 'providerId', type: 'string', description: "Healthcare provider's Stellar address" },
      { name: 'grantedAt', type: 'number', description: 'Timestamp when consent was granted (Unix seconds)' },
      { name: 'revokedAt', type: 'number', description: 'Timestamp when consent was revoked (Unix seconds)', optional: true },
      { name: 'status', type: 'ConsentStatus', description: 'Current consent status' },
      { name: 'scope', type: 'string[]', description: 'Optional data access scope', optional: true },
      { name: 'expiresAt', type: 'number', description: 'Optional consent expiration timestamp (Unix seconds)', optional: true },
    ],
  },
  VerificationMethod: {
    description: 'Verification method (public key) for W3C DID',
    fields: [
      { name: 'id', type: 'string', description: 'Fragment identifier' },
      { name: 'methodType', type: 'VerificationMethodType', description: 'Type of verification method' },
      { name: 'controller', type: 'string', description: 'Controller address' },
      { name: 'publicKey', type: 'string', description: 'Base64-encoded public key' },
      { name: 'isActive', type: 'boolean', description: 'Whether this key is active' },
      { name: 'created', type: 'number', description: 'Creation timestamp (Unix seconds)' },
      { name: 'lastRotated', type: 'number', description: 'Last rotation timestamp (Unix seconds, 0 if never)' },
    ],
  },
  ServiceEndpoint: {
    description: 'Service endpoint for W3C DID',
    fields: [
      { name: 'id', type: 'string', description: 'Service identifier' },
      { name: 'type', type: 'string', description: 'Service type' },
      { name: 'url', type: 'string', description: 'Service endpoint URL' },
    ],
  },
  IdentityDocument: {
    contractName: 'identity_registry',
    description: 'Identity document (Decentralized Identifier) per W3C DID spec',
    fields: [
      { name: 'id', type: 'string', description: 'The DID identifier' },
      { name: 'context', type: 'string[]', description: 'JSON-LD context URLs' },
      { name: 'verificationMethods', type: 'VerificationMethod[]', description: 'Public key information' },
      { name: 'authenticationMethods', type: 'VerificationRelationship[]', description: 'Auth key relationships', optional: true },
      { name: 'assertionMethods', type: 'VerificationRelationship[]', description: 'Assertion key relationships', optional: true },
      { name: 'serviceEndpoints', type: 'ServiceEndpoint[]', description: 'Service URLs', optional: true },
      { name: 'created', type: 'number', description: 'Creation timestamp (Unix seconds)' },
      { name: 'updated', type: 'number', description: 'Last update timestamp (Unix seconds)', optional: true },
      { name: 'proof', type: 'string', description: 'Cryptographic proof', optional: true },
    ],
  },
  PaymentStatus: {
    contractName: 'healthcare_payment',
    description: 'Payment status for healthcare claims and payments',
    fields: [
      { name: 'id', type: 'string', description: 'Unique payment identifier' },
      { name: 'patientId', type: 'string', description: "Patient's Stellar address" },
      { name: 'providerId', type: 'string', description: "Provider's Stellar address" },
      { name: 'amount', type: 'number', description: 'Payment amount in smallest unit' },
      { name: 'currency', type: 'string', description: 'Currency code (e.g., "USDC")' },
      { name: 'status', type: 'PaymentStatusEnum', description: 'Current payment status' },
      { name: 'serviceId', type: 'string', description: 'Service identifier', optional: true },
      { name: 'policyId', type: 'string', description: 'Insurance policy ID', optional: true },
      { name: 'createdAt', type: 'number', description: 'Creation timestamp (Unix seconds)' },
      { name: 'updatedAt', type: 'number', description: 'Last update timestamp (Unix seconds)' },
      { name: 'completedAt', type: 'number', description: 'Completion timestamp (Unix seconds)', optional: true },
      { name: 'transactionHash', type: 'string', description: 'Blockchain transaction hash', optional: true },
    ],
  },
  AuditEntry: {
    contractName: 'audit',
    description: 'Audit log entry for compliance and forensics',
    fields: [
      { name: 'id', type: 'string', description: 'Unique audit entry identifier' },
      { name: 'actor', type: 'string', description: 'Address of the actor performing the action' },
      { name: 'action', type: 'ActionType', description: 'Type of action performed' },
      { name: 'resource', type: 'string', description: 'Resource identifier being acted upon', optional: true },
      { name: 'resourceType', type: 'string', description: 'Type of resource', optional: true },
      { name: 'result', type: 'string', description: 'Operation result (success/failure)', optional: true },
      { name: 'reason', type: 'string', description: 'Reason for the action', optional: true },
      { name: 'timestamp', type: 'number', description: 'Action timestamp (Unix seconds)' },
      { name: 'ipAddress', type: 'string', description: 'IP address of the actor', optional: true },
      { name: 'metadata', type: 'Record<string, string>', description: 'Additional context', optional: true },
    ],
  },
};

// ---------------------------------------------------------------------------
// Registry generation
// ---------------------------------------------------------------------------

function buildRegistry() {
  const contracts = [];

  // Group definitions by contractName; shared types go into a "shared_types" bucket.
  const contractMap = new Map();

  for (const [name, def] of Object.entries(ENUM_DEFINITIONS)) {
    const bucket = 'shared_types';
    if (!contractMap.has(bucket)) {
      contractMap.set(bucket, { name: bucket, version: '1.0.0', enums: [], interfaces: [] });
    }
    contractMap.get(bucket).enums.push({ name, ...def });
  }

  for (const [name, def] of Object.entries(INTERFACE_DEFINITIONS)) {
    const bucket = def.contractName || 'shared_types';
    if (!contractMap.has(bucket)) {
      contractMap.set(bucket, { name: bucket, version: '1.0.0', enums: [], interfaces: [] });
    }
    contractMap.get(bucket).interfaces.push({ name, ...def });
  }

  for (const entry of contractMap.values()) {
    contracts.push(entry);
  }

  contracts.sort((a, b) => a.name.localeCompare(b.name));

  return {
    registry_version: '1.0.0',
    generated_at: new Date().toISOString(),
    generator: {
      name: 'scripts/abi-compat.mjs',
      version: GENERATOR_VERSION,
    },
    contracts,
  };
}

// ---------------------------------------------------------------------------
// Diff / compatibility analysis
// ---------------------------------------------------------------------------

const BREAKING_CHANGES = [];
const NON_BREAKING_CHANGES = [];

function analyseEnumChanges(contractName, oldEntry, newEntry) {
  const oldMap = new Map((oldEntry.enums || []).map((e) => [e.name, e]));
  const newMap = new Map((newEntry.enums || []).map((e) => [e.name, e]));

  for (const [name, def] of newMap) {
    if (!oldMap.has(name)) {
      NON_BREAKING_CHANGES.push({
        contract: contractName,
        type: 'enum_added',
        detail: `Enum "${name}" added`,
      });
      continue;
    }
    const oldDef = oldMap.get(name);
    const oldValues = new Map(oldDef.values.map((v) => [v.member, v.value]));
    const newValues = new Map(def.values.map((v) => [v.member, v.value]));

    for (const [member, val] of newValues) {
      if (!oldValues.has(member)) {
        NON_BREAKING_CHANGES.push({
          contract: contractName,
          type: 'enum_variant_added',
          detail: `Enum "${name}": variant "${member}" added`,
        });
      } else if (oldValues.get(member) !== val) {
        BREAKING_CHANGES.push({
          contract: contractName,
          type: 'enum_value_changed',
          detail: `Enum "${name}": variant "${member}" value changed from "${oldValues.get(member)}" to "${val}"`,
        });
      }
    }

    for (const [member] of oldValues) {
      if (!newValues.has(member)) {
        BREAKING_CHANGES.push({
          contract: contractName,
          type: 'enum_variant_removed',
          detail: `Enum "${name}": variant "${member}" removed`,
        });
      }
    }
  }

  for (const [name] of oldMap) {
    if (!newMap.has(name)) {
      BREAKING_CHANGES.push({
        contract: contractName,
        type: 'enum_removed',
        detail: `Enum "${name}" removed`,
      });
    }
  }
}

function analyseInterfaceChanges(contractName, oldEntry, newEntry) {
  const oldMap = new Map((oldEntry.interfaces || []).map((i) => [i.name, i]));
  const newMap = new Map((newEntry.interfaces || []).map((i) => [i.name, i]));

  for (const [name, def] of newMap) {
    if (!oldMap.has(name)) {
      NON_BREAKING_CHANGES.push({
        contract: contractName,
        type: 'interface_added',
        detail: `Interface "${name}" added`,
      });
      continue;
    }
    const oldDef = oldMap.get(name);
    const oldFields = new Map(oldDef.fields.map((f) => [f.name, f]));
    const newFields = new Map(def.fields.map((f) => [f.name, f]));

    for (const [fname, fdef] of newFields) {
      if (!oldFields.has(fname)) {
        if (fdef.optional) {
          NON_BREAKING_CHANGES.push({
            contract: contractName,
            type: 'field_added_optional',
            detail: `Interface "${name}": optional field "${fname}" added`,
          });
        } else {
          BREAKING_CHANGES.push({
            contract: contractName,
            type: 'field_added_required',
            detail: `Interface "${name}": required field "${fname}" added (breaking — downstream code must supply this field)`,
          });
        }
      } else {
        const oldF = oldFields.get(fname);
        if (oldF.type !== fdef.type) {
          BREAKING_CHANGES.push({
            contract: contractName,
            type: 'field_type_changed',
            detail: `Interface "${name}": field "${fname}" type changed from "${oldF.type}" to "${fdef.type}"`,
          });
        }
        if (oldF.optional && !fdef.optional) {
          BREAKING_CHANGES.push({
            contract: contractName,
            type: 'field_became_required',
            detail: `Interface "${name}": field "${fname}" changed from optional to required`,
          });
        } else if (!oldF.optional && fdef.optional) {
          NON_BREAKING_CHANGES.push({
            contract: contractName,
            type: 'field_became_optional',
            detail: `Interface "${name}": field "${fname}" changed from required to optional`,
          });
        }
      }
    }

    for (const [fname] of oldFields) {
      if (!newFields.has(fname)) {
        BREAKING_CHANGES.push({
          contract: contractName,
          type: 'field_removed',
          detail: `Interface "${name}": field "${fname}" removed`,
        });
      }
    }
  }

  for (const [name] of oldMap) {
    if (!newMap.has(name)) {
      BREAKING_CHANGES.push({
        contract: contractName,
        type: 'interface_removed',
        detail: `Interface "${name}" removed`,
      });
    }
  }
}

function analyseChanges(oldRegistry, newRegistry) {
  const oldMap = new Map(oldRegistry.contracts.map((c) => [c.name, c]));
  const newMap = new Map(newRegistry.contracts.map((c) => [c.name, c]));

  for (const [name, entry] of newMap) {
    if (!oldMap.has(name)) {
      NON_BREAKING_CHANGES.push({
        contract: name,
        type: 'contract_added',
        detail: `Contract "${name}" added to registry`,
      });
      continue;
    }
    analyseEnumChanges(name, oldMap.get(name), entry);
    analyseInterfaceChanges(name, oldMap.get(name), entry);
  }

  for (const [name] of oldMap) {
    if (!newMap.has(name)) {
      BREAKING_CHANGES.push({
        contract: name,
        type: 'contract_removed',
        detail: `Contract "${name}" removed from registry`,
      });
    }
  }
}

// ---------------------------------------------------------------------------
// Report generation
// ---------------------------------------------------------------------------

function buildReport(newRegistry) {
  const lines = [];
  const breaking = BREAKING_CHANGES.length;
  const nonBreaking = NON_BREAKING_CHANGES.length;

  lines.push('## ABI Compatibility Report');
  lines.push('');
  lines.push(`Generated at: \`${newRegistry.generated_at}\``);
  lines.push(`Registry version: \`${newRegistry.registry_version}\``);
  lines.push(`Generator version: \`${GENERATOR_VERSION}\``);
  lines.push('');
  lines.push(`| Metric | Count |`);
  lines.push(`|--------|-------|`);
  lines.push(`| Contracts registered | ${newRegistry.contracts.length} |`);
  lines.push(`| Breaking changes | ${breaking} |`);
  lines.push(`| Non-breaking changes | ${nonBreaking} |`);
  lines.push('');

  if (breaking > 0) {
    lines.push('### Breaking Changes');
    lines.push('');
    for (const ch of BREAKING_CHANGES) {
      lines.push(`- **${ch.contract}**: ${ch.detail}`);
    }
    lines.push('');
  }

  if (nonBreaking > 0) {
    lines.push('### Non-Breaking Changes');
    lines.push('');
    for (const ch of NON_BREAKING_CHANGES) {
      lines.push(`- **${ch.contract}**: ${ch.detail}`);
    }
    lines.push('');
  }

  if (breaking === 0 && nonBreaking === 0) {
    lines.push('> No interface changes detected — baseline is in sync.');
    lines.push('');
  }

  lines.push('---');
  lines.push(`> Enforced by \`scripts/abi-compat.mjs\`. See [CONTRACT_COMPATIBILITY.md](docs/CONTRACT_COMPATIBILITY.md) for the breaking vs non-breaking change policy.`);
  return lines.join('\n');
}

function buildMachineReport(newRegistry) {
  const lines = [];
  lines.push(`ABI_COMPAT_BREAKING=${BREAKING_CHANGES.length}`);
  lines.push(`ABI_COMPAT_NON_BREAKING=${NON_BREAKING_CHANGES.length}`);
  lines.push(`ABI_COMPAT_CONTRACTS=${newRegistry.contracts.length}`);
  lines.push(`GENERATOR_VERSION=${GENERATOR_VERSION}`);
  lines.push('');

  for (const ch of BREAKING_CHANGES) {
    lines.push(`BREAKING=${ch.contract}|${ch.type}|${ch.detail}`);
  }
  for (const ch of NON_BREAKING_CHANGES) {
    lines.push(`NON_BREAKING=${ch.contract}|${ch.type}|${ch.detail}`);
  }

  return `${lines.join('\n').trimEnd()}\n`;
}

// ---------------------------------------------------------------------------
// File I/O
// ---------------------------------------------------------------------------

function readOrEmpty(filePath) {
  try {
    return fs.existsSync(filePath) ? fs.readFileSync(filePath, 'utf8') : '';
  } catch {
    return '';
  }
}

function ensureParentDir(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function main() {
  console.log('ABI Compatibility Checker v' + GENERATOR_VERSION);
  console.log('');

  const newRegistry = buildRegistry();

  if (CHECK_MODE) {
    const baselineRaw = readOrEmpty(REGISTRY_BASELINE);
    if (!baselineRaw) {
      console.log('No baseline registry found — treating as initial generation.');
      console.log('Run without --check first to create the baseline.');
      process.exit(0);
    }

    let baseline;
    try {
      baseline = JSON.parse(baselineRaw);
    } catch (err) {
      console.error('Failed to parse baseline registry:', err.message);
      process.exit(1);
    }

    analyseChanges(baseline, newRegistry);

    const report = buildReport(newRegistry);
    const machineReport = buildMachineReport(newRegistry);

    if (REPORT_PATH) {
      ensureParentDir(REPORT_PATH);
      fs.writeFileSync(REPORT_PATH, machineReport, 'utf8');
      console.log(`Machine report written to ${REPORT_PATH}`);
    }

    console.log(report);

    if (BREAKING_CHANGES.length > 0 && !ALLOW_BREAKING) {
      console.error('');
      console.error(`BREAKING CHANGES DETECTED (${BREAKING_CHANGES.length}).`);
      console.error('If intentional, re-run with --allow-breaking or add [approve-abi-change] to the PR body.');
      console.error('See docs/CONTRACT_COMPATIBILITY.md for guidance.');
      process.exit(1);
    }

    if (BREAKING_CHANGES.length === 0 && NON_BREAKING_CHANGES.length === 0) {
      console.log('ABI compatibility: OK — no changes from baseline.');
    } else if (BREAKING_CHANGES.length === 0) {
      console.log('ABI compatibility: OK — only non-breaking changes.');
    } else {
      console.log('ABI compatibility: breaking changes approved.');
    }
    process.exit(0);
  }

  // Generation mode — write the baseline.
  ensureParentDir(REGISTRY_BASELINE);
  fs.writeFileSync(
    REGISTRY_BASELINE,
    JSON.stringify(newRegistry, null, 2) + '\n',
    'utf8',
  );
  console.log(`Baseline written to ${path.relative(projectRoot, REGISTRY_BASELINE)}`);
  console.log(`${newRegistry.contracts.length} contract(s) registered.`);
}

main();
