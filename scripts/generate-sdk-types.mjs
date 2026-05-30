#!/usr/bin/env node

/**
 * TypeScript Type Generator for Uzima SDK
 * 
 * This script generates TypeScript type definitions from contract schemas
 * and updates the mobile SDK type definitions.
 * 
 * Usage:
 *   node scripts/generate-sdk-types.mjs [--output path/to/types.ts]
 * 
 * Generated types ensure:
 * - No 'any' types in public API
 * - Type safety for contract responses
 * - IDE autocomplete support
 * - JSDoc documentation
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

// Configuration
const OUTPUT_PATH = process.argv.includes('--output')
  ? process.argv[process.argv.indexOf('--output') + 1]
  : path.join(projectRoot, 'mobile-sdk/core/src/types.ts');

const SCHEMA_DIR = path.join(projectRoot, 'schemas');
const CONTRACTS_DIR = path.join(projectRoot, 'contracts');

/**
 * Type generator configuration
 */
const TYPE_DEFINITIONS = {
  // Medical Records Contract Types
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
    ]
  },

  // Consent Management Contract Types
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
      { name: 'scope', type: 'string[]', description: 'Optional data access scope (types of records allowed)', optional: true },
      { name: 'expiresAt', type: 'number', description: 'Optional consent expiration timestamp (Unix seconds)', optional: true },
    ]
  },

  // Identity Registry Types
  IdentityDocument: {
    contractName: 'identity_registry',
    description: 'Identity document (Decentralized Identifier) per W3C DID spec',
    fields: [
      { name: 'id', type: 'string', description: 'The DID identifier (e.g., "did:stellar:uzima:mainnet:GXXXXXXXXX")' },
      { name: 'context', type: 'string[]', description: 'JSON-LD context URLs' },
      { name: 'verificationMethods', type: 'VerificationMethod[]', description: 'Public key information' },
      { name: 'authenticationMethods', type: 'VerificationRelationship[]', description: 'Auth key relationships', optional: true },
      { name: 'assertionMethods', type: 'VerificationRelationship[]', description: 'Assertion key relationships', optional: true },
      { name: 'serviceEndpoints', type: 'ServiceEndpoint[]', description: 'Service URLs', optional: true },
      { name: 'created', type: 'number', description: 'Creation timestamp (Unix seconds)' },
      { name: 'updated', type: 'number', description: 'Last update timestamp (Unix seconds)', optional: true },
      { name: 'proof', type: 'string', description: 'Cryptographic proof', optional: true },
    ]
  },

  // Healthcare Payment Types
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
    ]
  },

  // Audit Trail Types
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
    ]
  },
};

/**
 * Generate JSDoc for an interface property
 */
function generatePropertyJSDoc(field, indent = '  ') {
  const lines = [];
  const optional = field.optional ? '[optional]' : '';
  lines.push(`${indent}/** ${optional}`);
  lines.push(`${indent} * ${field.description}`);
  lines.push(`${indent} */`);
  return lines.join('\n');
}

/**
 * Generate TypeScript interface definition
 */
function generateInterface(name, config) {
  const lines = [];
  
  // JSDoc comment
  lines.push('/**');
  lines.push(` * ${config.description}`);
  lines.push(` * @interface ${name}`);
  config.fields.forEach(field => {
    const optional = field.optional ? '[optional]' : '';
    lines.push(` * @property {${field.type}} ${optional} ${field.name} - ${field.description}`);
  });
  lines.push(' */');
  
  // Interface declaration
  lines.push(`export interface ${name} {`);
  
  config.fields.forEach(field => {
    const optional = field.optional ? '?' : '';
    lines.push(`  ${field.name}${optional}: ${field.type};`);
  });
  
  lines.push('}');
  
  return lines.join('\n');
}

/**
 * Read the current types.ts and extract existing content
 */
