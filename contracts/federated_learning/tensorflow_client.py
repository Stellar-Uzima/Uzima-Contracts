"""
Federated Learning Client for TensorFlow Integration
Supports secure federated learning with privacy preservation
"""

import tensorflow as tf
import numpy as np
from typing import Dict, List, Optional, Tuple, Any
import hashlib
import time
import logging
from dataclasses import dataclass
from abc import ABC, abstractmethod

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class FederatedConfig:
    """Configuration for federated learning participation"""
    institution_id: str
    coordinator_address: str
    privacy_budget: float = 1.0
    noise_scale: float = 0.1
    clipping_bound: float = 1.0
    learning_rate: float = 0.01
    local_epochs: int = 5
    batch_size: int = 32
    framework: str = "tensorflow"


@dataclass
class TrainingMetrics:
    """Metrics for federated learning contribution"""
    loss: float
    accuracy: float
    convergence: float
    privacy_loss: float
    training_time: float
    data_size: int


class PrivacyPreserver:
    """Implements differential privacy mechanisms"""
    
    def __init__(self, epsilon: float, delta: float, clipping_bound: float):
        self.epsilon = epsilon
        self.delta = delta
        self.clipping_bound = clipping_bound
        self.noise_scale = clipping_bound * epsilon
    
    def clip_gradients(self, gradients: List[np.ndarray]) -> List[np.ndarray]:
        """Clip gradients to bound sensitivity"""
        total_norm = np.sqrt(sum(np.sum(g**2) for g in gradients))
        clip_factor = min(1.0, self.clipping_bound / (total_norm + 1e-8))
        
        return [g * clip_factor for g in gradients]
    
    def add_noise(self, gradients: List[np.ndarray]) -> List[np.ndarray]:
        """Add Gaussian noise for differential privacy"""
        noisy_gradients = []
        for grad in gradients:
            noise = np.random.normal(0, self.noise_scale, grad.shape)
            noisy_gradients.append(grad + noise)
        
        return noisy_gradients
    
    def compute_privacy_spent(self, num_examples: int, epochs: int) -> float:
        """Compute privacy loss using moments accountant"""
        # Simplified privacy accounting
        q = min(1.0, (num_examples * epochs) / 100000)  # Sampling probability
        privacy_spent = epochs * q * self.epsilon
        return privacy_spent


class TensorFlowFederatedClient:
    """TensorFlow-based federated learning client"""
    
    def __init__(self, config: FederatedConfig):
        self.config = config
        self.model = None
        self.privacy_preserver = PrivacyPreserver(
            config.privacy_budget, 1e-5, config.clipping_bound
        )
        self.training_history = []
        
    def setup_model(self, model: tf.keras.Model) -> None:
        """Initialize the model for federated learning"""
        self.model = model
        # Create optimizer for local training
        self.optimizer = tf.keras.optimizers.Adam(learning_rate=self.config.learning_rate)
        # Loss function
        self.loss_fn = tf.keras.losses.CategoricalCrossentropy()
        
    def compute_gradient_hash(self, gradients: List[np.ndarray]) -> str:
        """Compute hash for gradient verification"""
        gradient_bytes = b''.join([g.tobytes() for g in gradients])
        return hashlib.sha256(gradient_bytes).hexdigest()
    
    def train_locally(
        self, 
        x_train: np.ndarray, 
        y_train: np.ndarray,
        validation_data: Optional[Tuple[np.ndarray, np.ndarray]] = None
    ) -> Tuple[List[np.ndarray], TrainingMetrics]:
        """Perform local training with privacy preservation"""
        if self.model is None:
            raise ValueError("Model not initialized. Call setup_model() first.")
        
        start_time = time.time()
        initial_weights = self.model.get_weights()
        
        # Local training loop
        for epoch in range(self.config.local_epochs):
            epoch_loss = 0.0
            num_batches = 0
            
            # Mini-batch training
            for i in range(0, len(x_train), self.config.batch_size):
                x_batch = x_train[i:i+self.config.batch_size]
                y_batch = y_train[i:i+self.config.batch_size]
                
                with tf.GradientTape() as tape:
                    predictions = self.model(x_batch, training=True)
                    loss = self.loss_fn(y_batch, predictions)
                
                gradients = tape.gradient(loss, self.model.trainable_variables)
                self.optimizer.apply_gradients(zip(gradients, self.model.trainable_variables))
                
                epoch_loss += loss.numpy()
                num_batches += 1
            
            avg_loss = epoch_loss / num_batches if num_batches > 0 else 0
            logger.info(f"Epoch {epoch+1}/{self.config.local_epochs}, Loss: {avg_loss:.4f}")
        
        # Compute gradients (difference from initial weights)
        final_weights = self.model.get_weights()
        gradients = [(final - initial) * self.config.learning_rate 
                     for final, initial in zip(final_weights, initial_weights)]
        
        # Apply privacy preservation
        clipped_gradients = self.privacy_preserver.clip_gradients(gradients)
        private_gradients = self.privacy_preserver.add_noise(clipped_gradients)
        
        # Compute metrics
        training_time = time.time() - start_time
        privacy_loss = self.privacy_preserver.compute_privacy_spent(
            len(x_train), self.config.local_epochs
        )
        
        # Evaluate on validation set if provided
        accuracy = 0.0
        convergence = 0.0
        if validation_data:
            x_val, y_val = validation_data
            val_predictions = self.model.predict(x_val)
            val_loss = self.loss_fn(y_val, val_predictions).numpy()
            accuracy = np.mean(np.argmax(val_predictions, axis=1) == np.argmax(y_val, axis=1))
            convergence = max(0, 100 - val_loss * 10)
        
        metrics = TrainingMetrics(
            loss=avg_loss,
            accuracy=accuracy * 100,  # Convert to percentage
            convergence=convergence,
            privacy_loss=privacy_loss,
            training_time=training_time,
            data_size=len(x_train)
        )
        
        return private_gradients, metrics
    
    def prepare_submission(
        self, 
        gradients: List[np.ndarray], 
        metrics: TrainingMetrics
    ) -> Dict[str, Any]:
        """Prepare federated learning submission"""
        gradient_hash = self.compute_gradient_hash(gradients)
        
        # Prepare quality metrics for smart contract
        quality_metrics = {
            "loss": int(metrics.loss * 100),  # Scale for smart contract
            "accuracy": int(metrics.accuracy),
            "convergence": int(metrics.convergence),
            "privacy_loss": int(metrics.privacy_loss * 100),
            "training_time": int(metrics.training_time),
            "data_size": metrics.data_size
        }
        
        # Privacy proof (simplified - in production would use ZK proofs)
        privacy_proof = hashlib.sha256(
            f"{gradient_hash}{self.config.institution_id}{time.time()}".encode()
        ).hexdigest()
        
        return {
            "gradient_hash": gradient_hash,
            "quality_metrics": quality_metrics,
            "privacy_proof": privacy_proof,
            "gradients": gradients,  # For actual transmission to coordinator
            "metrics": metrics
        }
    
    def update_model_from_aggregation(self, new_weights: List[np.ndarray]) -> None:
        """Update local model with aggregated weights"""
        if self.model is None:
            raise ValueError("Model not initialized")
        
        self.model.set_weights(new_weights)
        logger.info("Model updated with aggregated weights")


