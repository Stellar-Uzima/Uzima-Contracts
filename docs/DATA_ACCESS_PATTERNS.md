# Data Access Pattern Diagrams

## Healthcare Data Access Architecture

```mermaid
graph TD
    %% Data Request Layer
    subgraph "Data Requesters"
        PATIENT[Patient]
        DOCTOR[Doctor/Provider]
        RESEARCHER[Researcher]
        INSURANCE[Insurance Company]
        EMR[EMR System]
        PHARMA[Pharma Company]
    end

    %% Access Control Layer
    subgraph "Access Control System"
        RBAC[Role-Based Access Control]
        CONSENT[Patient Consent Management]
        ABE[Attribute-Based Encryption]
        ZKP[Zero Knowledge Access]
        MFA[Multi-Factor Authentication]
    end

    %% Identity Verification Layer
    subgraph "Identity Verification"
        IR[Identity Registry]
        CRED[Credential Registry]
        BIO[Biometric Verification]
        DID[W3C DID Verification]
    end

    %% Data Storage Layer
    subgraph "Data Storage"
        MR[Medical Records Contract]
        IPFS[IPFS Storage]
        ENCRYPTED[Encrypted Data Vault]
        BACKUP[Backup Storage]
        AUDIT[Audit Log Storage]
    end

    %% Data Processing Layer
    subgraph "Data Processing"
        AI[AI Analytics]
        ML[Federated Learning]
        DP[Differential Privacy]
        AGG[Data Aggregation]
    end

    %% Access Flow
    PATIENT -->|Self Access| RBAC
    DOCTOR -->|Patient Care| RBAC
    RESEARCHER -->|Research Request| RBAC
    INSURANCE -->|Claims Processing| RBAC
    EMR -->|System Integration| RBAC
    PHARMA -->|Drug Research| RBAC

    RBAC -->|Permission Check| CONSENT
    CONSENT -->|Consent Verification| ABE
    ABE -->|Decrypt Access| ZKP
    ZKP -->|Identity Verify| MFA

    MFA -->|Authentication| IR
    IR -->|Credential Check| CRED
    CRED -->|Biometric Verify| BIO
    BIO -->|DID Validation| DID

    DID -->|Access Granted| MR
    MR -->|Data Retrieval| IPFS
    IPFS -->|Encrypted Data| ENCRYPTED
    ENCRYPTED -->|Data Access| BACKUP

    MR -->|Access Log| AUDIT
    AUDIT -->|Analytics Data| AI
    AI -->|ML Processing| ML
    ML -->|Privacy Protection| DP
    DP -->|Aggregated Results| AGG

    %% Styling
    classDef requester fill:#e1f5fe
    classDef access fill:#e8f5e8
    classDef identity fill:#fff3e0
    classDef storage fill:#f3e5f5
    classDef processing fill:#fce4ec

    class PATIENT,DOCTOR,RESEARCHER,INSURANCE,EMR,PHARMA requester
    class RBAC,CONSENT,ABE,ZKP,MFA access
    class IR,CRED,BIO,DID identity
    class MR,IPFS,ENCRYPTED,BACKUP,AUDIT storage
    class AI,ML,DP,AGG processing
```

## Patient-Initiated Data Access Flow

