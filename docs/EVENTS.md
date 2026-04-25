# Contract Events

This document is auto-generated from on-chain event emissions found in `contracts/**/src/**/*.rs`.

- Registry format version: `1.0.0`
- Generated at: `2026-04-25T04:43:58.181Z`

## ai_analytics

| Topics | Payload | Source |
|---|---:|---|
| `RndStart` | single (1) | `contracts/ai_analytics/src/lib.rs:130` |

## aml

| Topics | Payload | Source |
|---|---:|---|
| `AML` · `STATUS` | tuple (2) | `contracts/aml/src/lib.rs:129` |

## anomaly_detection

| Topics | Payload | Source |
|---|---:|---|
| `AlertAck` | single (1) | `contracts/anomaly_detection/src/lib.rs:472` |
| `AlertRes` | single (1) | `contracts/anomaly_detection/src/lib.rs:503` |
| `AnomDet` | tuple (4) | `contracts/anomaly_detection/src/lib.rs:341` |
| `CfgUpdate` | single (1) | `contracts/anomaly_detection/src/lib.rs:205` |
| `FalsePos` | tuple (2) | `contracts/anomaly_detection/src/lib.rs:532` |
| `Feedback` | tuple (3) | `contracts/anomaly_detection/src/lib.rs:566` |

## anomaly_detector

| Topics | Payload | Source |
|---|---:|---|
| `AccAnm` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:626` |
| `AlertCrt` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:674` |
| `AlertRes` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:736` |
| `FedUpd` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:867` |
| `Feedback` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:830` |
| `Infer` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:426` |
| `Init` | single (1) | `contracts/anomaly_detector/src/lib.rs:208` |
| `MdlReg` | single (1) | `contracts/anomaly_detector/src/lib.rs:302` |
| `Paused` | single (1) | `contracts/anomaly_detector/src/lib.rs:237` |
| `PrescAnm` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:526` |
| `Unpaused` | single (1) | `contracts/anomaly_detector/src/lib.rs:245` |
| `ValRmvd` | single (1) | `contracts/anomaly_detector/src/lib.rs:229` |

## appointment_booking_escrow

| Topics | Payload | Source |
|---|---:|---|
| `APPT` · `BOOK` | tuple (5) | `contracts/appointment_booking_escrow/src/events.rs:11` |
| `APPT` · `CONF` | tuple (3) | `contracts/appointment_booking_escrow/src/events.rs:23` |
| `APPT` · `REFUND` | tuple (4) | `contracts/appointment_booking_escrow/src/events.rs:36` |
| `APPT` · `RELEASE` | tuple (4) | `contracts/appointment_booking_escrow/src/events.rs:49` |

## audit

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT` · `LOG` | tuple (3) | `contracts/audit/src/lib.rs:74` |

