# Uzima Contracts — Error Code Registry

## Convention

- All error enums must be named `Error` (not `IoTError`, `ZkError`, etc.)
- All must carry `#[contracterror]`, `#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]`, and `#[repr(u32)]`
- Each contract must expose `pub fn get_suggestion(error: Error) -> Symbol`
- Codes `x00–x09` within each range are reserved for canonical/shared errors
- Contract-specific codes start at `x10+`
- Gaps within a range are allowed — do not renumber when adding new variants

## Category Ranges

| Range   | Category                        |
|---------|---------------------------------|
| 100–199 | Access Control & Authorization  |
| 200–299 | Input Validation                |
| 300–399 | Lifecycle & State               |
| 400–499 | Entity Existence                |
| 500–599 | Financial & Resource            |
| 600–699 | Cryptography & ZK               |
| 700–799 | Cross-Chain & Integration       |
| 800–899 | Domain-Specific (AI/Medical/IoT)|
| 900–999 | Reserved                        |

## Canonical (Shared) Errors

| Code | Name                    | Suggestion    | Meaning                                    | Contracts Using It                                                                 |
|------|-------------------------|---------------|--------------------------------------------|------------------------------------------------------------------------------------|
| 100  | `Unauthorized`          | `CHK_AUTH`    | Caller lacks authorization                 | emergency_access_override, patient_consent_management, appointment_booking_escrow, medical_record_hash_registry, notification_system, clinical_nlp, iot_device_management, medical_records |
| 101  | `InsufficientPermissions` | `CHK_AUTH`  | Caller has insufficient permissions        | clinical_nlp                                                                       |
| 102  | `NotAdmin`              | `CHK_AUTH`    | Caller is not admin                        | iot_device_management                                                              |
| 104  | `HIPAAComplianceViolation` | `CHK_PHI`  | HIPAA compliance failure                   | clinical_nlp                                                                       |
| 200  | `InvalidInput`          | `CHK_LEN`     | Generic invalid input                      | medical_records                                                                    |
| 201  | `InputTooLong`          | `CHK_LEN`     | Input exceeds maximum length               | iot_device_management, clinical_nlp                                                |
| 202  | `InputTooShort`         | `CHK_LEN`     | Input below minimum length                 | iot_device_management                                                              |
| 205  | `InvalidAmount`         | `CHK_AMT`     | Invalid monetary amount                    | appointment_booking_escrow                                                         |
| 206  | `InvalidId`             | `CHK_ID`      | Invalid identifier                         | medical_record_hash_registry, iot_device_management                                |
| 207  | `InvalidSignature`      | `CONTACT`     | Signature verification failed              | medical_record_hash_registry                                                       |
| 208  | `BatchTooLarge`         | `REDUCE`      | Batch size exceeds limit                   | notification_system, clinical_nlp, medical_records                                 |
| 209  | `EmptyField`            | `ADD_TEXT`    | Required field is empty                    | notification_system (RecipientsEmpty)                                              |
| 300  | `NotInitialized`        | `INIT_CTR`    | Contract has not been initialized          | All contracts                                                                      |
| 301  | `AlreadyInitialized`    | `ALREADY`     | Contract already initialized               | All contracts                                                                      |
| 302  | `ContractPaused`        | `RE_TRY_L`    | Contract is paused                         | medical_record_hash_registry, clinical_nlp, iot_device_management, medical_records |
| 303  | `NotPaused`             | `CONTACT`     | Contract is not paused                     | iot_device_management                                                              |
| 304  | `InvalidState`          | `CONTACT`     | Operation invalid in current state         | appointment_booking_escrow, medical_records                                        |
| 306  | `DeadlineExceeded`      | `RE_TRY_L`    | Operation past deadline                    | medical_record_hash_registry                                                       |
| 307  | `RateLimitExceeded`     | `RE_TRY_L`    | Too many requests                          | notification_system, clinical_nlp                                                  |
| 308  | `Timeout`               | `RE_TRY_L`    | Processing timed out                       | clinical_nlp                                                                       |
| 402  | `DuplicateRecord`       | `CHK_ID`      | Record already exists                      | medical_record_hash_registry                                                       |
| 403  | `RecordNotFound`        | `CHK_ID`      | Record does not exist                      | emergency_access_override, medical_record_hash_registry, clinical_nlp, medical_records |
| 500  | `InsufficientFunds`     | `ADD_FUND`    | Insufficient funds for operation           | appointment_booking_escrow, medical_record_hash_registry                           |
| 501  | `TokenTransferFailed`   | `CONTACT`     | Token transfer failed                      | appointment_booking_escrow                                                         |
| 502  | `StorageFull`           | `CLN_OLD`     | Storage capacity reached                   | medical_record_hash_registry                                                       |
| 600  | `InvalidProof`          | `CONTACT`     | ZK proof is invalid                        | zk_verifier                                                                        |
| 601  | `VerificationFailed`    | `CONTACT`     | ZK verification failed                     | zk_verifier                                                                        |
| 602  | `InvalidEncryptionKey`  | `CONTACT`     | Encryption key is invalid                  | iot_device_management                                                              |
| 605  | `CredentialExpired`     | `CONTACT`     | Credential has expired                     | medical_records                                                                    |
| 606  | `CredentialRevoked`     | `CONTACT`     | Credential has been revoked                | medical_records                                                                    |
| 700  | `CrossChainAccessDenied`| `CHK_AUTH`    | Cross-chain access was denied              | medical_records                                                                    |
| 702  | `CrossChainTimeout`     | `RE_TRY_L`    | Cross-chain operation timed out            | medical_record_hash_registry                                                       |
| 703  | `InvalidChain`          | `CONTACT`     | Chain identifier is invalid                | medical_records                                                                    |
| 704  | `IntegrationFailed`     | `CONTACT`     | External integration failed                | clinical_nlp                                                                       |
| 705  | `ExternalContractNotSet`| `SET_CNTR`    | Required external contract not configured  | clinical_nlp                                                                       |

