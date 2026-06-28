"""
Federated Learning Coordinator
Aggregates model updates from multiple institutions with privacy preservation
"""

import numpy as np
from typing import Dict, List, Optional, Tuple, Any
import hashlib
import time
import logging
from dataclasses import dataclass
from enum import Enum
import json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class AggregationMethod(Enum):
    """Methods for aggregating federated updates"""
    FED_AVG = "fed_avg"
    FED_PROX = "fed_prox"
    SECURE_AGG = "secure_agg"
    KRUM = "krum"
    MULTI_KRUM = "multi_krum"
    TRIMMED_MEAN = "trimmed_mean"


@dataclass
class AggregationConfig:
    """Configuration for model aggregation"""
    method: AggregationMethod
    num_byzantine: int = 0  # For Byzantine-robust methods
    trim_fraction: float = 0.1  # For trimmed mean
    mu: float = 0.01  # For FedProx regularization
    min_contributors: int = 2
    max_contributors: int = 100


@dataclass
class AggregatedResult:
    """Result of federated aggregation"""
    aggregated_weights: Dict[str, np.ndarray]
    contributor_weights: List[str]  # Hashes of contributing gradients
    aggregation_score: float
    privacy_guarantee: float
    robustness_score: float
    communication_cost: int


class AttackDetector:
    """Detects potential poisoning attacks in federated learning"""
    
    def __init__(self, threshold: float = 0.3):
        self.threshold = threshold
        self.history = []
    
    def detect_anomalies(
        self, 
        updates: Dict[str, Dict[str, np.ndarray]],
        base_model: Optional[Dict[str, np.ndarray]] = None
    ) -> Dict[str, float]:
        """Detect anomalous updates that might indicate attacks"""
        anomaly_scores = {}
        
        if not updates:
            return anomaly_scores
        
        # Compute similarity scores between updates
        update_list = list(updates.values())
        num_updates = len(update_list)
        
        for contributor_id, update in updates.items():
            score = 0.0
            comparisons = 0
            
            # Compare with other updates
            for other_id, other_update in updates.items():
                if contributor_id != other_id:
                    similarity = self._compute_update_similarity(update, other_update)
                    score += similarity
                    comparisons += 1
            
            # Average similarity score
            avg_similarity = score / comparisons if comparisons > 0 else 0
            
            # Compare with base model if available
            if base_model:
                base_similarity = self._compute_base_similarity(update, base_model)
                avg_similarity = (avg_similarity + base_similarity) / 2
            
            # Anomaly score is inverse of similarity
            anomaly_score = 1.0 - avg_similarity
            anomaly_scores[contributor_id] = anomaly_score
        
        return anomaly_scores
    
    def _compute_update_similarity(
        self, 
        update1: Dict[str, np.ndarray], 
        update2: Dict[str, np.ndarray]
    ) -> float:
        """Compute similarity between two model updates"""
        if not update1 or not update2:
            return 0.0
        
        similarities = []
        for key in update1:
            if key in update2:
                # Cosine similarity
                flat1 = update1[key].flatten()
                flat2 = update2[key].flatten()
                
                if np.linalg.norm(flat1) > 0 and np.linalg.norm(flat2) > 0:
                    similarity = np.dot(flat1, flat2) / (np.linalg.norm(flat1) * np.linalg.norm(flat2))
                    similarities.append(similarity)
        
        return np.mean(similarities) if similarities else 0.0
    
    def _compute_base_similarity(
        self, 
        update: Dict[str, np.ndarray], 
        base_model: Dict[str, np.ndarray]
    ) -> float:
        """Compute similarity of update with base model direction"""
        if not update or not base_model:
            return 0.0
        
        # For simplicity, compute magnitude similarity
        update_norm = sum(np.linalg.norm(param.flatten()) for param in update.values())
        base_norm = sum(np.linalg.norm(param.flatten()) for param in base_model.values())
        
        if base_norm == 0:
            return 0.0
        
        # Similarity based on relative magnitude
        relative_magnitude = min(update_norm / base_norm, 1.0)
        return relative_magnitude
    
    def is_attack_detected(self, anomaly_scores: Dict[str, float]) -> bool:
        """Determine if an attack is detected based on anomaly scores"""
        if not anomaly_scores:
            return False
        
        max_anomaly = max(anomaly_scores.values())
        return max_anomaly > self.threshold


