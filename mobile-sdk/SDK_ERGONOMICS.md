# SDK Client Ergonomics Standard

## Overview

This document defines the standardized client ergonomics patterns across all
Uzima SDK implementations (Rust, TypeScript, Python). All SDK clients MUST
follow these conventions to ensure a consistent developer experience.

## Core Principles

1. **Uniform initialization**: All SDKs use a builder/config pattern
2. **Consistent error handling**: Result types with typed errors
3. **Lazy initialization**: Resources allocated on first use
4. **Async-first**: All I/O operations are async
5. **Drop/dispose cleanup**: Automatic resource cleanup

## Client Interface Contract

### Constructor Pattern

All SDKs must accept a configuration object/struct:

`ust
// Rust
let client = UzimaClient::new(UzimaConfig { ... })?;
`

`	ypescript
// TypeScript
const client = new UzimaClient({ ... });
`

`python
# Python
client = UzimaClient(config=UzimaConfig(...))
`

### Method Naming Conventions

| Operation | Rust | TypeScript | Python |
|-----------|------|-----------|--------|
| Create | create_record() | createRecord() | create_record() |
| Read | ead_record() | eadRecord() | ead_record() |
| Update | update_record() | updateRecord() | update_record() |
| Delete | delete_record() | deleteRecord() | delete_record() |
| List | list_records() | listRecords() | list_records() |
| Count | count_records() | countRecords() | count_records() |

### Error Handling

Rust uses Result<T, UzimaError>, TypeScript uses Promise<T>, Python uses
exceptions. All SDKs must map errors to a common error taxonomy.

### Authentication

All SDKs must support:
1. Key pair authentication
2. Session token authentication
3. Biometric authentication (where available)

### Offline Support

All SDKs must provide:
1. Offline queue for write operations
2. Cache for read operations
3. Sync mechanism for pending operations

## Configuration Schema

All SDKs share the same configuration shape:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| apiEndpoint | string | yes | - | Backend API URL |
| contractId | string | yes | - | Soroban contract ID |
| networkPassphrase | string | yes | - | Stellar network passphrase |
| serverURL | string | yes | - | Soroban RPC URL |
| encryptionKey | string | no | null | Data at rest encryption key |
| offlineEnabled | boolean | no | false | Enable offline mode |
| notificationsEnabled | boolean | no | false | Enable push notifications |
| biometricEnabled | boolean | no | false | Enable biometric auth |
| requestTimeout | number | no | 30000 | HTTP timeout (ms) |
| cacheEnabled | boolean | no | true | Enable response caching |
| cacheTTL | number | no | 300000 | Cache TTL (ms) |

## Testing Requirements

All SDK implementations must include:
1. Unit tests for each manager class
2. Integration tests with mock backend
3. End-to-end tests with testnet
4. Performance benchmarks for critical paths