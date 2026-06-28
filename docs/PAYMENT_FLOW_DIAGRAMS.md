# Payment Flow Diagrams

## Healthcare Payment Processing Flow

```mermaid
graph TD
    %% Participants
    P[Patient]
    I[Insurance Company]
    D[Doctor/Provider]
    H[Hospital]
    PH[Pharmacy]
    
    %% Payment Contracts
    PR[Payment Router Contract]
    ESC[Escrow Contract]
    SUT[SUT Token Contract]
    REP[Reputation Contract]
    ABE[ABE Access Control]
    
    %% Financial Systems
    BANK[Bank/Gateway]
    STELLAR[Stellar Network]
    TOKEN[Token Issuer]
    
    %% Payment Flow
    P -->|Insurance Claim| I
    I -->|Pre-authorization| PR
    PR -->|Route Payment| ESC
    ESC -->|Hold Funds| STELLAR
    STELLAR -->|Token Transfer| SUT
    
    %% Service Delivery
    D -->|Provide Service| P
    H -->|Treatment| P
    PH -->|Medication| P
    
    %% Payment Release
    D -->|Invoice| PR
    H -->|Billing| PR
    PH -->|Claim| PR
    
    %% Fee Distribution
    PR -->|Platform Fee| ABE
    PR -->|Provider Payment| D
    PR -->|Hospital Payment| H
    PR -->|Pharmacy Payment| PH
    
    %% Reputation Updates
    PR -->|Payment Success| REP
    REP -->|Update Reputation| D
    REP -->|Update Reputation| H
    
    %% Styling
    classDef participant fill:#e1f5fe
    classDef contract fill:#e8f5e8
    classDef system fill:#fff3e0
    classDef token fill:#f3e5f5
    
    class P,I,D,H,PH participant
    class PR,ESC,SUT,REP,ABE contract
    class BANK,STELLAR,TOKEN system
```

## Detailed Payment Transaction Flow

```mermaid
sequenceDiagram
    participant P as Patient
    participant I as Insurance
    participant PR as Payment Router
    participant ESC as Escrow
    participant D as Doctor
    participant REP as Reputation
    participant AUD as Audit
    participant STELLAR as Stellar Network

    %% Step 1: Service Pre-authorization
    P->>I: Request Coverage
    I->>PR: Check Provider Network
    PR->>I: Return Provider Status
    I->>P: Approve Coverage

    %% Step 2: Service Delivery
    P->>D: Receive Treatment
    D->>PR: Submit Pre-claim
    PR->>ESC: Create Escrow
    ESC->>STELLAR: Lock Funds
    
    %% Step 3: Claim Processing
    D->>PR: Submit Final Claim
    PR->>I: Verify Insurance Details
    I->>PR: Approve Payment Amount
    PR->>ESC: Release Payment Conditions
    
    %% Step 4: Payment Distribution
    ESC->>PR: Execute Payment Split
    PR->>D: Transfer Provider Payment
    PR->>REP: Update Provider Reputation
    PR->>AUD: Log Transaction
    
    %% Step 5: Confirmation
    D->>P: Send Receipt
    P->>I: Confirm Service Received
    I->>PR: Close Claim
```

## Escrow-Based Appointment Booking Flow

```mermaid
graph TD
    %% Participants
    PAT[Patient]
    DOC[Doctor]
    HOSP[Hospital]
    
    %% Contracts
    ABES[Appointment Booking Escrow]
    PR[Payment Router]
    SUT[SUT Token]
    CAL[Calendar Contract]
    NOT[Notification Contract]
    
    %% Process Flow
    PAT -->|Book Appointment| CAL
    CAL -->|Check Availability| DOC
    DOC -->|Confirm Slot| CAL
    CAL -->|Create Booking| ABES
    
    %% Escrow Process
    PAT -->|Deposit Payment| ABES
    ABES -->|Lock Funds| SUT
    ABES -->|Notify Doctor| NOT
    NOT -->|Appointment Confirmed| DOC
    
    %% Service Completion
    DOC -->|Provide Service| PAT
    DOC -->|Mark Complete| ABES
    ABES -->|Release Payment| PR
    PR -->|Pay Doctor| DOC
    PR -->|Pay Platform Fee| HOSP
    
    %% No-show Handling
    PAT -->|Cancel/No-show| ABES
    ABES -->|Apply Penalty| PAT
    ABES -->|Partial Refund| PAT
    ABES -->|Compensate Doctor| DOC
    
    classDef person fill:#e1f5fe
    classDef contract fill:#e8f5e8
    classDef token fill:#f3e5f5
    
    class PAT,DOC,HOSP person
    class ABES,PR,CAL,NOT contract
    class SUT token
```