## Recovery Suggestion Symbols

| Symbol    | Meaning                                             |
|-----------|-----------------------------------------------------|
| `CHK_AUTH`| Check caller authorization                          |
| `INIT_CTR`| Initialize the contract first                       |
| `ALREADY` | Already initialized — no action needed              |
| `RE_TRY_L`| Retry later (contract paused / rate limited)        |
| `CHK_LEN` | Check input length                                  |
| `ADD_FUND`| Add sufficient funds                                |
| `CHK_ID`  | Verify the referenced ID exists                     |
| `CLN_OLD` | Clean up old entries to free storage                |
| `ADD_TEXT`| Input must not be empty                             |
| `FIX_LANG`| Use a supported language code                       |
| `SET_CNTR`| Admin must configure external contract address      |
| `CHK_PHI` | PHI/HIPAA compliance failure — check data handling  |
| `CONTACT` | Contact support / admin                             |

## Per-Contract Error Codes

### appointment_booking_escrow

| Code | Name                        | Suggestion  | Description                          |
|------|-----------------------------|-------------|--------------------------------------|
| 100  | `Unauthorized`              | `CHK_AUTH`  | Caller not authorized                |
| 110  | `OnlyPatientCanRefund`      | `CHK_AUTH`  | Only patient can initiate refund     |
| 111  | `OnlyProviderCanConfirm`    | `CHK_AUTH`  | Only provider can confirm            |
| 205  | `InvalidAmount`             | `CHK_LEN`   | Payment amount invalid               |
| 210  | `InvalidPatient`            | `CHK_ID`    | Patient address invalid              |
| 211  | `InvalidProvider`           | `CHK_ID`    | Provider address invalid             |
| 300  | `NotInitialized`            | `INIT_CTR`  | Contract not initialized             |
| 301  | `AlreadyInitialized`        | `ALREADY`   | Contract already initialized         |
| 304  | `InvalidState`              | `CONTACT`   | Appointment in wrong state           |
| 410  | `AppointmentNotFound`       | `CHK_ID`    | Appointment does not exist           |
| 411  | `AppointmentAlreadyConfirmed`| `ALREADY`  | Appointment already confirmed        |
| 412  | `AppointmentAlreadyRefunded` | `ALREADY`  | Appointment already refunded         |
| 500  | `InsufficientFunds`         | `ADD_FUND`  | Insufficient funds                   |
| 501  | `TokenTransferFailed`       | `CONTACT`   | Token transfer failed                |
| 505  | `DoubleWithdrawal`          | `CONTACT`   | Withdrawal already processed         |

### clinical_nlp

| Code | Name                        | Suggestion  | Description                          |
|------|-----------------------------|-------------|--------------------------------------|
| 100  | `Unauthorized`              | `CHK_AUTH`  | Caller not authorized                |
| 101  | `InsufficientPermissions`   | `CHK_AUTH`  | Caller has insufficient permissions  |
| 104  | `HIPAAComplianceViolation`  | `CHK_PHI`   | HIPAA violation detected             |
| 112  | `RecordAccessDenied`        | `CHK_AUTH`  | Access to record denied              |
| 201  | `InputTooLong`              | `CHK_LEN`   | Input exceeds maximum length         |
| 208  | `BatchTooLarge`             | `REDUCE`    | Batch size too large                 |
| 209  | `EmptyClinicalNote`         | `ADD_TEXT`  | Clinical note must not be empty      |
| 300  | `NotInitialized`            | `INIT_CTR`  | Contract not initialized             |
| 301  | `AlreadyInitialized`        | `ALREADY`   | Contract already initialized         |
| 302  | `ContractPaused`            | `RE_TRY_L`  | Contract is paused                   |
| 307  | `RateLimitExceeded`         | `RE_TRY_L`  | Rate limit exceeded                  |
| 308  | `Timeout`                   | `RE_TRY_L`  | Processing timed out                 |
| 310  | `InvalidConfiguration`      | `CONTACT`   | Configuration invalid                |
| 403  | `RecordNotFound`            | `CHK_ID`    | Record does not exist                |
| 704  | `IntegrationFailed`         | `CONTACT`   | External integration failed          |
| 705  | `ExternalContractNotSet`    | `SET_CNTR`  | Medical records contract not set     |
| 800  | `NLPEngineNotInitialized`   | `INIT_CTR`  | NLP engine not initialized           |
| 801  | `EntityExtractionFailed`    | `CONTACT`   | Entity extraction failed             |
| 810  | `ICD10CodeNotFound`         | `CONTACT`   | ICD-10 code not found                |
| 811  | `CPTCodeNotFound`           | `CONTACT`   | CPT code not found                   |

