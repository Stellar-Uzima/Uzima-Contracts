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

---

## 6) Cross-Contract Interaction Matrix

The matrix below lists every known contract-to-contract call in the system. Each row is a **caller**, each column is a **callee**. The cell value describes the purpose of the call.

| Caller → Callee | `identity_registry` | `rbac` | `patient_consent_management` | `medical_records` | `audit` | `zk_verifier` | `credential_registry` | `governor` | `timelock` | `upgrade_manager` | `treasury_controller` | `healthcare_payment` | `payment_router` | `escrow` | `cross_chain_bridge` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **`medical_records`** | verify identity | check role & permission | check patient consent | — | log record access | optional ZK proof gate | — | — | — | — | — | — | — | — | sync record hash |
| **`audit`** | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| **`patient_consent_management`** | verify patient DID | check caller role | — | — | log consent change | — | — | — | — | — | — | — | — | — | — |
| **`governor`** | — | — | — | — | — | — | — | — | queue proposal | — | — | — | — | — | — |
| **`timelock`** | — | — | — | — | — | — | — | confirm execution | — | execute upgrade | execute treasury tx | — | — | — | — |
| **`upgrade_manager`** | — | check admin role | — | — | log upgrade event | — | — | — | — | — | — | — | — | — | — |
| **`treasury_controller`** | — | check admin role | — | — | log treasury action | — | — | — | — | — | — | invoke token transfer | — | release escrow | — |
| **`healthcare_payment`** | verify payer identity | check payer role | check payment consent | — | log payment event | — | — | — | — | — | — | — | route payment | lock escrow | — |
| **`appointment_booking_escrow`** | — | check provider role | check appointment consent | — | log booking event | — | — | — | — | — | — | route refund | — | — | — |
| **`cross_chain_bridge`** | verify cross-chain DID | check bridge role | — | read/write synced records | log sync event | — | — | — | — | — | — | — | — | — | — |
| **`regional_node_manager`** | — | check node role | — | — | log node event | — | — | — | — | — | — | — | — | — | sync region data |
| **`mfa`** | verify identity | check MFA role | — | — | — | — | verify MFA credential | — | — | — | — | — | — | — | — |

> Empty cells indicate no direct on-chain call relationship. Off-chain consumers (indexers, monitoring) interact only via emitted events, not contract-to-contract calls.

---

## 7) Additional Sequence Diagrams for Common Healthcare Workflows

### Patient Registration and Identity Setup

```mermaid
sequenceDiagram
    participant P as Patient (wallet)
    participant IR as identity_registry
    participant RB as rbac
    participant MR as medical_records
    participant AUD as audit

    P->>IR: register_did(did_document, pubkey)
    IR-->>P: DID registered, identity_id returned
    P->>RB: (admin grants Patient role off-chain admin action)
    RB-->>P: role=patient assigned
    P->>MR: initialize_record(patient_id)
    MR->>IR: verify_identity(patient_address)
    IR-->>MR: identity verified
    MR->>AUD: log_event(RECORD_INITIALIZED, patient_id)
    AUD-->>MR: logged
    MR-->>P: record initialized, record_id returned
```

### Doctor Writes a Medical Record (Full Flow)

```mermaid
sequenceDiagram
    participant D as Doctor (wallet)
    participant IR as identity_registry
    participant RB as rbac
    participant PC as patient_consent_management
    participant MR as medical_records
    participant AUD as audit

    D->>IR: verify_identity(doctor_address)
    IR-->>D: identity OK
    D->>RB: has_role(doctor_address, doctor)
    RB-->>D: true
    D->>PC: check_consent(patient_id, doctor_address, WRITE)
    PC-->>D: consent granted
    D->>MR: create_record(patient_id, encrypted_data, metadata)
    MR->>IR: verify_identity(doctor_address)
    IR-->>MR: OK
    MR->>RB: has_role(doctor_address, doctor)
    RB-->>MR: true
    MR->>PC: check_consent(patient_id, doctor_address, WRITE)
    PC-->>MR: granted
    MR->>AUD: log_event(RECORD_CREATED, patient_id, doctor_address, record_id)
    AUD-->>MR: logged
    MR-->>D: record_id, ledger_sequence
```

### Patient Revokes Doctor Access

```mermaid
sequenceDiagram
    participant P as Patient (wallet)
    participant PC as patient_consent_management
    participant AUD as audit
    participant IDX as Off-chain Indexer

    P->>PC: revoke_access(patient_id, doctor_address)
    PC->>PC: require_auth(patient_address)
    PC->>AUD: log_event(CONSENT_REVOKED, patient_id, doctor_address)
    AUD-->>PC: logged
    PC-->>P: access revoked, tx confirmed
    PC-)IDX: emit event: ConsentRevoked(patient_id, doctor_address, ledger)
    IDX->>IDX: invalidate cache entry for (patient_id, doctor_address)
```

### Healthcare Payment Processing

