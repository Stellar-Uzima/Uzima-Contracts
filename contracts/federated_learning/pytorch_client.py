"""
Federated Learning Client for PyTorch Integration
Supports secure federated learning with privacy preservation
"""

import torch
import torch.nn as nn
import torch.optim as optim
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
    framework: str = "pytorch"


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
    """Implements differential privacy mechanisms for PyTorch"""
    
    def __init__(self, epsilon: float, delta: float, clipping_bound: float):
        self.epsilon = epsilon
        self.delta = delta
        self.clipping_bound = clipping_bound
        self.noise_scale = clipping_bound * epsilon
    
    def clip_gradients(self, model: nn.Module) -> float:
        """Clip gradients to bound sensitivity and return clipping norm"""
        total_norm = 0.0
        for p in model.parameters():
            if p.grad is not None:
                param_norm = p.grad.data.norm(2)
                total_norm += param_norm.item() ** 2
        total_norm = total_norm ** (1. / 2)
        
        clip_coef = self.clipping_bound / (total_norm + 1e-8)
        if clip_coef < 1:
            for p in model.parameters():
                if p.grad is not None:
                    p.grad.data.mul_(clip_coef)
        
        return min(total_norm, self.clipping_bound)
    
    def add_noise(self, model: nn.Module) -> None:
        """Add Gaussian noise to gradients for differential privacy"""
        with torch.no_grad():
            for p in model.parameters():
                if p.grad is not None:
                    noise = torch.randn_like(p.grad) * self.noise_scale
                    p.grad.data.add_(noise)
    
    def compute_privacy_spent(self, num_examples: int, epochs: int) -> float:
        """Compute privacy loss using moments accountant"""
        # Simplified privacy accounting
        q = min(1.0, (num_examples * epochs) / 100000)  # Sampling probability
        privacy_spent = epochs * q * self.epsilon
        return privacy_spent


