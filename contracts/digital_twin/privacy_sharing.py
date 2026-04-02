"""
Privacy-Preserving Data Sharing for Digital Twin Research
Implements differential privacy, anonymization, and secure data sharing mechanisms
"""

import numpy as np
import pandas as pd
import json
import logging
from typing import Dict, List, Optional, Any, Tuple, Union
from dataclasses import dataclass, asdict
from enum import Enum
from datetime import datetime, timedelta
import hashlib
import secrets
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import base64

# Privacy libraries
from diffprivlib import mechanisms
from diffprivlib.accountant import BudgetAccountant
import syft as sy
from syft.frameworks.torch.dp import pate

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class PrivacyLevel(Enum):
    """Privacy protection levels"""
    MINIMAL = "minimal"      # Basic anonymization
    STANDARD = "standard"    # Differential privacy with ε=1.0
    HIGH = "high"           # Differential privacy with ε=0.1
    MAXIMUM = "maximum"     # Differential privacy with ε=0.01


class AnonymizationMethod(Enum):
    """Data anonymization methods"""
    GENERALIZATION = "generalization"
    SUPPRESSION = "suppression"
    PERTURBATION = "perturbation"
    SYNTHETIC_DATA = "synthetic_data"
    K_ANONYMITY = "k_anonymity"
    L_DIVERSITY = "l_diversity"
    T_CLOSENESS = "t_closeness"


@dataclass
class PrivacyConfig:
    """Configuration for privacy protection"""
    privacy_level: PrivacyLevel
    anonymization_method: AnonymizationMethod
    epsilon: float
    delta: float
    k_anonymity: int
    l_diversity: int
    t_closeness: float
    sensitive_attributes: List[str]
    quasi_identifiers: List[str]
    retention_days: int
    access_log_required: bool


@dataclass
class ResearchRequest:
    """Request for research data access"""
    request_id: str
    researcher_id: str
    institution: str
    purpose: str
    data_types: List[str]
    time_range: Tuple[datetime, datetime]
    privacy_level: PrivacyLevel
    intended_use: str
    irb_approved: bool
    data_use_agreement: str


@dataclass
class DataSnapshot:
    """Privacy-preserved data snapshot"""
    snapshot_id: str
    twin_id: str
    created_at: datetime
    expires_at: datetime
    privacy_config: PrivacyConfig
    data_hash: str
    access_count: int
    max_access: int
    encryption_key_hash: str
    metadata: Dict[str, Any]


class DifferentialPrivacyMechanism:
    """Implements differential privacy mechanisms"""
    
    def __init__(self, epsilon: float, delta: float = 1e-5):
        self.epsilon = epsilon
        self.delta = delta
        self.accountant = BudgetAccountant(epsilon, delta)
        
    def add_laplace_noise(self, data: Union[float, np.ndarray], sensitivity: float = 1.0) -> Union[float, np.ndarray]:
        """Add Laplace noise for differential privacy"""
        mechanism = mechanisms.Laplace(epsilon=self.epsilon, sensitivity=sensitivity)
        
        if isinstance(data, (int, float)):
            return mechanism.randomise(data)
        elif isinstance(data, np.ndarray):
            return np.array([mechanism.randomise(x) for x in data])
        else:
            raise ValueError("Unsupported data type for Laplace noise")
    
    def add_gaussian_noise(self, data: Union[float, np.ndarray], sensitivity: float = 1.0) -> Union[float, np.ndarray]:
        """Add Gaussian noise for differential privacy"""
        mechanism = mechanisms.Gaussian(epsilon=self.epsilon, delta=self.delta, sensitivity=sensitivity)
        
        if isinstance(data, (int, float)):
            return mechanism.randomise(data)
        elif isinstance(data, np.ndarray):
            return np.array([mechanism.randomise(x) for x in data])
        else:
            raise ValueError("Unsupported data type for Gaussian noise")
    
    def count_with_dp(self, data: List[Any]) -> int:
        """Count with differential privacy"""
        mechanism = mechanisms.Laplace(epsilon=self.epsilon, sensitivity=1.0)
        return int(mechanism.randomise(len(data)))
    
    def average_with_dp(self, data: List[float]) -> float:
        """Compute average with differential privacy"""
        if not data:
            return 0.0
        
        sensitivity = 1.0 / len(data)  # Sensitivity of average
        mechanism = mechanisms.Laplace(epsilon=self.epsilon, sensitivity=sensitivity)
        return mechanism.randomise(np.mean(data))


