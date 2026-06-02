# Contract Events

This document is auto-generated from on-chain event emissions found in `contracts/**/src/**/*.rs`.

- Registry format version: `1.0.0`
- Generated at: `2026-06-02T00:41:40.134Z`

## access_control

| Topics | Payload | Source |
|---|---:|---|
| `AC` · `ADMIN` | single (1) | `contracts/access_control/src/lib.rs:126` |
| `AC` · `REVOKE` | single (1) | `contracts/access_control/src/lib.rs:151` |
| `AC` | tuple (2) | `contracts/access_control/src/lib.rs:221` |

## ai_analytics

| Topics | Payload | Source |
|---|---:|---|
| `RndStart` | single (1) | `contracts/ai_analytics/src/rounds.rs:41` |
| `SER_ADDR` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:123` |
| `SER_BYTESN` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:116` |
| `SER_EMPTY` · `MAP` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:94` |
| `SER_EMPTY` · `STR` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:104` |
| `SER_EMPTY` · `VEC` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:83` |
| `SER_WARN` · `EMPTY_DESC` | tuple (0) | `contracts/ai_analytics/src/types.rs:80` |
| `SER_WARN` · `NO_REFS` | tuple (0) | `contracts/ai_analytics/src/types.rs:84` |
| `SER_WARN` · `NO_UPDATES` | tuple (0) | `contracts/ai_analytics/src/types.rs:28` |
| `SER_WARN` · `ZERO_MIN` | tuple (0) | `contracts/ai_analytics/src/types.rs:24` |
| `SER_WARN` · `ZERO_SAMP` | tuple (0) | `contracts/ai_analytics/src/types.rs:52` |
| `SER_WARN` · `ZERO_TS` | tuple (0) | `contracts/ai_analytics/src/types.rs:88` |

## aml

| Topics | Payload | Source |
|---|---:|---|
| `AML` · `STATUS` | tuple (2) | `contracts/aml/src/lib.rs:251` |
| `Init` | single (1) | `contracts/aml/src/lib.rs:53` |

## anomaly_detection

| Topics | Payload | Source |
|---|---:|---|
| `AlertAck` | single (1) | `contracts/anomaly_detection/src/lib.rs:473` |
| `AlertRes` | single (1) | `contracts/anomaly_detection/src/lib.rs:504` |
| `AnomDet` | tuple (4) | `contracts/anomaly_detection/src/lib.rs:342` |
| `CfgUpdate` | single (1) | `contracts/anomaly_detection/src/lib.rs:206` |
| `FalsePos` | tuple (2) | `contracts/anomaly_detection/src/lib.rs:533` |
| `Feedback` | tuple (3) | `contracts/anomaly_detection/src/lib.rs:567` |

## anomaly_detector

| Topics | Payload | Source |
|---|---:|---|
| `AccAnm` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:689` |
| `AlertCrt` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:737` |
| `AlertRes` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:799` |
| `FedUpd` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:930` |
| `Feedback` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:893` |
| `Infer` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:489` |
| `Init` | single (1) | `contracts/anomaly_detector/src/lib.rs:209` |
| `MdlReg` | single (1) | `contracts/anomaly_detector/src/lib.rs:365` |
| `Paused` | single (1) | `contracts/anomaly_detector/src/lib.rs:238` |
| `PrescAnm` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:589` |
| `Unpaused` | single (1) | `contracts/anomaly_detector/src/lib.rs:246` |
| `ValRmvd` | single (1) | `contracts/anomaly_detector/src/lib.rs:230` |

## audit

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT` · `EXPORT` | tuple (3) | `contracts/audit/src/lib.rs:354` |
| `AUDIT` · `GRANT` | tuple (2) | `contracts/audit/src/lib.rs:256` |
| `AUDIT` · `LOG` | tuple (3) | `contracts/audit/src/lib.rs:118` |
| `AUDIT` · `REVOKE` | tuple (2) | `contracts/audit/src/lib.rs:271` |
| `Init` | single (1) | `contracts/audit/src/lib.rs:53` |

## audit_forensics

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT` · `ARCHIVE` | single (1) | `contracts/audit_forensics/src/lib.rs:520` |
| `AUDIT` · `COMPRESS` | tuple (3) | `contracts/audit_forensics/src/lib.rs:508` |
| `AUDIT` · `LOG` | tuple (3) | `contracts/audit_forensics/src/lib.rs:222` |
| `AUDIT` · `RUN` | tuple (3) | `contracts/audit_forensics/src/lib.rs:303` |
| `AUDIT` · `SHARE` | tuple (4) | `contracts/audit_forensics/src/lib.rs:552` |
| `AUDIT` · `XCSYNC` | tuple (2) | `contracts/audit_forensics/src/lib.rs:535` |

## clinical_decision_support

