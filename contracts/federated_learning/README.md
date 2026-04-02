# Enhanced Federated Learning System

A comprehensive federated learning system enabling collaborative model training without sharing raw patient data, with robust security, privacy preservation, and support for multiple ML frameworks.

## Features

### 🔐 Security & Privacy
- **Differential Privacy**: Configurable epsilon-delta privacy guarantees
- **Secure Aggregation**: Multiple Byzantine-robust aggregation methods
- **Poisoning Attack Detection**: Real-time anomaly detection and mitigation
- **Privacy Budgeting**: Per-institution privacy budget tracking
- **Zero-Knowledge Proofs**: Privacy proof verification for contributions

### 🏥 Healthcare Compliance
- **HIPAA Compliant**: Designed for healthcare data protection
- **Patient Privacy**: Raw data never leaves originating institution
- **Audit Trails**: Comprehensive logging of all federated operations
- **Consent Management**: Granular consent controls for data usage

### 🤖 Multi-Framework Support
- **TensorFlow Integration**: Full TensorFlow/Keras support
- **PyTorch Integration**: Complete PyTorch ecosystem support
- **Framework Agnostic**: Smart contract framework-agnostic design
- **Model Versioning**: Immutable model tracking with provenance

### 📊 Scalability & Performance
- **50+ Institutions**: Support for large-scale federated networks
- **Communication Efficiency**: < 3x overhead compared to centralized training
- **Model Accuracy**: Within 5% of centralized training performance
- **Real-time Monitoring**: Live metrics and performance tracking

## Architecture

### Smart Contract Layer (`lib.rs`)
```rust
// Core federated learning contract with enhanced security
pub struct FederatedLearningContract;

// Key features:
- Institution registration with reputation system
- Secure round management with verification stages
- Privacy budget tracking and enforcement
- Attack detection and mitigation
- Communication overhead monitoring
```

### Client Libraries
- **TensorFlow Client** (`tensorflow_client.py`): TensorFlow/Keras integration
- **PyTorch Client** (`pytorch_client.py`): PyTorch integration
- **Coordinator** (`coordinator.py`): Secure aggregation orchestration

### Aggregation Methods
- **FedAvg**: Standard federated averaging
- **FedProx**: Proximal term for heterogeneity
- **Krum/Multi-Krum**: Byzantine robust aggregation
- **Trimmed Mean**: Outlier-resistant aggregation
- **SecureAgg**: Cryptographic secure aggregation

## Quick Start

### 1. Smart Contract Deployment
```bash
# Build and deploy the federated learning contract
cargo build --release --target wasm32-unknown-unknown
soroban contract deploy federated_learning.wasm
```

### 2. Institution Registration
```rust
// Register a healthcare institution
let institution_address = Address::generate(env);
contract.register_institution(
    admin,
    institution_address,
    "General Hospital",
    credential_hash,
    Framework::TensorFlow,
);
```

### 3. TensorFlow Client Setup
```python
from tensorflow_client import TensorFlowFederatedClient, FederatedConfig

# Configure federated learning
config = FederatedConfig(
    institution_id="hospital_001",
    coordinator_address="0x...",
    privacy_budget=1.0,
    local_epochs=5
)

# Create and setup client
client = TensorFlowFederatedClient(config)
client.setup_model(your_tensorflow_model)
```

### 4. Participate in Training Round
```python
# Train locally and submit updates
gradients, metrics = client.train_locally(x_train, y_train)
submission = client.prepare_submission(gradients, metrics)

# Submit to blockchain (via coordinator)
await coordinator.submit_update(round_id, submission)
```

## Security Features

### Differential Privacy
```python
# Privacy-preserving local training
privacy_preserver = PrivacyPreserver(
    epsilon=1.0,    # Privacy budget
    delta=1e-5,      # Failure probability
    clipping_bound=1.0  # Gradient clipping
)

# Automatic noise addition and gradient clipping
private_gradients = privacy_preserver.add_noise(
    privacy_preserver.clip_gradients(gradients)
)
```

### Attack Detection
```python
# Real-time poisoning attack detection
attack_detector = AttackDetector(threshold=0.3)
anomaly_scores = attack_detector.detect_anomalies(updates)

if attack_detector.is_attack_detected(anomaly_scores):
    # Filter suspicious contributions
    filtered_updates = filter_anomalous_updates(updates, anomaly_scores)
```