## Cross-Chain Payment Settlement Flow

```mermaid
sequenceDiagram
    participant STELLAR as Stellar Network
    participant PR as Payment Router
    participant CC as Cross-Chain Bridge
    participant ETH as Ethereum
    participant POL as Polygon
    participant BANK as Banking System

    %% Multi-chain Payment Initiation
    STELLAR->>PR: Initiate Payment
    PR->>PR: Calculate Route
    PR->>CC: Bridge Transfer Request
    
    %% Cross-Chain Routing
    CC->>ETH: Route to Ethereum
    ETH->>CC: Confirmation
    CC->>POL: Route to Polygon
    POL->>CC: Confirmation
    
    %% Settlement
    CC->>PR: All Routes Complete
    PR->>BANK: Final Settlement
    BANK->>PR: Settlement Confirmation
    PR->>STELLAR: Update Payment Status
```

## Insurance Claim Processing Flow

```mermaid
graph TD
    %% Claim Participants
    PAT[Patient]
    DOC[Doctor]
    INS[Insurance Company]
    BILL[Billing Department]
    ADJ[Claims Adjuster]
    
    %% Processing Contracts
    PR[Payment Router]
    ESC[Escrow Contract]
    VEF[Verification Contract]
    AUD[Audit Contract]
    REP[Reputation Contract]
    
    %% Claim Flow
    PAT -->|Receive Treatment| DOC
    DOC -->|Submit Claim| BILL
    BILL -->|Process Claim| INS
    INS -->|Assign Adjuster| ADJ
    
    %% Verification Process
    ADJ -->|Verify Services| VEF
    VEF -->|Check Medical Necessity| DOC
    DOC -->|Provide Documentation| VEF
    VEF -->|Validate Claim| INS
    
    %% Payment Processing
    INS -->|Approve Payment| PR
    PR -->|Create Escrow| ESC
    ESC -->|Hold Funds| PR
    PR -->|Release Payment| DOC
    
    %% Audit & Reputation
    PR -->|Log Transaction| AUD
    AUD -->|Update Records| REP
    REP -->|Update Provider Score| DOC
    
    %% Denial Handling
    INS -->|Deny Claim| BILL
    BILL -->|Appeal Process| ADJ
    ADJ -->|Review Appeal| INS
    
    classDef participant fill:#e1f5fe
    classDef contract fill:#e8f5e8
    classDef process fill:#fff3e0
    
    class PAT,DOC,INS,BILL,ADJ participant
    class PR,ESC,VEF,AUD,REP contract
    class process fill:#fff3e0
```

## Token Economics and Fee Structure

```mermaid
graph LR
    %% Token Flow
    SUT[SUT Token]
    LIQ[Liquidity Pool]
    STAKE[Staking Pool]
    BURN[Burn Mechanism]
    
    %% Fee Collection
    PLATFORM[Platform Fees 2%]
    PROVIDER[Provider Fees 1%]
    NETWORK[Network Fees 0.5%]
    
    %% Fee Distribution
    TREASURY[Treasury]
    REWARDS[Staking Rewards]
    DEV[Development Fund]
    
    %% Flow Connections
    SUT -->|Transaction Fees| PLATFORM
    SUT -->|Service Fees| PROVIDER
    SUT -->|Gas Fees| NETWORK
    
    PLATFORM -->|70%| TREASURY
    PLATFORM -->|20%| REWARDS
    PLATFORM -->|10%| DEV
    
    PROVIDER -->|Provider Payouts| TREASURY
    NETWORK -->|Network Operations| BURN
    
    REWARDS -->|Stake Rewards| STAKE
    STAKE -->|Liquidity| LIQ
    LIQ -->|Trading| SUT
    
    classDef token fill:#f3e5f5
    classDef fee fill:#ffebee
    classDef dist fill:#e8f5e8
    
    class SUT,LIQ,STAKE,BURN token
    class PLATFORM,PROVIDER,NETWORK fee
    class TREASURY,REWARDS,DEV dist
```