| Topics | Payload | Source |
|---|---:|---|
| `cdss` · `learning_update` | tuple (3) | `contracts/clinical_decision_support/src/lib.rs:205` |

## clinical_trial

| Topics | Payload | Source |
|---|---:|---|
| `AdverseEvent` | tuple (5) | `contracts/clinical_trial/src/lib.rs:274` |
| `ConsentRecorded` | tuple (3) | `contracts/clinical_trial/src/lib.rs:235` |
| `PatientRecruited` | tuple (3) | `contracts/clinical_trial/src/lib.rs:205` |
| `TrialCapacityReached` | tuple (2) | `contracts/clinical_trial/src/lib.rs:199` |

## contract_monitoring

| Topics | Payload | Source |
|---|---:|---|
| `MON` · `ALERT` | single (1) | `contracts/contract_monitoring/src/lib.rs:225` |
| `MON` · `ALERT` | single (1) | `contracts/contract_monitoring/src/lib.rs:358` |
| `MON` · `ALERT` | single (1) | `contracts/contract_monitoring/src/lib.rs:365` |

## contract_usage_analytics

| Topics | Payload | Source |
|---|---:|---|
| `usage` | tuple (4) | `contracts/contract_usage_analytics/src/lib.rs:183` |

## contract_verification

| Topics | Payload | Source |
|---|---:|---|
| `VERIFY` · `ABI` | single (1) | `contracts/contract_verification/src/lib.rs:187` |
| `VERIFY` · `META` | tuple (2) | `contracts/contract_verification/src/lib.rs:142` |
| `VERIFY` · `OK` | single (1) | `contracts/contract_verification/src/lib.rs:209` |

## credential_notifications

| Topics | Payload | Source |
|---|---:|---|
| `CRED` · `ADD_NTF` | single (1) | `contracts/credential_notifications/src/lib.rs:65` |
| `CRED` · `NOTIFY` | tuple (4) | `contracts/credential_notifications/src/lib.rs:103` |
| `CRED` · `RM_NTF` | single (1) | `contracts/credential_notifications/src/lib.rs:86` |

## credential_registry

| Topics | Payload | Source |
|---|---:|---|
| `CREDREG` · `BROOT` | tuple (2) | `contracts/credential_registry/src/lib.rs:311` |
| `CREDREG` · `IADMIN` | tuple (2) | `contracts/credential_registry/src/lib.rs:81` |
| `CREDREG` · `ROOT` | tuple (2) | `contracts/credential_registry/src/lib.rs:141` |

## cross_chain_access

| Topics | Payload | Source |
|---|---:|---|
| `AccessControlInitialized` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:275` |
| `AccessGranted` | tuple (4) | `contracts/cross_chain_access/src/lib.rs:323` |
| `AccessLogged` | tuple (5) | `contracts/cross_chain_access/src/lib.rs:692` |
| `AccessRequested` | tuple (6) | `contracts/cross_chain_access/src/lib.rs:461` |
| `DelegationCreated` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:579` |
| `DelegationRevoked` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:605` |
| `EmergencyAutoApproved` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1183` |
| `EmergencyConfigured` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:644` |
| `Paused` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1000` |
| `RequestProcessed` | tuple (3) | `contracts/cross_chain_access/src/lib.rs:532` |
| `SwapAccepted` | tuple (3) | `contracts/cross_chain_access/src/lib.rs:807` |
| `SwapProposed` | tuple (4) | `contracts/cross_chain_access/src/lib.rs:754` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1014` |

## cross_chain_bridge

| Topics | Payload | Source |
|---|---:|---|
| `AtomicTxInitiated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:868` |
| `EventSynced` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:1387` |
| `MessageConfirmed` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:705` |
| `MessageExecuted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:748` |
| `MessageFailed` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:782` |
| `MessageRetried` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:834` |
| `MessageSubmitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:624` |
| `MessageVerified` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:699` |
| `OperationCreated` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:1462` |
| `OperationRefunded` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1493` |
| `OperationStatusUpdated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1578` |
| `OracleDataAggregated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1214` |
| `OracleReportSubmitted` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:1162` |
| `Paused` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:543` |
| `ProofSubmitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1258` |
| `ProofVerified` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1299` |
| `RecordRefRegistered` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1009` |
| `RefundProcessed` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:2029` |
| `RollbackInitiated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1641` |
| `SyncStatusUpdated` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:1041` |
| `TimeoutExtended` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1545` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:557` |
| `ValidatorDeactivated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:488` |
| `bridge` · `rel_add` | single (1) | `contracts/cross_chain_bridge/src/lib.rs:2056` |
| `bridge` · `rel_rm` | single (1) | `contracts/cross_chain_bridge/src/lib.rs:2073` |

## cross_chain_enhancements