## audit_forensics

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT` · `ARCHIVE` | single (1) | `contracts/audit_forensics/src/lib.rs:516` |
| `AUDIT` · `COMPRESS` | tuple (3) | `contracts/audit_forensics/src/lib.rs:504` |
| `AUDIT` · `LOG` | tuple (3) | `contracts/audit_forensics/src/lib.rs:219` |
| `AUDIT` · `RUN` | tuple (3) | `contracts/audit_forensics/src/lib.rs:299` |
| `AUDIT` · `SHARE` | tuple (4) | `contracts/audit_forensics/src/lib.rs:548` |
| `AUDIT` · `XCSYNC` | tuple (2) | `contracts/audit_forensics/src/lib.rs:531` |

## clinical_decision_support

| Topics | Payload | Source |
|---|---:|---|
| `cdss` · `learning_update` | tuple (3) | `contracts/clinical_decision_support/src/lib.rs:204` |

## clinical_nlp

| Topics | Payload | Source |
|---|---:|---|
| `BATCH` | tuple (2) | `contracts/clinical_nlp/src/events.rs:157` |
| `CODING` | tuple (2) | `contracts/clinical_nlp/src/events.rs:148` |
| `ENTITY` | tuple (2) | `contracts/clinical_nlp/src/events.rs:130` |
| `NLP_PROC` | tuple (2) | `contracts/clinical_nlp/src/events.rs:121` |
| `SENTIM` | tuple (2) | `contracts/clinical_nlp/src/events.rs:139` |

## clinical_trial

| Topics | Payload | Source |
|---|---:|---|
| `AdverseEvent` | tuple (5) | `contracts/clinical_trial/src/lib.rs:230` |
| `ConsentRecorded` | tuple (3) | `contracts/clinical_trial/src/lib.rs:192` |
| `PatientRecruited` | tuple (3) | `contracts/clinical_trial/src/lib.rs:164` |

## credential_registry

| Topics | Payload | Source |
|---|---:|---|
| `CREDREG` · `IADMIN` | tuple (2) | `contracts/credential_registry/src/lib.rs:70` |
| `CREDREG` · `ROOT` | tuple (2) | `contracts/credential_registry/src/lib.rs:118` |

## cross_chain_access

| Topics | Payload | Source |
|---|---:|---|
| `AccessControlInitialized` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:274` |
| `AccessGranted` | tuple (4) | `contracts/cross_chain_access/src/lib.rs:322` |
| `AccessLogged` | tuple (5) | `contracts/cross_chain_access/src/lib.rs:691` |
| `AccessRequested` | tuple (6) | `contracts/cross_chain_access/src/lib.rs:460` |
| `DelegationCreated` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:578` |
| `DelegationRevoked` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:604` |
| `EmergencyAutoApproved` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1182` |
| `EmergencyConfigured` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:643` |
| `Paused` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:999` |
| `RequestProcessed` | tuple (3) | `contracts/cross_chain_access/src/lib.rs:531` |
| `SwapAccepted` | tuple (3) | `contracts/cross_chain_access/src/lib.rs:806` |
| `SwapProposed` | tuple (4) | `contracts/cross_chain_access/src/lib.rs:753` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1013` |

## cross_chain_bridge

