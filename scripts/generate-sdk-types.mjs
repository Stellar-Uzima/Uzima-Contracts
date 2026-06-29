#!/usr/bin/env node

/**
 * SDK type generator for Uzima contract bindings.
 *
 * Generates TypeScript and Python (stellar-py style) bindings from the shared
 * contract schema and enforces drift detection in CI.
 *
 * Usage:
 *   node scripts/generate-sdk-types.mjs            # write bindings
 *   node scripts/generate-sdk-types.mjs --check    # fail if committed copy drifts
 *   node scripts/generate-sdk-types.mjs --report reports/sdk_bindings_drift.txt
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const args = process.argv.slice(2);
const CHECK_MODE = args.includes('--check');
const reportFlagIndex = args.indexOf('--report');
const REPORT_PATH =
  reportFlagIndex !== -1 ? args[reportFlagIndex + 1] : null;

const OUTPUT_TS = resolveArg('--output-ts', path.join(
  projectRoot,
  'mobile-sdk/core/src/generated/contract-bindings.ts',
));
const OUTPUT_PY = resolveArg('--output-py', path.join(
  projectRoot,
  'mobile-sdk/python/uzima_sdk/contract_bindings.py',
));
const OUTPUT_PY_INIT = path.join(
  projectRoot,
  'mobile-sdk/python/uzima_sdk/__init__.py',
);

function resolveArg(flag, defaultValue) {
  const index = args.indexOf(flag);
  return index !== -1 ? args[index + 1] : defaultValue;
}

/**
 * Shared contract schema mirrored from on-chain Soroban types.
 * Update this table when contract event/struct variants change.
 */