## Payment Security and Compliance Flow

```mermaid
graph TD
    %% Security Layers
    KYC[Identity Verification]
    AML[Anti-Money Laundering]
    ABE[Access Control]
    AUDIT[Audit Trail]
    
    %% Compliance Checks
    HIPAA[HIPAA Compliance]
    GDPR[GDPR Compliance]
    PCI[PCI DSS Compliance]
    
    %% Payment Security
    MFA[Multi-Factor Auth]
    ZKP[Zero Knowledge Proof]
    ENC[End-to-End Encryption]
    
    %% Monitoring
    FRAUD[Fraud Detection]
    RISK[Risk Assessment]
    ALERT[Alert System]
    
    %% Security Flow
    PAYMENT[Payment Request] --> KYC
    KYC --> AML
    AML --> ABE
    ABE --> AUDIT
    
    AUDIT --> HIPAA
    AUDIT --> GDPR
    AUDIT --> PCI
    
    ABE --> MFA
    MFA --> ZKP
    ZKP --> ENC
    
    ENC --> FRAUD
    FRAUD --> RISK
    RISK --> ALERT
    
    ALERT -->|Approved| SETTLEMENT[Payment Settlement]
    ALERT -->|Blocked| INVESTIGATION[Investigation Required]
    
    classDef security fill:#ffebee
    classDef compliance fill:#e8f5e8
    classDef monitoring fill:#fff3e0
    
    class KYC,AML,ABE,AUDIT security
    class HIPAA,GDPR,PCI compliance
    class MFA,ZKP,ENC,FRAUD,RISK,ALERT monitoring
```

## Emergency Payment Override Flow

```mermaid
sequenceDiagram
    participant PAT as Patient
    participant DOC as Doctor
    participant EAO as Emergency Access Override
    participant PR as Payment Router
    participant ESC as Escrow
    participant AUD as Audit
    participant GOV as Governance

    %% Emergency Situation
    PAT->>DOC: Emergency Treatment Needed
    DOC->>EAO: Request Emergency Access
    EAO->>GOV: Emergency Authorization
    GOV->>EAO: Override Granted
    
    %% Immediate Payment Processing
    EAO->>PR: Emergency Payment Request
    PR->>ESC: Create Emergency Escrow
    ESC->>PR: Immediate Fund Release
    PR->>DOC: Emergency Payment
    
    %% Post-Emergency Settlement
    DOC->>PR: Submit Final Bill
    PR->>ESC: Adjust Settlement
    ESC->>PR: Final Payment Calculation
    PR->>DOC: Settlement Payment
    PR->>AUD: Log Emergency Transaction
    
    %% Audit Trail
    AUD->>GOV: Emergency Usage Report
    GOV->>EAO: Review Emergency Access
```

## Key Payment Features

### **1. Multi-Token Support**
- **SUT Token**: Platform native token
- **Stablecoins**: USDC, USDT for price stability
- **Traditional**: Integration with banking systems

### **2. Smart Escrow System**
- **Conditional Release**: Based on service completion
- **Dispute Resolution**: Automated dispute handling
- **Refund Protection**: Patient and provider safeguards

### **3. Insurance Integration**
- **Pre-authorization**: Coverage verification
- **Claims Processing**: Automated claim handling
- **Coordination of Benefits**: Multiple insurer support

### **4. Cross-Chain Compatibility**
- **Multi-chain Routing**: Optimized payment paths
- **Bridge Integration**: Seamless cross-chain transfers
- **Liquidity Management**: Efficient fund allocation

### **5. Compliance & Security**
- **HIPAA Compliant**: Healthcare data protection
- **AML/KYC**: Regulatory compliance
- **Audit Trails**: Complete transaction history

This payment system provides a comprehensive, secure, and efficient solution for healthcare financial transactions while maintaining regulatory compliance and user privacy.