| Topics | Payload | Source |
|---|---:|---|
| `AtomicTxInitiated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:720` |
| `EventSynced` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:1239` |
| `MessageConfirmed` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:643` |
| `MessageExecuted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:686` |
| `MessageSubmitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:570` |
| `MessageVerified` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:637` |
| `OracleDataAggregated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1066` |
| `OracleReportSubmitted` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:1014` |
| `Paused` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:494` |
| `ProofSubmitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1110` |
| `ProofVerified` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1151` |
| `RecordRefRegistered` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:861` |
| `RollbackInitiated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1321` |
| `SyncStatusUpdated` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:893` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:508` |
| `ValidatorDeactivated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:439` |

## cross_chain_enhancements

| Topics | Payload | Source |
|---|---:|---|
| `rl` · `set` | tuple (2) | `contracts/cross_chain_enhancements/src/lib.rs:314` |
| `zk` · `integrity` | tuple (3) | `contracts/cross_chain_enhancements/src/lib.rs:249` |
| `zk` · `own_proof` | tuple (3) | `contracts/cross_chain_enhancements/src/lib.rs:166` |
| `zk` · `verified` | tuple (2) | `contracts/cross_chain_enhancements/src/lib.rs:205` |

## cross_chain_identity

| Topics | Payload | Source |
|---|---:|---|
| `AttestationAdded` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:460` |
| `IdentityContractInitialized` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:200` |
| `IdentityRevoked` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:491` |
| `IdentityVerified` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:733` |
| `Paused` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:311` |
| `SyncInitiated` | tuple (4) | `contracts/cross_chain_identity/src/lib.rs:546` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:325` |
| `ValidatorDeactivated` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:257` |
| `VerificationApproved` | tuple (4) | `contracts/cross_chain_identity/src/lib.rs:448` |
| `VerificationRequested` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:375` |

## crypto_registry

| Topics | Payload | Source |
|---|---:|---|
| `crypto` · `bundle` | tuple (2) | `contracts/crypto_registry/src/lib.rs:211` |
| `crypto` · `revoke` | tuple (2) | `contracts/crypto_registry/src/lib.rs:236` |

## differential_privacy

| Topics | Payload | Source |
|---|---:|---|
| `dp` · `budget` | tuple (3) | `contracts/differential_privacy/src/lib.rs:132` |
| `dp` · `gaussian` | tuple (3) | `contracts/differential_privacy/src/lib.rs:267` |
| `dp` · `laplace` | tuple (3) | `contracts/differential_privacy/src/lib.rs:199` |

## digital_twin

| Topics | Payload | Source |
|---|---:|---|
| `DT_CREATED` | tuple (2) | `contracts/digital_twin/src/lib.rs:373` |
| `DT_DATAPOINT` | single (1) | `contracts/digital_twin/src/lib.rs:543` |
| `DT_GD_SET` | single (1) | `contracts/digital_twin/src/lib.rs:313` |
| `DT_INIT` | single (1) | `contracts/digital_twin/src/lib.rs:285` |
| `DT_MODEL` | tuple (2) | `contracts/digital_twin/src/lib.rs:591` |
| `DT_MR_SET` | single (1) | `contracts/digital_twin/src/lib.rs:299` |
| `DT_PREDICTION` | tuple (2) | `contracts/digital_twin/src/lib.rs:648` |
| `DT_SIM` | tuple (2) | `contracts/digital_twin/src/lib.rs:706` |
| `DT_SIM_COMP` | single (1) | `contracts/digital_twin/src/lib.rs:746` |
| `DT_SNAPSHOT` | tuple (2) | `contracts/digital_twin/src/lib.rs:814` |
| `DT_STATUS` | tuple (2) | `contracts/digital_twin/src/lib.rs:412` |
| `DT_STREAM` | tuple (2) | `contracts/digital_twin/src/lib.rs:483` |
| `DT_SYNC` | tuple (2) | `contracts/digital_twin/src/lib.rs:874` |

## drug_discovery

| Topics | Payload | Source |
|---|---:|---|
| `CfgInt` | single (1) | `contracts/drug_discovery/src/lib.rs:272` |

## emergency_access_override

| Topics | Payload | Source |
|---|---:|---|
| `EMER` · `APPR` | tuple (4) | `contracts/emergency_access_override/src/events.rs:23` |
| `EMER` · `CHECK` | tuple (4) | `contracts/emergency_access_override/src/events.rs:49` |
| `EMER` · `DUPA` | tuple (4) | `contracts/emergency_access_override/src/events.rs:36` |
| `EMER` · `GRANT` | tuple (4) | `contracts/emergency_access_override/src/events.rs:10` |
| `EMER` · `REVOKE` | tuple (3) | `contracts/emergency_access_override/src/events.rs:61` |

## escrow

| Topics | Payload | Source |
|---|---:|---|
| `EscNew` | tuple (4) | `contracts/escrow/src/lib.rs:269` |
| `EscRel` | tuple (6) | `contracts/escrow/src/lib.rs:369` |
| `Refunded` | tuple (4) | `contracts/escrow/src/lib.rs:421` |

## explainable_ai

| Topics | Payload | Source |
|---|---:|---|
| `ExpFull` | tuple (3) | `contracts/explainable_ai/src/lib.rs:301` |
| `ExpReq` | tuple (3) | `contracts/explainable_ai/src/lib.rs:228` |
| `cf` · `created` | tuple (2) | `contracts/explainable_ai/src/lib.rs:545` |
| `shap` · `created` | tuple (2) | `contracts/explainable_ai/src/lib.rs:472` |

## failover_detector

| Topics | Payload | Source |
|---|---:|---|
| `FD_CRIT` | single (1) | `contracts/failover_detector/src/lib.rs:260` |
| `FD_DEAC` | single (1) | `contracts/failover_detector/src/lib.rs:504` |
| `FD_DETC` | single (1) | `contracts/failover_detector/src/lib.rs:257` |
| `FD_EXEC` | single (1) | `contracts/failover_detector/src/lib.rs:418` |
| `FD_INIT` | single (1) | `contracts/failover_detector/src/lib.rs:141` |
| `FD_PLAN` | single (1) | `contracts/failover_detector/src/lib.rs:324` |
| `FD_REC` | single (1) | `contracts/failover_detector/src/lib.rs:471` |

## federated_learning

| Topics | Payload | Source |
|---|---:|---|
| `AggStart` | tuple (2) | `contracts/federated_learning/src/lib.rs:647` |
| `InstBlack` | tuple (2) | `contracts/federated_learning/src/lib.rs:847` |
| `RndStart` | single (1) | `contracts/federated_learning/src/lib.rs:388` |

## forensics

| Topics | Payload | Source |
|---|---:|---|
| `FORENSIC` · `B_LIST` | single (1) | `contracts/forensics/src/lib.rs:163` |
| `FORENSIC` · `COLLECT` | tuple (5) | `contracts/forensics/src/lib.rs:66` |
| `FORENSIC` · `REPORT` | tuple (3) | `contracts/forensics/src/lib.rs:127` |

## genomic_data

| Topics | Payload | Source |
|---|---:|---|
| `LOG` | single (1) | `contracts/genomic_data/src/lib.rs:257` |

## governor

| Topics | Payload | Source |
|---|---:|---|
| `Vote` | tuple (3) | `contracts/governor/src/lib.rs:218` |

## health_data_access_logging

| Topics | Payload | Source |
|---|---:|---|
| `ACCESS` · `LOG` | tuple (6) | `contracts/health_data_access_logging/src/lib.rs:113` |

## healthcare_analytics_dashboard

| Topics | Payload | Source |
|---|---:|---|
| `AiSync` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:983` |
| `CompAuto` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:939` |
| `DashInit` | single (1) | `contracts/healthcare_analytics_dashboard/src/lib.rs:308` |
| `DashSnap` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:770` |
| `LakeCfg` | tuple (2) | `contracts/healthcare_analytics_dashboard/src/lib.rs:480` |
| `LakeOpt` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:623` |
| `LakeSync` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:563` |
| `PrivAgg` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:694` |
| `TplCreate` | single (1) | `contracts/healthcare_analytics_dashboard/src/lib.rs:806` |

## healthcare_compliance

| Topics | Payload | Source |
|---|---:|---|
| `audit_event` | tuple (6) | `contracts/healthcare_compliance/src/lib.rs:581` |
| `breach_reported` | tuple (5) | `contracts/healthcare_compliance/src/lib.rs:660` |
| `compliance_report_submitted` | tuple (4) | `contracts/healthcare_compliance/src/lib.rs:1160` |
| `consent_granted` | tuple (3) | `contracts/healthcare_compliance/src/lib.rs:424` |
| `consent_revoked` | tuple (3) | `contracts/healthcare_compliance/src/lib.rs:486` |

## healthcare_data_marketplace

| Topics | Payload | Source |
|---|---:|---|
| `settled` | tuple (3) | `contracts/healthcare_data_marketplace/src/lib.rs:474` |

## healthcare_payment

| Topics | Payload | Source |
|---|---:|---|
| `CLAIM_EDI` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:589` |
| `CLAIM_PD` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:871` |
| `COV_834` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:628` |
| `ELIG` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:473` |
| `EOB` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:819` |

