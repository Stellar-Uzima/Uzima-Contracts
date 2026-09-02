# Contract Events

This document is auto-generated from on-chain event emissions found in `contracts/**/src/**/*.rs`.

- Registry format version: `1.0.0`
- Generated at: `2026-09-01T19:11:31.118Z`

## access_control

| Topics | Payload | Source |
|---|---:|---|
| `AC` · `ADMIN` | single (1) | `contracts/access_control/src/lib.rs:143` |
| `AC` · `BATCH_G` | tuple (2) | `contracts/access_control/src/lib.rs:359` |
| `AC` · `BATCH_PERM` | tuple (2) | `contracts/access_control/src/lib.rs:310` |
| `AC` · `BATCH_R` | tuple (2) | `contracts/access_control/src/lib.rs:383` |
| `AC` · `BATCH_REV` | tuple (2) | `contracts/access_control/src/lib.rs:334` |
| `AC` · `REVOKE` | single (1) | `contracts/access_control/src/lib.rs:170` |
| `AC` · `REVOKE` | single (1) | `contracts/access_control/src/lib.rs:282` |
| `AC` | tuple (2) | `contracts/access_control/src/lib.rs:241` |

## ai_analytics

| Topics | Payload | Source |
|---|---:|---|
| `RndStart` | single (1) | `contracts/ai_analytics/src/rounds.rs:49` |
| `SER_ADDR` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:132` |
| `SER_BYTESN` | tuple (0) | `contracts/ai_analytics/src/serialization_utils.rs:125` |
| `SER_WARN` · `EMPTY_DESC` | tuple (0) | `contracts/ai_analytics/src/types.rs:92` |
| `SER_WARN` · `NO_REFS` | tuple (0) | `contracts/ai_analytics/src/types.rs:99` |
| `SER_WARN` · `NO_UPDATES` | tuple (0) | `contracts/ai_analytics/src/types.rs:32` |
| `SER_WARN` · `ZERO_MIN` | tuple (0) | `contracts/ai_analytics/src/types.rs:25` |
| `SER_WARN` · `ZERO_SAMP` | tuple (0) | `contracts/ai_analytics/src/types.rs:60` |
| `SER_WARN` · `ZERO_TS` | tuple (0) | `contracts/ai_analytics/src/types.rs:106` |
| `analytics` · `computed` | tuple (2) | `contracts/ai_analytics/src/lazy_analytics.rs:53` |
| `cap_grant` | tuple (3) | `contracts/ai_analytics/src/capability.rs:118` |
| `cap_grant` | tuple (3) | `contracts/ai_analytics/src/capability.rs:147` |
| `cap_revoke` | tuple (2) | `contracts/ai_analytics/src/capability.rs:166` |

## aml

| Topics | Payload | Source |
|---|---:|---|
| `AML` · `STATUS` | tuple (2) | `contracts/aml/src/lib.rs:262` |
| `AML` · `VIOLATION` | tuple (2) | `contracts/aml/src/lib.rs:319` |
| `Init` | single (1) | `contracts/aml/src/lib.rs:61` |

## anomaly_detection

| Topics | Payload | Source |
|---|---:|---|
| `AccAnm` | tuple (4) | `contracts/anomaly_detection/src/batch.rs:1600` |
| `AlertAck` | single (1) | `contracts/anomaly_detection/src/batch.rs:480` |
| `AlertCrt` | tuple (3) | `contracts/anomaly_detection/src/batch.rs:1648` |
| `AlertCrt` | tuple (3) | `contracts/anomaly_detection/src/batch.rs:1703` |
| `AlertRes` | tuple (3) | `contracts/anomaly_detection/src/batch.rs:1768` |
| `AlertRes` | single (1) | `contracts/anomaly_detection/src/batch.rs:511` |
| `AnomDet` | tuple (4) | `contracts/anomaly_detection/src/batch.rs:349` |
| `AnomDet` | tuple (4) | `contracts/anomaly_detection/src/lib.rs:360` |
| `CfgUpdate` | single (1) | `contracts/anomaly_detection/src/batch.rs:213` |
| `FalsePos` | tuple (2) | `contracts/anomaly_detection/src/batch.rs:540` |
| `FalsePos` | tuple (2) | `contracts/anomaly_detection/src/lib.rs:551` |
| `FedUpd` | tuple (3) | `contracts/anomaly_detection/src/batch.rs:1899` |
| `Feedback` | tuple (4) | `contracts/anomaly_detection/src/batch.rs:1862` |
| `Feedback` | tuple (3) | `contracts/anomaly_detection/src/batch.rs:574` |
| `Feedback` | tuple (3) | `contracts/anomaly_detection/src/lib.rs:585` |
| `Infer` | tuple (4) | `contracts/anomaly_detection/src/batch.rs:1400` |
| `Init` | single (1) | `contracts/anomaly_detection/src/batch.rs:1120` |
| `MdlReg` | single (1) | `contracts/anomaly_detection/src/batch.rs:1276` |
| `Paused` | single (1) | `contracts/anomaly_detection/src/batch.rs:1149` |
| `PrescAnm` | tuple (4) | `contracts/anomaly_detection/src/batch.rs:1500` |
| `SER_ADDR` | tuple (0) | `contracts/anomaly_detection/src/payload.rs:372` |
| `SER_BYTESN` | tuple (0) | `contracts/anomaly_detection/src/payload.rs:366` |
| `Unpaused` | single (1) | `contracts/anomaly_detection/src/batch.rs:1157` |
| `ValRmvd` | single (1) | `contracts/anomaly_detection/src/batch.rs:1141` |
| `alert_ack` | single (1) | `contracts/anomaly_detection/src/lib.rs:491` |
| `alert_res` | single (1) | `contracts/anomaly_detection/src/lib.rs:522` |
| `cfg_update` | single (1) | `contracts/anomaly_detection/src/lib.rs:224` |

## anomaly_detector

| Topics | Payload | Source |
|---|---:|---|
| `AccAnm` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:715` |
| `AlertCrt` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:763` |
| `AlertCrt` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:818` |
| `AlertRes` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:883` |
| `FedUpd` | tuple (3) | `contracts/anomaly_detector/src/lib.rs:1014` |
| `Feedback` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:977` |
| `Infer` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:515` |
| `PrescAnm` | tuple (4) | `contracts/anomaly_detector/src/lib.rs:615` |
| `init` | single (1) | `contracts/anomaly_detector/src/lib.rs:241` |
| `mdl_reg` | single (1) | `contracts/anomaly_detector/src/lib.rs:391` |
| `paused` | single (1) | `contracts/anomaly_detector/src/lib.rs:267` |
| `unpaused` | single (1) | `contracts/anomaly_detector/src/lib.rs:274` |
| `val_rmvd` | single (1) | `contracts/anomaly_detector/src/lib.rs:260` |

## appointment_booking_escrow

| Topics | Payload | Source |
|---|---:|---|
| `APPT` · `BOOK` | tuple (5) | `contracts/appointment_booking_escrow/src/events.rs:11` |
| `APPT` · `CONF` | tuple (3) | `contracts/appointment_booking_escrow/src/events.rs:23` |
| `APPT` · `NOSHOW` | tuple (4) | `contracts/appointment_booking_escrow/src/events.rs:62` |
| `APPT` · `REFUND` | tuple (4) | `contracts/appointment_booking_escrow/src/events.rs:36` |
| `APPT` · `RELEASE` | tuple (4) | `contracts/appointment_booking_escrow/src/events.rs:49` |
| `APPT` · `REMINDR` | tuple (4) | `contracts/appointment_booking_escrow/src/events.rs:75` |
| `DIAG` · `AUTHFAIL` | single (1) | `contracts/appointment_booking_escrow/src/events.rs:125` |
| `DIAG` · `ENTER` | single (1) | `contracts/appointment_booking_escrow/src/events.rs:90` |
| `DIAG` · `ERR` | tuple (2) | `contracts/appointment_booking_escrow/src/events.rs:135` |
| `DIAG` · `EXIT` | single (1) | `contracts/appointment_booking_escrow/src/events.rs:98` |
| `DIAG` · `STATE` | tuple (3) | `contracts/appointment_booking_escrow/src/events.rs:106` |
| `DIAG` · `VALFAIL` | tuple (3) | `contracts/appointment_booking_escrow/src/events.rs:114` |

## audit

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT` · `AUTH_DENY` | tuple (2) | `contracts/audit/src/lib.rs:853` |
| `AUDIT` · `EXPORT` | tuple (3) | `contracts/audit/src/lib.rs:528` |
| `AUDIT` · `EXPORT` | tuple (3) | `contracts/audit/src/vec.rs:317` |
| `AUDIT` · `GRANT` | tuple (2) | `contracts/audit/src/lib.rs:272` |
| `AUDIT` · `GRANT` | tuple (2) | `contracts/audit/src/vec.rs:226` |
| `AUDIT` · `LOG` | tuple (3) | `contracts/audit/src/lib.rs:144` |
| `AUDIT` · `LOG` | tuple (3) | `contracts/audit/src/vec.rs:97` |
| `AUDIT` · `POLICY` | tuple (2) | `contracts/audit/src/lib.rs:904` |
| `AUDIT` · `PURGE` | tuple (2) | `contracts/audit/src/lib.rs:433` |
| `AUDIT` · `RETPOL` | tuple (3) | `contracts/audit/src/lib.rs:327` |
| `AUDIT` · `REVOKE` | tuple (2) | `contracts/audit/src/lib.rs:287` |
| `AUDIT` · `REVOKE` | tuple (2) | `contracts/audit/src/vec.rs:241` |
| `Init` | single (1) | `contracts/audit/src/lib.rs:83` |
| `Init` | single (1) | `contracts/audit/src/vec.rs:56` |
| `audit` · `entry` | tuple (4) | `contracts/audit/src/batch_audit.rs:88` |
| `audit` · `flushed` | tuple (2) | `contracts/audit/src/batch_audit.rs:97` |
| `cdss` · `learn_upd` | tuple (3) | `contracts/audit/src/bud.rs:205` |