### emergency_access_override

| Code | Name                | Suggestion  | Description                    |
|------|---------------------|-------------|--------------------------------|
| 100  | `Unauthorized`      | `CHK_AUTH`  | Caller not authorized          |
| 230  | `InvalidThreshold`  | `CHK_LEN`   | Threshold value invalid        |
| 231  | `InvalidDuration`   | `CHK_LEN`   | Duration value invalid         |
| 300  | `NotInitialized`    | `INIT_CTR`  | Contract not initialized       |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized   |
| 403  | `RecordNotFound`    | `CHK_ID`    | Record does not exist          |

### iot_device_management

| Code | Name                        | Suggestion  | Description                          |
|------|-----------------------------|-------------|--------------------------------------|
| 100  | `Unauthorized`              | `CHK_AUTH`  | Caller not authorized                |
| 102  | `NotAdmin`                  | `CHK_AUTH`  | Caller is not admin                  |
| 115  | `NotDeviceOperator`         | `CHK_AUTH`  | Caller is not a device operator      |
| 116  | `NotManufacturer`           | `CHK_AUTH`  | Caller is not a manufacturer         |
| 201  | `InputTooLong`              | `CHK_LEN`   | String input too long                |
| 202  | `InputTooShort`             | `CHK_LEN`   | String input too short               |
| 240  | `InvalidDeviceType`         | `CONTACT`   | Device type unknown                  |
| 250  | `InvalidFirmwareHash`       | `CONTACT`   | Firmware hash invalid                |
| 260  | `InvalidMetricValue`        | `CONTACT`   | Metric value out of range            |
| 270  | `InvalidTimestamp`          | `CONTACT`   | Timestamp is invalid                 |
| 300  | `NotInitialized`            | `INIT_CTR`  | Contract not initialized             |
| 301  | `AlreadyInitialized`        | `ALREADY`   | Contract already initialized         |
| 302  | `ContractPaused`            | `RE_TRY_L`  | Contract is paused                   |
| 303  | `NotPaused`                 | `CONTACT`   | Contract is not paused               |
| 405  | `DeviceNotFound`            | `CHK_ID`    | Device does not exist                |
| 420  | `DeviceAlreadyRegistered`   | `ALREADY`   | Device already registered            |
| 425  | `ManufacturerNotRegistered` | `CHK_ID`    | Manufacturer not found               |
| 426  | `ManufacturerAlreadyRegistered`| `ALREADY`| Manufacturer already registered      |
| 430  | `FirmwareVersionNotFound`   | `CHK_ID`    | Firmware version not found           |
| 431  | `FirmwareAlreadyExists`     | `ALREADY`   | Firmware already exists              |
| 440  | `ChannelNotFound`           | `CHK_ID`    | Communication channel not found      |
| 602  | `InvalidEncryptionKey`      | `CONTACT`   | Encryption key is invalid            |
| 603  | `KeyRotationTooFrequent`    | `RE_TRY_L`  | Key rotation too frequent            |
| 820  | `DeviceDecommissioned`      | `CONTACT`   | Device has been decommissioned       |
| 821  | `FirmwareNotApproved`       | `CONTACT`   | Firmware not approved                |
| 822  | `HeartbeatTooFrequent`      | `RE_TRY_L`  | Heartbeat sent too frequently        |
| 823  | `DeviceNotActive`           | `CONTACT`   | Device is not active                 |
| 824  | `DeviceSuspended`           | `CONTACT`   | Device is suspended                  |
| 825  | `DowngradeNotAllowed`       | `CONTACT`   | Firmware downgrade not allowed       |
| 826  | `DeviceOffline`             | `CONTACT`   | Device is offline                    |

### medical_record_hash_registry