## healthcare_reputation

| Topics | Payload | Source |
|---|---:|---|
| `HLTHREP` · `CONDUCT` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:497` |
| `HLTHREP` · `CRED_ADD` | tuple (2) | `contracts/healthcare_reputation/src/lib.rs:268` |
| `HLTHREP` · `CRED_VER` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:311` |
| `HLTHREP` · `DISPUTE` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:583` |
| `HLTHREP` · `DISP_RES` | tuple (2) | `contracts/healthcare_reputation/src/lib.rs:624` |
| `HLTHREP` · `FEEDBACK` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:403` |

## homomorphic_registry

| Topics | Payload | Source |
|---|---:|---|
| `he` · `ctx` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:649` |
| `he` · `key` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:230` |
| `he` · `submit` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:736` |

## identity_registry

| Topics | Payload | Source |
|---|---:|---|
| `Attested` | tuple (3) | `contracts/identity_registry/src/lib.rs:1541` |
| `CredentialIssued` | tuple (4) | `contracts/identity_registry/src/lib.rs:850` |
| `CredentialRevoked` | tuple (2) | `contracts/identity_registry/src/lib.rs:926` |
| `DIDCreated` | tuple (2) | `contracts/identity_registry/src/lib.rs:454` |
| `DIDUpdated` | tuple (2) | `contracts/identity_registry/src/lib.rs:520` |
| `GuardianAdded` | tuple (3) | `contracts/identity_registry/src/lib.rs:1005` |
| `Initialized` | tuple (2) | `contracts/identity_registry/src/lib.rs:333` |
| `RecoveryApproved` | tuple (2) | `contracts/identity_registry/src/lib.rs:1179` |
| `RecoveryCancelled` | tuple (2) | `contracts/identity_registry/src/lib.rs:1323` |
| `RecoveryExecuted` | tuple (2) | `contracts/identity_registry/src/lib.rs:1279` |
| `RecoveryInitiated` | tuple (2) | `contracts/identity_registry/src/lib.rs:1133` |
| `ServiceRemoved` | tuple (2) | `contracts/identity_registry/src/lib.rs:1410` |
| `ThresholdUpdated` | tuple (2) | `contracts/identity_registry/src/lib.rs:1052` |
| `VerificationMethodAdded` | tuple (2) | `contracts/identity_registry/src/lib.rs:618` |
| `VerificationMethodRevoked` | tuple (2) | `contracts/identity_registry/src/lib.rs:767` |

## ihe_integration

| Topics | Payload | Source |
|---|---:|---|
| `ATNA` · `AUTH` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1048` |
| `ATNA` · `AUTO` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1744` |
| `ATNA` · `LOG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:990` |
| `BPPC` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1325` |
| `BPPC` · `REVOKE` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1348` |
| `CONN` · `TEST` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1622` |
| `CT` · `SYNC` | tuple (4) | `contracts/ihe_integration/src/lib.rs:1264` |
| `DSG` · `SIGN` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1432` |
| `HPD` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1505` |
| `MPI` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1154` |
| `PIX` · `MERGE` | tuple (2) | `contracts/ihe_integration/src/lib.rs:842` |
| `PIX` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:752` |
| `SVS` · `REG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1556` |
| `XDM` · `PKG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1237` |
| `XDR` · `SEND` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1198` |
| `XDS` · `DEPR` | tuple (2) | `contracts/ihe_integration/src/lib.rs:598` |
| `XDS` · `REG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:560` |
| `XDS` · `SUBMIT` | tuple (2) | `contracts/ihe_integration/src/lib.rs:695` |

## iot_device_management

| Topics | Payload | Source |
|---|---:|---|
| `dev_reg` · `IoT` | tuple (3) | `contracts/iot_device_management/src/events.rs:16` |
| `dev_sts` · `IoT` | tuple (3) | `contracts/iot_device_management/src/events.rs:28` |
| `fw_pub` · `IoT` | tuple (3) | `contracts/iot_device_management/src/events.rs:40` |
| `fw_sts` · `IoT` | tuple (3) | `contracts/iot_device_management/src/events.rs:52` |
| `fw_upd` · `IoT` | tuple (4) | `contracts/iot_device_management/src/events.rs:65` |
| `hbeat` · `IoT` | tuple (2) | `contracts/iot_device_management/src/events.rs:72` |
| `keyrot` · `IoT` | tuple (2) | `contracts/iot_device_management/src/events.rs:79` |

## medical_consent_nft

| Topics | Payload | Source |
|---|---:|---|
| `consent` · `delegated` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1081` |
| `consent` · `emerg_ovr` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1345` |
| `consent` · `issued` | tuple (4) | `contracts/medical_consent_nft/src/lib.rs:421` |
| `consent` · `mkt_list` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1450` |
| `consent` · `mkt_purch` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1514` |
| `consent` · `perm_upd` | tuple (2) | `contracts/medical_consent_nft/src/lib.rs:780` |
| `consent` · `revoked` | tuple (2) | `contracts/medical_consent_nft/src/lib.rs:571` |
| `consent` · `transfer` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:643` |
| `consent` · `upd_dyn` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1593` |
| `consent` · `updated` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:489` |