## audit_forensics

| Topics | Payload | Source |
|---|---:|---|
| `AUDIT_DONE` | tuple (2) | `contracts/audit_forensics/src/events.rs:20` |
| `COMPL_RPT` | tuple (2) | `contracts/audit_forensics/src/events.rs:28` |
| `EVID` · `NEW` | tuple (3) | `contracts/audit_forensics/src/lib.rs:871` |
| `EXPORT` · `CFG_SET` | tuple (3) | `contracts/audit_forensics/src/lib.rs:944` |
| `EXPORT` · `LOGS` | tuple (4) | `contracts/audit_forensics/src/lib.rs:1007` |
| `PROV` · `NEW` | tuple (2) | `contracts/audit_forensics/src/lib.rs:768` |
| `RULE_CFG` | tuple (2) | `contracts/audit_forensics/src/events.rs:12` |
| `audit` · `archive` | single (1) | `contracts/audit_forensics/src/lib.rs:577` |
| `audit` · `compress` | tuple (3) | `contracts/audit_forensics/src/lib.rs:565` |
| `audit` · `log` | tuple (3) | `contracts/audit_forensics/src/lib.rs:279` |
| `audit` · `run` | tuple (3) | `contracts/audit_forensics/src/lib.rs:360` |
| `audit` · `share` | tuple (4) | `contracts/audit_forensics/src/lib.rs:609` |
| `audit` · `xcsync` | tuple (2) | `contracts/audit_forensics/src/lib.rs:592` |

## bridge_dispute_mediation

| Topics | Payload | Source |
|---|---:|---|
| `DisputeAccepted` | tuple (2) | `contracts/bridge_dispute_mediation/src/lib.rs:471` |
| `DisputeEscalated` | tuple (2) | `contracts/bridge_dispute_mediation/src/lib.rs:517` |
| `DisputeFiled` | tuple (2) | `contracts/bridge_dispute_mediation/src/lib.rs:403` |
| `DisputeResolved` | tuple (2) | `contracts/bridge_dispute_mediation/src/lib.rs:673` |
| `DisputeWithdrawn` | tuple (2) | `contracts/bridge_dispute_mediation/src/lib.rs:435` |
| `VoteCast` | tuple (3) | `contracts/bridge_dispute_mediation/src/lib.rs:600` |

## clinical_decision_support

| Topics | Payload | Source |
|---|---:|---|
| `cdss` · `learn_upd` | tuple (3) | `contracts/clinical_decision_support/src/lib.rs:203` |

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
| `ParticipantEnrolled` | tuple (3) | `contracts/clinical_trial/src/lib.rs:470` |
| `TrialCapacityReached` | tuple (2) | `contracts/clinical_trial/src/lib.rs:316` |
| `TrialCapacityReached` | tuple (2) | `contracts/clinical_trial/src/lib.rs:464` |
| `adverse_event` | tuple (5) | `contracts/clinical_trial/src/lib.rs:400` |
| `consent_recorded` | tuple (3) | `contracts/clinical_trial/src/lib.rs:356` |
| `patient_recruited` | tuple (3) | `contracts/clinical_trial/src/lib.rs:322` |

## code_ownership

| Topics | Payload | Source |
|---|---:|---|
| `OWNER` · `REG` | tuple (2) | `contracts/code_ownership/src/events.rs:10` |
| `OWNER` · `ROUTE` | tuple (2) | `contracts/code_ownership/src/events.rs:24` |
| `OWNER` · `UPD` | tuple (2) | `contracts/code_ownership/src/events.rs:17` |

## common_auth

| Topics | Payload | Source |
|---|---:|---|
| `AUTH_DENY` | tuple (3) | `contracts/common_auth/src/lib.rs:222` |
| `POLICY` | tuple (3) | `contracts/common_auth/src/lib.rs:286` |
| `policy` · `role_chk` | tuple (2) | `contracts/common_auth/src/policy_engine.rs:243` |

## common_error

| Topics | Payload | Source |
|---|---:|---|
| `pause` · `unpause` | tuple (2) | `contracts/common_error/src/pause.rs:22` |

## contract_monitoring

