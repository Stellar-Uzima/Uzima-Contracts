"""
Federated Learning Integration Tests
Tests for the enhanced federated learning system
"""

import pytest
import numpy as np
import tensorflow as tf
import torch
import torch.nn as nn
from unittest.mock import Mock, patch
import tempfile
import os
import sys

# Add the federated learning modules to path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from tensorflow_client import TensorFlowFederatedClient, FederatedConfig, TrainingMetrics
from pytorch_client import PyTorchFederatedClient, CNNModel
from coordinator import FederatedAggregator, AggregationConfig, AggregationMethod


class TestTensorFlowClient:
    """Test cases for TensorFlow federated learning client"""
    
    @pytest.fixture
    def config(self):
        """Create test configuration"""
        return FederatedConfig(
            institution_id="test_hospital",
            coordinator_address="0x1234567890123456789012345678901234567890",
            privacy_budget=1.0,
            noise_scale=0.1,
            clipping_bound=1.0,
            local_epochs=2,
            batch_size=32
        )
    
    @pytest.fixture
    def tf_model(self):
        """Create a simple TensorFlow model for testing"""
        model = tf.keras.Sequential([
            tf.keras.layers.Dense(64, activation='relu', input_shape=(10,)),
            tf.keras.layers.Dense(32, activation='relu'),
            tf.keras.layers.Dense(10, activation='softmax')
        ])
        return model
    
    @pytest.fixture
    def sample_data(self):
        """Create sample training data"""
        x_train = np.random.random((100, 10))
        y_train = tf.keras.utils.to_categorical(np.random.randint(0, 10, 100), 10)
        return x_train, y_train
    
    def test_client_initialization(self, config):
        """Test client initialization"""
        client = TensorFlowFederatedClient(config)
        assert client.config.institution_id == "test_hospital"
        assert client.config.privacy_budget == 1.0
        assert client.privacy_preserver is not None
    
    def test_model_setup(self, config, tf_model):
        """Test model setup"""
        client = TensorFlowFederatedClient(config)
        client.setup_model(tf_model)
        
        assert client.model is not None
        assert client.optimizer is not None
        assert client.loss_fn is not None
    
    def test_local_training(self, config, tf_model, sample_data):
        """Test local training with privacy preservation"""
        client = TensorFlowFederatedClient(config)
        client.setup_model(tf_model)
        
        x_train, y_train = sample_data
        gradients, metrics = client.train_locally(x_train, y_train)
        
        assert isinstance(gradients, list)
        assert len(gradients) > 0
        assert isinstance(metrics, TrainingMetrics)
        assert metrics.loss >= 0
        assert 0 <= metrics.accuracy <= 100
        assert metrics.data_size == len(x_train)
    
    def test_gradient_hash_computation(self, config, tf_model):
        """Test gradient hash computation"""
        client = TensorFlowFederatedClient(config)
        client.setup_model(tf_model)
        
        # Create dummy gradients
        gradients = [np.random.randn(10, 10) for _ in range(3)]
        hash_value = client.compute_gradient_hash(gradients)
        
        assert isinstance(hash_value, str)
        assert len(hash_value) == 64  # SHA256 hex length
    
    def test_privacy_preservation(self, config, tf_model, sample_data):
        """Test privacy preservation mechanisms"""
        client = TensorFlowFederatedClient(config)
        client.setup_model(tf_model)
        
        x_train, y_train = sample_data
        gradients, metrics = client.train_locally(x_train, y_train)
        
        # Check that privacy loss is tracked
        assert metrics.privacy_loss > 0
        
        # Check that gradients are different from raw weights
        initial_weights = tf_model.get_weights()
        final_weights = tf_model.get_weights()
        
        # After training, weights should change
        for init, final in zip(initial_weights, final_weights):
            assert not np.allclose(init, final)


