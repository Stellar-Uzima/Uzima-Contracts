# Identity Verification Flow Diagrams

## W3C DID-Based Identity Verification Architecture

```mermaid
graph TD
    %% Identity Participants
    USER[User/Patient]
    ISSUER[Identity Issuer]
    VERIFIER[Service Verifier]
    REGISTRY[Identity Registry]
    
    %% Identity Contracts
    IR[Identity Registry Contract]
    CR[Credential Registry Contract]
    MFA[Multi-Factor Auth Contract]
    ZKP[Zero Knowledge Verifier]
    RBAC[Role-Based Access Control]
    
    %% External Systems
    KYC[Know Your Customer Service]
    BIO[Biometric Verification]
    GOV[Government ID Systems]
    HL7[Healthcare Systems]
    
    %% Identity Flow
    USER -->|Request DID| ISSUER
    ISSUER -->|Verify Identity| KYC
    KYC -->|Identity Check| GOV
    GOV -->|Confirmation| KYC
    KYC -->|Verified| ISSUER
    ISSUER -->|Create DID| IR
    IR -->|Register DID| REGISTRY
    
    %% Credential Issuance
    ISSUER -->|Issue Credentials| CR
    CR -->|Store Credentials| ZKP
    USER -->|Receive Credentials| MFA
    
    %% Verification Process
    USER -->|Access Request| VERIFIER
    VERIFIER -->|Verify DID| IR
    IR -->|DID Validation| RBAC
    RBAC -->|Permission Check| VERIFIER
    
    %% Multi-Factor Authentication
    VERIFIER -->|Request Auth| MFA
    MFA -->|Biometric Check| BIO
    MFA -->|Factor Verification| USER
    USER -->|Auth Factors| MFA
    MFA -->|Auth Success| VERIFIER
    
    %% Healthcare Integration
    USER -->|Medical Records| HL7
    HL7 -->|Identity Verification| IR
    IR -->|Access Grant| USER
    
    %% Styling
    classDef participant fill:#e1f5fe
    classDef contract fill:#e8f5e8
    classDef external fill:#fff3e0
    classDef identity fill:#f3e5f5
    
    class USER,ISSUER,VERIFIER,REGISTRY participant
    class IR,CR,MFA,ZKP,RBAC contract
    class KYC,BIO,GOV,HL7 external
    class identity fill:#f3e5f5
```

## Complete Identity Verification Sequence

```mermaid
sequenceDiagram
    participant U as User
    participant ISS as Identity Issuer
    participant IR as Identity Registry
    participant CR as Credential Registry
    participant MFA as Multi-Factor Auth
    participant ZKP as ZK Verifier
    participant RBAC as RBAC Contract
    participant HL7 as Healthcare System
    participant AUD as Audit Contract

    %% Step 1: DID Creation
    U->>ISS: Request Identity Creation
    ISS->>IR: Create W3C DID
    IR->>IR: Generate did:stellar:uzima:network:address
    IR->>ISS: Return DID Document
    ISS->>U: Provide DID and Keys

    %% Step 2: Credential Issuance
    U->>ISS: Submit Verification Documents
    ISS->>ISS: Verify Identity (KYC/AML)
    ISS->>CR: Issue Verifiable Credentials
    CR->>ZKP: Create ZK Proofs
    ZKP->>U: Provide Credential Proofs

    %% Step 3: MFA Setup
    U->>MFA: Register Authentication Factors
    MFA->>MFA: Setup Biometric, SMS, Hardware Token
    MFA->>U: Confirm MFA Configuration

    %% Step 4: Healthcare Access Request
    U->>HL7: Request Medical Records Access
    HL7->>IR: Verify User DID
    IR->>HL7: Return DID Validation
    HL7->>RBAC: Check User Permissions
    RBAC->>HL7: Return Permission Set

    %% Step 5: Authentication Challenge
    HL7->>MFA: Initiate Authentication
    MFA->>U: Request Multi-Factor Authentication
    U->>MFA: Provide Authentication Factors
    MFA->>MFA: Verify All Factors
    MFA->>HL7: Authentication Success

    %% Step 6: Zero-Knowledge Verification
    HL7->>ZKP: Request Credential Proof
    ZKP->>U: Request ZK Proof Generation
    U->>ZKP: Generate Privacy-Preserving Proof
    ZKP->>HL7: Verify Proof Without Revealing Data

    %% Step 7: Access Grant
    HL7->>U: Grant Access to Records
    HL7->>AUD: Log Access Event
    AUD->>IR: Update Identity Usage Stats

    %% Step 8: Ongoing Verification
    AUD->>MFA: Periodic Re-authentication
    MFA->>U: Request Fresh Authentication
    U->>MFA: Provide Updated Factors
    MFA->>AUD: Re-authentication Complete
```