## medical_imaging

| Topics | Payload | Source |
|---|---:|---|
| `DISCREP` | single (1) | `contracts/medical_imaging/src/lib.rs:1472` |
| `IMG_MDL` | single (1) | `contracts/medical_imaging/src/lib.rs:634` |

## medical_imaging_ai

| Topics | Payload | Source |
|---|---:|---|
| `MDL_REG` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:319` |
| `MDL_RET` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:351` |
| `SEG` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:515` |

## medical_record_backup

| Topics | Payload | Source |
|---|---:|---|
| `BKP_POL` | single (1) | `contracts/medical_record_backup/src/lib.rs:388` |
| `BKP_REST` | tuple (2) | `contracts/medical_record_backup/src/lib.rs:672` |
| `BKP_RUN` | tuple (3) | `contracts/medical_record_backup/src/lib.rs:984` |

## medical_record_hash_registry

| Topics | Payload | Source |
|---|---:|---|
| `MEDREG` · `DUP` | tuple (2) | `contracts/medical_record_hash_registry/src/events.rs:33` |
| `MEDREG` · `STORE` | tuple (3) | `contracts/medical_record_hash_registry/src/events.rs:9` |
| `MEDREG` · `VERIFY` | tuple (3) | `contracts/medical_record_hash_registry/src/events.rs:21` |

## medical_record_search

| Topics | Payload | Source |
|---|---:|---|
| `SRCH_AUD` | tuple (3) | `contracts/medical_record_search/src/lib.rs:740` |

## medical_records

| Topics | Payload | Source |
|---|---:|---|
| `DQ_VALID` · `EVENT` | tuple (3) | `contracts/medical_records/src/events.rs:812` |
| `LOG` | single (1) | `contracts/medical_records/src/lib.rs:801` |

## medication_management

| Topics | Payload | Source |
|---|---:|---|
| `CAT_SYNC` | single (1) | `contracts/medication_management/src/lib.rs:305` |
| `MED_SYNC` | single (1) | `contracts/medication_management/src/lib.rs:759` |

## meta_tx_forwarder

| Topics | Payload | Source |
|---|---:|---|
| `fwd` | tuple (5) | `contracts/meta_tx_forwarder/src/lib.rs:177` |
| `init` | tuple (3) | `contracts/meta_tx_forwarder/src/lib.rs:129` |
| `reg_relay` | tuple (2) | `contracts/meta_tx_forwarder/src/lib.rs:269` |

## mfa

| Topics | Payload | Source |
|---|---:|---|
| `MFA` | single (1) | `contracts/mfa/src/lib.rs:228` |

## mpc_manager

| Topics | Payload | Source |
|---|---:|---|
| `mpc` · `commit` | tuple (2) | `contracts/mpc_manager/src/lib.rs:293` |
| `mpc` · `final` | tuple (2) | `contracts/mpc_manager/src/lib.rs:402` |
| `mpc` · `ml` | tuple (4) | `contracts/mpc_manager/src/lib.rs:667` |
| `mpc` · `proof` | tuple (2) | `contracts/mpc_manager/src/lib.rs:559` |
| `mpc` · `reveal` | tuple (2) | `contracts/mpc_manager/src/lib.rs:343` |
| `mpc` · `start` | tuple (2) | `contracts/mpc_manager/src/lib.rs:246` |
| `mpc` · `stats` | tuple (4) | `contracts/mpc_manager/src/lib.rs:610` |

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

## notification_system

| Topics | Payload | Source |
|---|---:|---|
| `ALRT_DEL` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:171` |
| `ALRT_NEW` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:136` |
| `ALRT_TRG` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:191` |
| `ALRT_UPD` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:157` |
| `NOTIF_ARC` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:119` |
| `NOTIF_NEW` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:93` |
| `NOTIF_RD` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:108` |
| `PREF_UPD` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:204` |
| `SNDR_ADD` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:216` |
| `SNDR_RMV` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:228` |
| `TMPL_SET` · `NOTIF` | single (1) | `contracts/notification_system/src/events.rs:240` |

## patient_consent_management

| Topics | Payload | Source |
|---|---:|---|
| `CONSENT` · `CHECK` | tuple (3) | `contracts/patient_consent_management/src/events.rs:40` |
| `CONSENT` · `GRANT` | tuple (3) | `contracts/patient_consent_management/src/events.rs:4` |
| `CONSENT` · `REVOKE` | tuple (3) | `contracts/patient_consent_management/src/events.rs:11` |
| `CONSENT` · `UNAUTH` | tuple (3) | `contracts/patient_consent_management/src/events.rs:28` |

## patient_gamification

| Topics | Payload | Source |
|---|---:|---|
| `AchCreate` | tuple (3) | `contracts/patient_gamification/src/lib.rs:330` |
| `AchEarn` | tuple (3) | `contracts/patient_gamification/src/lib.rs:401` |
| `ChalComp` | tuple (3) | `contracts/patient_gamification/src/lib.rs:594` |
| `ChalCrt` | tuple (3) | `contracts/patient_gamification/src/lib.rs:477` |
| `ChalJoin` | tuple (2) | `contracts/patient_gamification/src/lib.rs:546` |
| `ConfigUpd` | single (1) | `contracts/patient_gamification/src/lib.rs:1248` |
| `GamInit` | single (1) | `contracts/patient_gamification/src/lib.rs:262` |
| `MetricRec` | tuple (3) | `contracts/patient_gamification/src/lib.rs:1113` |
| `ProfCrt` | single (1) | `contracts/patient_gamification/src/lib.rs:860` |
| `PtsRedeem` | tuple (2) | `contracts/patient_gamification/src/lib.rs:704` |
| `RndCmt` | tuple (3) | `contracts/patient_gamification/src/lib.rs:750` |
| `RndRvl` | tuple (3) | `contracts/patient_gamification/src/lib.rs:802` |

## patient_risk_stratification

| Topics | Payload | Source |
|---|---:|---|
| `ModelReg` | single (1) | `contracts/patient_risk_stratification/src/lib.rs:195` |
| `RiskAsses` | tuple (4) | `contracts/patient_risk_stratification/src/lib.rs:263` |

## predictive_analytics

| Topics | Payload | Source |
|---|---:|---|
| `CfgUpdate` | single (1) | `contracts/predictive_analytics/src/lib.rs:192` |
| `PredMade` | tuple (4) | `contracts/predictive_analytics/src/lib.rs:289` |

## public_health_surveillance

| Topics | Payload | Source |
|---|---:|---|
| `phs` · `alert_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:569` |
| `phs` · `amr_alert` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1191` |
| `phs` · `amr_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:760` |
| `phs` · `auto_alrt` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1105` |
| `phs` · `colab_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:932` |
| `phs` · `cov_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:639` |
| `phs` · `env_alert` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1150` |
| `phs` · `env_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:703` |
| `phs` · `intv_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:872` |
| `phs` · `model_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:511` |
| `phs` · `out_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:441` |
| `phs` · `sdoh_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:809` |