class FederatedAggregator:
    """Aggregates federated learning updates with various methods"""
    
    def __init__(self, config: AggregationConfig):
        self.config = config
        self.attack_detector = AttackDetector()
        self.aggregation_history = []
    
    def aggregate_updates(
        self,
        updates: Dict[str, Dict[str, np.ndarray]],
        base_model: Optional[Dict[str, np.ndarray]] = None,
        contributor_reputations: Optional[Dict[str, float]] = None
    ) -> AggregatedResult:
        """Aggregate model updates from multiple contributors"""
        logger.info(f"Aggregating {len(updates)} updates using {self.config.method.value}")
        
        if not updates:
            raise ValueError("No updates to aggregate")
        
        if len(updates) < self.config.min_contributors:
            raise ValueError(f"Insufficient contributors: {len(updates)} < {self.config.min_contributors}")
        
        # Detect potential attacks
        anomaly_scores = self.attack_detector.detect_anomalies(updates, base_model)
        attack_detected = self.attack_detector.is_attack_detected(anomaly_scores)
        
        if attack_detected:
            logger.warning(f"Potential attack detected! Anomaly scores: {anomaly_scores}")
            # Filter out high-anomaly updates
            filtered_updates = {
                contributor_id: update 
                for contributor_id, update in updates.items()
                if anomaly_scores.get(contributor_id, 0) <= self.attack_detector.threshold
            }
            
            if len(filtered_updates) < self.config.min_contributors:
                raise ValueError("Too many anomalous updates - cannot aggregate safely")
            
            updates = filtered_updates
            logger.info(f"Filtered to {len(updates)} updates after anomaly detection")
        
        # Apply aggregation method
        if self.config.method == AggregationMethod.FED_AVG:
            aggregated_weights = self._federated_averaging(updates, contributor_reputations)
        elif self.config.method == AggregationMethod.FED_PROX:
            aggregated_weights = self._fed_prox(updates, base_model, contributor_reputations)
        elif self.config.method == AggregationMethod.KRUM:
            aggregated_weights = self._krum_aggregation(updates)
        elif self.config.method == AggregationMethod.MULTI_KRUM:
            aggregated_weights = self._multi_krum_aggregation(updates)
        elif self.config.method == AggregationMethod.TRIMMED_MEAN:
            aggregated_weights = self._trimmed_mean_aggregation(updates)
        else:
            raise ValueError(f"Unsupported aggregation method: {self.config.method}")
        
        # Compute metrics
        aggregation_score = self._compute_aggregation_score(updates, aggregated_weights)
        privacy_guarantee = self._compute_privacy_guarantee(len(updates))
        robustness_score = 1.0 - max(anomaly_scores.values()) if anomaly_scores else 1.0
        communication_cost = self._estimate_communication_cost(updates)
        
        result = AggregatedResult(
            aggregated_weights=aggregated_weights,
            contributor_weights=list(updates.keys()),
            aggregation_score=aggregation_score,
            privacy_guarantee=privacy_guarantee,
            robustness_score=robustness_score,
            communication_cost=communication_cost
        )
        
        self.aggregation_history.append(result)
        logger.info(f"Aggregation completed. Score: {aggregation_score:.3f}, "
                   f"Privacy: {privacy_guarantee:.3f}, Robustness: {robustness_score:.3f}")
        
        return result
    
    def _federated_averaging(
        self, 
        updates: Dict[str, Dict[str, np.ndarray]],
        contributor_reputations: Optional[Dict[str, float]] = None
    ) -> Dict[str, np.ndarray]:
        """Standard federated averaging with optional reputation weighting"""
        if not updates:
            return {}
        
        # Get first update to determine structure
        first_update = next(iter(updates.values()))
        aggregated = {}
        
        for key in first_update.keys():
            weighted_sum = None
            total_weight = 0.0
            
            for contributor_id, update in updates.items():
                if key in update:
                    weight = 1.0
                    if contributor_reputations:
                        weight = contributor_reputations.get(contributor_id, 1.0)
                    
                    if weighted_sum is None:
                        weighted_sum = update[key] * weight
                    else:
                        weighted_sum += update[key] * weight
                    
                    total_weight += weight
            
            aggregated[key] = weighted_sum / total_weight if total_weight > 0 else weighted_sum
        
        return aggregated
    
    def _fed_prox(
        self,
        updates: Dict[str, Dict[str, np.ndarray]],
        base_model: Optional[Dict[str, np.ndarray]],
        contributor_reputations: Optional[Dict[str, float]] = None
    ) -> Dict[str, np.ndarray]:
        """FedProx aggregation with proximal term"""
        if not base_model:
            # Fall back to standard FedAvg if no base model
            return self._federated_averaging(updates, contributor_reputations)
        
        # Apply proximal term to each update
        prox_updates = {}
        for contributor_id, update in updates.items():
            prox_update = {}
            for key in update:
                if key in base_model:
                    # Add proximal term: μ * (w - w0)
                    prox_update[key] = update[key] + self.config.mu * (base_model[key] - update[key])
                else:
                    prox_update[key] = update[key]
            prox_updates[contributor_id] = prox_update
        
        return self._federated_averaging(prox_updates, contributor_reputations)
    
    def _krum_aggregation(self, updates: Dict[str, Dict[str, np.ndarray]]) -> Dict[str, np.ndarray]:
        """Krum aggregation for Byzantine robustness"""
        if len(updates) <= self.config.num_byzantine:
            raise ValueError("Insufficient updates for Krum aggregation")
        
        # Flatten all updates for distance computation
        flattened_updates = []
        for contributor_id, update in updates.items():
            flattened = np.concatenate([param.flatten() for param in update.values()])
            flattened_updates.append((contributor_id, flattened))
        
        # Compute pairwise distances
        num_updates = len(flattened_updates)
        distances = np.zeros((num_updates, num_updates))
        
        for i in range(num_updates):
            for j in range(i + 1, num_updates):
                dist = np.linalg.norm(flattened_updates[i][1] - flattened_updates[j][1])
                distances[i][j] = dist
                distances[j][i] = dist
        
        # Compute scores (sum of closest distances)
        num_closest = num_updates - self.config.num_byzantine - 2
        scores = []
        
        for i in range(num_updates):
            closest_distances = np.sort(distances[i])[:num_closest]
            score = np.sum(closest_distances)
            scores.append((flattened_updates[i][0], score))
        
        # Select update with minimum score
        best_contributor = min(scores, key=lambda x: x[1])[0]
        return updates[best_contributor]
    
    def _multi_krum_aggregation(self, updates: Dict[str, Dict[str, np.ndarray]]) -> Dict[str, np.ndarray]:
        """Multi-Krum aggregation (average of multiple Krum updates)"""
        num_byzantine = self.config.num_byzantine
        num_selected = len(updates) - num_byzantine - 2
        
        if num_selected <= 0:
            raise ValueError("Insufficient updates for Multi-Krum")
        
        # Use Krum to find the best updates
        krum_result = self._krum_aggregation(updates)
        
        # For simplicity, return the Krum result
        # In practice, Multi-Krum would average multiple good updates
        return krum_result
    
    def _trimmed_mean_aggregation(self, updates: Dict[str, Dict[str, np.ndarray]]) -> Dict[str, np.ndarray]:
        """Trimmed mean aggregation for robustness"""
        if not updates:
            return {}
        
        first_update = next(iter(updates.values()))
        aggregated = {}
        
        for key in first_update.keys():
            # Collect all parameter values for this layer
            param_values = []
            for update in updates.values():
                if key in update:
                    param_values.append(update[key].flatten())
            
            if not param_values:
                continue
            
            # Stack and compute trimmed mean
            stacked_params = np.vstack(param_values)
            num_params = stacked_params.shape[0]
            
            # Number to trim from each end
            trim_count = int(num_params * self.config.trim_fraction)
            
            if trim_count > 0:
                # Sort and trim
                sorted_params = np.sort(stacked_params, axis=0)
                trimmed_params = sorted_params[trim_count:num_params-trim_count]
                mean_params = np.mean(trimmed_params, axis=0)
            else:
                mean_params = np.mean(stacked_params, axis=0)
            
            # Reshape back to original shape
            original_shape = first_update[key].shape
            aggregated[key] = mean_params.reshape(original_shape)
        
        return aggregated
    
    def _compute_aggregation_score(
        self, 
        updates: Dict[str, Dict[str, np.ndarray]], 
        aggregated: Dict[str, np.ndarray]
    ) -> float:
        """Compute aggregation quality score"""
        if not updates or not aggregated:
            return 0.0
        
        similarities = []
        for update in updates.values():
            similarity = self._compute_model_similarity(update, aggregated)
            similarities.append(similarity)
        
        return np.mean(similarities) if similarities else 0.0
    
    def _compute_model_similarity(
        self, 
        model1: Dict[str, np.ndarray], 
        model2: Dict[str, np.ndarray]
    ) -> float:
        """Compute similarity between two models"""
        similarities = []
        for key in model1:
            if key in model2:
                flat1 = model1[key].flatten()
                flat2 = model2[key].flatten()
                
                if np.linalg.norm(flat1) > 0 and np.linalg.norm(flat2) > 0:
                    similarity = np.dot(flat1, flat2) / (np.linalg.norm(flat1) * np.linalg.norm(flat2))
                    similarities.append(similarity)
        
        return np.mean(similarities) if similarities else 0.0
    
    def _compute_privacy_guarantee(self, num_contributors: int) -> float:
        """Compute privacy guarantee based on number of contributors"""
        # Simple model: privacy improves with more contributors
        return min(1.0, num_contributors / 50.0)
    
    def _estimate_communication_cost(self, updates: Dict[str, Dict[str, np.ndarray]]) -> int:
        """Estimate communication cost in bytes"""
        total_bytes = 0
        for update in updates.values():
            for param in update.values():
                total_bytes += param.nbytes
        
        return total_bytes