| Code | Name                | Suggestion  | Description                    |
|------|---------------------|-------------|--------------------------------|
| 100  | `Unauthorized`      | `CHK_AUTH`  | Caller not authorized          |
| 206  | `InvalidId`         | `CHK_ID`    | Patient ID invalid             |
| 207  | `InvalidSignature`  | `CONTACT`   | Signature verification failed  |
| 251  | `InvalidRecordHash` | `CONTACT`   | Record hash invalid            |
| 300  | `NotInitialized`    | `INIT_CTR`  | Contract not initialized       |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized   |
| 302  | `ContractPaused`    | `RE_TRY_L`  | Contract is paused             |
| 306  | `DeadlineExceeded`  | `RE_TRY_L`  | Operation past deadline        |
| 402  | `DuplicateRecord`   | `CHK_ID`    | Record already exists          |
| 403  | `RecordNotFound`    | `CHK_ID`    | Record does not exist          |
| 500  | `InsufficientFunds` | `ADD_FUND`  | Insufficient funds             |
| 502  | `StorageFull`       | `CLN_OLD`   | Storage capacity reached       |
| 702  | `CrossChainTimeout` | `RE_TRY_L`  | Cross-chain timeout            |

### notification_system

| Code | Name                    | Suggestion  | Description                        |
|------|-------------------------|-------------|------------------------------------|
| 100  | `Unauthorized`          | `CHK_AUTH`  | Caller not authorized              |
| 120  | `SenderNotAuthorized`   | `CHK_AUTH`  | Sender not authorized              |
| 208  | `BatchTooLarge`         | `REDUCE`    | Batch too large                    |
| 209  | `RecipientsEmpty`       | `ADD_TEXT`  | Recipients list is empty           |
| 221  | `TitleTooLong`          | `SHORTEN`   | Title exceeds max length           |
| 222  | `MessageTooLong`        | `SHORTEN`   | Message exceeds max length         |
| 223  | `NameTooLong`           | `SHORTEN`   | Name exceeds max length            |
| 224  | `LocaleTooLong`         | `FIX_LANG`  | Locale string too long             |
| 241  | `InvalidNotifType`      | `CONTACT`   | Notification type invalid          |
| 242  | `TooManyEnabledTypes`   | `REDUCE`    | Too many enabled notification types|
| 300  | `NotInitialized`        | `INIT_CTR`  | Contract not initialized           |
| 301  | `AlreadyInitialized`    | `ALREADY`   | Contract already initialized       |
| 307  | `RateLimitExceeded`     | `RE_TRY_L`  | Rate limit exceeded                |
| 330  | `AlreadyRead`           | `ALREADY`   | Notification already read          |
| 331  | `AlreadyArchived`       | `ALREADY`   | Notification already archived      |
| 450  | `NotificationNotFound`  | `CHK_ID`    | Notification not found             |
| 451  | `AlertRuleNotFound`     | `CHK_ID`    | Alert rule not found               |
| 452  | `TemplateNotFound`      | `CHK_ID`    | Template not found                 |
| 453  | `SenderNotFound`        | `CHK_ID`    | Sender not found                   |
| 510  | `MaxSendersReached`     | `CLN_OLD`   | Maximum senders reached            |
| 511  | `MaxRulesReached`       | `CLN_OLD`   | Maximum rules reached              |
| 512  | `MaxNotificationsReached`| `CLN_OLD`  | Maximum notifications reached      |
| 513  | `MaxTemplatesReached`   | `CLN_OLD`   | Maximum templates reached          |

### patient_consent_management

| Code | Name                  | Suggestion  | Description                      |
|------|-----------------------|-------------|----------------------------------|
| 100  | `Unauthorized`        | `CHK_AUTH`  | Caller not authorized            |
| 210  | `InvalidPatient`      | `CHK_ID`    | Patient address invalid          |
| 211  | `InvalidProvider`     | `CHK_ID`    | Provider address invalid         |
| 300  | `NotInitialized`      | `INIT_CTR`  | Contract not initialized         |
| 301  | `AlreadyInitialized`  | `ALREADY`   | Contract already initialized     |
| 406  | `ConsentNotFound`     | `CHK_ID`    | Consent record not found         |
| 460  | `ConsentAlreadyExists`| `ALREADY`   | Consent already exists           |

### zk_verifier

| Code | Name                | Suggestion  | Description                 |
|------|---------------------|-------------|-----------------------------|
| 100  | `Unauthorized`      | `CHK_AUTH`  | Caller not authorized       |
| 200  | `InvalidInput`      | `CHK_LEN`   | Generic invalid input       |
| 300  | `NotInitialized`    | `INIT_CTR`  | Contract not initialized    |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized|
| 430  | `VersionNotFound`   | `CHK_ID`    | Version does not exist      |
| 600  | `InvalidProof`      | `CONTACT`   | ZK proof is invalid         |
| 601  | `VerificationFailed`| `CONTACT`   | ZK verification failed      |

### medical_records