class PyTorchFederatedClient:
    """PyTorch-based federated learning client"""
    
    def __init__(self, config: FederatedConfig):
        self.config = config
        self.model = None
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.privacy_preserver = PrivacyPreserver(
            config.privacy_budget, 1e-5, config.clipping_bound
        )
        self.training_history = []
        
    def setup_model(self, model: nn.Module) -> None:
        """Initialize the model for federated learning"""
        self.model = model.to(self.device)
        # Create optimizer for local training
        self.optimizer = optim.Adam(self.model.parameters(), lr=self.config.learning_rate)
        # Loss function
        self.loss_fn = nn.CrossEntropyLoss()
        
    def compute_gradient_hash(self, state_dict: Dict[str, torch.Tensor]) -> str:
        """Compute hash for gradient verification"""
        gradient_bytes = b''.join([
            param.cpu().numpy().tobytes() 
            for param in state_dict.values()
        ])
        return hashlib.sha256(gradient_bytes).hexdigest()
    
    def train_locally(
        self, 
        train_loader: torch.utils.data.DataLoader,
        validation_loader: Optional[torch.utils.data.DataLoader] = None
    ) -> Tuple[Dict[str, torch.Tensor], TrainingMetrics]:
        """Perform local training with privacy preservation"""
        if self.model is None:
            raise ValueError("Model not initialized. Call setup_model() first.")
        
        start_time = time.time()
        initial_state_dict = {k: v.clone() for k, v in self.model.state_dict().items()}
        
        self.model.train()
        total_loss = 0.0
        num_batches = 0
        
        # Local training loop
        for epoch in range(self.config.local_epochs):
            epoch_loss = 0.0
            epoch_batches = 0
            
            for batch_idx, (data, target) in enumerate(train_loader):
                data, target = data.to(self.device), target.to(self.device)
                
                self.optimizer.zero_grad()
                output = self.model(data)
                loss = self.loss_fn(output, target)
                loss.backward()
                
                # Apply privacy preservation
                clipping_norm = self.privacy_preserver.clip_gradients(self.model)
                self.privacy_preserver.add_noise(self.model)
                
                self.optimizer.step()
                
                epoch_loss += loss.item()
                epoch_batches += 1
                
                if batch_idx % 100 == 0:
                    logger.info(f"Epoch {epoch+1}/{self.config.local_epochs}, "
                              f"Batch {batch_idx}, Loss: {loss.item():.4f}")
            
            avg_epoch_loss = epoch_loss / epoch_batches if epoch_batches > 0 else 0
            total_loss += avg_epoch_loss
            num_batches += 1
            
            logger.info(f"Epoch {epoch+1}/{self.config.local_epochs}, "
                       f"Avg Loss: {avg_epoch_loss:.4f}")
        
        # Compute gradients (difference from initial weights)
        final_state_dict = self.model.state_dict()
        gradients = {}
        for key in final_state_dict:
            gradients[key] = (final_state_dict[key] - initial_state_dict[key]) * self.config.learning_rate
        
        # Compute metrics
        training_time = time.time() - start_time
        privacy_loss = self.privacy_preserver.compute_privacy_spent(
            len(train_loader.dataset), self.config.local_epochs
        )
        
        # Evaluate on validation set if provided
        accuracy = 0.0
        convergence = 0.0
        if validation_loader:
            accuracy, val_loss = self.evaluate(validation_loader)
            convergence = max(0, 100 - val_loss * 10)
        
        avg_loss = total_loss / num_batches if num_batches > 0 else 0
        
        metrics = TrainingMetrics(
            loss=avg_loss,
            accuracy=accuracy * 100,  # Convert to percentage
            convergence=convergence,
            privacy_loss=privacy_loss,
            training_time=training_time,
            data_size=len(train_loader.dataset)
        )
        
        return gradients, metrics
    
    def evaluate(self, data_loader: torch.utils.data.DataLoader) -> Tuple[float, float]:
        """Evaluate model on given data loader"""
        self.model.eval()
        total_loss = 0.0
        correct = 0
        total = 0
        
        with torch.no_grad():
            for data, target in data_loader:
                data, target = data.to(self.device), target.to(self.device)
                output = self.model(data)
                loss = self.loss_fn(output, target)
                
                total_loss += loss.item()
                pred = output.argmax(dim=1, keepdim=True)
                correct += pred.eq(target.view_as(pred)).sum().item()
                total += target.size(0)
        
        avg_loss = total_loss / len(data_loader)
        accuracy = correct / total if total > 0 else 0
        
        return accuracy, avg_loss
    
    def prepare_submission(
        self, 
        gradients: Dict[str, torch.Tensor], 
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
    
    def update_model_from_aggregation(self, new_state_dict: Dict[str, torch.Tensor]) -> None:
        """Update local model with aggregated weights"""
        if self.model is None:
            raise ValueError("Model not initialized")
        
        self.model.load_state_dict(new_state_dict)
        logger.info("Model updated with aggregated weights")


class FederatedLearningOrchestrator:
    """Orchestrates the federated learning process"""
    
    def __init__(self, client: PyTorchFederatedClient):
        self.client = client
        self.current_round = None
        
    async def participate_in_round(
        self,
        round_id: int,
        train_loader: torch.utils.data.DataLoader,
        validation_loader: Optional[torch.utils.data.DataLoader] = None
    ) -> Dict[str, Any]:
        """Participate in a federated learning round"""
        logger.info(f"Starting participation in round {round_id}")
        
        # Train locally
        gradients, metrics = self.client.train_locally(train_loader, validation_loader)
        
        # Prepare submission
        submission = self.client.prepare_submission(gradients, metrics)
        
        # In a real implementation, this would interact with the blockchain
        # For now, we return the prepared submission
        logger.info(f"Round {round_id} training completed. Accuracy: {metrics.accuracy:.2f}%")
        
        return submission
    
    def receive_aggregated_weights(self, state_dict: Dict[str, torch.Tensor]) -> None:
        """Receive and apply aggregated weights from coordinator"""
        self.client.update_model_from_aggregation(state_dict)
        logger.info("Received and applied aggregated weights")


# Example model definitions
class CNNModel(nn.Module):
    """CNN model for image classification"""
    
    def __init__(self, input_channels: int = 1, num_classes: int = 10):
        super(CNNModel, self).__init__()
        self.conv1 = nn.Conv2d(input_channels, 32, 3, 1)
        self.conv2 = nn.Conv2d(32, 64, 3, 1)
        self.dropout1 = nn.Dropout(0.25)
        self.dropout2 = nn.Dropout(0.5)
        self.fc1 = nn.Linear(9216, 128)
        self.fc2 = nn.Linear(128, num_classes)

    def forward(self, x):
        x = self.conv1(x)
        x = torch.relu(x)
        x = self.conv2(x)
        x = torch.relu(x)
        x = torch.max_pool2d(x, 2)
        x = self.dropout1(x)
        x = torch.flatten(x, 1)
        x = self.fc1(x)
        x = torch.relu(x)
        x = self.dropout2(x)
        x = self.fc2(x)
        output = torch.log_softmax(x, dim=1)
        return output


class LSTMModel(nn.Module):
    """LSTM model for sequence classification"""
    
    def __init__(self, input_size: int, hidden_size: int = 128, num_layers: int = 2, num_classes: int = 10):
        super(LSTMModel, self).__init__()
        self.hidden_size = hidden_size
        self.num_layers = num_layers
        self.lstm = nn.LSTM(input_size, hidden_size, num_layers, batch_first=True)
        self.fc = nn.Linear(hidden_size, num_classes)
        
    def forward(self, x):
        # Initialize hidden state
        h0 = torch.zeros(self.num_layers, x.size(0), self.hidden_size).to(x.device)
        c0 = torch.zeros(self.num_layers, x.size(0), self.hidden_size).to(x.device)
        
        # Forward propagate LSTM
        out, _ = self.lstm(x, (h0, c0))
        
        # Decode the hidden state of the last time step
        out = self.fc(out[:, -1, :])
        return out


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
    client = PyTorchFederatedClient(config)
    
    # Create and setup model
    model = CNNModel(input_channels=1, num_classes=10)  # MNIST example
    client.setup_model(model)
    
    # Create orchestrator
    orchestrator = FederatedLearningOrchestrator(client)
    
    # Generate sample data (in practice, this would be real medical data)
    # Create dummy dataset
    class DummyDataset(torch.utils.data.Dataset):
        def __init__(self, size=1000):
            self.size = size
            
        def __len__(self):
            return self.size
            
        def __getitem__(self, idx):
            # Generate random image-like data (1, 28, 28)
            data = torch.randn(1, 28, 28)
            # Random label
            label = torch.randint(0, 10, (1,)).item()
            return data, label
    
    dataset = DummyDataset(1000)
    train_loader = torch.utils.data.DataLoader(dataset, batch_size=config.batch_size, shuffle=True)
    
    # Participate in federated learning round
    submission = await orchestrator.participate_in_round(
        round_id=1,
        train_loader=train_loader
    )
    
    print(f"Submission prepared: {submission['gradient_hash']}")
    print(f"Training metrics: Accuracy={submission['metrics'].accuracy:.2f}%, "
          f"Loss={submission['metrics'].loss:.4f}")
    
    return submission


if __name__ == "__main__":
    import asyncio
    asyncio.run(example_federated_workflow())