## rbac

| Topics | Payload | Source |
|---|---:|---|
| `ROLE` · `ASSIGN` | tuple (2) | `contracts/rbac/src/lib.rs:80` |
| `ROLE` · `REMOVE` | tuple (2) | `contracts/rbac/src/lib.rs:118` |

## regional_node_manager

| Topics | Payload | Source |
|---|---:|---|
| `RNM_CFG` | single (1) | `contracts/regional_node_manager/src/lib.rs:467` |
| `RNM_HLTH` | single (1) | `contracts/regional_node_manager/src/lib.rs:330` |
| `RNM_INIT` | single (1) | `contracts/regional_node_manager/src/lib.rs:136` |
| `RNM_REG` | single (1) | `contracts/regional_node_manager/src/lib.rs:200` |
| `RNM_REPL` | single (1) | `contracts/regional_node_manager/src/lib.rs:393` |
| `RNM_SYNC` | single (1) | `contracts/regional_node_manager/src/lib.rs:432` |
| `RNM_UPD` | single (1) | `contracts/regional_node_manager/src/lib.rs:282` |

## remote_patient_monitoring

| Topics | Payload | Source |
|---|---:|---|
| `alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:275` |
| `caregiver_alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:194` |
| `caregiver_alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:280` |

## reputation_access_control