| Code | Name                          | Suggestion  | Description                                  |
|------|-------------------------------|-------------|----------------------------------------------|
| 100  | `Unauthorized`                | `CHK_AUTH`  | Caller not authorized                        |
| 150  | `NotAICoordinator`            | `CHK_AUTH`  | Caller is not the AI coordinator             |
| 160  | `EmergencyAccessExpired`      | `NEW_EMER`  | Emergency access has expired                 |
| 200  | `InvalidInput`                | `CONTACT`   | Generic invalid input                        |
| 201  | `InputTooLong`                | `CHK_LEN`   | Input exceeds maximum length                 |
| 207  | `InvalidSignature`            | `CONTACT`   | Signature invalid                            |
| 208  | `BatchTooLarge`               | `REDUCE`    | Batch size too large                         |
| 250  | `InvalidDataRefLength`        | `CHK_LEN`   | Data reference length invalid                |
| 251  | `InvalidDataRefCharset`       | `CHK_LEN`   | Data reference charset invalid               |
| 252  | `InvalidDiagnosisLength`      | `CHK_LEN`   | Diagnosis field too long                     |
| 253  | `InvalidTreatmentLength`      | `CHK_LEN`   | Treatment field too long                     |
| 254  | `InvalidPurposeLength`        | `CHK_LEN`   | Purpose field too long                       |
| 255  | `InvalidTagLength`            | `CHK_LEN`   | Tag field too long                           |
| 256  | `InvalidModelVersionLength`   | `CHK_LEN`   | Model version field too long                 |
| 257  | `InvalidExplanationLength`    | `CHK_LEN`   | Explanation field too long                   |
| 258  | `InvalidTreatmentTypeLength`  | `CHK_LEN`   | Treatment type field too long                |
| 280  | `InvalidCategory`             | `FIX_CAT`   | Category is invalid                          |
| 281  | `EmptyTreatment`              | `FILL_FLD`  | Treatment field must not be empty            |
| 282  | `EmptyDiagnosis`              | `FILL_FLD`  | Diagnosis field must not be empty            |
| 283  | `EmptyTag`                    | `CONTACT`   | Tag must not be empty                        |
| 284  | `EmptyDataRef`                | `CONTACT`   | Data reference must not be empty             |
| 290  | `InvalidAddress`              | `CONTACT`   | Address is invalid                           |
| 291  | `SameAddress`                 | `CONTACT`   | Source and destination are the same          |
| 292  | `InvalidBatch`                | `CHK_DATA`  | Batch data is invalid                        |
| 293  | `NumberOutOfBounds`           | `CONTACT`   | Number is out of bounds                      |
| 300  | `NotInitialized`              | `INIT_CTR`  | Contract not initialized                     |
| 302  | `ContractPaused`              | `RE_TRY_L`  | Contract is paused                           |
| 306  | `DeadlineExceeded`            | `CONTACT`   | Operation past deadline                      |
| 307  | `RateLimitExceeded`           | `RE_TRY_L`  | Rate limit exceeded                          |
| 320  | `ProposalAlreadyExecuted`     | `ALREADY`   | Proposal already executed                    |
| 321  | `TimelockNotElapsed`          | `RE_TRY_L`  | Timelock period has not elapsed              |
| 322  | `NotEnoughApproval`           | `CONTACT`   | Insufficient approvals for execution         |
| 340  | `CryptoRegistryNotSet`        | `SET_CNTR`  | Crypto registry contract not configured      |
| 341  | `EncryptionRequired`          | `CONTACT`   | Record requires encryption                   |
| 342  | `IdentityRegistryNotSet`      | `SET_CNTR`  | Identity registry contract not configured    |
| 403  | `RecordNotFound`              | `CHK_ID`    | Record does not exist                        |
| 460  | `EmergencyAccessNotFound`     | `CHK_ID`    | Emergency access not found                   |
| 470  | `DIDNotFound`                 | `CHK_ID`    | DID does not exist                           |
| 471  | `DIDNotActive`                | `CONTACT`   | DID is not active                            |
| 480  | `RecordAlreadySynced`         | `ALREADY`   | Record already synced cross-chain            |
| 500  | `InsufficientFunds`           | `ADD_FUND`  | Insufficient funds                           |
| 502  | `StorageFull`                 | `CLN_OLD`   | Storage capacity reached                     |
| 605  | `CredentialExpired`           | `CONTACT`   | Credential has expired                       |
| 606  | `CredentialRevoked`           | `CONTACT`   | Credential has been revoked                  |
| 640  | `InvalidCredential`           | `CONTACT`   | Credential is invalid                        |
| 641  | `MissingRequiredCredential`   | `CONTACT`   | Required credential missing                  |
| 700  | `CrossChainAccessDenied`      | `CHK_AUTH`  | Cross-chain access denied                    |
| 702  | `CrossChainTimeout`           | `RE_TRY_L`  | Cross-chain operation timed out              |
| 703  | `InvalidChain`                | `CONTACT`   | Chain identifier is invalid                  |
| 710  | `CrossChainNotEnabled`        | `CONTACT`   | Cross-chain is not enabled                   |
| 711  | `CrossChainContractsNotSet`   | `SET_CNTR`  | Cross-chain contracts not configured         |
| 830  | `AIConfigNotSet`              | `SET_CNTR`  | AI configuration not set                     |
| 831  | `InvalidAIScore`              | `CONTACT`   | AI score is invalid                          |
| 832  | `InvalidScore`                | `CONTACT`   | Score is invalid                             |
| 833  | `InvalidDPEpsilon`            | `CONTACT`   | Differential privacy epsilon invalid         |
| 834  | `InvalidParticipantCount`     | `CONTACT`   | Participant count invalid                    |