class TestPyTorchClient:
    """Test cases for PyTorch federated learning client"""
    
    @pytest.fixture
    def config(self):
        """Create test configuration"""
        return FederatedConfig(
            institution_id="test_hospital_pytorch",
            coordinator_address="0x1234567890123456789012345678901234567890",
            privacy_budget=1.0,
            noise_scale=0.1,
            clipping_bound=1.0,
            local_epochs=2,
            batch_size=32
        )
    
    @pytest.fixture
    def pytorch_model(self):
        """Create a simple PyTorch model for testing"""
        model = nn.Sequential(
            nn.Linear(10, 64),
            nn.ReLU(),
            nn.Linear(64, 32),
            nn.ReLU(),
            nn.Linear(32, 10)
        )
        return model
    
    @pytest.fixture
    def sample_dataset(self):
        """Create sample dataset"""
        class DummyDataset(torch.utils.data.Dataset):
            def __init__(self, size=100):
                self.size = size
                
            def __len__(self):
                return self.size
                
            def __getitem__(self, idx):
                data = torch.randn(10)
                label = torch.randint(0, 10, (1,)).item()
                return data, label
        
        return DummyDataset(100)
    
    def test_pytorch_client_initialization(self, config):
        """Test PyTorch client initialization"""
        client = PyTorchFederatedClient(config)
        assert client.config.institution_id == "test_hospital_pytorch"
        assert client.device is not None
        assert client.privacy_preserver is not None
    
    def test_model_setup(self, config, pytorch_model):
        """Test PyTorch model setup"""
        client = PyTorchFederatedClient(config)
        client.setup_model(pytorch_model)
        
        assert client.model is not None
        assert client.optimizer is not None
        assert client.loss_fn is not None
    
    def test_local_training_pytorch(self, config, pytorch_model, sample_dataset):
        """Test PyTorch local training"""
        client = PyTorchFederatedClient(config)
        client.setup_model(pytorch_model)
        
        train_loader = torch.utils.data.DataLoader(sample_dataset, batch_size=16)
        gradients, metrics = client.train_locally(train_loader)
        
        assert isinstance(gradients, dict)
        assert len(gradients) > 0
        assert isinstance(metrics, TrainingMetrics)
        assert metrics.loss >= 0
        assert metrics.data_size == len(sample_dataset)
    
    def test_gradient_clipping(self, config):
        """Test gradient clipping functionality"""
        from pytorch_client import PrivacyPreserver
        
        privacy_preserver = PrivacyPreserver(1.0, 1e-5, 1.0)
        
        # Create a model with large gradients
        model = nn.Linear(10, 1)
        # Set large gradients
        for param in model.parameters():
            if param.grad is not None:
                param.grad.data.fill_(10.0)
        
        clipping_norm = privacy_preserver.clip_gradients(model)
        assert clipping_norm <= privacy_preserver.clipping_bound


class TestFederatedAggregator:
    """Test cases for federated aggregation"""
    
    @pytest.fixture
    def aggregation_config(self):
        """Create aggregation configuration"""
        return AggregationConfig(
            method=AggregationMethod.FED_AVG,
            min_contributors=2,
            max_contributors=10
        )
    
    @pytest.fixture
    def sample_updates(self):
        """Create sample model updates"""
        updates = {}
        
        for i in range(3):
            update = {}
            for layer_name in ['layer1', 'layer2', 'layer3']:
                # Random gradient-like data
                if layer_name == 'layer1':
                    update[layer_name] = np.random.randn(64, 32)
                elif layer_name == 'layer2':
                    update[layer_name] = np.random.randn(32, 16)
                else:
                    update[layer_name] = np.random.randn(16)
            
            contributor_id = f"hospital_{i+1:03d}"
            updates[contributor_id] = update
        
        return updates
    
    def test_aggregator_initialization(self, aggregation_config):
        """Test aggregator initialization"""
        aggregator = FederatedAggregator(aggregation_config)
        assert aggregator.config == aggregation_config
        assert aggregator.attack_detector is not None
    
    def test_federated_averaging(self, aggregation_config, sample_updates):
        """Test federated averaging aggregation"""
        aggregator = FederatedAggregator(aggregation_config)
        
        result = aggregator.aggregate_updates(sample_updates)
        
        assert result.aggregated_weights is not None
        assert len(result.contributor_weights) == 3
        assert result.aggregation_score > 0
        assert 0 <= result.privacy_guarantee <= 1
        assert result.communication_cost > 0
    
    def test_byzantine_robust_methods(self, sample_updates):
        """Test Byzantine-robust aggregation methods"""
        # Test Krum
        krum_config = AggregationConfig(
            method=AggregationMethod.KRUM,
            num_byzantine=1,
            min_contributors=3
        )
        krum_aggregator = FederatedAggregator(krum_config)
        krum_result = krum_aggregator.aggregate_updates(sample_updates)
        assert krum_result.aggregated_weights is not None
        
        # Test Trimmed Mean
        trimmed_config = AggregationConfig(
            method=AggregationMethod.TRIMMED_MEAN,
            trim_fraction=0.2,
            min_contributors=3
        )
        trimmed_aggregator = FederatedAggregator(trimmed_config)
        trimmed_result = trimmed_aggregator.aggregate_updates(sample_updates)
        assert trimmed_result.aggregated_weights is not None
    
    def test_attack_detection(self, aggregation_config, sample_updates):
        """Test attack detection functionality"""
        aggregator = FederatedAggregator(aggregation_config)
        
        # Add an anomalous update
        anomalous_update = {}
        for layer_name in ['layer1', 'layer2', 'layer3']:
            anomalous_update[layer_name] = np.random.randn(*sample_updates['hospital_001'][layer_name].shape) * 100
        
        sample_updates['malicious_hospital'] = anomalous_update
        
        result = aggregator.aggregate_updates(sample_updates)
        
        # Should detect anomaly and potentially filter
        assert result.robustness_score < 1.0  # Some anomaly detected
    
    def test_reputation_weighting(self, aggregation_config, sample_updates):
        """Test reputation-based weighting"""
        aggregator = FederatedAggregator(aggregation_config)
        
        reputations = {
            'hospital_001': 0.9,
            'hospital_002': 0.7,
            'hospital_003': 0.5
        }
        
        result = aggregator.aggregate_updates(sample_updates, contributor_reputations=reputations)
        assert result.aggregated_weights is not None
    
    def test_insufficient_contributors(self, aggregation_config):
        """Test behavior with insufficient contributors"""
        aggregator = FederatedAggregator(aggregation_config)
        
        # Only one contributor
        single_update = {'hospital_001': {'layer1': np.random.randn(10, 5)}}
        
        with pytest.raises(ValueError, match="Insufficient contributors"):
            aggregator.aggregate_updates(single_update)