```mermaid
sequenceDiagram
    participant PATIENT as Patient
    participant PORTAL as Patient Portal
    participant RBAC as RBAC Contract
    participant CONSENT as Consent Management
    participant IR as Identity Registry
    participant MFA as Multi-Factor Auth
    participant MR as Medical Records
    participant AUDIT as Audit Contract
    participant IPFS as IPFS Storage

    %% Step 1: Patient Login
    PATIENT->>PORTAL: Login Request
    PORTAL->>IR: Verify Patient DID
    IR->>PORTAL: DID Validated
    PORTAL->>MFA: Request MFA
    MFA->>PATIENT: Authentication Challenge
    PATIENT->>MFA: Provide Factors
    MFA->>PORTAL: Authentication Success

    %% Step 2: Data Access Request
    PORTAL->>RBAC: Request Own Data Access
    RBAC->>RBAC: Check Patient Role
    RBAC->>CONSENT: Verify Self-Access Consent
    CONSENT->>RBAC: Self-Access Granted
    RBAC->>PORTAL: Access Permission Granted

    %% Step 3: Data Retrieval
    PORTAL->>MR: Request Medical Records
    MR->>IPFS: Retrieve Encrypted Data
    IPFS->>MR: Return Encrypted Records
    MR->>PORTAL: Decrypt and Return Records

    %% Step 4: Audit Logging
    PORTAL->>AUDIT: Log Access Event
    AUDIT->>MR: Update Access Statistics
    MR->>PATIENT: Display Medical Records

    %% Step 5: Consent Management
    PATIENT->>CONSENT: Update Consent Preferences
    CONSENT->>AUDIT: Log Consent Changes
    AUDIT->>PORTAL: Confirmation of Update
```

## Provider-Initiated Data Access Flow

```mermaid
graph TD
    %% Provider Request
    PROVIDER[Healthcare Provider]
    EMR[EMR System]
    
    %% Access Verification
    RBAC[RBAC Contract]
    CRED[Credential Registry]
    LICENSE[License Verification]
    
    %% Consent Layer
    CONSENT[Patient Consent Management]
    PATIENT_APPROVAL[Patient Approval]
    EMERGENCY_OVERRIDE[Emergency Override]
    
    %% Data Access
    MR[Medical Records Contract]
    ABE[Attribute-Based Encryption]
    ZKP[Zero Knowledge Proofs]
    AUDIT[Audit Contract]
    
    %% Access Flow
    PROVIDER -->|Patient Care Request| EMR
    EMR -->|Provider Credentials| RBAC
    RBAC -->|Verify License| CRED
    CRED -->|License Status| LICENSE
    LICENSE -->|License Valid| RBAC
    
    RBAC -->|Check Consent| CONSENT
    CONSENT -->|Patient Approval| PATIENT_APPROVAL
    PATIENT_APPROVAL -->|Consent Granted| CONSENT
    
    %% Emergency Path
    CONSENT -->|Emergency Situation| EMERGENCY_OVERRIDE
    EMERGENCY_OVERRIDE -->|Immediate Access| RBAC
    
    %% Data Retrieval
    RBAC -->|Access Granted| MR
    MR -->|Decrypt Access| ABE
    ABE -->|Privacy Verification| ZKP
    ZKP -->|Data Access| AUDIT
    AUDIT -->|Log Access| PROVIDER
    
    %% Styling
    classDef provider fill:#e1f5fe
    classDef verification fill:#e8f5e8
    classDef consent fill:#fff3e0
    classDef data fill:#f3e5f5
    
    class PROVIDER,EMR provider
    class RBAC,CRED,LICENSE verification
    class CONSENT,PATIENT_APPROVAL,EMERGENCY_OVERRIDE consent
    class MR,ABE,ZKP,AUDIT data
```

## Research Data Access with Privacy Protection