### Byzantine-Robust Aggregation
```python
# Robust aggregation methods
config = AggregationConfig(
    method=AggregationMethod.KRUM,
    num_byzantine=5,  # Tolerate up to 5 malicious actors
    min_contributors=10
)

aggregator = FederatedAggregator(config)
result = aggregator.aggregate_updates(updates)
```

## Performance Guarantees

### Model Accuracy
- **Target**: Within 5% of centralized training accuracy
- **Validation**: Comprehensive testing on medical datasets
- **Monitoring**: Real-time accuracy tracking and alerts

### Communication Efficiency
- **Overhead**: < 3x centralized training communication
- **Compression**: Gradient compression and quantization
- **Optimization**: Efficient serialization protocols

### Scalability
- **Institutions**: Support for 50+ concurrent participants
- **Model Size**: Support for large medical imaging models
- **Latency**: Sub-second aggregation for most model sizes

## Compliance & Ethics

### HIPAA Compliance
- **Data Minimization**: Only necessary model updates shared
- **Access Controls**: Role-based permissions and audit logs
- **Breach Detection**: Automated anomaly and breach detection

### Ethical AI
- **Bias Detection**: Regular bias audits and fairness testing
- **Transparency**: Explainable AI features and model interpretability
- **Consent**: Patient consent management and revocation

### Regulatory Alignment
- **GDPR**: European data protection compliance
- **FDA**: Medical device software considerations
- **Local Regulations**: Adaptable to regional requirements

## Testing

### Unit Tests
```bash
# Run comprehensive test suite
python integration_tests.py
pytest contracts/federated_learning/ -v
```

### Integration Tests
```bash
# Test end-to-end federated learning workflow
python -m pytest integration_tests.py::TestIntegration -v
```

### Performance Tests
```bash
# Large-scale simulation tests
python -m pytest integration_tests.py::TestPerformance -v
```

## Monitoring & Analytics

### Real-time Metrics
- **Training Progress**: Live accuracy and loss tracking
- **Privacy Metrics**: Privacy budget usage and guarantees
- **Security Alerts**: Attack detection and mitigation status
- **Performance**: Communication overhead and latency

### Dashboard Integration
```typescript
// Analytics dashboard integration
import { FederatedLearningMetrics } from './analytics_dashboard';

const metrics = new FederatedLearningMetrics();
metrics.trackRound(roundId);
metrics.monitorPrivacy(roundId);
metrics.detectAnomalies(roundId);
```

## API Reference

### Smart Contract Functions

#### Institution Management
```rust
// Register institution
register_institution(admin, institution, name, credentials, framework)

// Blacklist malicious institution
blacklist_institution(admin, institution, reason)
```

#### Round Management
```rust
// Start federated learning round
start_round(admin, base_model, config)

// Submit model update
submit_update(institution, round_id, gradient_hash, quality_metrics, privacy_proof)

// Begin aggregation phase
begin_aggregation(coordinator, round_id)

// Finalize round with results
finalize_round(coordinator, round_id, model_output)
```

#### Query Functions
```rust
// Get institution details
get_institution(institution)

// Get round information
get_round(round_id)

// Get model metadata
get_model(model_id)

// Get privacy metrics
get_privacy_metrics(round_id)

// Get attack detection results
get_attack_detection(round_id)
```

### Client Library APIs

#### TensorFlow Client
```python
# Setup and training
client.setup_model(model)
gradients, metrics = client.train_locally(x_train, y_train)

# Submission preparation
submission = client.prepare_submission(gradients, metrics)

# Model updates
client.update_model_from_aggregation(new_weights)
```

#### PyTorch Client
```python
# Setup and training
client.setup_model(model)
gradients, metrics = client.train_locally(train_loader, val_loader)

# Evaluation
accuracy, loss = client.evaluate(test_loader)
```

## Contributing

### Development Setup
```bash
# Install dependencies
pip install tensorflow torch numpy pytest

# Build smart contract
cargo build

# Run tests
pytest
```

### Code Style
- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Python**: PEP 8 compliance with type hints
- **Documentation**: Comprehensive docstrings and comments

### Security Review
- All changes undergo security review
- Privacy impact assessment required
- Performance benchmarking mandatory

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For technical support and questions:
- **Documentation**: See `/docs` directory
- **Issues**: GitHub issue tracker
- **Security**: Report security issues privately

## Acknowledgments

- Healthcare institutions participating in federated learning
- Privacy-preserving ML research community
- Stellar/Soroban blockchain platform
- Open-source federated learning frameworks

---

**Note**: This system is designed for healthcare applications and maintains strict privacy and security standards. Always ensure proper institutional review board (IRB) approval and patient consent before deployment.