| Topics | Payload | Source |
|---|---:|---|
| `rl` · `set` | tuple (2) | `contracts/cross_chain_enhancements/src/lib.rs:317` |
| `zk` · `integrity` | tuple (3) | `contracts/cross_chain_enhancements/src/lib.rs:252` |
| `zk` · `own_proof` | tuple (3) | `contracts/cross_chain_enhancements/src/lib.rs:168` |
| `zk` · `verified` | tuple (2) | `contracts/cross_chain_enhancements/src/lib.rs:207` |

## cross_chain_identity

| Topics | Payload | Source |
|---|---:|---|
| `AttestationAdded` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:461` |
| `IdentityContractInitialized` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:201` |
| `IdentityRevoked` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:492` |
| `IdentityVerified` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:734` |
| `Paused` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:312` |
| `SyncInitiated` | tuple (4) | `contracts/cross_chain_identity/src/lib.rs:547` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:326` |
| `ValidatorDeactivated` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:258` |
| `VerificationApproved` | tuple (4) | `contracts/cross_chain_identity/src/lib.rs:449` |
| `VerificationRequested` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:376` |

## crypto_registry

| Topics | Payload | Source |
|---|---:|---|
| `KeyRotated` | tuple (3) | `contracts/crypto_registry/src/lib.rs:469` |
| `crypto` · `bundle` | tuple (2) | `contracts/crypto_registry/src/lib.rs:216` |
| `crypto` · `revoke` | tuple (2) | `contracts/crypto_registry/src/lib.rs:241` |

## differential_privacy

| Topics | Payload | Source |
|---|---:|---|
| `dp` · `budget` | tuple (3) | `contracts/differential_privacy/src/lib.rs:133` |
| `dp` · `gaussian` | tuple (3) | `contracts/differential_privacy/src/lib.rs:268` |
| `dp` · `laplace` | tuple (3) | `contracts/differential_privacy/src/lib.rs:200` |

## digital_twin

| Topics | Payload | Source |
|---|---:|---|
| `DT_CREATED` | tuple (2) | `contracts/digital_twin/src/lib.rs:374` |
| `DT_DATAPOINT` | single (1) | `contracts/digital_twin/src/lib.rs:544` |
| `DT_GD_SET` | single (1) | `contracts/digital_twin/src/lib.rs:314` |
| `DT_INIT` | single (1) | `contracts/digital_twin/src/lib.rs:286` |
| `DT_MODEL` | tuple (2) | `contracts/digital_twin/src/lib.rs:592` |
| `DT_MR_SET` | single (1) | `contracts/digital_twin/src/lib.rs:300` |
| `DT_PREDICTION` | tuple (2) | `contracts/digital_twin/src/lib.rs:649` |
| `DT_SIM` | tuple (2) | `contracts/digital_twin/src/lib.rs:707` |
| `DT_SIM_COMP` | single (1) | `contracts/digital_twin/src/lib.rs:747` |
| `DT_SNAPSHOT` | tuple (2) | `contracts/digital_twin/src/lib.rs:815` |
| `DT_STATUS` | tuple (2) | `contracts/digital_twin/src/lib.rs:413` |
| `DT_STREAM` | tuple (2) | `contracts/digital_twin/src/lib.rs:484` |
| `DT_SYNC` | tuple (2) | `contracts/digital_twin/src/lib.rs:875` |

## drug_discovery

| Topics | Payload | Source |
|---|---:|---|
| `CfgInt` | single (1) | `contracts/drug_discovery/src/lib.rs:273` |

## emergency_access_override

| Topics | Payload | Source |
|---|---:|---|
| `EmergencyAccessGranted` | tuple (2) | `contracts/emergency_access_override/src/lib.rs:513` |
| `EmergencyApproval` | tuple (2) | `contracts/emergency_access_override/src/lib.rs:507` |
| `EmergencyRequested` | tuple (3) | `contracts/emergency_access_override/src/lib.rs:472` |

## escrow

| Topics | Payload | Source |
|---|---:|---|
| `EscNew` | tuple (4) | `contracts/escrow/src/lib.rs:280` |
| `EscRel` | tuple (6) | `contracts/escrow/src/lib.rs:384` |
| `Refunded` | tuple (4) | `contracts/escrow/src/lib.rs:436` |

## explainable_ai

| Topics | Payload | Source |
|---|---:|---|
| `ExpFull` | tuple (3) | `contracts/explainable_ai/src/lib.rs:302` |
| `ExpReq` | tuple (3) | `contracts/explainable_ai/src/lib.rs:229` |
| `cf` · `created` | tuple (2) | `contracts/explainable_ai/src/lib.rs:546` |
| `shap` · `created` | tuple (2) | `contracts/explainable_ai/src/lib.rs:473` |

## failover_detector