class DataAnonymizer:
    """Implements various data anonymization techniques"""
    
    def __init__(self, config: PrivacyConfig):
        self.config = config
        self.generalization_hierarchy = self._build_generalization_hierarchy()
        
    def _build_generalization_hierarchy(self) -> Dict[str, Dict]:
        """Build generalization hierarchies for common attributes"""
        return {
            'age': {
                5: 'child',
                10: 'adolescent',
                20: 'young_adult',
                40: 'middle_aged',
                60: 'senior'
            },
            'zip_code': {
                1000: 'first_3_digits',
                10000: 'first_2_digits',
                100000: 'state_level'
            },
            'date': {
                1: 'month',
                12: 'season',
                365: 'year'
            }
        }
    
    def generalize_value(self, value: Any, attribute: str, level: int) -> Any:
        """Generalize a value to specified level"""
        if attribute not in self.generalization_hierarchy:
            return value
        
        hierarchy = self.generalization_hierarchy[attribute]
        
        if isinstance(value, (int, float)):
            for threshold, generalized in sorted(hierarchy.items()):
                if value < threshold:
                    return generalized
        elif isinstance(value, str) and attribute == 'date':
            # Handle date generalization
            try:
                date_obj = datetime.strptime(value, '%Y-%m-%d')
                if level == 1:
                    return date_obj.strftime('%Y-%m')
                elif level == 2:
                    season = self._get_season(date_obj.month)
                    return f"{date_obj.year}_{season}"
                elif level == 3:
                    return str(date_obj.year)
            except:
                pass
        
        return value
    
    def _get_season(self, month: int) -> str:
        """Get season from month"""
        if month in [12, 1, 2]:
            return 'winter'
        elif month in [3, 4, 5]:
            return 'spring'
        elif month in [6, 7, 8]:
            return 'summer'
        else:
            return 'fall'
    
    def suppress_attribute(self, data: pd.DataFrame, attribute: str) -> pd.DataFrame:
        """Suppress (remove) an attribute"""
        return data.drop(columns=[attribute])
    
    def achieve_k_anonymity(self, data: pd.DataFrame, quasi_identifiers: List[str], k: int) -> pd.DataFrame:
        """Achieve k-anonymity through generalization and suppression"""
        current_data = data.copy()
        
        # Iteratively generalize until k-anonymity is achieved
        for level in range(1, 4):  # Up to 3 levels of generalization
            # Check current anonymity
            groups = current_data.groupby(quasi_identifiers).size()
            
            if all(group_size >= k for group_size in groups):
                break
            
            # Generalize quasi-identifiers
            for qi in quasi_identifiers:
                if qi in current_data.columns:
                    current_data[qi] = current_data[qi].apply(
                        lambda x: self.generalize_value(x, qi, level)
                    )
        
        # Suppress records that still don't meet k-anonymity
        groups = current_data.groupby(quasi_identifiers).size()
        valid_groups = groups[groups >= k].index
        
        current_data = current_data.set_index(quasi_identifiers)
        current_data = current_data.loc[valid_groups].reset_index()
        
        return current_data
    
    def synthetic_data_generation(self, data: pd.DataFrame, num_samples: int) -> pd.DataFrame:
        """Generate synthetic data preserving statistical properties"""
        synthetic_data = []
        
        for _ in range(num_samples):
            sample = {}
            for column in data.columns:
                if data[column].dtype in ['int64', 'float64']:
                    # Sample from normal distribution
                    mean = data[column].mean()
                    std = data[column].std()
                    sample[column] = np.random.normal(mean, std)
                else:
                    # Sample from existing values
                    sample[column] = np.random.choice(data[column].values)
            
            synthetic_data.append(sample)
        
        return pd.DataFrame(synthetic_data)