| Topics | Payload | Source |
|---|---:|---|
| `MON` · `ALERT` | single (1) | `contracts/contract_monitoring/src/lib.rs:290` |
| `MON` · `ALERT` | single (1) | `contracts/contract_monitoring/src/lib.rs:522` |
| `MON` · `ALERT` | single (1) | `contracts/contract_monitoring/src/lib.rs:542` |
| `SEC` · `ALERT` | tuple (3) | `contracts/contract_monitoring/src/security_telemetry.rs:359` |
| `SEC` · `EVT` | tuple (5) | `contracts/contract_monitoring/src/security_telemetry.rs:374` |
| `SEC` · `LOCK` | tuple (2) | `contracts/contract_monitoring/src/security_telemetry.rs:218` |
| `SEC` · `UNLOCK` | tuple (2) | `contracts/contract_monitoring/src/security_telemetry.rs:258` |
| `telemetry` | single (1) | `contracts/contract_monitoring/src/telemetry.rs:465` |

## contract_template

| Topics | Payload | Source |
|---|---:|---|
| `adm_xfer` | tuple (2) | `contracts/contract_template/src/events.rs:9` |

## contract_usage_analytics

| Topics | Payload | Source |
|---|---:|---|
| `usage` | tuple (4) | `contracts/contract_usage_analytics/src/lib.rs:192` |

## contract_verification

| Topics | Payload | Source |
|---|---:|---|
| `VERIFY` · `ABI` | single (1) | `contracts/contract_verification/src/lib.rs:199` |
| `VERIFY` · `META` | tuple (2) | `contracts/contract_verification/src/lib.rs:154` |
| `VERIFY` · `OK` | single (1) | `contracts/contract_verification/src/lib.rs:221` |

## credential_notifications

| Topics | Payload | Source |
|---|---:|---|
| `CRED` · `NOTIFY` | tuple (4) | `contracts/credential_notifications/src/lib.rs:107` |

## credential_registry

| Topics | Payload | Source |
|---|---:|---|
| `CREDREG` · `BROOT` | tuple (2) | `contracts/credential_registry/src/lib.rs:358` |
| `CREDREG` · `IADMIN` | tuple (2) | `contracts/credential_registry/src/lib.rs:128` |
| `CREDREG` · `ROOT` | tuple (2) | `contracts/credential_registry/src/lib.rs:188` |

## cross_chain_access

| Topics | Payload | Source |
|---|---:|---|
| `Paused` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1024` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1038` |
| `access_control_initialized` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:304` |
| `access_granted` | tuple (4) | `contracts/cross_chain_access/src/lib.rs:352` |
| `access_logged` | tuple (5) | `contracts/cross_chain_access/src/lib.rs:716` |
| `access_requested` | tuple (6) | `contracts/cross_chain_access/src/lib.rs:490` |
| `delegation_created` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:603` |
| `delegation_revoked` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:629` |
| `emergency_auto_approved` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:1213` |
| `emergency_configured` | tuple (2) | `contracts/cross_chain_access/src/lib.rs:668` |
| `request_processed` | tuple (3) | `contracts/cross_chain_access/src/lib.rs:556` |
| `swap_accepted` | tuple (3) | `contracts/cross_chain_access/src/lib.rs:831` |
| `swap_proposed` | tuple (4) | `contracts/cross_chain_access/src/lib.rs:778` |

## cross_chain_bridge

| Topics | Payload | Source |
|---|---:|---|
| `JUR_CHECK` | tuple (2) | `contracts/cross_chain_bridge/src/events.rs:13` |
| `MessageFailed` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:937` |
| `MessageRetried` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:989` |
| `MessageSubmitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:770` |
| `OperationCreated` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:1712` |
| `OperationRefunded` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1743` |
| `OperationStatusUpdated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1828` |
| `REQUIRED` · `JurisdictionCheck` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:2428` |
| `RefundProcessed` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:2375` |
| `TimeoutExtended` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1795` |
| `atomic_tx_initiated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1023` |
| `bridge` · `rel_add` | single (1) | `contracts/cross_chain_bridge/src/lib.rs:2174` |
| `bridge` · `rel_rm` | single (1) | `contracts/cross_chain_bridge/src/lib.rs:2194` |
| `event_synced` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:1633` |
| `message_confirmed` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:860` |
| `message_executed` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:903` |
| `message_submitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:692` |
| `message_verified` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:854` |
| `oracle_data_aggregated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1396` |
| `oracle_report_submitted` | tuple (4) | `contracts/cross_chain_bridge/src/lib.rs:1332` |
| `paused` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:598` |
| `proof_submitted` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1455` |
| `proof_verified` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1534` |
| `record_ref_registered` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1170` |
| `rollback_initiated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:1891` |
| `sync_status_updated` | tuple (3) | `contracts/cross_chain_bridge/src/lib.rs:1213` |
| `unpaused` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:611` |
| `validator_deactivated` | tuple (2) | `contracts/cross_chain_bridge/src/lib.rs:546` |

## cross_chain_enhancements

| Topics | Payload | Source |
|---|---:|---|
| `rl` · `set` | tuple (2) | `contracts/cross_chain_enhancements/src/lib.rs:343` |
| `zk` · `integrity` | tuple (3) | `contracts/cross_chain_enhancements/src/lib.rs:272` |
| `zk` · `own_proof` | tuple (3) | `contracts/cross_chain_enhancements/src/lib.rs:188` |
| `zk` · `verified` | tuple (2) | `contracts/cross_chain_enhancements/src/lib.rs:227` |

## cross_chain_identity

| Topics | Payload | Source |
|---|---:|---|
| `Paused` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:336` |
| `Unpaused` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:350` |
| `attestation_added` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:485` |
| `identity_contract_initialized` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:225` |
| `identity_revoked` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:516` |
| `identity_verified` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:762` |
| `sync_initiated` | tuple (4) | `contracts/cross_chain_identity/src/lib.rs:571` |
| `validator_deactivated` | tuple (2) | `contracts/cross_chain_identity/src/lib.rs:282` |
| `verification_approved` | tuple (4) | `contracts/cross_chain_identity/src/lib.rs:473` |
| `verification_requested` | tuple (3) | `contracts/cross_chain_identity/src/lib.rs:400` |

## crypto_registry

| Topics | Payload | Source |
|---|---:|---|
| `KeyRotated` | tuple (3) | `contracts/crypto_registry/src/lib.rs:492` |
| `crypto` · `bundle` | tuple (2) | `contracts/crypto_registry/src/lib.rs:233` |
| `crypto` · `revoke` | tuple (2) | `contracts/crypto_registry/src/lib.rs:258` |

## deprecation_framework

| Topics | Payload | Source |
|---|---:|---|
| `DEPREC` · `CHECKLIST` | single (1) | `contracts/deprecation_framework/src/events.rs:45` |
| `DEPREC` · `COMM` | tuple (2) | `contracts/deprecation_framework/src/events.rs:38` |
| `DEPREC` · `DONE` | tuple (2) | `contracts/deprecation_framework/src/events.rs:52` |
| `DEPREC` · `GUIDE` | tuple (2) | `contracts/deprecation_framework/src/events.rs:24` |
| `DEPREC` · `MARKED` | tuple (2) | `contracts/deprecation_framework/src/events.rs:10` |
| `DEPREC` · `PHASE` | tuple (2) | `contracts/deprecation_framework/src/events.rs:31` |
| `DEPREC` · `TIMELINE` | tuple (2) | `contracts/deprecation_framework/src/events.rs:17` |