| Topics | Payload | Source |
|---|---:|---|
| `FD_CRIT` | single (1) | `contracts/failover_detector/src/lib.rs:253` |
| `FD_DEAC` | single (1) | `contracts/failover_detector/src/lib.rs:497` |
| `FD_DETC` | single (1) | `contracts/failover_detector/src/lib.rs:250` |
| `FD_EXEC` | single (1) | `contracts/failover_detector/src/lib.rs:411` |
| `FD_INIT` | single (1) | `contracts/failover_detector/src/lib.rs:140` |
| `FD_PLAN` | single (1) | `contracts/failover_detector/src/lib.rs:317` |
| `FD_REC` | single (1) | `contracts/failover_detector/src/lib.rs:464` |

## federated_learning

| Topics | Payload | Source |
|---|---:|---|
| `AggStart` | tuple (2) | `contracts/federated_learning/src/lib.rs:676` |
| `RndFin` | tuple (4) | `contracts/federated_learning/src/lib.rs:813` |
| `RndStart` | single (1) | `contracts/federated_learning/src/lib.rs:389` |
| `UpdSub` | tuple (3) | `contracts/federated_learning/src/lib.rs:514` |

## fhir_integration

| Topics | Payload | Source |
|---|---:|---|
| `DataExportRequested` | tuple (3) | `contracts/fhir_integration/src/lib.rs:826` |

## forensics

| Topics | Payload | Source |
|---|---:|---|
| `FORENSIC` · `B_LIST` | single (1) | `contracts/forensics/src/lib.rs:176` |
| `FORENSIC` · `COLLECT` | tuple (5) | `contracts/forensics/src/lib.rs:79` |
| `FORENSIC` · `REPORT` | tuple (3) | `contracts/forensics/src/lib.rs:140` |

## genomic_data

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT` · `GENOMIC_CONSENT` | single (1) | `contracts/genomic_data/src/lib.rs:618` |
| `LOG` | single (1) | `contracts/genomic_data/src/lib.rs:303` |
| `WITHDRAWAL` · `GENOMIC_CONSENT` | single (1) | `contracts/genomic_data/src/lib.rs:673` |

## governor

| Topics | Payload | Source |
|---|---:|---|
| `Vote` | tuple (3) | `contracts/governor/src/lib.rs:201` |

## health_data_access_logging

| Topics | Payload | Source |
|---|---:|---|
| `ACCESS` · `LOG` | tuple (6) | `contracts/health_data_access_logging/src/lib.rs:114` |

## healthcare_analytics_dashboard

| Topics | Payload | Source |
|---|---:|---|
| `AiSync` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:1008` |
| `CompAuto` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:964` |
| `DPNoise` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:1153` |
| `DashInit` | single (1) | `contracts/healthcare_analytics_dashboard/src/lib.rs:333` |
| `DashSnap` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:795` |
| `LakeCfg` | tuple (2) | `contracts/healthcare_analytics_dashboard/src/lib.rs:505` |
| `LakeOpt` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:648` |
| `LakeSync` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:588` |
| `PrivAgg` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:719` |
| `TplCreate` | single (1) | `contracts/healthcare_analytics_dashboard/src/lib.rs:831` |

## healthcare_compliance

| Topics | Payload | Source |
|---|---:|---|
| `audit_event` | tuple (6) | `contracts/healthcare_compliance/src/lib.rs:605` |
| `breach_reported` | tuple (5) | `contracts/healthcare_compliance/src/lib.rs:684` |
| `compliance_report_submitted` | tuple (4) | `contracts/healthcare_compliance/src/lib.rs:1184` |
| `consent_granted` | tuple (3) | `contracts/healthcare_compliance/src/lib.rs:448` |
| `consent_revoked` | tuple (3) | `contracts/healthcare_compliance/src/lib.rs:510` |
| `health_check` | tuple (2) | `contracts/healthcare_compliance/src/lib.rs:375` |

## healthcare_data_marketplace

| Topics | Payload | Source |
|---|---:|---|
| `TierPurchased` | tuple (3) | `contracts/healthcare_data_marketplace/src/lib.rs:661` |
| `settled` | tuple (3) | `contracts/healthcare_data_marketplace/src/lib.rs:475` |

## healthcare_payment

| Topics | Payload | Source |
|---|---:|---|
| `CB_ANOM` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1683` |
| `CLAIM_EDI` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:724` |
| `CLAIM_PD` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1027` |
| `CLAIM_PD` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1085` |
| `COV_834` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:763` |
| `COV_PROOF` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1428` |
| `COV_VER` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1468` |
| `DIAG` · `ENTER` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:965` |
| `DIAG` · `EXIT` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:1032` |
| `DIAG` · `STATE` | tuple (4) | `contracts/healthcare_payment/src/lib.rs:1004` |
| `DIAG` · `VALFAIL` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:981` |
| `ELIG` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:605` |
| `EOB` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:954` |

## healthcare_reputation