## Multi-Factor Authentication Flow

```mermaid
graph TD
    %% Authentication Factors
    USER[User]
    BIO[Biometric - Fingerprint/Face]
    SMS[SMS/Email OTP]
    HW[Hardware Token/YubiKey]
    PW[Password/PIN]
    LOC[Location Verification]
    
    %% MFA Contract
    MFA[Multi-Factor Auth Contract]
    SESSION[Auth Session Manager]
    VAULT[Recovery Vault]
    AUDIT[Audit Logger]
    
    %% Factor Verification
    USER -->|Init Login| MFA
    MFA -->|Factor 1 Required| PW
    PW -->|Password Check| MFA
    
    MFA -->|Factor 2 Required| BIO
    BIO -->|Biometric Scan| USER
    USER -->|Biometric Data| BIO
    BIO -->|Verification| MFA
    
    MFA -->|Factor 3 Required| SMS
    SMS -->|Send OTP| USER
    USER -->|Enter OTP| SMS
    SMS -->|OTP Verification| MFA
    
    %% Optional Factors
    MFA -->|Optional Factor| HW
    HW -->|Hardware Token| USER
    USER -->|Token Input| HW
    HW -->|Token Verify| MFA
    
    MFA -->|Location Check| LOC
    LOC -->|GPS Verification| USER
    USER -->|Location Data| LOC
    LOC -->|Location Confirm| MFA
    
    %% Session Management
    MFA -->|Create Session| SESSION
    SESSION -->|Session Token| USER
    SESSION -->|Log Activity| AUDIT
    
    %% Recovery Process
    USER -->|Lost Access| VAULT
    VAULT -->|Recovery Process| MFA
    MFA -->|Multi-Party Recovery| SMS
    SMS -->|Recovery Codes| USER
    
    classDef user fill:#e1f5fe
    classDef factor fill:#e8f5e8
    classDef contract fill:#fff3e0
    classDef security fill:#ffebee
    
    class USER user
    class BIO,SMS,HW,PW,LOC factor
    class MFA,SESSION,VAULT,AUDIT contract
    class security fill:#ffebee
```

## Credential Verification and Revocation Flow

```mermaid
sequenceDiagram
    participant USER as User
    participant ISS as Issuer
    participant CR as Credential Registry
    participant ZKP as ZK Verifier
    participant VER as Verifier
    participant REV as Revocation Registry
    participant AUD as Audit Contract

    %% Credential Issuance
    USER->>ISS: Request Credential
    ISS->>ISS: Validate User Identity
    ISS->>CR: Issue Verifiable Credential
    CR->>CR: Store Credential Hash
    CR->>ZKP: Generate ZK Proof
    ZKP->>USER: Provide Credential Proof

    %% Credential Verification
    USER->>VER: Present Credential Proof
    VER->>ZKP: Verify ZK Proof
    ZKP->>CR: Check Credential Status
    CR->>REV: Check Revocation Status
    REV->>CR: Return Active Status
    CR->>ZKP: Credential Valid
    ZKP->>VER: Proof Verified
    VER->>USER: Access Granted

    %% Credential Revocation
    USER->>ISS: Request Revocation
    ISS->>CR: Revoke Credential
    CR->>REV: Add to Revocation List
    REV->>AUD: Log Revocation Event
    AUD->>VER: Notify Revocation

    %% Credential Renewal
    USER->>ISS: Request Renewal
    ISS->>CR: Issue New Credential
    CR->>ZKP: Update ZK Proofs
    ZKP->>USER: Provide New Proofs
```

## Healthcare Provider Identity Verification