class SecureDataSharing:
    """Manages secure data sharing with privacy preservation"""
    
    def __init__(self, config: PrivacyConfig):
        self.config = config
        self.dp_mechanism = DifferentialPrivacyMechanism(config.epsilon, config.delta)
        self.anonymizer = DataAnonymizer(config)
        self.encryption_key = self._generate_encryption_key()
        self.access_log: List[Dict] = []
        
    def _generate_encryption_key(self) -> bytes:
        """Generate encryption key"""
        password = b"digital_twin_secure_key"
        salt = b"digital_twin_salt"
        kdf = PBKDF2HMAC(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            iterations=100000,
        )
        key = base64.urlsafe_b64encode(kdf.derive(password))
        return key
    
    def create_privacy_preserved_snapshot(
        self,
        twin_id: str,
        data: pd.DataFrame,
        researcher_id: str,
        duration_days: int = 30
    ) -> DataSnapshot:
        """Create privacy-preserved data snapshot"""
        logger.info(f"Creating privacy-preserved snapshot for twin {twin_id}")
        
        # Apply privacy protection based on configuration
        protected_data = self._apply_privacy_protection(data)
        
        # Encrypt the data
        encrypted_data = self._encrypt_data(protected_data)
        
        # Create snapshot
        snapshot_id = f"snapshot_{datetime.now().strftime('%Y%m%d_%H%M%S')}_{secrets.token_hex(8)}"
        data_hash = self._compute_data_hash(protected_data)
        
        snapshot = DataSnapshot(
            snapshot_id=snapshot_id,
            twin_id=twin_id,
            created_at=datetime.now(),
            expires_at=datetime.now() + timedelta(days=duration_days),
            privacy_config=self.config,
            data_hash=data_hash,
            access_count=0,
            max_access=10,
            encryption_key_hash=hashlib.sha256(self.encryption_key).hexdigest(),
            metadata={
                'researcher_id': researcher_id,
                'original_rows': len(data),
                'protected_rows': len(protected_data),
                'privacy_level': self.config.privacy_level.value,
                'anonymization_method': self.config.anonymization_method.value
            }
        )
        
        # Log access
        self._log_access(snapshot_id, researcher_id, 'create')
        
        return snapshot
    
    def _apply_privacy_protection(self, data: pd.DataFrame) -> pd.DataFrame:
        """Apply privacy protection based on configuration"""
        protected_data = data.copy()
        
        # Apply anonymization method
        if self.config.anonymization_method == AnonymizationMethod.GENERALIZATION:
            protected_data = self._apply_generalization(protected_data)
        elif self.config.anonymization_method == AnonymizationMethod.SUPPRESSION:
            protected_data = self._apply_suppression(protected_data)
        elif self.config.anonymization_method == AnonymizationMethod.K_ANONYMITY:
            protected_data = self.anonymizer.achieve_k_anonymity(
                protected_data, self.config.quasi_identifiers, self.config.k_anonymity
            )
        elif self.config.anonymization_method == AnonymizationMethod.SYNTHETIC_DATA:
            protected_data = self.anonymizer.synthetic_data_generation(
                protected_data, len(protected_data)
            )
        
        # Apply differential privacy to numerical columns
        for column in protected_data.select_dtypes(include=[np.number]).columns:
            if column not in self.config.sensitive_attributes:
                protected_data[column] = protected_data[column].apply(
                    lambda x: self.dp_mechanism.add_laplace_noise(x, sensitivity=1.0)
                )
        
        return protected_data
    
    def _apply_generalization(self, data: pd.DataFrame) -> pd.DataFrame:
        """Apply data generalization"""
        generalized_data = data.copy()
        
        for qi in self.config.quasi_identifiers:
            if qi in generalized_data.columns:
                # Apply 2 levels of generalization
                generalized_data[qi] = generalized_data[qi].apply(
                    lambda x: self.anonymizer.generalize_value(x, qi, 2)
                )
        
        return generalized_data
    
    def _apply_suppression(self, data: pd.DataFrame) -> pd.DataFrame:
        """Apply data suppression"""
        suppressed_data = data.copy()
        
        # Suppress sensitive attributes
        for attr in self.config.sensitive_attributes:
            if attr in suppressed_data.columns:
                suppressed_data = self.anonymizer.suppress_attribute(suppressed_data, attr)
        
        return suppressed_data
    
    def _encrypt_data(self, data: pd.DataFrame) -> bytes:
        """Encrypt data"""
        fernet = Fernet(self.encryption_key)
        data_json = data.to_json()
        encrypted_data = fernet.encrypt(data_json.encode())
        return encrypted_data
    
    def _decrypt_data(self, encrypted_data: bytes) -> pd.DataFrame:
        """Decrypt data"""
        fernet = Fernet(self.encryption_key)
        decrypted_json = fernet.decrypt(encrypted_data).decode()
        return pd.read_json(decrypted_json)
    
    def _compute_data_hash(self, data: pd.DataFrame) -> str:
        """Compute hash of data for integrity verification"""
        data_json = data.to_json(sort_keys=True)
        return hashlib.sha256(data_json.encode()).hexdigest()
    
    def _log_access(self, snapshot_id: str, researcher_id: str, action: str) -> None:
        """Log data access"""
        log_entry = {
            'timestamp': datetime.now().isoformat(),
            'snapshot_id': snapshot_id,
            'researcher_id': researcher_id,
            'action': action
        }
        self.access_log.append(log_entry)
    
    def grant_access(self, snapshot: DataSnapshot, researcher_id: str) -> Optional[pd.DataFrame]:
        """Grant access to snapshot data"""
        # Check if snapshot is still valid
        if datetime.now() > snapshot.expires_at:
            logger.warning(f"Snapshot {snapshot.snapshot_id} has expired")
            return None
        
        # Check access limits
        if snapshot.access_count >= snapshot.max_access:
            logger.warning(f"Snapshot {snapshot.snapshot_id} has reached max access")
            return None
        
        # Verify researcher authorization (simplified)
        if snapshot.metadata.get('researcher_id') != researcher_id:
            logger.warning(f"Unauthorized access attempt by {researcher_id}")
            return None
        
        # Log access
        self._log_access(snapshot.snapshot_id, researcher_id, 'access')
        
        # In a real implementation, you would retrieve and decrypt the actual data
        # For now, return a placeholder
        logger.info(f"Access granted to snapshot {snapshot.snapshot_id}")
        
        return pd.DataFrame({
            'status': ['access_granted'],
            'snapshot_id': [snapshot.snapshot_id],
            'researcher_id': [researcher_id]
        })
    
    def verify_data_integrity(self, snapshot: DataSnapshot, data: pd.DataFrame) -> bool:
        """Verify data integrity"""
        computed_hash = self._compute_data_hash(data)
        return computed_hash == snapshot.data_hash
    
    def get_privacy_budget_usage(self) -> Dict[str, float]:
        """Get privacy budget usage"""
        return {
            'epsilon_used': self.dp_mechanism.accountant.epsilon,
            'delta_used': self.dp_mechanism.accountant.delta,
            'epsilon_remaining': max(0, self.config.epsilon - self.dp_mechanism.accountant.epsilon),
            'delta_remaining': max(0, self.config.delta - self.dp_mechanism.accountant.delta)
        }