| Topics | Payload | Source |
|---|---:|---|
| `HLTHREP` · `CONDUCT` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:499` |
| `HLTHREP` · `CRED_ADD` | tuple (2) | `contracts/healthcare_reputation/src/lib.rs:269` |
| `HLTHREP` · `CRED_VER` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:312` |
| `HLTHREP` · `DISPUTE` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:585` |
| `HLTHREP` · `DISP_RES` | tuple (2) | `contracts/healthcare_reputation/src/lib.rs:626` |
| `HLTHREP` · `FEEDBACK` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:404` |

## homomorphic_registry

| Topics | Payload | Source |
|---|---:|---|
| `he` · `ctx` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:650` |
| `he` · `key` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:231` |
| `he` · `submit` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:737` |

## identity_registry

| Topics | Payload | Source |
|---|---:|---|
| `Attested` | tuple (3) | `contracts/identity_registry/src/lib.rs:1689` |
| `CredentialIssued` | tuple (4) | `contracts/identity_registry/src/lib.rs:947` |
| `CredentialRevoked` | tuple (2) | `contracts/identity_registry/src/lib.rs:1023` |
| `DIDCreated` | tuple (2) | `contracts/identity_registry/src/lib.rs:554` |
| `DIDUpdated` | tuple (2) | `contracts/identity_registry/src/lib.rs:620` |
| `GuardianAdded` | tuple (3) | `contracts/identity_registry/src/lib.rs:1102` |
| `HealthCheck` | tuple (2) | `contracts/identity_registry/src/lib.rs:382` |
| `Initialized` | tuple (2) | `contracts/identity_registry/src/lib.rs:364` |
| `Paused` | tuple (2) | `contracts/identity_registry/src/lib.rs:427` |
| `RecoveryApproved` | tuple (2) | `contracts/identity_registry/src/lib.rs:1279` |
| `RecoveryCancelled` | tuple (2) | `contracts/identity_registry/src/lib.rs:1424` |
| `RecoveryExecuted` | tuple (2) | `contracts/identity_registry/src/lib.rs:1379` |
| `RecoveryInitiated` | tuple (2) | `contracts/identity_registry/src/lib.rs:1232` |
| `ServiceRemoved` | tuple (2) | `contracts/identity_registry/src/lib.rs:1517` |
| `StakeDeposited` | tuple (3) | `contracts/identity_registry/src/lib.rs:1992` |
| `StakeSlashed` | tuple (3) | `contracts/identity_registry/src/lib.rs:2058` |
| `StakeWithdrawn` | tuple (2) | `contracts/identity_registry/src/lib.rs:2029` |
| `ThresholdUpdated` | tuple (2) | `contracts/identity_registry/src/lib.rs:1151` |
| `Unpaused` | tuple (2) | `contracts/identity_registry/src/lib.rs:435` |
| `VerificationMethodAdded` | tuple (2) | `contracts/identity_registry/src/lib.rs:718` |
| `VerificationMethodRevoked` | tuple (2) | `contracts/identity_registry/src/lib.rs:867` |

## ihe_integration

| Topics | Payload | Source |
|---|---:|---|
| `ATNA` · `AUTH` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1049` |
| `ATNA` · `AUTO` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1745` |
| `ATNA` · `LOG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:991` |
| `BPPC` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1326` |
| `BPPC` · `REVOKE` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1349` |
| `CONN` · `TEST` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1623` |
| `CT` · `SYNC` | tuple (4) | `contracts/ihe_integration/src/lib.rs:1265` |
| `DSG` · `SIGN` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1433` |
| `HPD` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1506` |
| `MPI` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1155` |
| `PIX` · `MERGE` | tuple (2) | `contracts/ihe_integration/src/lib.rs:843` |
| `PIX` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:753` |
| `SVS` · `REG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1557` |
| `XDM` · `PKG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1238` |
| `XDR` · `SEND` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1199` |
| `XDS` · `DEPR` | tuple (2) | `contracts/ihe_integration/src/lib.rs:599` |
| `XDS` · `REG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:561` |
| `XDS` · `SUBMIT` | tuple (2) | `contracts/ihe_integration/src/lib.rs:696` |

## load_testing

| Topics | Payload | Source |
|---|---:|---|
| `LOAD` · `DONE` | single (1) | `contracts/load_testing/src/lib.rs:143` |

## medical_consent_nft

| Topics | Payload | Source |
|---|---:|---|
| `consent` · `delegated` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1082` |
| `consent` · `emerg_ovr` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1346` |
| `consent` · `issued` | tuple (4) | `contracts/medical_consent_nft/src/lib.rs:422` |
| `consent` · `mkt_list` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1451` |
| `consent` · `mkt_purch` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1515` |
| `consent` · `perm_upd` | tuple (2) | `contracts/medical_consent_nft/src/lib.rs:781` |
| `consent` · `revoked` | tuple (2) | `contracts/medical_consent_nft/src/lib.rs:572` |
| `consent` · `transfer` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:644` |
| `consent` · `upd_dyn` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1594` |
| `consent` · `updated` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:490` |