# Example usage
async def example_aggregation():
    """Example of federated aggregation"""
    
    # Configuration
    config = AggregationConfig(
        method=AggregationMethod.FED_AVG,
        min_contributors=2,
        max_contributors=50
    )
    
    # Create aggregator
    aggregator = FederatedAggregator(config)
    
    # Simulate updates from different institutions
    updates = {}
    for i in range(3):
        update = {}
        for layer_name in ['conv1', 'conv2', 'fc1', 'fc2']:
            # Random gradient-like data
            update[layer_name] = np.random.randn(32, 3, 3) if 'conv' in layer_name else np.random.randn(128)
        
        contributor_id = f"hospital_{i+1:03d}"
        updates[contributor_id] = update
    
    # Add reputation scores
    reputations = {
        "hospital_001": 0.9,
        "hospital_002": 0.7,
        "hospital_003": 0.8
    }
    
    # Aggregate updates
    result = aggregator.aggregate_updates(updates, contributor_reputations=reputations)
    
    print(f"Aggregation completed:")
    print(f"  Contributors: {len(result.contributor_weights)}")
    print(f"  Aggregation score: {result.aggregation_score:.3f}")
    print(f"  Privacy guarantee: {result.privacy_guarantee:.3f}")
    print(f"  Robustness score: {result.robustness_score:.3f}")
    print(f"  Communication cost: {result.communication_cost} bytes")
    
    return result


if __name__ == "__main__":
    import asyncio
    asyncio.run(example_aggregation())