### audit

| Code | Name                | Suggestion  | Description                  |
|------|---------------------|-------------|------------------------------|
| 100  | `Unauthorized`      | `CHK_AUTH`  | Caller not authorized        |
| 300  | `NotInitialized`    | `INIT_CTR`  | Contract not initialized     |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized |
| 403  | `RecordNotFound`    | `CHK_ID`    | Audit record does not exist  |

### rbac

| Code | Name                | Suggestion  | Description                  |
|------|---------------------|-------------|------------------------------|
| 100  | `Unauthorized`      | `CHK_AUTH`  | Caller not authorized        |
| 300  | `NotInitialized`    | `INIT_CTR`  | Contract not initialized     |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized |

### identity_registry

| Code | Name                          | Suggestion  | Description                               |
|------|-------------------------------|-------------|-------------------------------------------|
| 100  | `Unauthorized`                | `CHK_AUTH`  | Caller not authorized                     |
| 110  | `NotVerifier`                 | `CHK_AUTH`  | Caller is not a verifier                  |
| 111  | `CannotRemoveOwner`           | `CHK_AUTH`  | Cannot remove contract owner              |
| 120  | `InvalidRecoveryGuardian`     | `CHK_AUTH`  | Recovery guardian is invalid              |
| 121  | `InsufficientGuardianApprovals`| `CHK_AUTH` | Insufficient guardian approvals           |
| 250  | `InvalidVerificationMethod`   | `CONTACT`   | Verification method is invalid            |
| 251  | `InvalidCredentialType`       | `CONTACT`   | Credential type is invalid                |
| 252  | `InvalidServiceEndpoint`      | `CONTACT`   | Service endpoint is invalid               |
| 300  | `NotInitialized`              | `INIT_CTR`  | Contract not initialized                  |
| 301  | `AlreadyInitialized`          | `ALREADY`   | Contract already initialized              |
| 360  | `RecoveryNotInitiated`        | `CONTACT`   | Recovery has not been initiated           |
| 361  | `RecoveryAlreadyPending`      | `ALREADY`   | Recovery is already pending               |
| 362  | `RecoveryTimelockNotElapsed`  | `RE_TRY_L`  | Recovery timelock not elapsed             |
| 450  | `VerificationMethodNotFound`  | `CHK_ID`    | Verification method not found             |
| 460  | `CredentialNotFound`          | `CHK_ID`    | Credential not found                      |
| 461  | `AttestationNotFound`         | `CHK_ID`    | Attestation not found                     |
| 462  | `ServiceNotFound`             | `CHK_ID`    | Service endpoint not found                |
| 470  | `DIDNotFound`                 | `CHK_ID`    | DID does not exist                        |
| 471  | `DIDAlreadyExists`            | `ALREADY`   | DID already exists                        |
| 472  | `DIDDeactivated`              | `CONTACT`   | DID has been deactivated                  |
| 603  | `KeyRotationCooldown`         | `RE_TRY_L`  | Key rotation cooldown in effect           |
| 605  | `CredentialExpired`           | `CONTACT`   | Credential has expired                    |
| 606  | `CredentialRevoked`           | `CONTACT`   | Credential has been revoked               |

### governor

| Code | Name                    | Suggestion  | Description                          |
|------|-------------------------|-------------|--------------------------------------|
| 280  | `InvalidVoteType`       | `CONTACT`   | Vote type is invalid                 |
| 300  | `NotInitialized`        | `INIT_CTR`  | Contract not initialized             |
| 301  | `AlreadyInitialized`    | `ALREADY`   | Contract already initialized         |
| 304  | `InvalidState`          | `CONTACT`   | Proposal in wrong state              |
| 370  | `VotingClosed`          | `RE_TRY_L`  | Voting period is closed              |
| 371  | `AlreadyVoted`          | `ALREADY`   | Caller has already voted             |
| 372  | `NotQueued`             | `RE_TRY_L`  | Proposal is not queued               |
| 373  | `ProposalDisputed`      | `CONTACT`   | Proposal is disputed                 |
| 450  | `ProposalNotFound`      | `CHK_ID`    | Proposal does not exist              |
| 451  | `ProposalNotSuccessful` | `CHK_ID`    | Proposal did not succeed             |
| 452  | `AlreadyExecuted`       | `ALREADY`   | Proposal already executed            |
| 530  | `ProposalThresholdNotMet`| `CONTACT`  | Proposal threshold not reached       |
| 531  | `NoVotingPower`         | `CONTACT`   | Caller has no voting power           |
| 580  | `Overflow`              | `CONTACT`   | Arithmetic overflow                  |