class ResearchDataManager:
    """Manages research data requests and access"""
    
    def __init__(self):
        self.pending_requests: List[ResearchRequest] = []
        self.approved_requests: List[ResearchRequest] = []
        self.snapshots: Dict[str, DataSnapshot] = {}
        self.privacy_configs: Dict[PrivacyLevel, PrivacyConfig] = self._initialize_privacy_configs()
        
    def _initialize_privacy_configs(self) -> Dict[PrivacyLevel, PrivacyConfig]:
        """Initialize privacy configurations for different levels"""
        return {
            PrivacyLevel.MINIMAL: PrivacyConfig(
                privacy_level=PrivacyLevel.MINIMAL,
                anonymization_method=AnonymizationMethod.SUPPRESSION,
                epsilon=10.0,
                delta=1e-3,
                k_anonymity=5,
                l_diversity=2,
                t_closeness=0.2,
                sensitive_attributes=['name', 'email', 'phone'],
                quasi_identifiers=['age', 'zip_code', 'gender'],
                retention_days=365,
                access_log_required=True
            ),
            PrivacyLevel.STANDARD: PrivacyConfig(
                privacy_level=PrivacyLevel.STANDARD,
                anonymization_method=AnonymizationMethod.K_ANONYMITY,
                epsilon=1.0,
                delta=1e-5,
                k_anonymity=10,
                l_diversity=5,
                t_closeness=0.1,
                sensitive_attributes=['name', 'email', 'phone', 'ssn'],
                quasi_identifiers=['age', 'zip_code', 'gender', 'date_of_birth'],
                retention_days=180,
                access_log_required=True
            ),
            PrivacyLevel.HIGH: PrivacyConfig(
                privacy_level=PrivacyLevel.HIGH,
                anonymization_method=AnonymizationMethod.GENERALIZATION,
                epsilon=0.1,
                delta=1e-6,
                k_anonymity=20,
                l_diversity=10,
                t_closeness=0.05,
                sensitive_attributes=['name', 'email', 'phone', 'ssn', 'medical_record_number'],
                quasi_identifiers=['age', 'zip_code', 'gender', 'date_of_birth', 'ethnicity'],
                retention_days=90,
                access_log_required=True
            ),
            PrivacyLevel.MAXIMUM: PrivacyConfig(
                privacy_level=PrivacyLevel.MAXIMUM,
                anonymization_method=AnonymizationMethod.SYNTHETIC_DATA,
                epsilon=0.01,
                delta=1e-8,
                k_anonymity=50,
                l_diversity=20,
                t_closeness=0.01,
                sensitive_attributes=['name', 'email', 'phone', 'ssn', 'medical_record_number'],
                quasi_identifiers=['age', 'zip_code', 'gender', 'date_of_birth', 'ethnicity', 'income'],
                retention_days=30,
                access_log_required=True
            )
        }
    
    def submit_research_request(self, request: ResearchRequest) -> bool:
        """Submit a research data request"""
        # Validate request
        if not request.irb_approved:
            logger.error("Request must be IRB approved")
            return False
        
        if not request.data_use_agreement:
            logger.error("Data use agreement required")
            return False
        
        self.pending_requests.append(request)
        logger.info(f"Research request {request.request_id} submitted")
        return True
    
    def approve_request(self, request_id: str, approver_id: str) -> bool:
        """Approve a research request"""
        for i, request in enumerate(self.pending_requests):
            if request.request_id == request_id:
                # Move to approved requests
                approved_request = self.pending_requests.pop(i)
                self.approved_requests.append(approved_request)
                
                logger.info(f"Research request {request_id} approved by {approver_id}")
                return True
        
        logger.error(f"Request {request_id} not found")
        return False
    
    def create_research_snapshot(
        self,
        twin_id: str,
        request_id: str,
        data: pd.DataFrame
    ) -> Optional[DataSnapshot]:
        """Create research snapshot from approved request"""
        # Find approved request
        request = None
        for req in self.approved_requests:
            if req.request_id == request_id:
                request = req
                break
        
        if not request:
            logger.error(f"Approved request {request_id} not found")
            return None
        
        # Get privacy configuration
        privacy_config = self.privacy_configs[request.privacy_level]
        
        # Create secure data sharing instance
        secure_sharing = SecureDataSharing(privacy_config)
        
        # Create snapshot
        snapshot = secure_sharing.create_privacy_preserved_snapshot(
            twin_id, data, request.researcher_id
        )
        
        # Store snapshot
        self.snapshots[snapshot.snapshot_id] = snapshot
        
        logger.info(f"Research snapshot {snapshot.snapshot_id} created")
        return snapshot
    
    def get_research_data(self, snapshot_id: str, researcher_id: str) -> Optional[pd.DataFrame]:
        """Get research data from snapshot"""
        if snapshot_id not in self.snapshots:
            logger.error(f"Snapshot {snapshot_id} not found")
            return None
        
        snapshot = self.snapshots[snapshot_id]
        privacy_config = snapshot.privacy_config
        
        # Create secure data sharing instance
        secure_sharing = SecureDataSharing(privacy_config)
        
        # Grant access
        return secure_sharing.grant_access(snapshot, researcher_id)
    
    def get_request_status(self, request_id: str) -> Optional[str]:
        """Get status of research request"""
        for request in self.pending_requests:
            if request.request_id == request_id:
                return "PENDING"
        
        for request in self.approved_requests:
            if request.request_id == request_id:
                return "APPROVED"
        
        return "NOT_FOUND"
    
    def get_privacy_report(self, snapshot_id: str) -> Optional[Dict[str, Any]]:
        """Get privacy report for snapshot"""
        if snapshot_id not in self.snapshots:
            return None
        
        snapshot = self.snapshots[snapshot_id]
        privacy_config = snapshot.privacy_config
        
        secure_sharing = SecureDataSharing(privacy_config)
        
        report = {
            'snapshot_id': snapshot_id,
            'privacy_level': privacy_config.privacy_level.value,
            'anonymization_method': privacy_config.anonymization_method.value,
            'epsilon': privacy_config.epsilon,
            'delta': privacy_config.delta,
            'k_anonymity': privacy_config.k_anonymity,
            'l_diversity': privacy_config.l_diversity,
            't_closeness': privacy_config.t_closeness,
            'access_count': snapshot.access_count,
            'max_access': snapshot.max_access,
            'created_at': snapshot.created_at.isoformat(),
            'expires_at': snapshot.expires_at.isoformat(),
            'privacy_budget_usage': secure_sharing.get_privacy_budget_usage()
        }
        
        return report


