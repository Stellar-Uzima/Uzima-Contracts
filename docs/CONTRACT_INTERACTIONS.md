# Contract interaction diagrams

This document complements the existing visual docs by focusing specifically on **contract-to-contract interactions** and their **control/data flows**.

> All diagrams use Mermaid. See `docs/DIAGRAMS_INDEX.md` for rendering tips and standards.

## Major contracts (interaction-focused)

- **Core**: `medical_records`, `identity_registry`, `patient_consent_management`, `rbac`, `audit`
- **Security**: `mfa`, `credential_registry`, `zk_verifier`, `zkp_registry`
- **Governance/upgradeability**: `governor`, `timelock`, `upgrade_manager`
- **Payments/treasury**: `healthcare_payment`, `payment_router`, `escrow`, `appointment_booking_escrow`, `treasury_controller`
- **Cross-chain**: `cross_chain_bridge`, `cross_chain_access`, `cross_chain_identity`, `regional_node_manager`

## 1) Data flow diagrams

### Medical record write + audit + optional ZK gate

```mermaid
graph TD
    UI[Client / Portal / EMR] --> MR[medical_records]
    UI --> IR[identity_registry]
    UI --> PC[patient_consent_management]

    MR -->|write metadata| MR_STORE[(on-chain storage)]
    MR -->|store payload ref| IPFS[(off-chain: IPFS / external storage)]

    MR -->|emit events| EV[Event stream]
    MR -->|log access| AUD[audit]

    MR -. optional .-> ZK[zk_verifier]
    ZK -. attested by .-> CR[credential_registry]

    classDef core fill:#e8f5e8
    classDef sec fill:#fce4ec
    classDef ext fill:#e0f2f1
    classDef infra fill:#fff3e0

    class MR,IR,PC,AUD core
    class ZK,CR sec
    class IPFS,UI ext
    class EV,MR_STORE infra
```

### Treasury governance execution (token transfer)

```mermaid
graph LR
    GOV[governor/timelock] -->|admin auth| TREAS[treasury_controller]
    TREAS -->|invoke_contract: transfer| TOKEN[token contract]
    TREAS -->|emit GOV_EXEC| EV[Event stream]

    classDef gov fill:#fafafa
    classDef core fill:#e8f5e8
    classDef ext fill:#e0f2f1
    classDef infra fill:#fff3e0

    class GOV gov
    class TREAS core
    class TOKEN ext
    class EV infra
```

## 2) Call sequence diagrams

### Consent-gated record read (provider)

```mermaid
sequenceDiagram
    participant UI as Client/EMR
    participant IR as identity_registry
    participant RB as rbac
    participant PC as patient_consent_management
    participant MR as medical_records
    participant AUD as audit

    UI->>IR: verify identity (DID / credential)
    IR-->>UI: identity OK
    UI->>RB: check role + permission
    RB-->>UI: allowed/denied
    UI->>PC: check patient consent
    PC-->>UI: consent OK/denied
    UI->>MR: get_record(...)
    MR->>AUD: publish access log
    AUD-->>MR: logged
    MR-->>UI: encrypted record + metadata (or error)
```

### ZK attestation gating (tests demonstrate multi-contract setup)

```mermaid
sequenceDiagram
    participant Admin as Admin
    participant MR as medical_records
    participant CR as credential_registry
    participant ZK as zk_verifier
    participant Att as Attestor

    Admin->>CR: initialize + set_credential_root(...)
    Admin->>ZK: initialize + register_verifying_key(...)
    Admin->>MR: set_credential_registry_contract(CR)
    Admin->>MR: set_zk_verifier_contract(ZK)
    Admin->>MR: set_zk_enforced(true)

    Att->>ZK: submit_attestation(vk_version, pi_hash, proof_hash, verified)
    MR->>ZK: verify access proof / check attestations
    ZK-->>MR: verified/denied
```

## 3) State machine diagrams

### Consent grant lifecycle (high level)

```mermaid
stateDiagram-v2
    [*] --> NoConsent
    NoConsent --> Active : grant
    Active --> Revoked : revoke
    Active --> Expired : time passes (expiry)
    Expired --> Active : renew/grant
    Revoked --> Active : grant
```

### Treasury proposal execution (conceptual)

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Proposed : propose
    Proposed --> Approved : approvals >= threshold
    Approved --> Executed : execute
    Proposed --> Cancelled : cancel
    Approved --> Cancelled : cancel
```

## 4) Permission inheritance diagrams

```mermaid
graph TD
    Role[Role] --> Admin[Admin]
    Role --> Doctor[Doctor]
    Role --> Patient[Patient]

    Admin -->|inherits| ManageUsers[ManageUsers]
    Admin -->|inherits| ManageSystem[ManageSystem]
    Admin -->|inherits| ReadRecord[ReadRecord]
    Admin -->|inherits| ReadConfidential[ReadConfidential]

    Doctor -->|inherits| CreateRecord[CreateRecord]
    Doctor -->|inherits| ReadRecord

    Patient -->|inherits| ReadRecord

    RBAC[rbac] -->|enforces| Role
    PC[patient_consent_management] -->|additional gate| ReadRecord
    ZK[zk_verifier] -. optional .-> ReadConfidential

    classDef core fill:#e8f5e8
    classDef sec fill:#fce4ec
    classDef infra fill:#fff3e0
    class RBAC,PC core
    class ZK sec
    class Role,Admin,Doctor,Patient,ManageUsers,ManageSystem,CreateRecord,ReadRecord,ReadConfidential infra
```

## 5) Message flow diagrams

### Event emission and off-chain consumers

```mermaid
graph TB
    subgraph OnChain[On-chain]
        MR[medical_records]
        TREAS[treasury_controller]
        AUD[audit]
    end

    subgraph Events[Event stream]
        EV[(Soroban events)]
    end

    subgraph OffChain[Off-chain consumers]
        IDX[Indexer]
        MON[Monitoring/alerts]
        ETL[Analytics pipeline]
        NOTIF[Notification service]
    end

    MR --> EV
    TREAS --> EV
    AUD --> EV

    EV --> IDX
    EV --> MON
    EV --> ETL
    EV --> NOTIF
```

## Update process (how to keep diagrams correct)

When changing contract behavior or cross-contract wiring:

1. **Update code/tests first**
2. **Update diagrams** in `docs/CONTRACT_INTERACTIONS.md` and/or the specific subsystem doc
3. **Confirm diagram renders** locally (Mermaid preview)
4. **Link new diagrams** from `docs/DIAGRAMS_INDEX.md`
5. If you changed event topics/payloads, also run `npm run events:validate`