```mermaid
graph TD
    %% Provider Participants
    DOC[Doctor/Provider]
    HOSP[Hospital/Admin]
    MED[Medical Board]
    LIC[Licensing Authority]
    DEA[DEA Registration]
    
    %% Verification Contracts
    IR[Identity Registry]
    CR[Credential Registry]
    RBAC[Role-Based Access Control]
    REP[Reputation Contract]
    AUD[Audit Contract]
    
    %% Credential Types
    MED_LICENSE[Medical License]
    DEA_CERT[DEA Certificate]
    HOSP_PRIVILEGES[Hospital Privileges]
    BOARD_CERT[Board Certification]
    
    %% Verification Flow
    DOC -->|Apply for Credentials| HOSP
    HOSP -->|Verify Employment| MED
    MED -->|License Verification| LIC
    LIC -->|License Status| MED
    MED -->|License Confirmed| HOSP
    
    HOSP -->|Request DEA Verification| DEA
    DEA -->|DEA Registration Check| DOC
    DEA -->|DEA Status| HOSP
    
    HOSP -->|Create Provider DID| IR
    IR -->|Register Provider| CR
    CR -->|Issue Credentials| DOC
    
    %% Role Assignment
    HOSP -->|Assign Roles| RBAC
    RBAC -->|Doctor Role| DOC
    RBAC -->|Access Permissions| DOC
    
    %% Reputation System
    DOC -->|Patient Interactions| REP
    REP -->|Update Reputation| DOC
    REP -->|Quality Score| HOSP
    
    %% Audit Trail
    IR -->|Identity Events| AUD
    RBAC -->|Permission Changes| AUD
    REP -->|Reputation Updates| AUD
    
    classDef person fill:#e1f5fe
    classDef authority fill:#fff3e0
    classDef contract fill:#e8f5e8
    classDef credential fill:#f3e5f5
    
    class DOC,HOSP person
    class MED,LIC,DEA authority
    class IR,CR,RBAC,REP,AUD contract
    class MED_LICENSE,DEA_CERT,HOSP_PRIVILEGES,BOARD_CERT credential
```

## Cross-Chain Identity Verification

```mermaid
sequenceDiagram
    participant STELLAR as Stellar Identity
    participant CC as Cross-Chain Bridge
    participant ETH as Ethereum Identity
    participant POL as Polygon Identity
    participant IR as Identity Registry
    participant ZKP as ZK Verifier
    participant USER as User

    %% Identity Creation on Stellar
    USER->>STELLAR: Create Stellar DID
    STELLAR->>IR: Register Identity
    IR->>IR: Store did:stellar:uzima:*

    %% Cross-Chain Identity Mapping
    USER->>CC: Request Cross-Chain Identity
    CC->>IR: Verify Stellar Identity
    IR->>CC: Identity Confirmed
    CC->>ETH: Create Ethereum Identity
    ETH->>CC: Ethereum Address Generated
    CC->>POL: Create Polygon Identity
    POL->>CC: Polygon Address Generated

    %% Identity Synchronization
    CC->>IR: Update Cross-Chain Mapping
    IR->>ZKP: Create Cross-Chain Proofs
    ZKP->>USER: Provide Unified Proofs

    %% Cross-Chain Verification
    USER->>ETH: Access Ethereum Service
    ETH->>CC: Verify Cross-Chain Identity
    CC->>IR: Check Stellar Identity
    IR->>CC: Identity Valid
    CC->>ETH: Verification Complete
    ETH->>USER: Access Granted

    %% Reputation Synchronization
    ETH->>CC: Update Reputation
    CC->>POL: Sync Reputation
    CC->>IR: Update Global Reputation
```

## Emergency Identity Recovery Flow

```mermaid
graph TD
    %% Recovery Participants
    USER[User]
    GUARDIAN1[Recovery Guardian 1]
    GUARDIAN2[Recovery Guardian 2]
    GUARDIAN3[Recovery Guardian 3]
    NOTARY[Digital Notary]
    
    %% Recovery Contracts
    IR[Identity Registry]
    MFA[Multi-Factor Auth]
    VAULT[Recovery Vault]
    GOV[Governance Contract]
    AUD[Audit Contract]
    
    %% Recovery Process
    USER -->|Lost Access| VAULT
    VAULT -->|Initiate Recovery| IR
    IR -->|Guardian Verification| GUARDIAN1
    IR -->|Guardian Verification| GUARDIAN2
    IR -->|Guardian Verification| GUARDIAN3
    
    GUARDIAN1 -->|Verify Identity| NOTARY
    GUARDIAN2 -->|Verify Identity| NOTARY
    GUARDIAN3 -->|Verify Identity| NOTARY
    
    NOTARY -->|Notarize Recovery| IR
    IR -->|Quorum Check| GOV
    GOV -->|Approve Recovery| MFA
    MFA -->|Reset Factors| USER
    
    %% New Identity Setup
    USER -->|Create New DID| IR
    IR -->|Update Registry| AUD
    AUD -->|Log Recovery| GOV
    
    %% Social Recovery Alternative
    USER -->|Social Recovery| GUARDIAN1
    GUARDIAN1 -->|Threshold Signature| GUARDIAN2
    GUARDIAN2 -->|Combine Signatures| GUARDIAN3
    GUARDIAN3 -->|Recovery Complete| IR
    
    classDef person fill:#e1f5fe
    classDef guardian fill:#e8f5e8
    classDef contract fill:#fff3e0
    classDef recovery fill:#ffebee
    
    class USER person
    class GUARDIAN1,GUARDIAN2,GUARDIAN3,NOTARY guardian
    class IR,MFA,VAULT,GOV,AUD contract
    class recovery fill:#ffebee
```