## differential_privacy

| Topics | Payload | Source |
|---|---:|---|
| `dp` · `budget` | tuple (3) | `contracts/differential_privacy/src/lib.rs:150` |
| `dp` · `gaussian` | tuple (3) | `contracts/differential_privacy/src/lib.rs:285` |
| `dp` · `laplace` | tuple (3) | `contracts/differential_privacy/src/lib.rs:217` |

## digital_twin

| Topics | Payload | Source |
|---|---:|---|
| `DT_CREATED` | tuple (2) | `contracts/digital_twin/src/lib.rs:398` |
| `DT_DATAPOINT` | single (1) | `contracts/digital_twin/src/lib.rs:568` |
| `DT_GD_SET` | single (1) | `contracts/digital_twin/src/lib.rs:338` |
| `DT_INIT` | single (1) | `contracts/digital_twin/src/lib.rs:310` |
| `DT_MODEL` | tuple (2) | `contracts/digital_twin/src/lib.rs:616` |
| `DT_MR_SET` | single (1) | `contracts/digital_twin/src/lib.rs:324` |
| `DT_PREDICTION` | tuple (2) | `contracts/digital_twin/src/lib.rs:673` |
| `DT_SIM` | tuple (2) | `contracts/digital_twin/src/lib.rs:731` |
| `DT_SIM_COMP` | single (1) | `contracts/digital_twin/src/lib.rs:771` |
| `DT_SNAPSHOT` | tuple (2) | `contracts/digital_twin/src/lib.rs:843` |
| `DT_STATUS` | tuple (2) | `contracts/digital_twin/src/lib.rs:437` |
| `DT_STREAM` | tuple (2) | `contracts/digital_twin/src/lib.rs:508` |
| `DT_SYNC` | tuple (2) | `contracts/digital_twin/src/lib.rs:903` |

## dispute_resolution

| Topics | Payload | Source |
|---|---:|---|
| `DISP` · `CLOSED` | tuple (2) | `contracts/dispute_resolution/src/lib.rs:474` |
| `DISP` · `DELIB` | tuple (2) | `contracts/dispute_resolution/src/lib.rs:343` |
| `DISP` · `ESCAL` | tuple (3) | `contracts/dispute_resolution/src/lib.rs:435` |
| `DISP` · `EVD_SUB` | tuple (2) | `contracts/dispute_resolution/src/lib.rs:313` |
| `DISP` · `EVIDENCE` | tuple (2) | `contracts/dispute_resolution/src/lib.rs:256` |
| `DISP` · `FILED` | tuple (5) | `contracts/dispute_resolution/src/lib.rs:195` |
| `DISP` · `RESOLVED` | tuple (3) | `contracts/dispute_resolution/src/lib.rs:389` |
| `DISP` · `REVIEW` | tuple (2) | `contracts/dispute_resolution/src/lib.rs:226` |

## drug_discovery

| Topics | Payload | Source |
|---|---:|---|
| `CfgInt` | single (1) | `contracts/drug_discovery/src/lib.rs:301` |

## emergency_access_override

| Topics | Payload | Source |
|---|---:|---|
| `EMER` · `APPR` | tuple (4) | `contracts/emergency_access_override/src/events.rs:121` |
| `EMER` · `AUDIT` | tuple (3) | `contracts/emergency_access_override/src/events.rs:67` |
| `EMER` · `CDUPD` | tuple (2) | `contracts/emergency_access_override/src/events.rs:183` |
| `EMER` · `CHECK` | tuple (4) | `contracts/emergency_access_override/src/events.rs:147` |
| `EMER` · `DUPA` | tuple (4) | `contracts/emergency_access_override/src/events.rs:134` |
| `EMER` · `GRANT` | tuple (4) | `contracts/emergency_access_override/src/events.rs:108` |
| `EMER` · `RATELMT` | tuple (3) | `contracts/emergency_access_override/src/events.rs:176` |
| `EMER` · `REVOKE` | tuple (3) | `contracts/emergency_access_override/src/events.rs:159` |
| `EmergencyAccessGranted` | tuple (2) | `contracts/emergency_access_override/src/lib.rs:921` |
| `EmergencyApproval` | tuple (2) | `contracts/emergency_access_override/src/lib.rs:913` |
| `EmergencyRequested` | tuple (3) | `contracts/emergency_access_override/src/lib.rs:875` |
| `emergency` · `access` | tuple (3) | `contracts/emergency_access_override/src/admin_recovery.rs:269` |
| `recovery` · `approved` | tuple (2) | `contracts/emergency_access_override/src/admin_recovery.rs:204` |
| `recovery` · `executed` | tuple (2) | `contracts/emergency_access_override/src/admin_recovery.rs:236` |
| `recovery` · `proposed` | tuple (3) | `contracts/emergency_access_override/src/admin_recovery.rs:155` |

## escrow

| Topics | Payload | Source |
|---|---:|---|
| `EscNew` | tuple (4) | `contracts/escrow/src/lib.rs:275` |
| `EscRel` | tuple (6) | `contracts/escrow/src/lib.rs:379` |
| `Refunded` | tuple (4) | `contracts/escrow/src/lib.rs:431` |

## explainable_ai

| Topics | Payload | Source |
|---|---:|---|
| `ExpFull` | tuple (3) | `contracts/explainable_ai/src/lib.rs:309` |
| `ExpReq` | tuple (3) | `contracts/explainable_ai/src/lib.rs:236` |
| `cf` · `created` | tuple (2) | `contracts/explainable_ai/src/lib.rs:553` |
| `shap` · `created` | tuple (2) | `contracts/explainable_ai/src/lib.rs:480` |

## failover_detector

| Topics | Payload | Source |
|---|---:|---|
| `FD_CRIT` | single (1) | `contracts/failover_detector/src/lib.rs:280` |
| `FD_DEAC` | single (1) | `contracts/failover_detector/src/lib.rs:513` |
| `FD_INIT` | single (1) | `contracts/failover_detector/src/lib.rs:157` |
| `FD_PLAN` | single (1) | `contracts/failover_detector/src/lib.rs:344` |
| `FD_REC` | single (1) | `contracts/failover_detector/src/lib.rs:484` |

## federated_learning

| Topics | Payload | Source |
|---|---:|---|
| `RndFin` | tuple (4) | `contracts/federated_learning/src/lib.rs:812` |
| `UpdSub` | tuple (3) | `contracts/federated_learning/src/lib.rs:520` |
| `agg_start` | single (1) | `contracts/federated_learning/src/lib.rs:678` |
| `rnd_start` | single (1) | `contracts/federated_learning/src/lib.rs:395` |

## fhir_integration

| Topics | Payload | Source |
|---|---:|---|
| `DataExportRequested` | tuple (3) | `contracts/fhir_integration/src/lib.rs:843` |

## forensics

