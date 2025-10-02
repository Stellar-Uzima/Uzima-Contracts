# Medical Consent NFT Contract

## Overview

The Medical Consent NFT contract provides a secure, blockchain-based system for managing patient consent in healthcare settings. It allows authorized healthcare providers (issuers) to mint consent tokens for patients, track consent history, and manage consent lifecycle including updates and revocations.

### Key Features

- **NFT-based Consent Management**: Each consent is represented as a unique NFT
- **Authorized Issuers**: Only pre-approved healthcare providers can issue consent tokens
- **Consent Lifecycle Management**: Support for issuing, updating, and revoking consent
- **Audit Trail**: Complete history of all consent actions for compliance
- **Privacy-First Design**: Only metadata pointers/hashes stored on-chain, actual data stored off-chain
- **Transfer Restrictions**: Revoked consent cannot be transferred
- **Expiry Management**: Optional consent expiration timestamps

## Contract Architecture

### Storage Design

The contract uses efficient pointer/hash-based storage patterns:

- **Metadata URIs**: IPFS hashes or secure storage pointers instead of storing actual consent data on-chain
- **Hash-based Keys**: Efficient lookups using token IDs and addresses as keys
- **Minimal On-chain Data**: Only essential metadata and pointers stored on-chain

### Data Structures

#### ConsentMetadata
```rust
pub struct ConsentMetadata {
    pub metadata_uri: String,  // IPFS hash or secure storage pointer
    pub consent_type: String,  // Type of consent (treatment, research, etc.)
    pub issued_timestamp: u64, // When consent was issued
    pub expiry_timestamp: u64, // When consent expires (0 = no expiry)
    pub issuer: Address,       // Who issued the consent
    pub version: u32,          // Metadata version for updates
}
```

#### ConsentHistoryEntry
```rust
pub struct ConsentHistoryEntry {
    pub action: String, // "issued", "updated", "revoked"
    pub timestamp: u64,
    pub actor: Address,
    pub metadata_uri: String,
}
```

## Contract Functions

### Initialization & Administration

#### `initialize(admin: Address)`
Initializes the contract with an admin address.

#### `add_issuer(issuer: Address)`
Adds an authorized healthcare provider who can issue consent tokens.

#### `remove_issuer(issuer: Address)`
Removes an authorized issuer.

#### `is_issuer(address: Address) -> bool`
Checks if an address is an authorized issuer.

### Consent Management

#### `mint_consent(to: Address, metadata_uri: String, consent_type: String, expiry_timestamp: u64) -> u64`
Mints a new consent token for a patient.

#### `update_consent(token_id: u64, new_metadata_uri: String)`
Updates consent metadata (creates new version).

#### `revoke_consent(token_id: u64)`
Revokes a consent token, preventing transfers.

### Query Functions

#### `owner_of(token_id: u64) -> Address`
Returns the owner of a consent token.

#### `get_metadata(token_id: u64) -> ConsentMetadata`
Returns the metadata for a consent token.

#### `is_revoked(token_id: u64) -> bool`
Checks if a consent token is revoked.

#### `is_valid(token_id: u64) -> bool`
Checks if consent is valid (not revoked and not expired).

#### `get_history(token_id: u64) -> Vec<ConsentHistoryEntry>`
Returns the complete audit trail for a consent token.

#### `tokens_of_owner(owner: Address) -> Vec<u64>`
Returns all token IDs owned by an address.

### Transfer Functions

#### `transfer(from: Address, to: Address, token_id: u64)`
Transfers a consent token (blocked if revoked).

## Usage Examples

### 1. Contract Deployment and Initialization

```javascript
// Deploy the contract
const contractId = await deployContract('medical_consent_nft');

// Initialize with admin
const admin = 'GABC...'; // Admin address
await contract.initialize(admin);
```

### 2. Adding Healthcare Providers

```javascript
// Add authorized issuers (healthcare providers)
const hospital1 = 'GDEF...'; // Hospital address
const clinic1 = 'GHIJ...';   // Clinic address

await contract.add_issuer(hospital1);
await contract.add_issuer(clinic1);

// Verify issuer status
const isIssuer = await contract.is_issuer(hospital1);
console.log('Is authorized issuer:', isIssuer); // true
```

### 3. Issuing Consent Tokens