## medical_imaging

| Topics | Payload | Source |
|---|---:|---|
| `DISCREP` | single (1) | `contracts/medical_imaging/src/lib.rs:1474` |
| `IMG_MDL` | single (1) | `contracts/medical_imaging/src/lib.rs:635` |

## medical_imaging_ai

| Topics | Payload | Source |
|---|---:|---|
| `MDL_REG` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:320` |
| `MDL_RET` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:352` |
| `SEG` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:516` |

## medical_record_backup

| Topics | Payload | Source |
|---|---:|---|
| `BKP_POL` | single (1) | `contracts/medical_record_backup/src/lib.rs:389` |
| `BKP_REST` | tuple (2) | `contracts/medical_record_backup/src/lib.rs:673` |
| `BKP_RUN` | tuple (3) | `contracts/medical_record_backup/src/lib.rs:985` |

## medical_record_search

| Topics | Payload | Source |
|---|---:|---|
| `SRCH_AUD` | tuple (3) | `contracts/medical_record_search/src/lib.rs:741` |

## medical_records

| Topics | Payload | Source |
|---|---:|---|
| `DQ_VALID` | tuple (3) | `contracts/medical_records/src/events.rs:825` |
| `EXPORT` · `DATA` | tuple (3) | `contracts/medical_records/src/lib.rs:4694` |
| `LOG` | single (1) | `contracts/medical_records/src/lib.rs:852` |
| `TradRecAdded` | tuple (4) | `contracts/medical_records/src/lib.rs:6025` |

## medication_management

| Topics | Payload | Source |
|---|---:|---|
| `CAT_SYNC` | single (1) | `contracts/medication_management/src/lib.rs:306` |
| `MED_SYNC` | single (1) | `contracts/medication_management/src/lib.rs:850` |

## meta_tx_forwarder

| Topics | Payload | Source |
|---|---:|---|
| `fwd` | tuple (5) | `contracts/meta_tx_forwarder/src/lib.rs:355` |
| `init` | tuple (3) | `contracts/meta_tx_forwarder/src/lib.rs:130` |
| `reg_relay` | tuple (2) | `contracts/meta_tx_forwarder/src/lib.rs:240` |

## mfa

| Topics | Payload | Source |
|---|---:|---|
| `MFA` | single (1) | `contracts/mfa/src/lib.rs:229` |

## mpc_manager

| Topics | Payload | Source |
|---|---:|---|
| `mpc` · `commit` | tuple (2) | `contracts/mpc_manager/src/lib.rs:294` |
| `mpc` · `final` | tuple (2) | `contracts/mpc_manager/src/lib.rs:404` |
| `mpc` · `ml` | tuple (4) | `contracts/mpc_manager/src/lib.rs:669` |
| `mpc` · `proof` | tuple (2) | `contracts/mpc_manager/src/lib.rs:561` |
| `mpc` · `reveal` | tuple (2) | `contracts/mpc_manager/src/lib.rs:344` |
| `mpc` · `start` | tuple (2) | `contracts/mpc_manager/src/lib.rs:247` |
| `mpc` · `stats` | tuple (4) | `contracts/mpc_manager/src/lib.rs:612` |

## multi_region_orchestrator

| Topics | Payload | Source |
|---|---:|---|
| `DRO_FAIL` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:379` |
| `DRO_HLTH` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:476` |
| `DRO_INIT` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:170` |
| `DRO_REGI` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:238` |
| `DRO_SETP` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:548` |
| `DRO_SLAM` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:511` |
| `DRO_STAT` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:296` |
| `DRO_SYNC` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:433` |

## patient_consent_management

| Topics | Payload | Source |
|---|---:|---|
| `Paused` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:254` |
| `ProxyConsentGranted` | tuple (3) | `contracts/patient_consent_management/src/lib.rs:367` |
| `ProxyConsentRevoked` | tuple (3) | `contracts/patient_consent_management/src/lib.rs:394` |
| `ProxyDesignated` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:321` |
| `ProxyRevoked` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:332` |
| `Unpaused` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:265` |

## patient_gamification

| Topics | Payload | Source |
|---|---:|---|
| `AchCreate` | tuple (3) | `contracts/patient_gamification/src/lib.rs:331` |
| `AchEarn` | tuple (3) | `contracts/patient_gamification/src/lib.rs:402` |
| `ChalComp` | tuple (3) | `contracts/patient_gamification/src/lib.rs:595` |
| `ChalCrt` | tuple (3) | `contracts/patient_gamification/src/lib.rs:478` |
| `ChalJoin` | tuple (2) | `contracts/patient_gamification/src/lib.rs:547` |
| `ConfigUpd` | single (1) | `contracts/patient_gamification/src/lib.rs:1249` |
| `GamInit` | single (1) | `contracts/patient_gamification/src/lib.rs:263` |
| `MetricRec` | tuple (3) | `contracts/patient_gamification/src/lib.rs:1114` |
| `ProfCrt` | single (1) | `contracts/patient_gamification/src/lib.rs:861` |
| `PtsRedeem` | tuple (2) | `contracts/patient_gamification/src/lib.rs:705` |
| `RndCmt` | tuple (3) | `contracts/patient_gamification/src/lib.rs:751` |
| `RndRvl` | tuple (3) | `contracts/patient_gamification/src/lib.rs:803` |

## patient_risk_stratification

| Topics | Payload | Source |
|---|---:|---|
| `ModelReg` | single (1) | `contracts/patient_risk_stratification/src/lib.rs:196` |
| `RiskAsses` | tuple (4) | `contracts/patient_risk_stratification/src/lib.rs:264` |

## pharma_supply_chain

| Topics | Payload | Source |
|---|---:|---|
| `BATCH` · `CREATE` | tuple (3) | `contracts/pharma_supply_chain/src/lib.rs:290` |

## predictive_analytics

| Topics | Payload | Source |
|---|---:|---|
| `CfgUpdate` | single (1) | `contracts/predictive_analytics/src/config.rs:72` |
| `PredMade` | tuple (4) | `contracts/predictive_analytics/src/predictions.rs:84` |

## public_health_surveillance

| Topics | Payload | Source |
|---|---:|---|
| `phs` · `alert_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:570` |
| `phs` · `amr_alert` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1192` |
| `phs` · `amr_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:761` |
| `phs` · `auto_alrt` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1106` |
| `phs` · `colab_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:933` |
| `phs` · `cov_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:640` |
| `phs` · `env_alert` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1151` |
| `phs` · `env_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:704` |
| `phs` · `intv_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:873` |
| `phs` · `model_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:512` |
| `phs` · `out_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:442` |
| `phs` · `sdoh_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:810` |