| Topics | Payload | Source |
|---|---:|---|
| `REPUTAC` · `APPROVED` | single (1) | `contracts/reputation_access_control/src/lib.rs:275` |
| `REPUTAC` · `DENIED` | single (1) | `contracts/reputation_access_control/src/lib.rs:298` |
| `REPUTAC` · `EMERGENCY` | single (1) | `contracts/reputation_access_control/src/lib.rs:319` |
| `REPUTAC` · `POLICY` | single (1) | `contracts/reputation_access_control/src/lib.rs:149` |
| `REPUTAC` · `REQUEST` | single (1) | `contracts/reputation_access_control/src/lib.rs:246` |
| `REPUTAC` · `REVOKE_EM` · `REVOKE_EMERGENCY` | single (1) | `contracts/reputation_access_control/src/lib.rs:339` |
| `REPUTAC` · `THRESHOLD` | tuple (2) | `contracts/reputation_access_control/src/lib.rs:404` |

## reputation_integration

| Topics | Payload | Source |
|---|---:|---|
| `REPUTINT` · `AUTO_SYNC` | single (1) | `contracts/reputation_integration/src/lib.rs:205` |
| `REPUTINT` · `BASE_UPD` | tuple (2) | `contracts/reputation_integration/src/lib.rs:432` |
| `REPUTINT` · `MAP_UPD` | single (1) | `contracts/reputation_integration/src/lib.rs:238` |
| `REPUTINT` · `SET_UPD` | single (1) | `contracts/reputation_integration/src/lib.rs:258` |
| `REPUTINT` · `SYNC` | tuple (2) | `contracts/reputation_integration/src/lib.rs:156` |

