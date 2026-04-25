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

## Advanced Features

The contract now includes advanced consent management features for enhanced control, security, and compliance:

### 1. Dynamic Consent Updates and Versioning

The contract supports dynamic consent updates with full version history tracking.

#### `enable_dynamic_updates(token_id: u64)`
Enables dynamic updates for a consent token. Only the patient can enable this feature.

#### `update_consent_dynamic(token_id: u64, new_metadata_uri: String, change_summary: String)`
Updates consent with version tracking. Creates a new version entry in the version history.

#### `get_version_history(token_id: u64) -> Vec<VersionHistoryEntry>`
Returns the complete version history for a consent token.

**Example:**
```javascript
// Enable dynamic updates
await contract.enable_dynamic_updates(tokenId);

// Update consent with change summary
await contract.update_consent_dynamic(
    tokenId,
    newMetadataUri,
    "Updated treatment plan based on new diagnosis"
);

// Get version history
const history = await contract.get_version_history(tokenId);
console.log('Version history:', history);
```

### 2. Granular Permission System

Fine-grained permissions for different data types within a consent.

#### Data Types
- `Demographics`: Basic patient information
- `MedicalHistory`: Medical history records
- `LabResults`: Laboratory test results
- `Imaging`: Medical imaging data
- `Medications`: Medication records
- `Procedures`: Medical procedures
- `Allergies`: Allergy information
- `Research`: Research data
- `Financial`: Financial/insurance information

#### Permission Levels
- `None`: No access
- `Read`: Read-only access
- `Write`: Read and write access
- `Full`: Full access including deletion

#### `set_granular_permissions(token_id: u64, permissions: GranularPermissions)`
Sets granular permissions for different data types. Can be called by patient or issuer.

#### `get_granular_permissions(token_id: u64) -> GranularPermissions`
Retrieves the granular permissions for a consent token.

#### `has_permission(token_id: u64, requester: Address, data_type: DataType, required_level: PermissionLevel) -> bool`
Checks if a requester has the required permission level for a specific data type.

**Example:**
```javascript
// Set permissions
const permissions = {
    permissions: new Map([
        [DataType.LabResults, PermissionLevel.Read],
        [DataType.MedicalHistory, PermissionLevel.Write],
        [DataType.Financial, PermissionLevel.None]
    ])
};
await contract.set_granular_permissions(tokenId, permissions);

// Check permission
const hasAccess = await contract.has_permission(
    tokenId,
    doctorAddress,
    DataType.LabResults,
    PermissionLevel.Read
);
```

### 3. Time-Based and Condition-Based Access Controls

Advanced access controls with time windows, day restrictions, and usage limits.

#### Access Condition Types
- `TimeWindow`: Access allowed only within a specific time range
- `DayOfWeek`: Access restricted to specific days of the week
- `TimeOfDay`: Access restricted to specific hours of the day
- `LocationBased`: Location-based access restrictions
- `PurposeBased`: Purpose-based access restrictions
- `EmergencyOnly`: Access only through emergency override

#### `set_access_controls(token_id: u64, access_control: AccessControl)`
Sets access controls for a consent token. Only the patient can set access controls.

#### `check_access_allowed(token_id: u64, requester: Address) -> bool`
Checks if access is currently allowed based on all access control conditions.

#### `record_access(token_id: u64, requester: Address)`
Records an access attempt and updates access count.

**Example:**
```javascript
// Set time-based access control
const accessControl = {
    conditions: [
        {
            type: AccessCondition.TimeWindow,
            start: currentTimestamp,
            end: currentTimestamp + 86400 // 1 day
        },
        {
            type: AccessCondition.DayOfWeek,
            days: [1, 2, 3, 4, 5] // Monday to Friday
        }
    ],
    max_access_count: 10,
    current_access_count: 0,
    last_access_timestamp: 0
};
await contract.set_access_controls(tokenId, accessControl);

// Check if access is allowed
const allowed = await contract.check_access_allowed(tokenId, requester);
if (allowed) {
    await contract.record_access(tokenId, requester);
}
```

### 4. Consent Inheritance and Delegation

Support for delegating consent to other parties and creating consent hierarchies.

#### `delegate_consent(token_id: u64, delegate: Address, permissions: GranularPermissions, expiry_timestamp: u64)`
Delegates consent to another address with specific permissions. Only the patient can delegate.

#### `revoke_delegation(token_id: u64, delegate: Address)`
Revokes a delegation. Only the patient can revoke delegations.

#### `get_delegations(token_id: u64) -> Vec<Delegation>`
Returns all active delegations for a consent token.

#### `set_inheritance(child_token_id: u64, parent_token_id: u64, inherited_permissions: GranularPermissions)`
Sets up consent inheritance where a child consent inherits permissions from a parent. Includes cycle detection.

**Example:**
```javascript
// Delegate consent to a specialist
const delegatePermissions = {
    permissions: new Map([
        [DataType.LabResults, PermissionLevel.Read],
        [DataType.Imaging, PermissionLevel.Read]
    ])
};
await contract.delegate_consent(
    tokenId,
    specialistAddress,
    delegatePermissions,
    currentTimestamp + 2592000 // 30 days
);

// Get active delegations
const delegations = await contract.get_delegations(tokenId);

// Set up inheritance (child consent inherits from parent)
await contract.set_inheritance(childTokenId, parentTokenId, inheritedPermissions);
```