## regional_node_manager

| Topics | Payload | Source |
|---|---:|---|
| `RNM_CFG` | single (1) | `contracts/regional_node_manager/src/lib.rs:468` |
| `RNM_HLTH` | single (1) | `contracts/regional_node_manager/src/lib.rs:331` |
| `RNM_INIT` | single (1) | `contracts/regional_node_manager/src/lib.rs:137` |
| `RNM_REG` | single (1) | `contracts/regional_node_manager/src/lib.rs:201` |
| `RNM_REPL` | single (1) | `contracts/regional_node_manager/src/lib.rs:394` |
| `RNM_SYNC` | single (1) | `contracts/regional_node_manager/src/lib.rs:433` |
| `RNM_UPD` | single (1) | `contracts/regional_node_manager/src/lib.rs:283` |

## remote_patient_monitoring

| Topics | Payload | Source |
|---|---:|---|
| `alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:276` |
| `caregiver_alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:195` |
| `caregiver_alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:281` |

## reputation_access_control

| Topics | Payload | Source |
|---|---:|---|
| `REPUTAC` · `APPROVED` | single (1) | `contracts/reputation_access_control/src/lib.rs:276` |
| `REPUTAC` · `DENIED` | single (1) | `contracts/reputation_access_control/src/lib.rs:299` |
| `REPUTAC` · `EMERGENCY` | single (1) | `contracts/reputation_access_control/src/lib.rs:320` |
| `REPUTAC` · `POLICY` | single (1) | `contracts/reputation_access_control/src/lib.rs:150` |
| `REPUTAC` · `REQUEST` | single (1) | `contracts/reputation_access_control/src/lib.rs:247` |
| `REPUTAC` · `REVOKE_EM` · `REVOKE_EMERGENCY` | single (1) | `contracts/reputation_access_control/src/lib.rs:340` |
| `REPUTAC` · `THRESHOLD` | tuple (2) | `contracts/reputation_access_control/src/lib.rs:405` |

## reputation_integration

| Topics | Payload | Source |
|---|---:|---|
| `REPUTINT` · `AUTO_SYNC` | single (1) | `contracts/reputation_integration/src/lib.rs:206` |
| `REPUTINT` · `BASE_UPD` | tuple (2) | `contracts/reputation_integration/src/lib.rs:433` |
| `REPUTINT` · `MAP_UPD` | single (1) | `contracts/reputation_integration/src/lib.rs:239` |
| `REPUTINT` · `SET_UPD` | single (1) | `contracts/reputation_integration/src/lib.rs:259` |
| `REPUTINT` · `SYNC` | tuple (2) | `contracts/reputation_integration/src/lib.rs:157` |

## storage_cleanup

| Topics | Payload | Source |
|---|---:|---|
| `CLEANUP` · `ALL` | tuple (2) | `contracts/storage_cleanup/src/lib.rs:202` |