class FederatedLearningOrchestrator:
    """Orchestrates the federated learning process"""
    
    def __init__(self, client: TensorFlowFederatedClient):
        self.client = client
        self.current_round = None
        
    async def participate_in_round(
        self,
        round_id: int,
        x_train: np.ndarray,
        y_train: np.ndarray,
        validation_data: Optional[Tuple[np.ndarray, np.ndarray]] = None
    ) -> Dict[str, Any]:
        """Participate in a federated learning round"""
        logger.info(f"Starting participation in round {round_id}")
        
        # Train locally
        gradients, metrics = self.client.train_locally(x_train, y_train, validation_data)
        
        # Prepare submission
        submission = self.client.prepare_submission(gradients, metrics)
        
        # In a real implementation, this would interact with the blockchain
        # For now, we return the prepared submission
        logger.info(f"Round {round_id} training completed. Accuracy: {metrics.accuracy:.2f}%")
        
        return submission
    
    def receive_aggregated_weights(self, weights: List[np.ndarray]) -> None:
        """Receive and apply aggregated weights from coordinator"""
        self.client.update_model_from_aggregation(weights)
        logger.info("Received and applied aggregated weights")


# Example usage
def create_sample_model(input_shape: Tuple[int, ...], num_classes: int) -> tf.keras.Model:
    """Create a sample CNN model for federated learning"""
    model = tf.keras.Sequential([
        tf.keras.layers.Conv2D(32, (3, 3), activation='relu', input_shape=input_shape),
        tf.keras.layers.MaxPooling2D((2, 2)),
        tf.keras.layers.Conv2D(64, (3, 3), activation='relu'),
        tf.keras.layers.MaxPooling2D((2, 2)),
        tf.keras.layers.Flatten(),
        tf.keras.layers.Dense(128, activation='relu'),
        tf.keras.layers.Dense(num_classes, activation='softmax')
    ])
    
    return model


# Example federated learning workflow
async def example_federated_workflow():
    """Example of federated learning workflow"""
    
    # Configuration
    config = FederatedConfig(
        institution_id="hospital_001",
        coordinator_address="0x1234567890123456789012345678901234567890",
        privacy_budget=1.0,
        noise_scale=0.1,
        clipping_bound=1.0,
        local_epochs=5
    )
    
    # Create client
    client = TensorFlowFederatedClient(config)
    
    # Create and setup model
    model = create_sample_model((28, 28, 1), 10)  # MNIST example
    client.setup_model(model)
    
    # Create orchestrator
    orchestrator = FederatedLearningOrchestrator(client)
    
    # Generate sample data (in practice, this would be real medical data)
    x_train = np.random.random((1000, 28, 28, 1))
    y_train = tf.keras.utils.to_categorical(np.random.randint(0, 10, 1000), 10)
    
    # Participate in federated learning round
    submission = await orchestrator.participate_in_round(
        round_id=1,
        x_train=x_train,
        y_train=y_train
    )
    
    print(f"Submission prepared: {submission['gradient_hash']}")
    print(f"Training metrics: Accuracy={submission['metrics'].accuracy:.2f}%, "
          f"Loss={submission['metrics'].loss:.4f}")
    
    return submission


if __name__ == "__main__":
    import asyncio
    asyncio.run(example_federated_workflow())