```javascript
// Prepare consent metadata (store off-chain first)
const consentData = {
    patientId: "P12345",
    consentType: "treatment",
    procedures: ["surgery", "anesthesia"],
    risks: ["bleeding", "infection"],
    alternatives: ["conservative treatment"],
    // ... other consent details
};

// Upload to IPFS or secure storage
const metadataUri = await uploadToIPFS(consentData);
// metadataUri = "ipfs://QmXxx..."

// Mint consent token
const patient = 'GKLM...'; // Patient address
const consentType = "treatment";
const expiryTimestamp = 0; // No expiry (0 = permanent)

const tokenId = await contract.mint_consent(
    patient,
    metadataUri,
    consentType,
    expiryTimestamp
);

console.log('Consent token minted with ID:', tokenId);
```

### 4. Querying Consent Information

```javascript
// Get token owner
const owner = await contract.owner_of(tokenId);
console.log('Token owner:', owner);

// Get consent metadata
const metadata = await contract.get_metadata(tokenId);
console.log('Consent metadata:', {
    uri: metadata.metadata_uri,
    type: metadata.consent_type,
    issued: new Date(metadata.issued_timestamp * 1000),
    issuer: metadata.issuer,
    version: metadata.version
});

// Check if consent is valid
const isValid = await contract.is_valid(tokenId);
console.log('Consent is valid:', isValid);

// Get consent history
const history = await contract.get_history(tokenId);
console.log('Consent history:', history);
```

### 5. Updating Consent

```javascript
// Update consent with new information
const updatedConsentData = {
    ...consentData,
    additionalProcedures: ["post-operative care"],
    updatedRisks: ["bleeding", "infection", "scarring"]
};

const newMetadataUri = await uploadToIPFS(updatedConsentData);

await contract.update_consent(tokenId, newMetadataUri);

// Verify update
const updatedMetadata = await contract.get_metadata(tokenId);
console.log('Updated version:', updatedMetadata.version); // Should be 2
```

### 6. Revoking Consent

```javascript
// Revoke consent
await contract.revoke_consent(tokenId);

// Verify revocation
const isRevoked = await contract.is_revoked(tokenId);
console.log('Consent revoked:', isRevoked); // true

const isValid = await contract.is_valid(tokenId);
console.log('Consent valid:', isValid); // false
```

### 7. Transferring Consent

```javascript
// Transfer consent to another party (e.g., specialist)
const specialist = 'GNOP...'; // Specialist address

await contract.transfer(patient, specialist, tokenId);

// Verify transfer
const newOwner = await contract.owner_of(tokenId);
console.log('New owner:', newOwner); // Should be specialist

// Attempt to transfer revoked consent (will fail)
await contract.revoke_consent(tokenId);
try {
    await contract.transfer(specialist, patient, tokenId);
} catch (error) {
    console.log('Transfer failed:', error.message); // ConsentRevoked error
}
```

### 8. Batch Operations

```javascript
// Get all tokens owned by a patient
const patientTokens = await contract.tokens_of_owner(patient);
console.log('Patient tokens:', patientTokens);

// Check validity of all tokens
for (const tokenId of patientTokens) {
    const isValid = await contract.is_valid(tokenId);
    const metadata = await contract.get_metadata(tokenId);
    console.log(`Token ${tokenId}: ${metadata.consent_type} - Valid: ${isValid}`);
}
```

## Events

The contract emits the following events for tracking:

- `consent_issued`: When a new consent token is minted
- `consent_updated`: When consent metadata is updated
- `consent_revoked`: When consent is revoked
- `consent_transfer`: When consent is transferred

## Security Considerations

1. **Access Control**: Only authorized issuers can mint consent tokens
2. **Authentication**: All operations require proper authentication
3. **Revocation**: Revoked consent cannot be transferred
4. **Audit Trail**: Complete history maintained for compliance
5. **Privacy**: Actual consent data stored off-chain, only pointers on-chain

## Compliance Features

- **HIPAA Considerations**: Contract designed with privacy-first approach
- **Audit Trail**: Complete history of all consent actions
- **Revocation Support**: Patients can revoke consent at any time
- **Version Control**: Metadata updates create new versions while preserving history

## Error Handling

The contract defines specific error types:

- `NotAuthorized`: Caller is not authorized for the operation
- `TokenNotFound`: Token ID does not exist
- `ConsentRevoked`: Operation attempted on revoked consent
- `AlreadyInitialized`: Contract already initialized
- `NotTokenOwner`: Caller is not the token owner

## Testing

Run the test suite:

```bash
cargo test
```

The test suite covers:
- Contract initialization
- Issuer management
- Consent minting
- Consent revocation
- Transfer restrictions
- Metadata updates

## Deployment

Deploy the contract using the provided scripts:

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Deploy using the deployment script
./scripts/deploy.sh medical_consent_nft
```

## Integration

This contract is designed to integrate with:
- Healthcare management systems
- Electronic Health Records (EHR)
- Patient portals
- Compliance monitoring systems
- Audit and reporting tools