| Topics | Payload | Source |
|---|---:|---|
| `FORENSIC` · `B_LIST` | single (1) | `contracts/forensics/src/lib.rs:185` |
| `FORENSIC` · `COLLECT` | tuple (5) | `contracts/forensics/src/lib.rs:88` |
| `FORENSIC` · `REPORT` | tuple (3) | `contracts/forensics/src/lib.rs:149` |

## genomic_data

| Topics | Payload | Source |
|---|---:|---|
| `LOG` | single (1) | `contracts/genomic_data/src/lib.rs:305` |
| `WITHDRAWAL` · `GENOMIC_CONSENT` | single (1) | `contracts/genomic_data/src/lib.rs:693` |

## governor

| Topics | Payload | Source |
|---|---:|---|
| `CLEANUP` · `PROPS` | tuple (2) | `contracts/governor/src/lib.rs:383` |
| `Vote` | tuple (3) | `contracts/governor/src/lib.rs:273` |

## health_data_access_logging

| Topics | Payload | Source |
|---|---:|---|
| `ACCESS` · `LOG` | tuple (6) | `contracts/health_data_access_logging/src/lib.rs:114` |
| `ACCESS` · `PURGE` | tuple (2) | `contracts/health_data_access_logging/src/lib.rs:366` |

## healthcare_analytics_dashboard

| Topics | Payload | Source |
|---|---:|---|
| `AiSync` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:1036` |
| `CompAuto` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:992` |
| `DPNoise` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:1223` |
| `DashSnap` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:823` |
| `LakeCfg` | tuple (2) | `contracts/healthcare_analytics_dashboard/src/lib.rs:533` |
| `LakeOpt` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:676` |
| `LakeSync` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/lib.rs:616` |
| `PrivAgg` | tuple (4) | `contracts/healthcare_analytics_dashboard/src/lib.rs:747` |
| `cap_grant` | tuple (3) | `contracts/healthcare_analytics_dashboard/src/analytics_capability.rs:74` |
| `cap_revoke` | tuple (2) | `contracts/healthcare_analytics_dashboard/src/analytics_capability.rs:93` |
| `dash_init` | single (1) | `contracts/healthcare_analytics_dashboard/src/lib.rs:358` |
| `tpl_create` | single (1) | `contracts/healthcare_analytics_dashboard/src/lib.rs:859` |

## healthcare_compliance

| Topics | Payload | Source |
|---|---:|---|
| `audit_event` | tuple (6) | `contracts/healthcare_compliance/src/lib.rs:641` |
| `breach_reported` | tuple (5) | `contracts/healthcare_compliance/src/lib.rs:720` |
| `compliance_report_submitted` | tuple (4) | `contracts/healthcare_compliance/src/lib.rs:1224` |
| `consent_granted` | tuple (3) | `contracts/healthcare_compliance/src/lib.rs:484` |
| `consent_revoked` | tuple (3) | `contracts/healthcare_compliance/src/lib.rs:546` |
| `health_check` | tuple (2) | `contracts/healthcare_compliance/src/lib.rs:406` |

## healthcare_data_marketplace

| Topics | Payload | Source |
|---|---:|---|
| `TierPurchased` | tuple (3) | `contracts/healthcare_data_marketplace/src/lib.rs:688` |
| `settled` | tuple (3) | `contracts/healthcare_data_marketplace/src/lib.rs:497` |

## healthcare_oracle_network

| Topics | Payload | Source |
|---|---:|---|
| `DUPLICATE_SUBMISSION` | tuple (4) | `contracts/healthcare_oracle_network/src/utils.rs:141` |
| `MISBEHAVIOR_REPORTED` | tuple (5) | `contracts/healthcare_oracle_network/src/submissions.rs:232` |
| `ORACLE_SLASHED` | tuple (3) | `contracts/healthcare_oracle_network/src/utils.rs:115` |

## healthcare_payment

| Topics | Payload | Source |
|---|---:|---|
| `CB_ANOM` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1738` |
| `CLAIM_PD` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1142` |
| `COV_PROOF` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1485` |
| `COV_VER` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1523` |
| `DIAG` · `ENTER` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:1022` |
| `DIAG` · `EXIT` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:1089` |
| `DIAG` · `STATE` | tuple (4) | `contracts/healthcare_payment/src/lib.rs:1061` |
| `DIAG` · `VALFAIL` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1038` |
| `claim_edi` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:781` |
| `claim_pd` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1084` |
| `cov_834` | tuple (2) | `contracts/healthcare_payment/src/lib.rs:820` |
| `elig` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:661` |
| `eob` | tuple (3) | `contracts/healthcare_payment/src/lib.rs:1011` |

## healthcare_reputation