```mermaid
sequenceDiagram
    participant RESEARCHER as Researcher
    participant IRB as IRB Review Board
    participant RBAC as RBAC Contract
    participant CONSENT as Consent Management
    participant DP as Differential Privacy
    participant ML as Federated Learning
    participant MR as Medical Records
    participant AUDIT as Audit Contract
    participant PATIENT as Patient

    %% Step 1: Research Request
    RESEARCHER->>IRB: Submit Research Proposal
    IRB->>IRB: Review and Approve
    IRB->>RBAC: Grant Researcher Role
    RBAC->>RESEARCHER: Research Credentials

    %% Step 2: Patient Consent Collection
    RESEARCHER->>CONSENT: Request Patient Data Access
    CONSENT->>PATIENT: Request Research Consent
    PATIENT->>CONSENT: Grant Research Consent
    CONSENT->>RBAC: Update Consent Records

    %% Step 3: Privacy-Preserving Access
    RESEARCHER->>RBAC: Request Anonymized Data
    RBAC->>DP: Apply Differential Privacy
    DP->>ML: Prepare Federated Learning
    ML->>MR: Access Privacy-Protected Data
    MR->>DP: Return Protected Data
    DP->>RESEARCHER: Provide Anonymized Dataset

    %% Step 4: Audit and Compliance
    RESEARCHER->>AUDIT: Log Research Access
    AUDIT->>CONSENT: Verify Consent Compliance
    CONSENT->>PATIENT: Notify Data Usage
    AUDIT->>IRB: Report Research Activity

    %% Step 5: Results Processing
    RESEARCHER->>ML: Submit Analysis Results
    ML->>DP: Apply Privacy Protection
    DP->>AUDIT: Log Research Results
    AUDIT->>RESEARCHER: Publication Approval
```

## Insurance Claims Data Access Pattern

```mermaid
graph TD
    %% Insurance Participants
    INSURANCE[Insurance Company]
    ADJUSTER[Claims Adjuster]
    BILLING[Billing Department]
    
    %% Access Control
    RBAC[RBAC Contract]
    CRED[Credential Registry]
    COMPLIANCE[Compliance Verification]
    
    %% Data Access
    CONSENT[Patient Consent]
    MR[Medical Records]
    HL7[HL7/FHIR Gateway]
    AUDIT[Audit Contract]
    
    %% Claims Processing
    INSURANCE -->|Claims Processing| RBAC
    RBAC -->|Verify Credentials| CRED
    CRED -->|Insurance License| COMPLIANCE
    COMPLIANCE -->|Compliance Check| RBAC
    
    RBAC -->|Assign Adjuster| ADJUSTER
    ADJUSTER -->|Review Claim| BILLING
    BILLING -->|Medical Necessity| CONSENT
    
    CONSENT -->|Patient Authorization| MR
    MR -->|Standard Format| HL7
    HL7 -->|FHIR Resources| AUDIT
    AUDIT -->|Claims Data| INSURANCE
    
    %% Fraud Detection
    INSURANCE -->|Anomaly Detection| AUDIT
    AUDIT -->|Pattern Analysis| BILLING
    BILLING -->|Suspicious Activity| ADJUSTER
    
    classDef insurance fill:#e1f5fe
    classDef control fill:#e8f5e8
    classDef data fill:#fff3e0
    classDef processing fill:#f3e5f5
    
    class INSURANCE,ADJUSTER,BILLING insurance
    class RBAC,CRED,COMPLIANCE control
    class CONSENT,MR,HL7,AUDIT data
    class processing fill:#f3e5f5
```

## Emergency Access Override Pattern

```mermaid
sequenceDiagram
    participant EMERGENCY as Emergency Situation
    participant PROVIDER as Emergency Provider
    participant EAO as Emergency Access Override
    participant RBAC as RBAC Contract
    patient_consent_management as Patient Consent Management
    participant MR as Medical Records
    participant AUDIT as Audit Contract
    participant GOV as Governance Contract
    participant PATIENT as Patient (if available)

    %% Step 1: Emergency Declaration
    EMERGENCY->>PROVIDER: Medical Emergency
    PROVIDER->>EAO: Declare Emergency Access
    EAO->>GOV: Request Emergency Override
    GOV->>EAO: Emergency Override Granted

    %% Step 2: Bypass Normal Controls
    EAO->>RBAC: Override Access Controls
    RBAC->>patient_consent_management: Bypass Consent Check
    patient_consent_management->>EAO: Emergency Access Granted

    %% Step 3: Immediate Data Access
    EAO->>MR: Emergency Record Access
    MR->>EAO: Full Medical History
    EAO->>PROVIDER: Critical Patient Data

    %% Step 4: Emergency Treatment
    PROVIDER->>EMERGENCY: Provide Emergency Care
    EMERGENCY->>PROVIDER: Patient Stabilized

    %% Step 5: Post-Emergency Processing
    PROVIDER->>AUDIT: Log Emergency Access
    AUDIT->>GOV: Report Emergency Usage
    GOV->>PATIENT: Notify Emergency Access
    PATIENT->>patient_consent_management: Review Emergency Access

    %% Step 6: Audit and Review
    AUDIT->>EAO: Emergency Access Review
    EAO->>GOV: Justification Report
    GOV->>EAO: Emergency Access Validated
```