## sut_token

| Topics | Payload | Source |
|---|---:|---|
| `burn` | single (1) | `contracts/sut_token/src/lib.rs:469` |
| `mint` | single (1) | `contracts/sut_token/src/lib.rs:404` |

## sync_manager

| Topics | Payload | Source |
|---|---:|---|
| `SM_CONF` | single (1) | `contracts/sync_manager/src/lib.rs:450` |
| `SM_EXEC` | single (1) | `contracts/sync_manager/src/lib.rs:272` |
| `SM_INIT` | single (1) | `contracts/sync_manager/src/lib.rs:165` |
| `SM_INIT_S` | single (1) | `contracts/sync_manager/src/lib.rs:231` |
| `SM_LAG` | single (1) | `contracts/sync_manager/src/lib.rs:377` |
| `SM_RESO` | single (1) | `contracts/sync_manager/src/lib.rs:485` |
| `SM_RETR` | single (1) | `contracts/sync_manager/src/lib.rs:315` |
| `SM_SETP` | single (1) | `contracts/sync_manager/src/lib.rs:508` |

## timelock

| Topics | Payload | Source |
|---|---:|---|
| `Queued` | tuple (2) | `contracts/timelock/src/lib.rs:83` |

## token_sale

| Topics | Payload | Source |
|---|---:|---|
| `contribution` | tuple (4) | `contracts/token_sale/src/contract.rs:209` |
| `phase_added` | tuple (5) | `contracts/token_sale/src/contract.rs:90` |
| `sale_initialized` | tuple (4) | `contracts/token_sale/src/contract.rs:54` |
| `sale_paused` | tuple (0) | `contracts/token_sale/src/contract.rs:112` |
| `sale_unpaused` | tuple (0) | `contracts/token_sale/src/contract.rs:121` |
| `token_added` | tuple (2) | `contracts/token_sale/src/contract.rs:103` |
| `tokens_claimed` | tuple (2) | `contracts/token_sale/src/contract.rs:266` |
| `vesting_schedule_created` | tuple (4) | `contracts/token_sale/src/vesting.rs:70` |
| `vesting_schedule_updated` | tuple (5) | `contracts/token_sale/src/vesting.rs:220` |

## treasury_controller

| Topics | Payload | Source |
|---|---:|---|
| `APPROVED` | tuple (3) | `contracts/treasury_controller/src/lib.rs:354` |
| `EMERGENCY` | single (1) | `contracts/treasury_controller/src/lib.rs:475` |
| `EXECUTED` | tuple (3) | `contracts/treasury_controller/src/lib.rs:445` |
| `INIT` | single (1) | `contracts/treasury_controller/src/lib.rs:178` |
| `PROPOSAL` | tuple (3) | `contracts/treasury_controller/src/lib.rs:290` |
| `RESUMED` | single (1) | `contracts/treasury_controller/src/lib.rs:499` |

## zk_verifier

| Topics | Payload | Source |
|---|---:|---|
| `ZKVER` · `ATTEST` | tuple (2) | `contracts/zk_verifier/src/lib.rs:237` |
| `ZKVER` · `VKREG` | tuple (2) | `contracts/zk_verifier/src/lib.rs:147` |

## zkp_registry

| Topics | Payload | Source |
|---|---:|---|
| `zkp` · `circ_reg` | single (1) | `contracts/zkp_registry/src/lib.rs:310` |
| `zkp` · `cred_prf` | tuple (2) | `contracts/zkp_registry/src/lib.rs:549` |
| `zkp` · `med_proof` | tuple (2) | `contracts/zkp_registry/src/lib.rs:440` |
| `zkp` · `proof_sub` | tuple (3) | `contracts/zkp_registry/src/lib.rs:392` |
| `zkp` · `rec_proof` | tuple (3) | `contracts/zkp_registry/src/lib.rs:617` |
| `zkp` · `rng_proof` | tuple (4) | `contracts/zkp_registry/src/lib.rs:499` |