| Topics | Payload | Source |
|---|---:|---|
| `HLTHREP` · `CONDUCT` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:523` |
| `HLTHREP` · `CRED_ADD` | tuple (2) | `contracts/healthcare_reputation/src/lib.rs:293` |
| `HLTHREP` · `CRED_VER` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:336` |
| `HLTHREP` · `DISPUTE` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:609` |
| `HLTHREP` · `DISP_RES` | tuple (2) | `contracts/healthcare_reputation/src/lib.rs:650` |
| `HLTHREP` · `FEEDBACK` | tuple (3) | `contracts/healthcare_reputation/src/lib.rs:428` |

## homomorphic_registry

| Topics | Payload | Source |
|---|---:|---|
| `he` · `ctx` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:672` |
| `he` · `key` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:253` |
| `he` · `submit` | tuple (2) | `contracts/homomorphic_registry/src/lib.rs:759` |

## identity_registry

| Topics | Payload | Source |
|---|---:|---|
| `Attested` | tuple (3) | `contracts/identity_registry/src/lib.rs:1756` |
| `HealthCheck` | tuple (2) | `contracts/identity_registry/src/lib.rs:405` |
| `Paused` | tuple (2) | `contracts/identity_registry/src/lib.rs:468` |
| `StakeDeposited` | tuple (3) | `contracts/identity_registry/src/lib.rs:2202` |
| `StakeSlashed` | tuple (3) | `contracts/identity_registry/src/lib.rs:2265` |
| `StakeWithdrawn` | tuple (2) | `contracts/identity_registry/src/lib.rs:2236` |
| `Unpaused` | tuple (2) | `contracts/identity_registry/src/lib.rs:479` |
| `credential_issued` | tuple (4) | `contracts/identity_registry/src/lib.rs:1012` |
| `credential_revoked` | tuple (2) | `contracts/identity_registry/src/lib.rs:1088` |
| `did_created` | tuple (2) | `contracts/identity_registry/src/lib.rs:607` |
| `did_updated` | tuple (2) | `contracts/identity_registry/src/lib.rs:676` |
| `guardian_added` | tuple (3) | `contracts/identity_registry/src/lib.rs:1167` |
| `initialized` | tuple (2) | `contracts/identity_registry/src/lib.rs:387` |
| `recovery_approved` | tuple (2) | `contracts/identity_registry/src/lib.rs:1344` |
| `recovery_cancelled` | tuple (2) | `contracts/identity_registry/src/lib.rs:1489` |
| `recovery_executed` | tuple (2) | `contracts/identity_registry/src/lib.rs:1444` |
| `recovery_initiated` | tuple (2) | `contracts/identity_registry/src/lib.rs:1297` |
| `service_removed` | tuple (2) | `contracts/identity_registry/src/lib.rs:1587` |
| `threshold_updated` | tuple (2) | `contracts/identity_registry/src/lib.rs:1216` |
| `verification_method_added` | tuple (2) | `contracts/identity_registry/src/lib.rs:777` |
| `verification_method_revoked` | tuple (2) | `contracts/identity_registry/src/lib.rs:932` |

## ihe_integration

| Topics | Payload | Source |
|---|---:|---|
| `ATNA` · `AUTH` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1080` |
| `ATNA` · `AUTO` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1778` |
| `ATNA` · `LOG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1022` |
| `BPPC` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1357` |
| `BPPC` · `REVOKE` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1380` |
| `CONN` · `TEST` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1654` |
| `CT` · `SYNC` | tuple (4) | `contracts/ihe_integration/src/lib.rs:1296` |
| `DSG` · `SIGN` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1464` |
| `HPD` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1537` |
| `MPI` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:1186` |
| `PIX` · `MERGE` | tuple (2) | `contracts/ihe_integration/src/lib.rs:874` |
| `PIX` · `REG` | tuple (2) | `contracts/ihe_integration/src/lib.rs:784` |
| `SVS` · `REG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1588` |
| `XDM` · `PKG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1269` |
| `XDR` · `SEND` | tuple (3) | `contracts/ihe_integration/src/lib.rs:1230` |
| `XDS` · `DEPR` | tuple (2) | `contracts/ihe_integration/src/lib.rs:630` |
| `XDS` · `REG` | tuple (3) | `contracts/ihe_integration/src/lib.rs:592` |
| `XDS` · `SUBMIT` | tuple (2) | `contracts/ihe_integration/src/lib.rs:727` |

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

## load_testing

| Topics | Payload | Source |
|---|---:|---|
| `LOAD` · `DONE` | single (1) | `contracts/load_testing/src/lib.rs:146` |

## medical_consent_nft

| Topics | Payload | Source |
|---|---:|---|
| `consent` · `delegated` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1102` |
| `consent` · `emerg_ovr` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1366` |
| `consent` · `issued` | tuple (4) | `contracts/medical_consent_nft/src/lib.rs:442` |
| `consent` · `mkt_list` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1471` |
| `consent` · `mkt_purch` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1535` |
| `consent` · `perm_upd` | tuple (2) | `contracts/medical_consent_nft/src/lib.rs:801` |
| `consent` · `revoked` | tuple (2) | `contracts/medical_consent_nft/src/lib.rs:592` |
| `consent` · `transfer` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:664` |
| `consent` · `upd_dyn` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:1614` |
| `consent` · `updated` | tuple (3) | `contracts/medical_consent_nft/src/lib.rs:510` |

## medical_imaging

| Topics | Payload | Source |
|---|---:|---|
| `DISCREP` | single (1) | `contracts/medical_imaging/src/lib.rs:1502` |
| `img_mdl` | single (1) | `contracts/medical_imaging/src/lib.rs:663` |

## medical_imaging_ai

| Topics | Payload | Source |
|---|---:|---|
| `MDL_REG` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:343` |
| `MDL_RET` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:375` |
| `SEG` | single (1) | `contracts/medical_imaging_ai/src/lib.rs:539` |

## medical_record_backup

| Topics | Payload | Source |
|---|---:|---|
| `bkp_pol` | single (1) | `contracts/medical_record_backup/src/lib.rs:412` |
| `bkp_rest` | tuple (2) | `contracts/medical_record_backup/src/lib.rs:696` |
| `bkp_run` | tuple (3) | `contracts/medical_record_backup/src/lib.rs:1008` |

## medical_record_hash_registry

| Topics | Payload | Source |
|---|---:|---|
| `MEDREG` · `DUP` | tuple (2) | `contracts/medical_record_hash_registry/src/events.rs:33` |
| `MEDREG` · `STORE` | tuple (3) | `contracts/medical_record_hash_registry/src/events.rs:9` |
| `MEDREG` · `VERIFY` | tuple (3) | `contracts/medical_record_hash_registry/src/events.rs:21` |

## medical_record_search

| Topics | Payload | Source |
|---|---:|---|
| `ENC_IDX_CR` | tuple (2) | `contracts/medical_record_search/src/lib.rs:523` |
| `ENC_IDX_RM` | tuple (2) | `contracts/medical_record_search/src/lib.rs:621` |
| `ENC_IDX_UP` | tuple (2) | `contracts/medical_record_search/src/lib.rs:594` |
| `SRCH_AUD` | tuple (3) | `contracts/medical_record_search/src/lib.rs:934` |

## medical_records

| Topics | Payload | Source |
|---|---:|---|
| `COMP` · `CREATE` | tuple (2) | `contracts/medical_records/src/lib.rs:7440` |
| `COMP` · `GRANT` | tuple (2) | `contracts/medical_records/src/lib.rs:7482` |
| `COMP` · `REVOKE` | tuple (2) | `contracts/medical_records/src/lib.rs:7557` |
| `EXPORT` · `DATA` | tuple (3) | `contracts/medical_records/src/lib.rs:5507` |
| `LOG` | single (1) | `contracts/medical_records/src/lib.rs:1190` |
| `PAUSED` | single (1) | `contracts/medical_records/src/events.rs:234` |
| `RECORD` · `ATTACH` | tuple (3) | `contracts/medical_records/src/lib.rs:7649` |
| `SCH_BREAK` | tuple (6) | `contracts/medical_records/src/lib.rs:7823` |
| `SCH_EVO` | tuple (7) | `contracts/medical_records/src/lib.rs:7873` |
| `SCH_REG` | tuple (4) | `contracts/medical_records/src/lib.rs:7788` |
| `TradRecAdded` | tuple (5) | `contracts/medical_records/src/lib.rs:7062` |

## medication_management

| Topics | Payload | Source |
|---|---:|---|
| `CAT_SYNC` | single (1) | `contracts/medication_management/src/lib.rs:323` |
| `MED_SYNC` | single (1) | `contracts/medication_management/src/lib.rs:867` |

## meta_tx_forwarder

| Topics | Payload | Source |
|---|---:|---|
| `deact_rel` | single (1) | `contracts/meta_tx_forwarder/src/lib.rs:350` |
| `fwd` | tuple (5) | `contracts/meta_tx_forwarder/src/lib.rs:407` |
| `init` | tuple (3) | `contracts/meta_tx_forwarder/src/lib.rs:221` |
| `reg_key` | tuple (2) | `contracts/meta_tx_forwarder/src/lib.rs:246` |

## mfa

| Topics | Payload | Source |
|---|---:|---|
| `MFA` | single (1) | `contracts/mfa/src/lib.rs:232` |

## mpc_manager

| Topics | Payload | Source |
|---|---:|---|
| `mpc` · `commit` | tuple (2) | `contracts/mpc_manager/src/lib.rs:322` |
| `mpc` · `final` | tuple (2) | `contracts/mpc_manager/src/lib.rs:432` |
| `mpc` · `ml` | tuple (4) | `contracts/mpc_manager/src/lib.rs:697` |
| `mpc` · `proof` | tuple (2) | `contracts/mpc_manager/src/lib.rs:589` |
| `mpc` · `reveal` | tuple (2) | `contracts/mpc_manager/src/lib.rs:372` |
| `mpc` · `start` | tuple (2) | `contracts/mpc_manager/src/lib.rs:275` |
| `mpc` · `stats` | tuple (4) | `contracts/mpc_manager/src/lib.rs:640` |

## multi_region_orchestrator

| Topics | Payload | Source |
|---|---:|---|
| `DRO_FAIL` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:396` |
| `DRO_HLTH` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:493` |
| `DRO_INIT` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:187` |
| `DRO_REGI` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:255` |
| `DRO_SETP` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:565` |
| `DRO_SLAM` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:528` |
| `DRO_STAT` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:313` |
| `DRO_SYNC` | single (1) | `contracts/multi_region_orchestrator/src/lib.rs:450` |

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
| `CONSENT` · `CHECK` | tuple (3) | `contracts/patient_consent_management/src/events.rs:30` |
| `CONSENT` · `EXPIRED` | tuple (3) | `contracts/patient_consent_management/src/events.rs:37` |
| `CONSENT` · `EXP_SOON` | tuple (4) | `contracts/patient_consent_management/src/events.rs:72` |
| `CONSENT` · `GRANT` | tuple (3) | `contracts/patient_consent_management/src/events.rs:4` |
| `CONSENT` · `JURISDICT` | tuple (3) | `contracts/patient_consent_management/src/events.rs:50` |
| `CONSENT` · `POLICY` | tuple (2) | `contracts/patient_consent_management/src/events.rs:58` |
| `CONSENT` · `REVOKE` | tuple (3) | `contracts/patient_consent_management/src/events.rs:11` |
| `Paused` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:537` |
| `ProxyConsentGranted` | tuple (3) | `contracts/patient_consent_management/src/lib.rs:852` |
| `ProxyConsentRevoked` | tuple (3) | `contracts/patient_consent_management/src/lib.rs:874` |
| `ProxyDesignated` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:815` |
| `Unpaused` | tuple (2) | `contracts/patient_consent_management/src/lib.rs:548` |