const ENUM_DEFINITIONS = {
  RecordType: {
    description: 'Enumeration of medical record types',
    values: [
      ['DIAGNOSIS', 'diagnosis', 'Diagnosis record'],
      ['PRESCRIPTION', 'prescription', 'Prescription record'],
      ['LAB_RESULT', 'lab_result', 'Laboratory test result'],
      ['IMAGING', 'imaging', 'Medical imaging record'],
      ['CONSULTATION', 'consultation', 'Consultation notes'],
      ['VITAL_SIGNS', 'vital_signs', 'Vital signs measurement'],
      ['IMMUNIZATION', 'immunization', 'Immunization record'],
      ['MEDICATION_HISTORY', 'medication_history', 'Medication history'],
      ['ALLERGY', 'allergy', 'Allergy information'],
      ['PROCEDURE', 'procedure', 'Medical procedure record'],
    ],
  },
  EncryptionAlgorithm: {
    description: 'Encryption algorithm used for medical data',
    values: [
      ['NACL_BOX', 'nacl-box', 'NaCl box (public-key encryption)'],
      ['NACL_SECRETBOX', 'nacl-secretbox', 'NaCl secretbox (secret-key encryption)'],
      ['AES_256_GCM', 'aes-256-gcm', 'AES-256 GCM'],
    ],
  },
  ConsentStatus: {
    description: 'Consent grant status',
    values: [
      ['ACTIVE', 'active', 'Consent is active and valid'],
      ['REVOKED', 'revoked', 'Consent has been revoked'],
      ['PENDING', 'pending', 'Consent is pending approval'],
      ['EXPIRED', 'expired', 'Consent has expired'],
    ],
  },
  VerificationMethodType: {
    description: 'Verification method types per W3C DID specification',
    values: [
      ['ED25519_VERIFICATION_KEY_2020', 'Ed25519VerificationKey2020', 'Ed25519 Verification Key (2020)'],
      ['ECDSA_SECP256K1_VERIF_KEY_2019', 'EcdsaSecp256k1VerifKey2019', 'ECDSA Secp256k1 Verification Key (2019)'],
      ['X25519_KEY_AGREEMENT_KEY_2020', 'X25519KeyAgreementKey2020', 'X25519 Key Agreement Key (2020)'],
      ['JSON_WEB_KEY_2020', 'JsonWebKey2020', 'JSON Web Key (2020)'],
      ['FIDO2_ED_DSA_2024', 'Fido2EdDsa2024', 'FIDO2 EdDSA Key (2024)'],
      ['FIDO2_ES256_2024', 'Fido2Es2562024', 'FIDO2 ES256 Key (2024)'],
    ],
  },
  VerificationRelationship: {
    description: 'Verification relationship types per W3C DID specification',
    values: [
      ['AUTHENTICATION', 'Authentication', 'Authentication relationship'],
      ['ASSERTION_METHOD', 'AssertionMethod', 'Assertion method relationship'],
      ['KEY_AGREEMENT', 'KeyAgreement', 'Key agreement relationship'],
      ['CAPABILITY_INVOCATION', 'CapabilityInvocation', 'Capability invocation relationship'],
      ['CAPABILITY_DELEGATION', 'CapabilityDelegation', 'Capability delegation relationship'],
    ],
  },
  PaymentStatusEnum: {
    description: 'Payment/claim status enumeration',
    values: [
      ['SUBMITTED', 'submitted', 'Payment claim submitted'],
      ['VERIFIED', 'verified', 'Payment claim verified'],
      ['APPROVED', 'approved', 'Payment claim approved'],
      ['REJECTED', 'rejected', 'Payment claim rejected'],
      ['PAID', 'paid', 'Payment completed'],
      ['DISPUTED', 'disputed', 'Payment disputed'],
    ],
  },
  ActionType: {
    description: 'Audit action types',
    values: [
      ['DATA_READ', 'DataRead', 'Data read operation'],
      ['DATA_WRITE', 'DataWrite', 'Data write operation'],
      ['DATA_DELETE', 'DataDelete', 'Data delete operation'],
      ['DATA_EXPORT', 'DataExport', 'Data export operation'],
      ['PERMISSION_GRANT', 'PermissionGrant', 'Permission grant'],
      ['PERMISSION_REVOKE', 'PermissionRevoke', 'Permission revoke'],
      ['ROLE_ASSIGN', 'RoleAssign', 'Role assignment'],
      ['ROLE_REVOKE', 'RoleRevoke', 'Role revocation'],
      ['RECORD_CREATE', 'RecordCreate', 'Record creation'],
      ['RECORD_UPDATE', 'RecordUpdate', 'Record update'],
      ['RECORD_ARCHIVE', 'RecordArchive', 'Record archive'],
      ['RECORD_RESTORE', 'RecordRestore', 'Record restore'],
      ['AUTH_SUCCESS', 'AuthSuccess', 'Authentication success'],
      ['AUTH_FAILURE', 'AuthFailure', 'Authentication failure'],
      ['AUTH_LOGOUT', 'AuthLogout', 'User logout'],
      ['TOKEN_REFRESH', 'TokenRefresh', 'Token refresh'],
      ['CROSS_CHAIN_TRANSFER_INIT', 'CrossChainTransferInit', 'Cross-chain transfer initiated'],
      ['CROSS_CHAIN_TRANSFER_COMPLETED', 'CrossChainTransferCompleted', 'Cross-chain transfer completed'],
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

const GENERATOR_VERSION = '1.0.0';

function tsEnum(name, config) {
  const lines = [
    '/**',
    ` * ${config.description}`,
    ' * @enum {string}',
    ' */',
    `export enum ${name} {`,
  ];
  for (const [member, value, doc] of config.values) {
    lines.push(`  /** ${doc} */`);
    lines.push(`  ${member} = '${value}',`);
  }
  lines.push('}');
  return lines.join('\n');
}

function tsInterface(name, config) {
  const lines = [
    '/**',
    ` * ${config.description}`,
    ` * @interface ${name}`,
  ];
  for (const field of config.fields) {
    const optional = field.optional ? '[optional] ' : '';
    lines.push(` * @property {${field.type}} ${optional}${field.name} - ${field.description}`);
  }
  lines.push(' */');
  lines.push(`export interface ${name} {`);
  for (const field of config.fields) {
    const optional = field.optional ? '?' : '';
    lines.push(`  ${field.name}${optional}: ${field.type};`);
  }
  lines.push('}');
  return lines.join('\n');
}

function generateTypeScriptBindings() {
  const sections = [
    '/**',
    ' * Auto-generated contract bindings for the Uzima mobile SDK.',
    ' *',
    ' * DO NOT EDIT MANUALLY — run `node scripts/generate-sdk-types.mjs` instead.',
    ` * Generator version: ${GENERATOR_VERSION}`,
    ' * @module @uzima/sdk-core/generated',
    ' */',
    '',
    '// ==================== Contract Enums ====================',
    '',
  ];

  for (const [name, config] of Object.entries(ENUM_DEFINITIONS)) {
    sections.push(tsEnum(name, config), '');
  }

  sections.push('// ==================== Contract Interfaces ====================', '');

  for (const [name, config] of Object.entries(INTERFACE_DEFINITIONS)) {
    sections.push(tsInterface(name, config), '');
  }

  const content = `${sections.join('\n').trimEnd()}\n`;
  if (content.match(/:\s*any(?![a-zA-Z_])/)) {
    throw new Error('Generated TypeScript bindings contain unsafe `any` types');
  }
  return content;
}

function toSnakeCase(name) {
  return name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
}

function pyType(tsType) {
  if (tsType === 'string') return 'str';
  if (tsType === 'number') return 'int';
  if (tsType === 'boolean') return 'bool';
  if (tsType.endsWith('[]')) {
    const inner = tsType.slice(0, -2);
    if (inner === 'string') return 'List[str]';
    if (inner === 'VerificationMethod') return 'List[VerificationMethod]';
    if (inner === 'VerificationRelationship') return 'List[VerificationRelationship]';
    if (inner === 'ServiceEndpoint') return 'List[ServiceEndpoint]';
    if (inner === 'AccessLog') return 'List[AccessLog]';
    return `List[${inner}]`;
  }
  if (tsType === 'Record<string, string>') return 'Dict[str, str]';
  if (tsType.startsWith("'") && tsType.includes('|')) {
    const values = tsType.split('|').map((v) => v.trim().replace(/^'|'$/g, ''));
    return `Literal[${values.map((v) => `'${v}'`).join(', ')}]`;
  }
  return tsType;
}

function pyEnum(name, config) {
  const lines = [
    `class ${name}(str, Enum):`,
    `    """${config.description}"""`,
  ];
  for (const [member, value, doc] of config.values) {
    lines.push(`    ${member} = "${value}"  # ${doc}`);
  }
  return lines.join('\n');
}

function pyDataclass(name, config) {
  const lines = [
  `@dataclass`,
  `class ${name}:`,
  `    """${config.description}"""`,
  ];
  for (const field of config.fields) {
    const pyName = toSnakeCase(field.name);
    const typeName = pyType(field.type);
    if (field.optional) {
      lines.push(`    ${pyName}: Optional[${typeName}] = None`);
    } else {
      lines.push(`    ${pyName}: ${typeName}`);
    }
  }
  return lines.join('\n');
}

function generatePythonBindings() {
  const needsLiteral = Object.values(INTERFACE_DEFINITIONS).some((config) =>
    config.fields.some((field) => field.type.startsWith("'")),
  );

  const imports = [
    '"""',
    'Auto-generated contract bindings for the Uzima Python SDK.',
    '',
    'DO NOT EDIT MANUALLY — run `node scripts/generate-sdk-types.mjs` instead.',
    `Generator version: ${GENERATOR_VERSION}`,
    '"""',
    '',
    'from __future__ import annotations',
    '',
    'from dataclasses import dataclass',
    'from enum import Enum',
    'from typing import Dict, List, Optional',
  ];
  if (needsLiteral) {
    imports.push('from typing import Literal');
  }

  const sections = [...imports, '', '# ==================== Contract Enums ====================', ''];

  for (const [name, config] of Object.entries(ENUM_DEFINITIONS)) {
    sections.push(pyEnum(name, config), '');
  }

  sections.push('# ==================== Contract Dataclasses ====================', '');

  for (const [name, config] of Object.entries(INTERFACE_DEFINITIONS)) {
    sections.push(pyDataclass(name, config), '');
  }

  return `${sections.join('\n').trimEnd()}\n`;
}

function generatePythonInit() {
  const names = [
    ...Object.keys(ENUM_DEFINITIONS),
    ...Object.keys(INTERFACE_DEFINITIONS),
  ];
  return `"""Uzima Python SDK — contract bindings (auto-generated exports)."""

from .contract_bindings import (
${names.map((name) => `    ${name},`).join('\n')}
)

__all__ = [
${names.map((name) => `    "${name}",`).join('\n')}
]
`;
}

function ensureParentDir(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function writeOutputs(outputs) {
  for (const [filePath, content] of Object.entries(outputs)) {
    ensureParentDir(filePath);
    fs.writeFileSync(filePath, content, 'utf8');
  }
}

function readOrEmpty(filePath) {
  try {
    return fs.existsSync(filePath) ? fs.readFileSync(filePath, 'utf8') : '';
  } catch {
    return '';
  }
}

function buildDriftReport(diffs) {
  const lines = ['SDK_BINDINGS_DRIFT=1', `GENERATOR_VERSION=${GENERATOR_VERSION}`, ''];
  for (const diff of diffs) {
    lines.push(`FILE=${diff.path}`);
    lines.push(`STATUS=${diff.status}`);
    if (diff.diff) {
      lines.push('---DIFF---');
      lines.push(diff.diff.trimEnd());
      lines.push('---END---');
    }
    lines.push('');
  }
  return `${lines.join('\n').trimEnd()}\n`;
}

function unifiedDiff(pathLabel, before, after) {
  if (before === after) return '';
  const beforeLines = before.split('\n');
  const afterLines = after.split('\n');
  const max = Math.max(beforeLines.length, afterLines.length);
  const chunks = [`--- a/${pathLabel}`, `+++ b/${pathLabel}`];
  for (let i = 0; i < max; i++) {
    const left = beforeLines[i];
    const right = afterLines[i];
    if (left === right) continue;
    if (left !== undefined) chunks.push(`-${left}`);
    if (right !== undefined) chunks.push(`+${right}`);
  }
  return chunks.join('\n');
}

function compareOutputs(outputs) {
  const diffs = [];
  for (const [filePath, nextContent] of Object.entries(outputs)) {
    const previous = readOrEmpty(filePath);
    if (previous !== nextContent) {
      const rel = path.relative(projectRoot, filePath).replace(/\\/g, '/');
      diffs.push({
        path: rel,
        status: previous ? 'modified' : 'missing',
        diff: unifiedDiff(rel, previous, nextContent),
      });
    }
  }
  return diffs;
}

function main() {
  console.log('🚀 Uzima SDK binding generator\n');

  const outputs = {
    [OUTPUT_TS]: generateTypeScriptBindings(),
    [OUTPUT_PY]: generatePythonBindings(),
    [OUTPUT_PY_INIT]: generatePythonInit(),
  };

  const diffs = compareOutputs(outputs);

  if (REPORT_PATH) {
    ensureParentDir(REPORT_PATH);
    const report = diffs.length === 0
      ? `SDK_BINDINGS_DRIFT=0\nGENERATOR_VERSION=${GENERATOR_VERSION}\n`
      : buildDriftReport(diffs);
    fs.writeFileSync(REPORT_PATH, report, 'utf8');
    console.log(`📄 Drift report written to ${REPORT_PATH}`);
  }

  if (CHECK_MODE) {
    if (diffs.length === 0) {
      console.log('✅ SDK bindings are in sync with on-chain contract types.');
      return;
    }
    console.error(`❌ SDK bindings are out of sync (${diffs.length} file(s)):`);
    for (const diff of diffs) {
      console.error(`  - ${diff.path} (${diff.status})`);
      if (diff.diff) {
        console.error(diff.diff);
      }
    }
    console.error('\nRun `node scripts/generate-sdk-types.mjs` and commit the updated bindings.');
    process.exit(1);
  }

  writeOutputs(outputs);
  console.log(`✓ TypeScript: ${path.relative(projectRoot, OUTPUT_TS)}`);
  console.log(`✓ Python:     ${path.relative(projectRoot, OUTPUT_PY)}`);
  console.log(`✓ Python init: ${path.relative(projectRoot, OUTPUT_PY_INIT)}`);
  console.log('\n🎉 SDK binding generation complete.');
}

main();