## sut_token

| Topics | Payload | Source |
|---|---:|---|
| `burn` | single (1) | `contracts/sut_token/src/lib.rs:470` |
| `mint` | single (1) | `contracts/sut_token/src/lib.rs:405` |

## sync_manager

| Topics | Payload | Source |
|---|---:|---|
| `SM_CONF` | single (1) | `contracts/sync_manager/src/lib.rs:457` |
| `SM_EXEC` | single (1) | `contracts/sync_manager/src/lib.rs:276` |
| `SM_INIT` | single (1) | `contracts/sync_manager/src/lib.rs:167` |
| `SM_INIT_S` | single (1) | `contracts/sync_manager/src/lib.rs:234` |
| `SM_LAG` | single (1) | `contracts/sync_manager/src/lib.rs:383` |
| `SM_RESO` | single (1) | `contracts/sync_manager/src/lib.rs:493` |
| `SM_RETR` | single (1) | `contracts/sync_manager/src/lib.rs:320` |
| `SM_SETP` | single (1) | `contracts/sync_manager/src/lib.rs:516` |

## timelock

| Topics | Payload | Source |
|---|---:|---|
| `Queued` | tuple (2) | `contracts/timelock/src/lib.rs:81` |

## token_sale

| Topics | Payload | Source |
|---|---:|---|
| `contribution` | tuple (4) | `contracts/token_sale/src/contract.rs:228` |
| `phase_added` | tuple (5) | `contracts/token_sale/src/contract.rs:91` |
| `sale_initialized` | tuple (4) | `contracts/token_sale/src/contract.rs:55` |
| `sale_paused` | tuple (0) | `contracts/token_sale/src/contract.rs:113` |
| `sale_unpaused` | tuple (0) | `contracts/token_sale/src/contract.rs:122` |
| `token_added` | tuple (2) | `contracts/token_sale/src/contract.rs:104` |
| `tokens_claimed` | tuple (2) | `contracts/token_sale/src/contract.rs:285` |
| `vesting_schedule_created` | tuple (4) | `contracts/token_sale/src/vesting.rs:67` |
| `vesting_schedule_updated` | tuple (5) | `contracts/token_sale/src/vesting.rs:227` |

## treasury_controller

| Topics | Payload | Source |
|---|---:|---|
| `APPROVED` | tuple (3) | `contracts/treasury_controller/src/lib.rs:369` |
| `EMERGENCY` | single (1) | `contracts/treasury_controller/src/lib.rs:500` |
| `EXECUTED` | tuple (3) | `contracts/treasury_controller/src/lib.rs:470` |
| `INIT` | single (1) | `contracts/treasury_controller/src/lib.rs:183` |
| `PROPOSAL` | tuple (3) | `contracts/treasury_controller/src/lib.rs:300` |
| `RESUMED` | single (1) | `contracts/treasury_controller/src/lib.rs:524` |

## upgradeability

| Topics | Payload | Source |
|---|---:|---|
| `DeprecationsUpdated` | single (1) | `contracts/upgradeability/src/lib.rs:238` |

## zk_verifier

| Topics | Payload | Source |
|---|---:|---|
| `ZKVER` · `ATTEST` | tuple (2) | `contracts/zk_verifier/src/lib.rs:235` |
| `ZKVER` · `VKREG` | tuple (2) | `contracts/zk_verifier/src/lib.rs:144` |

## zkp_registry

| Topics | Payload | Source |
|---|---:|---|
| `admin` · `approved` | tuple (2) | `contracts/zkp_registry/src/lib.rs:461` |
| `admin` · `emer_exec` | single (1) | `contracts/zkp_registry/src/lib.rs:548` |
| `admin` · `executed` | single (1) | `contracts/zkp_registry/src/lib.rs:508` |
| `admin` · `proposed` | tuple (2) | `contracts/zkp_registry/src/lib.rs:416` |
| `zkp` · `circ_reg` | single (1) | `contracts/zkp_registry/src/lib.rs:597` |
| `zkp` · `cleanup` | tuple (2) | `contracts/zkp_registry/src/lib.rs:1071` |
| `zkp` · `cred_prf` | tuple (2) | `contracts/zkp_registry/src/lib.rs:949` |
| `zkp` · `med_proof` | tuple (2) | `contracts/zkp_registry/src/lib.rs:838` |
| `zkp` · `proof_sub` | tuple (3) | `contracts/zkp_registry/src/lib.rs:680` |
| `zkp` · `proof_sub` | tuple (3) | `contracts/zkp_registry/src/lib.rs:791` |
| `zkp` · `rec_proof` | tuple (3) | `contracts/zkp_registry/src/lib.rs:1023` |
| `zkp` · `rng_proof` | tuple (4) | `contracts/zkp_registry/src/lib.rs:898` |