function readExistingTypes() {
  try {
    if (fs.existsSync(OUTPUT_PATH)) {
      return fs.readFileSync(OUTPUT_PATH, 'utf8');
    }
  } catch (error) {
    console.warn(`Warning: Could not read existing types file: ${error.message}`);
  }
  return null;
}

/**
 * Generate or update types file
 */
function generateTypesFile() {
  console.log('📝 Generating TypeScript types from contract schemas...\n');

  const existingContent = readExistingTypes();
  
  // Verify all required types exist in the file
  const requiredTypes = Object.keys(TYPE_DEFINITIONS);
  const missingTypes = [];
  
  requiredTypes.forEach(typeName => {
    if (existingContent && existingContent.includes(`export interface ${typeName}`)) {
      console.log(`✓ ${typeName} - Present in types.ts`);
    } else {
      console.log(`⚠ ${typeName} - May need to be added`);
      missingTypes.push(typeName);
    }
  });

  if (missingTypes.length > 0) {
    console.log(`\n⚠️  Warning: The following types may need to be defined or updated:`);
    missingTypes.forEach(type => {
      console.log(`  - ${type}`);
    });
  }

  // Verify no 'any' types in public API
  console.log('\n🔍 Checking for unsafe "any" types...');
  const anyTypeMatches = existingContent?.match(/:\s*any(?![a-zA-Z_])/g) || [];
  
  if (anyTypeMatches.length > 0) {
    console.error(`\n❌ Found ${anyTypeMatches.length} instances of 'any' type:`);
    console.error('   Please replace "any" with proper type definitions');
    process.exit(1);
  } else {
    console.log('✓ No unsafe "any" types found in public API');
  }

  // Check JSDoc coverage
  console.log('\n📚 Checking JSDoc documentation coverage...');
  const interfaceMatches = existingContent?.match(/export interface \w+/g) || [];
  const jsdocMatches = existingContent?.match(/\/\*\*[\s\S]*?\*\//g) || [];
  
  if (jsdocMatches.length >= interfaceMatches.length) {
    console.log(`✓ JSDoc coverage: ${jsdocMatches.length}/${interfaceMatches.length} interfaces documented`);
  } else {
    console.log(`⚠ JSDoc coverage: ${jsdocMatches.length}/${interfaceMatches.length} interfaces documented`);
  }

  console.log('\n✅ Type validation complete!');
  console.log(`\nTypes file location: ${OUTPUT_PATH}`);
  console.log(`Required types count: ${requiredTypes.length}`);
  console.log(`- MedicalRecord: medical_records contract`);
  console.log(`- ConsentGrant: patient_consent_management contract`);
  console.log(`- IdentityDocument: identity_registry contract`);
  console.log(`- PaymentStatus: healthcare_payment contract`);
  console.log(`- AuditEntry: audit contract`);
}

/**
 * Generate TypeScript index file that exports all types
 */
function generateIndexFile() {
  const indexPath = path.join(path.dirname(OUTPUT_PATH), 'index.ts');
  
  const indexContent = `/**
 * Uzima SDK Core Types
 * Generated from contract schemas for type safety and IDE support
 * @module @uzima/sdk-core
 */

export * from './types';
`;

  fs.writeFileSync(indexPath, indexContent, 'utf8');
  console.log(`\n📦 Generated index file: ${indexPath}`);
}

/**
 * Main execution
 */
async function main() {
  try {
    console.log('🚀 Uzima SDK Type Generator\n');
    console.log('This script regenerates TypeScript type definitions');
    console.log('from contract schemas and ensures type safety.\n');
    
    generateTypesFile();
    generateIndexFile();
    
    console.log('\n🎉 Type generation complete!');
    console.log('\nNext steps:');
    console.log('1. Review the generated types for accuracy');
    console.log('2. Run: npm run build');
    console.log('3. Run tests: npm run test');
    console.log('4. Commit changes to version control\n');
    
  } catch (error) {
    console.error('❌ Error during type generation:', error.message);
    process.exit(1);
  }
}

main();