## Biometric Authentication Integration

```mermaid
graph LR
    %% Biometric Sources
    FACE[Face Recognition]
    FINGER[Fingerprint]
    VOICE[Voice Recognition]
    IRIS[Iris Scan]
    DNA[DNA Verification]
    
    %% Processing Layer
    BIO_API[Biometric API]
    ML_MODEL[ML Verification Models]
    SEC_STORE[Secure Biometric Storage]
    HASH_BIO[Biometric Hashing]
    
    %% Contract Integration
    MFA[Multi-Factor Auth]
    ZKP[Zero Knowledge Proofs]
    AUDIT[Audit Trail]
    
    %% Biometric Flow
    FACE -->|Face Data| BIO_API
    FINGER -->|Fingerprint| BIO_API
    VOICE -->|Voice Pattern| BIO_API
    IRIS -->|Iris Pattern| BIO_API
    DNA -->|DNA Sample| BIO_API
    
    BIO_API -->|Processed Data| ML_MODEL
    ML_MODEL -->|Verification Score| HASH_BIO
    HASH_BIO -->|Secure Hash| SEC_STORE
    SEC_STORE -->|Biometric Template| MFA
    
    MFA -->|Authentication Request| ZKP
    ZKP -->|Privacy Proof| AUDIT
    AUDIT -->|Auth Log| MFA
    
    %% Privacy Protection
    SEC_STORE -->|Encrypted Storage| ZKP
    ZKP -->|Zero-Knowledge Proof| MFA
    MFA -->|Verify Without Raw Data| USER
    
    classDef biometric fill:#e1f5fe
    classDef processing fill:#e8f5e8
    classDef storage fill:#fff3e0
    classDef contract fill:#f3e5f5
    
    class FACE,FINGER,VOICE,IRIS,DNA biometric
    class BIO_API,ML_MODEL,HASH_BIO processing
    class SEC_STORE storage
    class MFA,ZKP,AUDIT contract
```

## Key Identity Verification Features

### **1. W3C DID Compliance**
- **DID Method**: `did:stellar:uzima:<network>:<address>`
- **DID Documents**: Standardized identity documents
- **Verification Methods**: Multiple authentication methods
- **Service Endpoints**: Discoverable service endpoints

### **2. Multi-Factor Authentication**
- **Biometric Factors**: Fingerprint, face, voice recognition
- **Knowledge Factors**: Passwords, PINs
- **Possession Factors**: Hardware tokens, mobile devices
- **Location Factors**: Geolocation verification

### **3. Verifiable Credentials**
- **Medical Credentials**: Licenses, certifications
- **Identity Credentials**: Age, nationality verification
- **Access Credentials**: Role-based permissions
- **Revocation Support**: Dynamic credential revocation

### **4. Privacy-Preserving Verification**
- **Zero-Knowledge Proofs**: Verify without revealing data
- **Selective Disclosure**: Share only necessary information
- **Biometric Protection**: Hashed biometric templates
- **Minimal Data Collection**: Privacy-first design

### **5. Cross-Chain Identity**
- **Unified Identity**: Single identity across blockchains
- **Reputation Sync**: Cross-chain reputation transfer
- **Interoperable**: Work with multiple blockchain networks
- **Portable Identity**: Take your identity anywhere

### **6. Recovery and Backup**
- **Social Recovery**: Guardian-based recovery
- **Multi-Party Recovery**: Distributed recovery process
- **Emergency Override**: Critical access recovery
- **Secure Backup**: Encrypted identity backups

This identity verification system provides a comprehensive, secure, and privacy-preserving solution for healthcare identity management while maintaining compliance with global standards and regulations.