## Cross-Chain Data Access Pattern

```mermaid
graph TD
    %% Multi-Chain Access Request
    USER[User/Provider]
    CC_ACCESS[Cross-Chain Access]
    
    %% Chain-Specific Access
    STELLAR_RBAC[Stellar RBAC]
    ETH_RBAC[Ethereum RBAC]
    POL_RBAC[Polygon RBAC]
    
    %% Chain-Specific Data
    STELLAR_MR[Stellar Medical Records]
    ETH_MR[Ethereum Medical Records]
    POL_MR[Polygon Medical Records]
    
    %% Cross-Chain Coordination
    CC_BRIDGE[Cross-Chain Bridge]
    VALIDATOR[Validator Network]
    AUDIT[Unified Audit]

    %% Access Flow
    USER -->|Cross-Chain Request| CC_ACCESS
    CC_ACCESS -->|Check Stellar| STELLAR_RBAC
    CC_ACCESS -->|Check Ethereum| ETH_RBAC
    CC_ACCESS -->|Check Polygon| POL_RBAC
    
    STELLAR_RBAC -->|Stellar Permission| STELLAR_MR
    ETH_RBAC -->|Ethereum Permission| ETH_MR
    POL_RBAC -->|Polygon Permission| POL_MR
    
    %% Cross-Chain Data Retrieval
    STELLAR_MR -->|Primary Data| CC_BRIDGE
    ETH_MR -->|Mirror Data| CC_BRIDGE
    POL_MR -->|Cache Data| CC_BRIDGE
    
    CC_BRIDGE -->|Validate Data| VALIDATOR
    VALIDATOR -->|Confirmed Data| CC_ACCESS
    CC_ACCESS -->|Unified Access| USER
    
    %% Unified Auditing
    CC_ACCESS -->|Log All Access| AUDIT
    AUDIT -->|Cross-Chain Trail| CC_BRIDGE

    classDef user fill:#e1f5fe
    classDef access fill:#e8f5e8
    classDef chain fill:#fff3e0
    classDef data fill:#f3e5f5
    classDef bridge fill:#fce4ec

    class USER user
    class CC_ACCESS,STELLAR_RBAC,ETH_RBAC,POL_RBAC access
    class STELLAR_MR,ETH_MR,POL_MR data
    class CC_BRIDGE,VALIDATOR,AUDIT bridge
```

## API-Based Data Access Pattern

```mermaid
sequenceDiagram
    participant API as External API
    participant GATEWAY as API Gateway
    participant RBAC as RBAC Contract
    participant AUTH as Authentication Service
    participant RATE as Rate Limiter
    participant MR as Medical Records
    participant AUDIT as Audit Contract
    participant CACHE as Cache Layer

    %% Step 1: API Request
    API->>GATEWAY: Data Access Request
    GATEWAY->>AUTH: Authenticate API Key
    AUTH->>GATEWAY: Authentication Valid
    GATEWAY->>RATE: Check Rate Limits
    RATE->>GATEWAY: Rate Limit OK

    %% Step 2: Authorization
    GATEWAY->>RBAC: Check API Permissions
    RBAC->>RBAC: Validate API Role
    RBAC->>GATEWAY: API Access Granted

    %% Step 3: Data Retrieval
    GATEWAY->>CACHE: Check Cache
    alt Cache Hit
        CACHE->>GATEWAY: Return Cached Data
    else Cache Miss
        GATEWAY->>MR: Request Fresh Data
        MR->>GATEWAY: Return Medical Records
        GATEWAY->>CACHE: Update Cache
    end

    %% Step 4: Response
    GATEWAY->>API: Formatted Data Response
    API->>API: Process Data

    %% Step 5: Audit and Monitoring
    GATEWAY->>AUDIT: Log API Access
    AUDIT->>RATE: Update Usage Statistics
    RATE->>AUTH: Update Authentication Metrics
```