### 5. Emergency Override Mechanisms

Emergency access with full audit trails for life-threatening situations.

#### `add_emergency_authority(authority: Address)`
Adds an authorized emergency override authority. Only admin can add authorities.

#### `emergency_override(token_id: u64, reason: String, duration: u64) -> u64`
Creates an emergency override with full audit trail. Returns override ID.

**Example:**
```javascript
// Add emergency authority (admin only)
await contract.add_emergency_authority(hospitalEmergencyDept);

// Create emergency override
const overrideId = await contract.emergency_override(
    tokenId,
    "Life-threatening emergency - patient unconscious",
    3600 // 1 hour override
);
```

### 6. Consent Marketplace for Research

Marketplace functionality for patients to share their data for research purposes.

#### `set_marketplace_enabled(enabled: bool)`
Enables or disables the marketplace feature. Admin only.

#### `list_on_marketplace(token_id: u64, price: i128, data_types: Vec<DataType>, research_purpose: String, duration: u64)`
Lists a consent token on the marketplace for research purposes. Only the patient can list.

#### `get_marketplace_listing(token_id: u64) -> MarketplaceListing`
Retrieves marketplace listing information.

#### `purchase_marketplace_listing(token_id: u64, buyer: Address)`
Purchases access to a marketplace listing. Creates a delegation for the buyer.

**Example:**
```javascript
// Enable marketplace (admin)
await contract.set_marketplace_enabled(true);

// List consent for research
await contract.list_on_marketplace(
    tokenId,
    1000, // Price in tokens
    [DataType.LabResults, DataType.MedicalHistory],
    "Diabetes research study",
    2592000 // 30 days access
);

// Purchase listing
await contract.purchase_marketplace_listing(tokenId, researcherAddress);
```

### 7. Consent Analytics and Reporting

Comprehensive analytics and reporting capabilities.

#### `get_analytics() -> AnalyticsData`
Returns aggregated analytics data including:
- Total consents issued
- Active consents
- Revoked consents
- Total delegations
- Total emergency overrides
- Marketplace listings
- Total access count

#### `generate_consent_report(patient: Address) -> Vec<u64>`
Generates a report of all consent tokens for a patient.

**Example:**
```javascript
// Get analytics
const analytics = await contract.get_analytics();
console.log('Total consents:', analytics.total_consents);
console.log('Active consents:', analytics.active_consents);
console.log('Total delegations:', analytics.total_delegations);

// Generate patient report
const report = await contract.generate_consent_report(patientAddress);
console.log('Patient has', report.length, 'consent tokens');
```

## Enhanced Data Structures

### ConsentMetadata (Enhanced)
```rust
pub struct ConsentMetadata {
    pub metadata_uri: String,
    pub consent_type: String,
    pub issued_timestamp: u64,
    pub expiry_timestamp: u64,
    pub issuer: Address,
    pub patient: Address,
    pub version: u32,
    pub dynamic_updates_enabled: bool, // New field
}
```

### ConsentHistoryEntry (Enhanced)
```rust
pub struct ConsentHistoryEntry {
    pub action: String, // Now includes: "issued", "updated", "revoked", "delegated", "inherited", "emergency_override", "marketplace_listed", etc.
    pub timestamp: u64,
    pub actor: Address,
    pub metadata_uri: String,
    pub details: String, // New field for additional details
}
```

## Security Considerations

### Advanced Security Features

1. **Granular Access Control**: Fine-grained permissions prevent unauthorized access to specific data types
2. **Time-Based Restrictions**: Access can be restricted to specific time windows
3. **Emergency Protocols**: Controlled emergency access with full audit trails
4. **Delegation Management**: Secure delegation with expiry and revocation
5. **Inheritance Protection**: Cycle detection prevents infinite inheritance loops
6. **Marketplace Security**: Controlled marketplace with patient-only listing

### Compliance Features

- **Complete Audit Trail**: Every action is recorded with timestamp, actor, and details
- **Version History**: Full version history for dynamic consent updates
- **Access Logging**: All access attempts are logged
- **Emergency Documentation**: Emergency overrides include reason and duration
- **Analytics**: Comprehensive analytics for compliance reporting

## Best Practices

1. **Enable Dynamic Updates**: Only enable dynamic updates when necessary for flexibility
2. **Set Granular Permissions**: Use granular permissions to minimize data exposure
3. **Use Access Controls**: Implement time-based and condition-based access controls for sensitive data
4. **Monitor Analytics**: Regularly review analytics for compliance and security
5. **Emergency Protocols**: Establish clear emergency override procedures
6. **Marketplace Guidelines**: Only list data for legitimate research purposes

## Testing

The test suite now includes tests for all advanced features:

```bash
cargo test
```

Test coverage includes:
- Granular permissions
- Access controls
- Delegation and inheritance
- Emergency overrides
- Marketplace functionality
- Dynamic updates
- Analytics and reporting