## patient_gamification

| Topics | Payload | Source |
|---|---:|---|
| `AchCreate` | tuple (3) | `contracts/patient_gamification/src/lib.rs:357` |
| `AchEarn` | tuple (3) | `contracts/patient_gamification/src/lib.rs:428` |
| `ChalComp` | tuple (3) | `contracts/patient_gamification/src/lib.rs:621` |
| `ChalCrt` | tuple (3) | `contracts/patient_gamification/src/lib.rs:504` |
| `ChalJoin` | tuple (2) | `contracts/patient_gamification/src/lib.rs:573` |
| `ConfigUpd` | single (1) | `contracts/patient_gamification/src/lib.rs:1276` |
| `GamInit` | single (1) | `contracts/patient_gamification/src/lib.rs:287` |
| `MetricRec` | tuple (3) | `contracts/patient_gamification/src/lib.rs:1140` |
| `ProfCrt` | single (1) | `contracts/patient_gamification/src/lib.rs:887` |
| `PtsRedeem` | tuple (2) | `contracts/patient_gamification/src/lib.rs:731` |
| `RndCmt` | tuple (3) | `contracts/patient_gamification/src/lib.rs:777` |
| `RndRvl` | tuple (3) | `contracts/patient_gamification/src/lib.rs:829` |

## patient_risk_stratification

| Topics | Payload | Source |
|---|---:|---|
| `ModelReg` | single (1) | `contracts/patient_risk_stratification/src/lib.rs:217` |
| `RiskAsses` | tuple (4) | `contracts/patient_risk_stratification/src/lib.rs:285` |

## pharma_supply_chain

| Topics | Payload | Source |
|---|---:|---|
| `BATCH` · `CREATE` | tuple (3) | `contracts/pharma_supply_chain/src/lib.rs:311` |

## predictive_analytics

| Topics | Payload | Source |
|---|---:|---|
| `CfgUpdate` | single (1) | `contracts/predictive_analytics/src/config.rs:77` |
| `PredMade` | tuple (4) | `contracts/predictive_analytics/src/predictions.rs:84` |
| `cfg_update` | single (1) | `contracts/predictive_analytics/src/lib.rs:77` |

## public_health_surveillance

| Topics | Payload | Source |
|---|---:|---|
| `phs` · `alert_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:591` |
| `phs` · `amr_alert` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1216` |
| `phs` · `amr_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:782` |
| `phs` · `auto_alrt` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1129` |
| `phs` · `colab_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:954` |
| `phs` · `cov_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:661` |
| `phs` · `env_alert` | tuple (2) | `contracts/public_health_surveillance/src/lib.rs:1174` |
| `phs` · `env_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:725` |
| `phs` · `intv_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:894` |
| `phs` · `model_crt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:533` |
| `phs` · `out_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:463` |
| `phs` · `sdoh_rpt` | tuple (3) | `contracts/public_health_surveillance/src/lib.rs:831` |

## regional_node_manager

| Topics | Payload | Source |
|---|---:|---|
| `RNM_CFG` | single (1) | `contracts/regional_node_manager/src/lib.rs:483` |
| `RNM_HLTH` | single (1) | `contracts/regional_node_manager/src/lib.rs:346` |
| `RNM_INIT` | single (1) | `contracts/regional_node_manager/src/lib.rs:152` |
| `RNM_REG` | single (1) | `contracts/regional_node_manager/src/lib.rs:216` |
| `RNM_REPL` | single (1) | `contracts/regional_node_manager/src/lib.rs:409` |
| `RNM_SYNC` | single (1) | `contracts/regional_node_manager/src/lib.rs:448` |
| `RNM_UPD` | single (1) | `contracts/regional_node_manager/src/lib.rs:298` |

## remote_patient_monitoring

| Topics | Payload | Source |
|---|---:|---|
| `alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:279` |
| `caregiver_alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:198` |
| `caregiver_alert` | single (1) | `contracts/remote_patient_monitoring/src/lib.rs:284` |

## reputation_access_control

| Topics | Payload | Source |
|---|---:|---|
| `REPUTAC` · `APPROVED` | single (1) | `contracts/reputation_access_control/src/lib.rs:293` |
| `REPUTAC` · `DENIED` | single (1) | `contracts/reputation_access_control/src/lib.rs:316` |
| `REPUTAC` · `EMERGENCY` | single (1) | `contracts/reputation_access_control/src/lib.rs:337` |
| `REPUTAC` · `POLICY` | single (1) | `contracts/reputation_access_control/src/lib.rs:167` |
| `REPUTAC` · `REQUEST` | single (1) | `contracts/reputation_access_control/src/lib.rs:264` |
| `REPUTAC` · `REVOKE_EM` · `REVOKE_EMERGENCY` | single (1) | `contracts/reputation_access_control/src/lib.rs:357` |
| `REPUTAC` · `THRESHOLD` | tuple (2) | `contracts/reputation_access_control/src/lib.rs:422` |

## reputation_integration