### timelock

| Code | Name                | Suggestion  | Description                       |
|------|---------------------|-------------|-----------------------------------|
| 100  | `Unauthorized`      | `CHK_AUTH`  | Caller not authorized             |
| 207  | `InvalidSignature`  | `CONTACT`   | Signature verification failed     |
| 300  | `NotInitialized`    | `INIT_CTR`  | Contract not initialized          |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized      |
| 302  | `ContractPaused`    | `RE_TRY_L`  | Contract is paused                |
| 306  | `DeadlineExceeded`  | `RE_TRY_L`  | Operation past deadline           |
| 372  | `NotQueued`         | `CONTACT`   | Operation is not queued           |
| 375  | `AlreadyQueued`     | `ALREADY`   | Operation already queued          |
| 376  | `NotReady`          | `RE_TRY_L`  | Timelock not yet elapsed          |
| 500  | `InsufficientFunds` | `ADD_FUND`  | Insufficient funds                |
| 502  | `StorageFull`       | `CLN_OLD`   | Storage capacity reached          |
| 702  | `CrossChainTimeout` | `RE_TRY_L`  | Cross-chain operation timed out   |

### escrow

| Code | Name                    | Suggestion  | Description                        |
|------|-------------------------|-------------|------------------------------------|
| 100  | `Unauthorized`          | `CHK_AUTH`  | Caller not authorized              |
| 102  | `NotAdmin`              | `CHK_AUTH`  | Caller is not admin                |
| 120  | `InsufficientApprovals` | `CHK_AUTH`  | Insufficient approvals             |
| 205  | `InvalidAmount`         | `CHK_LEN`   | Amount is invalid                  |
| 260  | `InvalidFeeBps`         | `CHK_LEN`   | Fee basis points invalid           |
| 380  | `FeeNotSet`             | `CONTACT`   | Fee has not been configured        |
| 381  | `ReentrancyGuard`       | `CONTACT`   | Reentrancy detected                |
| 382  | `InvalidStateTransition`| `CONTACT`   | Invalid state transition           |
| 480  | `EscrowExists`          | `ALREADY`   | Escrow already exists              |
| 481  | `EscrowNotFound`        | `CHK_ID`    | Escrow does not exist              |
| 482  | `AlreadySettled`        | `ALREADY`   | Escrow already settled             |
| 560  | `NoBasisToRefund`       | `CONTACT`   | No basis to issue refund           |
| 561  | `NoCredit`              | `CONTACT`   | No credit available                |

### healthcare_payment

| Code | Name                        | Suggestion  | Description                            |
|------|-----------------------------|-------------|----------------------------------------|
| 100  | `Unauthorized`              | `CHK_AUTH`  | Caller not authorized                  |
| 205  | `InvalidAmount`             | `CHK_LEN`   | Amount is invalid                      |
| 207  | `InvalidSignature`          | `CONTACT`   | Signature verification failed          |
| 280  | `InvalidCoverage`           | `CONTACT`   | Coverage is invalid                    |
| 281  | `PolicyMismatch`            | `CONTACT`   | Policy does not match claim            |
| 300  | `NotInitialized`            | `INIT_CTR`  | Contract not initialized               |
| 301  | `AlreadyInitialized`        | `ALREADY`   | Contract already initialized           |
| 302  | `ContractPaused`            | `RE_TRY_L`  | Contract is paused                     |
| 304  | `InvalidStatus`             | `CONTACT`   | Claim status is invalid                |
| 306  | `DeadlineExceeded`          | `RE_TRY_L`  | Operation past deadline                |
| 480  | `ClaimNotFound`             | `CHK_ID`    | Claim does not exist                   |
| 481  | `PreAuthNotFound`           | `CHK_ID`    | Pre-authorization not found            |
| 482  | `PaymentPlanNotFound`       | `CHK_ID`    | Payment plan not found                 |
| 483  | `InsuranceProviderNotFound` | `CHK_ID`    | Insurance provider not found           |
| 484  | `CoveragePolicyNotFound`    | `CHK_ID`    | Coverage policy not found              |
| 485  | `EligibilityCheckNotFound`  | `CHK_ID`    | Eligibility check not found            |
| 486  | `ClaimSubmissionNotFound`   | `CHK_ID`    | Claim submission not found             |
| 487  | `EobNotFound`               | `CHK_ID`    | Explanation of benefits not found      |
| 500  | `InsufficientFunds`         | `ADD_FUND`  | Insufficient funds                     |
| 502  | `StorageFull`               | `CLN_OLD`   | Storage capacity reached               |
| 580  | `FraudDetected`             | `CONTACT`   | Fraud detected in claim                |
| 581  | `EscrowFailed`              | `CONTACT`   | Escrow operation failed                |
| 582  | `UnsupportedTransaction`    | `CONTACT`   | Transaction type not supported         |
| 702  | `CrossChainTimeout`         | `RE_TRY_L`  | Cross-chain operation timed out        |

