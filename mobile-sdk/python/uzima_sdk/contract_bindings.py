"""
Auto-generated contract bindings for the Uzima Python SDK.

DO NOT EDIT MANUALLY — run `node scripts/generate-sdk-types.mjs` instead.
Generator version: 1.0.0
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, List, Optional, Protocol
from typing import Literal

# ==================== Contract Enums ====================

class RecordType(str, Enum):
    """Enumeration of medical record types"""
    DIAGNOSIS = "diagnosis"  # Diagnosis record
    PRESCRIPTION = "prescription"  # Prescription record
    LAB_RESULT = "lab_result"  # Laboratory test result
    IMAGING = "imaging"  # Medical imaging record
    CONSULTATION = "consultation"  # Consultation notes
    VITAL_SIGNS = "vital_signs"  # Vital signs measurement
    IMMUNIZATION = "immunization"  # Immunization record
    MEDICATION_HISTORY = "medication_history"  # Medication history
    ALLERGY = "allergy"  # Allergy information
    PROCEDURE = "procedure"  # Medical procedure record

class EncryptionAlgorithm(str, Enum):
    """Encryption algorithm used for medical data"""
    NACL_BOX = "nacl-box"  # NaCl box (public-key encryption)
    NACL_SECRETBOX = "nacl-secretbox"  # NaCl secretbox (secret-key encryption)
    AES_256_GCM = "aes-256-gcm"  # AES-256 GCM

class ConsentStatus(str, Enum):
    """Consent grant status"""
    ACTIVE = "active"  # Consent is active and valid
    REVOKED = "revoked"  # Consent has been revoked
    PENDING = "pending"  # Consent is pending approval
    EXPIRED = "expired"  # Consent has expired

class VerificationMethodType(str, Enum):
    """Verification method types per W3C DID specification"""
    ED25519_VERIFICATION_KEY_2020 = "Ed25519VerificationKey2020"  # Ed25519 Verification Key (2020)
    ECDSA_SECP256K1_VERIF_KEY_2019 = "EcdsaSecp256k1VerifKey2019"  # ECDSA Secp256k1 Verification Key (2019)
    X25519_KEY_AGREEMENT_KEY_2020 = "X25519KeyAgreementKey2020"  # X25519 Key Agreement Key (2020)
    JSON_WEB_KEY_2020 = "JsonWebKey2020"  # JSON Web Key (2020)
    FIDO2_ED_DSA_2024 = "Fido2EdDsa2024"  # FIDO2 EdDSA Key (2024)
    FIDO2_ES256_2024 = "Fido2Es2562024"  # FIDO2 ES256 Key (2024)

class VerificationRelationship(str, Enum):
    """Verification relationship types per W3C DID specification"""
    AUTHENTICATION = "Authentication"  # Authentication relationship
    ASSERTION_METHOD = "AssertionMethod"  # Assertion method relationship
    KEY_AGREEMENT = "KeyAgreement"  # Key agreement relationship
    CAPABILITY_INVOCATION = "CapabilityInvocation"  # Capability invocation relationship
    CAPABILITY_DELEGATION = "CapabilityDelegation"  # Capability delegation relationship

class PaymentStatusEnum(str, Enum):
    """Payment/claim status enumeration"""
    SUBMITTED = "submitted"  # Payment claim submitted
    VERIFIED = "verified"  # Payment claim verified
    APPROVED = "approved"  # Payment claim approved
    REJECTED = "rejected"  # Payment claim rejected
    PAID = "paid"  # Payment completed
    DISPUTED = "disputed"  # Payment disputed

class ActionType(str, Enum):
    """Audit action types"""
    DATA_READ = "DataRead"  # Data read operation
    DATA_WRITE = "DataWrite"  # Data write operation
    DATA_DELETE = "DataDelete"  # Data delete operation
    DATA_EXPORT = "DataExport"  # Data export operation
    PERMISSION_GRANT = "PermissionGrant"  # Permission grant
    PERMISSION_REVOKE = "PermissionRevoke"  # Permission revoke
    ROLE_ASSIGN = "RoleAssign"  # Role assignment
    ROLE_REVOKE = "RoleRevoke"  # Role revocation
    RECORD_CREATE = "RecordCreate"  # Record creation
    RECORD_UPDATE = "RecordUpdate"  # Record update
    RECORD_ARCHIVE = "RecordArchive"  # Record archive
    RECORD_RESTORE = "RecordRestore"  # Record restore
    AUTH_SUCCESS = "AuthSuccess"  # Authentication success
    AUTH_FAILURE = "AuthFailure"  # Authentication failure
    AUTH_LOGOUT = "AuthLogout"  # User logout
    TOKEN_REFRESH = "TokenRefresh"  # Token refresh
    CROSS_CHAIN_TRANSFER_INIT = "CrossChainTransferInit"  # Cross-chain transfer initiated
    CROSS_CHAIN_TRANSFER_COMPLETED = "CrossChainTransferCompleted"  # Cross-chain transfer completed

# ==================== Contract Dataclasses ====================

@dataclass
class EncryptedData:
    """Encrypted data container"""
    ciphertext: str
    nonce: str
    algorithm: EncryptionAlgorithm

@dataclass
class AccessLog:
    """Access log entry for audit trail"""
    accessor: str
    access_time: int
    access_type: Literal['read', 'write', 'share']
    ip_address: Optional[str] = None

@dataclass
class RecordMetadata:
    """Metadata for medical records"""
    created_at: int
    updated_at: int
    access_log: List[AccessLog]
    tags: Optional[List[str]] = None
    is_traditional_healing: Optional[bool] = None

@dataclass
class MedicalRecord:
    """Medical record structure matching contract schema"""
    id: str
    patient_id: str
    provider_id: str
    record_type: RecordType
    data: EncryptedData
    metadata: RecordMetadata
    timestamp: int
    is_encrypted: bool
    signature: Optional[str] = None

@dataclass
class ConsentGrant:
    """Consent grant from patient to provider"""
    id: str
    patient_id: str
    provider_id: str
    granted_at: int
    revoked_at: Optional[int] = None
    status: ConsentStatus
    scope: Optional[List[str]] = None
    expires_at: Optional[int] = None

@dataclass
class VerificationMethod:
    """Verification method (public key) for W3C DID"""
    id: str
    method_type: VerificationMethodType
    controller: str
    public_key: str
    is_active: bool
    created: int
    last_rotated: int

@dataclass
class ServiceEndpoint:
    """Service endpoint for W3C DID"""
    id: str
    type: str
    url: str

@dataclass
class IdentityDocument:
    """Identity document (Decentralized Identifier) per W3C DID spec"""
    id: str
    context: List[str]
    verification_methods: List[VerificationMethod]
    authentication_methods: Optional[List[VerificationRelationship]] = None
    assertion_methods: Optional[List[VerificationRelationship]] = None
    service_endpoints: Optional[List[ServiceEndpoint]] = None
    created: int
    updated: Optional[int] = None
    proof: Optional[str] = None

@dataclass
class PaymentStatus:
    """Payment status for healthcare claims and payments"""
    id: str
    patient_id: str
    provider_id: str
    amount: int
    currency: str
    status: PaymentStatusEnum
    service_id: Optional[str] = None
    policy_id: Optional[str] = None
    created_at: int
    updated_at: int
    completed_at: Optional[int] = None
    transaction_hash: Optional[str] = None

@dataclass
class AuditEntry:
    """Audit log entry for compliance and forensics"""
    id: str
    actor: str
    action: ActionType
    resource: Optional[str] = None
    resource_type: Optional[str] = None
    result: Optional[str] = None
    reason: Optional[str] = None
    timestamp: int
    ip_address: Optional[str] = None
    metadata: Optional[Dict[str, str]] = None

# ==================== Contract Client Bindings ====================

class ContractMonitoringClient(Protocol):
    """Generated client binding for the contract_monitoring public interface."""

    def get_dashboard(self, ) -> Any:
        """Get dashboard snapshot"""
        ...

    def initialize(self, admin: str, alert_config: Any) -> None:
        """Initialize the monitoring contract"""
        ...

    def record_call(self, caller: str, function_name: str, gas_used: int) -> None:
        """Record a successful function call"""
        ...

    def record_error(self, function_name: str) -> None:
        """Record a failed function call"""
        ...

class IdentityRegistryClient(Protocol):
    """Generated client binding for the identity_registry public interface."""

    def initialize(self, admin: str) -> None:
        """Initialize the identity registry"""
        ...

    def register_identity(self, identity: str, role: str, metadata: str) -> None:
        """Register a new identity"""
        ...

    def revoke_identity(self, identity: str) -> None:
        """Revoke an identity"""
        ...

    def verify_identity(self, identity: str, role: str) -> bool:
        """Verify an identity exists and has role"""
        ...

class MedicalRecordsClient(Protocol):
    """Generated client binding for the medical_records public interface."""

    def create_record(self, patient: str, data: str, record_type: str) -> int:
        """Create a new medical record"""
        ...

    def delete_record(self, patient: str, record_id: int) -> None:
        """Soft-delete a medical record"""
        ...

    def grant_access(self, patient: str, provider: str) -> None:
        """Grant a provider access to patient records"""
        ...

    def initialize(self, admin: str, rbac_id: str) -> None:
        """Initialize the contract with admin and RBAC"""
        ...

    def read_record(self, patient: str, record_id: int) -> MedicalRecord:
        """Read a medical record by ID"""
        ...

    def revoke_access(self, patient: str, provider: str) -> None:
        """Revoke provider access to patient records"""
        ...

    def update_record(self, patient: str, record_id: int, data: str) -> None:
        """Update an existing medical record"""
        ...

class PatientConsentManagementClient(Protocol):
    """Generated client binding for the patient_consent_management public interface."""

    def check_consent(self, patient: str, provider: str, data_type: str) -> bool:
        """Check if consent is active"""
        ...

    def grant_consent(self, patient: str, provider: str, data_type: str, expiry_ledger: int) -> None:
        """Grant consent for data access"""
        ...

    def initialize(self, admin: str) -> None:
        """Initialize the consent contract"""
        ...

    def revoke_consent(self, patient: str, provider: str, data_type: str) -> None:
        """Revoke previously granted consent"""
        ...

# ==================== Binding Coverage ====================

SDK_CONTRACT_INTERFACE_COVERAGE: Dict[str, List[str]] = {
    "contract_monitoring": ["get_dashboard", "initialize", "record_call", "record_error"],
    "identity_registry": ["initialize", "register_identity", "revoke_identity", "verify_identity"],
    "medical_records": ["create_record", "delete_record", "grant_access", "initialize", "read_record", "revoke_access", "update_record"],
    "patient_consent_management": ["check_consent", "grant_consent", "initialize", "revoke_consent"],
}