```mermaid
sequenceDiagram
    participant PR as Provider (wallet)
    participant HP as healthcare_payment
    participant IR as identity_registry
    participant RB as rbac
    participant PC as patient_consent_management
    participant ESC as escrow
    participant TREAS as treasury_controller
    participant AUD as audit

    PR->>HP: initiate_payment(patient_id, provider_id, amount, service_code)
    HP->>IR: verify_identity(provider_id)
    IR-->>HP: OK
    HP->>RB: has_role(provider_id, doctor or nurse)
    RB-->>HP: true
    HP->>PC: check_consent(patient_id, provider_id, PAYMENT)
    PC-->>HP: consent granted
    HP->>ESC: lock_funds(payer, amount, conditions)
    ESC-->>HP: escrow_id
    HP->>AUD: log_event(PAYMENT_INITIATED, patient_id, provider_id, escrow_id)
    AUD-->>HP: logged
    HP-->>PR: escrow_id, status=PENDING

    Note over HP,TREAS: After service delivery confirmation
    PR->>HP: confirm_service_delivery(escrow_id, proof)
    HP->>ESC: release_funds(escrow_id, provider_address)
    ESC->>TREAS: transfer(provider_address, amount)
    TREAS-->>ESC: transfer confirmed
    HP->>AUD: log_event(PAYMENT_SETTLED, escrow_id, provider_id, amount)
    HP-->>PR: payment settled
```

### Contract Upgrade via Governance

```mermaid
sequenceDiagram
    participant GC as Governance Council
    participant GOV as governor
    participant TL as timelock
    participant UM as upgrade_manager
    participant AUD as audit

    GC->>GOV: propose(calldata=[upgrade_manager.upgrade(new_wasm_hash)], description_hash)
    GOV-->>GC: proposal_id, state=Created

    Note over GOV: 24h voting delay passes

    GC->>GOV: cast_vote(proposal_id, YES, weight)
    Note over GOV: 72h voting window; quorum and approval threshold met
    GOV->>GOV: state → Succeeded

    GOV->>TL: queue(proposal_id, calldata, eta=now+48h)
    TL-->>GOV: queued

    Note over TL: 48h timelock elapses

    GC->>TL: execute(proposal_id)
    TL->>UM: upgrade(new_wasm_hash)
    UM->>RB: has_role(timelock_address, admin)
    RB-->>UM: true
    UM->>AUD: log_event(CONTRACT_UPGRADED, old_hash, new_wasm_hash)
    UM-->>TL: upgrade complete
    TL-->>GC: execution confirmed, state=Executed
```

### Cross-Chain Record Synchronization

```mermaid
sequenceDiagram
    participant SRC as Source Chain Node
    participant CCB as cross_chain_bridge
    participant IR as identity_registry
    participant MR as medical_records
    participant AUD as audit
    participant RNM as regional_node_manager

    SRC->>CCB: sync_record(record_hash, source_chain_id, patient_did)
    CCB->>RNM: verify_node_authority(source_chain_id)
    RNM-->>CCB: authorized
    CCB->>IR: resolve_did(patient_did)
    IR-->>CCB: local patient_address
    CCB->>MR: write_synced_record(patient_address, record_hash, source_chain_id)
    MR->>AUD: log_event(RECORD_SYNCED, patient_address, source_chain_id, record_hash)
    AUD-->>MR: logged
    MR-->>CCB: sync confirmed
    CCB-->>SRC: ack, local_record_id
```

### Error Handling in Cross-Contract Calls

All cross-contract calls follow a consistent error propagation pattern:

```mermaid
sequenceDiagram
    participant Caller
    participant Callee
    participant AUD as audit

    Caller->>Callee: invoke(args)
    alt Success path
        Callee-->>Caller: Ok(result)
        Caller->>AUD: log_event(SUCCESS, ...)
    else Auth failure
        Callee-->>Caller: Err(AuthError)
        Caller->>AUD: log_event(AUTH_FAILED, caller, callee, function)
        Caller-->>Caller: panic with structured error code (see ERROR_CODES.md)
    else Consent denied
        Callee-->>Caller: Err(ConsentDenied)
        Caller->>AUD: log_event(CONSENT_DENIED, patient_id, requester)
        Caller-->>Caller: panic with ConsentDenied error
    else Contract logic error
        Callee-->>Caller: Err(ContractError(code))
        Caller->>AUD: log_event(CONTRACT_ERROR, callee, code)
        Caller-->>Caller: propagate error to original invoker
    end
```

**Error handling rules:**
- Every cross-contract call must be wrapped in a result check — panicking on unexpected `Ok` is not acceptable.
- Auth errors and consent denials must always be logged to the `audit` contract before propagating.
- Callers must not silently swallow errors — if a downstream call fails, the entire transaction must fail (atomicity).
- Error codes are defined in [`docs/ERROR_CODES.md`](./ERROR_CODES.md).