| Topics | Payload | Source |
|---|---:|---|
| `REPUTINT` · `AUTO_SYNC` | single (1) | `contracts/reputation_integration/src/lib.rs:222` |
| `REPUTINT` · `BASE_UPD` | tuple (2) | `contracts/reputation_integration/src/lib.rs:451` |
| `REPUTINT` · `MAP_UPD` | single (1) | `contracts/reputation_integration/src/lib.rs:255` |
| `REPUTINT` · `SET_UPD` | single (1) | `contracts/reputation_integration/src/lib.rs:275` |
| `REPUTINT` · `SYNC` | tuple (2) | `contracts/reputation_integration/src/lib.rs:173` |

## runtime_validation

| Topics | Payload | Source |
|---|---:|---|
| `VALID` · `INV_REG` | tuple (2) | `contracts/runtime_validation/src/events.rs:12` |
| `VALID` · `INV_VIOL` | tuple (2) | `contracts/runtime_validation/src/events.rs:47` |
| `VALID` · `PERM_REG` | single (1) | `contracts/runtime_validation/src/events.rs:26` |
| `VALID` · `PERM_VIOL` | tuple (2) | `contracts/runtime_validation/src/events.rs:61` |
| `VALID` · `RES_REG` | tuple (2) | `contracts/runtime_validation/src/events.rs:33` |
| `VALID` · `RES_UPD` | tuple (2) | `contracts/runtime_validation/src/events.rs:68` |
| `VALID` · `STATE_REG` | single (1) | `contracts/runtime_validation/src/events.rs:19` |
| `VALID` · `STATE_V` | tuple (2) | `contracts/runtime_validation/src/events.rs:54` |
| `VALID` · `VIOL` | tuple (2) | `contracts/runtime_validation/src/events.rs:40` |

## storage_cleanup

| Topics | Payload | Source |
|---|---:|---|
| `CLEANUP` · `ALL` | tuple (2) | `contracts/storage_cleanup/src/lib.rs:213` |

## storage_migration

| Topics | Payload | Source |
|---|---:|---|
| `MIGRATE` | tuple (5) | `contracts/storage_migration/src/lib.rs:127` |
| `MIGRATE` | tuple (5) | `contracts/storage_migration/src/lib.rs:190` |

## sut_token

| Topics | Payload | Source |
|---|---:|---|
| `burn` | single (1) | `contracts/sut_token/src/lib.rs:486` |
| `mint` | single (1) | `contracts/sut_token/src/lib.rs:421` |

## sync_manager

| Topics | Payload | Source |
|---|---:|---|
| `SM_INIT` | single (1) | `contracts/sync_manager/src/lib.rs:196` |
| `SM_LAG` | single (1) | `contracts/sync_manager/src/lib.rs:428` |
| `SM_SETP` | single (1) | `contracts/sync_manager/src/lib.rs:722` |
| `sync` · `enqueue` | tuple (3) | `contracts/sync_manager/src/lib.rs:967` |
| `sync` · `resolve` | tuple (2) | `contracts/sync_manager/src/lib.rs:998` |

## timelock

| Topics | Payload | Source |
|---|---:|---|
| `queued` | tuple (2) | `contracts/timelock/src/lib.rs:70` |

## token_sale

| Topics | Payload | Source |
|---|---:|---|
| `contribution` | tuple (4) | `contracts/token_sale/src/contract.rs:226` |
| `phase_added` | tuple (5) | `contracts/token_sale/src/contract.rs:89` |
| `sale_initialized` | tuple (4) | `contracts/token_sale/src/contract.rs:53` |
| `sale_paused` | tuple (0) | `contracts/token_sale/src/contract.rs:111` |
| `sale_unpaused` | tuple (0) | `contracts/token_sale/src/contract.rs:120` |
| `token_added` | tuple (2) | `contracts/token_sale/src/contract.rs:102` |
| `tokens_claimed` | tuple (2) | `contracts/token_sale/src/contract.rs:283` |
| `vesting_schedule_created` | tuple (4) | `contracts/token_sale/src/vesting.rs:67` |
| `vesting_schedule_updated` | tuple (5) | `contracts/token_sale/src/vesting.rs:227` |

## treasury_controller

| Topics | Payload | Source |
|---|---:|---|
| `APPROVED` | tuple (3) | `contracts/treasury_controller/src/lib.rs:386` |
| `EXECUTED` | tuple (3) | `contracts/treasury_controller/src/lib.rs:487` |
| `PROPOSAL` | tuple (3) | `contracts/treasury_controller/src/lib.rs:317` |
| `emergency` | single (1) | `contracts/treasury_controller/src/lib.rs:517` |
| `init` | single (1) | `contracts/treasury_controller/src/lib.rs:200` |
| `resumed` | single (1) | `contracts/treasury_controller/src/lib.rs:541` |

## upgradeability

| Topics | Payload | Source |
|---|---:|---|
| `DeprecationsUpdated` | single (1) | `contracts/upgradeability/src/lib.rs:307` |
| `lifecycle` · `transition` | tuple (2) | `contracts/upgradeability/src/lifecycle.rs:197` |
| `pausable` · `paused` | single (1) | `contracts/upgradeability/src/pausable.rs:57` |
| `pausable` · `unpaused` | single (1) | `contracts/upgradeability/src/pausable.rs:75` |

## zk_verifier

| Topics | Payload | Source |
|---|---:|---|
| `ZKVER` · `ATTEST` | tuple (2) | `contracts/zk_verifier/src/lib.rs:236` |
| `ZKVER` · `VKREG` | tuple (2) | `contracts/zk_verifier/src/lib.rs:145` |

## zkp_registry

| Topics | Payload | Source |
|---|---:|---|
| `admin` · `approved` | tuple (2) | `contracts/zkp_registry/src/lib.rs:616` |
| `admin` · `emer_exec` | single (1) | `contracts/zkp_registry/src/lib.rs:703` |
| `admin` · `executed` | single (1) | `contracts/zkp_registry/src/lib.rs:663` |
| `admin` · `proposed` | tuple (2) | `contracts/zkp_registry/src/lib.rs:571` |
| `zkp` · `circ_reg` | single (1) | `contracts/zkp_registry/src/lib.rs:758` |
| `zkp` · `cleanup` | tuple (2) | `contracts/zkp_registry/src/lib.rs:1487` |
| `zkp` · `cred_prf` | tuple (2) | `contracts/zkp_registry/src/lib.rs:1333` |
| `zkp` · `med_proof` | tuple (2) | `contracts/zkp_registry/src/lib.rs:1195` |
| `zkp` · `proof_sub` | tuple (3) | `contracts/zkp_registry/src/lib.rs:1015` |
| `zkp` · `proof_sub` | tuple (3) | `contracts/zkp_registry/src/lib.rs:1136` |
| `zkp` · `rec_proof` | tuple (3) | `contracts/zkp_registry/src/lib.rs:1439` |
| `zkp` · `rng_proof` | tuple (4) | `contracts/zkp_registry/src/lib.rs:1255` |
| `zkp` · `vk_roll` | tuple (2) | `contracts/zkp_registry/src/lib.rs:895` |
| `zkp` · `vk_rot` | tuple (2) | `contracts/zkp_registry/src/lib.rs:842` |