class TestIntegration:
    """Integration tests for the complete federated learning system"""
    
    def test_end_to_end_workflow(self):
        """Test complete federated learning workflow"""
        # Configuration
        config = FederatedConfig(
            institution_id="integration_test",
            coordinator_address="0x1234567890123456789012345678901234567890",
            local_epochs=2,
            batch_size=16
        )
        
        # Create TensorFlow client
        tf_model = tf.keras.Sequential([
            tf.keras.layers.Dense(32, activation='relu', input_shape=(5,)),
            tf.keras.layers.Dense(16, activation='relu'),
            tf.keras.layers.Dense(3, activation='softmax')
        ])
        
        tf_client = TensorFlowFederatedClient(config)
        tf_client.setup_model(tf_model)
        
        # Create training data
        x_train = np.random.random((50, 5))
        y_train = tf.keras.utils.to_categorical(np.random.randint(0, 3, 50), 3)
        
        # Train locally
        gradients, metrics = tf_client.train_locally(x_train, y_train)
        
        # Prepare submission
        submission = tf_client.prepare_submission(gradients, metrics)
        
        assert 'gradient_hash' in submission
        assert 'quality_metrics' in submission
        assert 'privacy_proof' in submission
        
        # Create aggregator
        agg_config = AggregationConfig(
            method=AggregationMethod.FED_AVG,
            min_contributors=1
        )
        aggregator = FederatedAggregator(agg_config)
        
        # Convert gradients to numpy format for aggregation
        updates = {'test_hospital': {f'layer_{i}': grad for i, grad in enumerate(gradients)}}
        
        # Aggregate
        result = aggregator.aggregate_updates(updates)
        
        assert result.aggregated_weights is not None
        assert result.aggregation_score > 0
    
    def test_privacy_budget_tracking(self):
        """Test privacy budget tracking across multiple rounds"""
        config = FederatedConfig(
            institution_id="privacy_test",
            coordinator_address="0x1234567890123456789012345678901234567890",
            privacy_budget=2.0,
            local_epochs=1
        )
        
        client = TensorFlowFederatedClient(config)
        
        model = tf.keras.Sequential([
            tf.keras.layers.Dense(16, activation='relu', input_shape=(4,)),
            tf.keras.layers.Dense(8, activation='relu'),
            tf.keras.layers.Dense(2, activation='softmax')
        ])
        client.setup_model(model)
        
        # Multiple rounds of training
        total_privacy_loss = 0
        for round_num in range(3):
            x_train = np.random.random((30, 4))
            y_train = tf.keras.utils.to_categorical(np.random.randint(0, 2, 30), 2)
            
            gradients, metrics = client.train_locally(x_train, y_train)
            total_privacy_loss += metrics.privacy_loss
            
            print(f"Round {round_num + 1}: Privacy loss = {metrics.privacy_loss:.4f}")
        
        assert total_privacy_loss > 0
        assert total_privacy_loss <= config.privacy_budget * 2  # Some tolerance


# Performance tests
class TestPerformance:
    """Performance tests for the federated learning system"""
    
    def test_large_scale_simulation(self):
        """Test performance with many simulated institutions"""
        config = AggregationConfig(
            method=AggregationMethod.FED_AVG,
            min_contributors=10,
            max_contributors=100
        )
        
        aggregator = FederatedAggregator(config)
        
        # Simulate 50 institutions
        updates = {}
        for i in range(50):
            update = {}
            for layer_name in ['conv1', 'conv2', 'fc1']:
                update[layer_name] = np.random.randn(64, 32) if 'conv' in layer_name else np.random.randn(128)
            
            updates[f'hospital_{i+1:03d}'] = update
        
        import time
        start_time = time.time()
        result = aggregator.aggregate_updates(updates)
        aggregation_time = time.time() - start_time
        
        assert aggregation_time < 5.0  # Should complete within 5 seconds
        assert len(result.contributor_weights) == 50
        assert result.aggregation_score > 0
    
    def test_communication_overhead(self):
        """Test communication overhead estimation"""
        config = AggregationConfig(method=AggregationMethod.FED_AVG)
        aggregator = FederatedAggregator(config)
        
        # Create updates with known size
        updates = {}
        for i in range(10):
            update = {
                'large_layer': np.random.randn(1000, 1000),  # ~8MB per layer
                'small_layer': np.random.randn(100, 100)     # ~80KB per layer
            }
            updates[f'hospital_{i+1:03d}'] = update
        
        result = aggregator.aggregate_updates(updates)
        
        # Check that communication cost is properly estimated
        expected_size = 10 * (1000 * 1000 + 100 * 100) * 8  # 8 bytes per float
        assert abs(result.communication_cost - expected_size) < expected_size * 0.1


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