## Data Access with Zero-Knowledge Proofs

```mermaid
graph TD
    %% ZKP Participants
    REQUESTER[Data Requester]
    VERIFIER[ZK Verifier]
    PROVER[ZK Prover]
    
    %% Privacy Layer
    ZKP_SYSTEM[Zero Knowledge System]
    CIRCUIT[ZK Circuit]
    WITNESS[Witness Data]
    
    %% Data Layer
    MR[Medical Records]
    ENCRYPTED[Encrypted Data]
    AUDIT[Audit Trail]
    
    %% ZKP Flow
    REQUESTER -->|Access Request| VERIFIER
    VERIFIER -->|Generate Challenge| ZKP_SYSTEM
    ZKP_SYSTEM -->|Create Circuit| CIRCUIT
    CIRCUIT -->|Compute Proof| PROVER
    PROVER -->|Use Witness| WITNESS
    WITNESS -->|Privacy Proof| ZKP_SYSTEM
    
    %% Verification Without Disclosure
    ZKP_SYSTEM -->|Verify Proof| VERIFIER
    VERIFIER -->|Proof Valid| MR
    MR -->|Access Without Revealing| ENCRYPTED
    ENCRYPTED -->|Selective Disclosure| AUDIT
    AUDIT -->|Access Log| REQUESTER
    
    %% Privacy Properties
    PROVER -->|No Data Leakage| REQUESTER
    VERIFIER -->|Mathematical Certainty| ZKP_SYSTEM
    CIRCUIT -->|Custom Logic| MR
    
    classDef participant fill:#e1f5fe
    classDef zkp fill:#e8f5e8
    classDef privacy fill:#fff3e0
    classDef data fill:#f3e5f5
    
    class REQUESTER,VERIFIER,PROVER participant
    class ZKP_SYSTEM,CIRCUIT,WITNESS zkp
    class privacy fill:#fff3e0
    class MR,ENCRYPTED,AUDIT data
```

## Key Data Access Patterns

### **1. Role-Based Access Control (RBAC)**
- **Hierarchical Permissions**: Doctor > Nurse > Admin > Patient
- **Scope Limitation**: Access only relevant medical records
- **Time-Based Access**: Temporary access for specific periods
- **Audit Trail**: Complete access logging

### **2. Consent-Driven Access**
- **Explicit Consent**: Patient must grant access
- **Granular Control**: Specific data type permissions
- **Revocation**: Dynamic consent withdrawal
- **Emergency Override**: Life-threatening situations

### **3. Privacy-Preserving Access**
- **Zero-Knowledge Proofs**: Verify without revealing data
- **Differential Privacy**: Statistical privacy protection
- **Data Anonymization**: Remove personal identifiers
- **Federated Learning**: Learn without data movement

### **4. Cross-Chain Access**
- **Unified Identity**: Single identity across chains
- **Permission Mapping**: Translate between chain permissions
- **Data Synchronization**: Consistent access across networks
- **Fallback Mechanisms**: Alternative access paths

### **5. Emergency Access**
- **Immediate Override**: Bypass normal controls
- **Time-Limited**: Temporary emergency access
- **Post-Review**: Audit after emergency
- **Multi-Party Approval**: Guardian-based verification

### **6. API-Based Access**
- **Rate Limiting**: Prevent abuse
- **Authentication**: Secure API key management
- **Caching**: Improve performance
- **Monitoring**: Real-time access tracking

These data access patterns provide a comprehensive, secure, and privacy-preserving framework for healthcare data access while maintaining regulatory compliance and patient control over their medical information.