### upgrade_manager

| Code | Name                | Suggestion  | Description                        |
|------|---------------------|-------------|------------------------------------|
| 110  | `NotAValidator`     | `CHK_AUTH`  | Caller is not a validator          |
| 120  | `NotEnoughApprovals`| `CHK_AUTH`  | Insufficient validator approvals   |
| 301  | `AlreadyInitialized`| `ALREADY`   | Contract already initialized       |
| 304  | `InvalidState`      | `CONTACT`   | Proposal in wrong state            |
| 376  | `TimelockNotExpired`| `RE_TRY_L`  | Timelock has not expired           |
| 390  | `ConfigNotFound`    | `CHK_ID`    | Configuration not found            |
| 450  | `ProposalNotFound`  | `CHK_ID`    | Proposal does not exist            |
| 451  | `AlreadyApproved`   | `ALREADY`   | Already approved by this validator |

### cross_chain_bridge

| Code | Name                      | Suggestion  | Description                              |
|------|---------------------------|-------------|------------------------------------------|
| 100  | `Unauthorized`            | `CHK_AUTH`  | Caller not authorized                    |
| 120  | `InsufficientConfirmations`| `CHK_AUTH` | Not enough confirmations                 |
| 121  | `InsufficientOracleReports`| `CHK_AUTH` | Not enough oracle reports                |
| 122  | `DuplicateOracleReport`   | `ALREADY`   | Oracle already reported                  |
| 207  | `InvalidSignature`        | `CONTACT`   | Signature verification failed            |
| 280  | `InvalidMessage`          | `CONTACT`   | Message is invalid                       |
| 281  | `InvalidNonce`            | `CONTACT`   | Nonce is invalid                         |
| 282  | `InvalidPayload`          | `CONTACT`   | Payload is invalid                       |
| 290  | `InvalidAddress`          | `CONTACT`   | Address is invalid                       |
| 301  | `AlreadyInitialized`      | `ALREADY`   | Contract already initialized             |
| 302  | `ContractPaused`          | `RE_TRY_L`  | Contract is paused                       |
| 480  | `MessageNotFound`         | `CHK_ID`    | Message does not exist                   |
| 481  | `MessageExpired`          | `CONTACT`   | Message has expired                      |
| 482  | `MessageAlreadyProcessed` | `ALREADY`   | Message already processed                |
| 483  | `ValidatorNotFound`       | `CHK_ID`    | Validator not found                      |
| 484  | `ValidatorNotActive`      | `CONTACT`   | Validator is not active                  |
| 485  | `DuplicateConfirmation`   | `ALREADY`   | Confirmation already submitted           |
| 486  | `AtomicTxNotFound`        | `CHK_ID`    | Atomic transaction not found             |
| 487  | `AtomicTxExpired`         | `CONTACT`   | Atomic transaction has expired           |
| 488  | `AtomicTxAlreadyProcessed`| `ALREADY`   | Atomic transaction already processed     |
| 489  | `RecordRefNotFound`       | `CHK_ID`    | Record reference not found               |
| 490  | `RollbackNotFound`        | `CHK_ID`    | Rollback operation not found             |
| 491  | `RollbackAlreadyProcessed`| `ALREADY`   | Rollback already processed               |
| 492  | `EventNotFound`           | `CHK_ID`    | Event not found                          |
| 580  | `Overflow`                | `CONTACT`   | Arithmetic overflow                      |
| 610  | `ProofNotFound`           | `CHK_ID`    | Proof not found                          |
| 611  | `ProofAlreadyVerified`    | `ALREADY`   | Proof already verified                   |
| 703  | `InvalidChain`            | `CONTACT`   | Chain identifier is invalid              |
| 720  | `ChainNotSupported`       | `CONTACT`   | Chain is not supported                   |
| 721  | `OracleNotFound`          | `CHK_ID`    | Oracle not found                         |
| 722  | `OracleNotActive`         | `CONTACT`   | Oracle is not active                     |

## Adding a New Error (step-by-step)

1. Pick the right range from the Category Ranges table above
2. Pick the next free code within the range (gaps are allowed)
3. If the semantics match an existing canonical error, use that code and name
4. Add a `get_suggestion()` arm returning the most helpful symbol
5. Update this document in the Per-Contract section
6. Add a test asserting `Error::YourVariant as u32 == <expected_code>`