# Example usage
def example_privacy_preserving_sharing():
    """Example of privacy-preserving data sharing"""
    
    # Create sample data
    np.random.seed(42)
    n_samples = 1000
    
    sample_data = pd.DataFrame({
        'patient_id': [f'patient_{i}' for i in range(n_samples)],
        'age': np.random.randint(18, 80, n_samples),
        'gender': np.random.choice(['M', 'F'], n_samples),
        'zip_code': [f'{np.random.randint(10000, 99999)}' for _ in range(n_samples)],
        'blood_pressure': np.random.normal(120, 15, n_samples),
        'cholesterol': np.random.normal(200, 40, n_samples),
        'diabetes': np.random.choice([0, 1], n_samples, p=[0.85, 0.15]),
        'name': [f'Patient_{i}' for i in range(n_samples)],
        'email': [f'patient{i}@example.com' for i in range(n_samples)]
    })
    
    # Create research request
    request = ResearchRequest(
        request_id="req_001",
        researcher_id="researcher_123",
        institution="Medical Research Institute",
        purpose="Cardiovascular disease prediction study",
        data_types=["vital_signs", "lab_results"],
        time_range=(datetime.now() - timedelta(days=365), datetime.now()),
        privacy_level=PrivacyLevel.STANDARD,
        intended_use="Machine learning model training",
        irb_approved=True,
        data_use_agreement="Signed data use agreement"
    )
    
    # Create research data manager
    manager = ResearchDataManager()
    
    # Submit and approve request
    manager.submit_research_request(request)
    manager.approve_request("req_001", "admin_001")
    
    # Create research snapshot
    snapshot = manager.create_research_snapshot(
        twin_id="twin_001",
        request_id="req_001",
        data=sample_data
    )
    
    if snapshot:
        print(f"Created research snapshot: {snapshot.snapshot_id}")
        print(f"Privacy level: {snapshot.privacy_config.privacy_level.value}")
        print(f"Original rows: {snapshot.metadata['original_rows']}")
        
        # Get privacy report
        report = manager.get_privacy_report(snapshot.snapshot_id)
        print(f"Privacy budget epsilon used: {report['privacy_budget_usage']['epsilon_used']:.4f}")
        
        # Grant access
        data = manager.get_research_data(snapshot.snapshot_id, "researcher_123")
        if data is not None:
            print("Access granted to research data")


if __name__ == "__main__":
    example_privacy_preserving_sharing()
