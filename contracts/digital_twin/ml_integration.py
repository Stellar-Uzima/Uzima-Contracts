"""
Machine Learning Integration for Digital Twin Platform
Integrates ML models with the digital twin system for enhanced predictions
"""

import numpy as np
import pandas as pd
import json
import logging
from typing import Dict, List, Optional, Any, Tuple, Union
from dataclasses import dataclass, asdict
from enum import Enum
from datetime import datetime, timedelta
import joblib
import pickle
from abc import ABC, abstractmethod

# ML libraries
import tensorflow as tf
from sklearn.ensemble import RandomForestClassifier, GradientBoostingRegressor
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score, mean_squared_error, classification_report
import xgboost as xgb
import lightgbm as lgb

# Deep learning
from tensorflow import keras
from tensorflow.keras import layers, models

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class MLFramework(Enum):
    """Supported ML frameworks"""
    TENSORFLOW = "tensorflow"
    PYTORCH = "pytorch"
    SCIKIT_LEARN = "scikit_learn"
    XGBOOST = "xgboost"
    LIGHTGBM = "lightgbm"


class ModelCategory(Enum):
    """Categories of ML models"""
    PREDICTIVE = "predictive"
    CLASSIFICATION = "classification"
    REGRESSION = "regression"
    TIME_SERIES = "time_series"
    ANOMALY_DETECTION = "anomaly_detection"
    CLUSTERING = "clustering"
    RECOMMENDATION = "recommendation"


@dataclass
class MLModelConfig:
    """Configuration for ML model"""
    model_id: str
    model_name: str
    framework: MLFramework
    category: ModelCategory
    target_variable: str
    input_features: List[str]
    hyperparameters: Dict[str, Any]
    training_config: Dict[str, Any]
    performance_threshold: float
    update_frequency_days: int
    version: str


@dataclass
class TrainingData:
    """Training data specification"""
    data_source: str
    time_range: Tuple[datetime, datetime]
    features: List[str]
    target: str
    preprocessing_steps: List[str]
    sample_size: int
    quality_score: float


@dataclass
class ModelPerformance:
    """Model performance metrics"""
    model_id: str
    accuracy: Optional[float]
    precision: Optional[float]
    recall: Optional[float]
    f1_score: Optional[float]
    mse: Optional[float]
    mae: Optional[float]
    r2_score: Optional[float]
    auc_roc: Optional[float]
    training_time: float
    inference_time: float
    last_updated: datetime


@dataclass
class PredictionRequest:
    """Request for model prediction"""
    model_id: str
    input_data: Dict[str, Any]
    patient_id: str
    timestamp: datetime
    context: Dict[str, Any]


@dataclass
class PredictionResponse:
    """Response from model prediction"""
    prediction_id: str
    model_id: str
    prediction: Any
    confidence: float
    explanation: Dict[str, Any]
    risk_assessment: str
    recommendations: List[str]
    processing_time: float
    timestamp: datetime


class BaseModelWrapper(ABC):
    """Abstract base class for ML model wrappers"""
    
    def __init__(self, config: MLModelConfig):
        self.config = config
        self.model = None
        self.scaler = None
        self.label_encoders = {}
        self.is_trained = False
        self.performance = None
        
    @abstractmethod
    def train(self, training_data: TrainingData) -> ModelPerformance:
        """Train the model"""
        pass
    
    @abstractmethod
    def predict(self, input_data: Dict[str, Any]) -> PredictionResponse:
        """Make prediction"""
        pass
    
    @abstractmethod
    def explain(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Explain prediction"""
        pass
    
    @abstractmethod
    def save_model(self, filepath: str) -> bool:
        """Save model to file"""
        pass
    
    @abstractmethod
    def load_model(self, filepath: str) -> bool:
        """Load model from file"""
        pass


class TensorFlowModelWrapper(BaseModelWrapper):
    """TensorFlow/Keras model wrapper"""
    
    def train(self, training_data: TrainingData) -> ModelPerformance:
        """Train TensorFlow model"""
        logger.info(f"Training TensorFlow model: {self.config.model_id}")
        
        # In a real implementation, this would load actual training data
        # For now, we'll create a sample model
        
        # Create model architecture
        self.model = self._create_model()
        
        # Generate sample training data
        X_train, y_train = self._generate_sample_data(training_data.sample_size)
        
        # Train model
        start_time = datetime.now()
        
        history = self.model.fit(
            X_train, y_train,
            epochs=self.config.training_config.get('epochs', 10),
            batch_size=self.config.training_config.get('batch_size', 32),
            validation_split=0.2,
            verbose=0
        )
        
        training_time = (datetime.now() - start_time).total_seconds()
        
        # Evaluate model
        loss, accuracy = self.model.evaluate(X_train, y_train, verbose=0)
        
        # Calculate inference time
        start_inference = datetime.now()
        self.model.predict(X_train[:1])
        inference_time = (datetime.now() - start_inference).total_seconds()
        
        # Create performance metrics
        performance = ModelPerformance(
            model_id=self.config.model_id,
            accuracy=accuracy,
            precision=None,  # Would need more detailed evaluation
            recall=None,
            f1_score=None,
            mse=loss,
            mae=None,
            r2_score=None,
            auc_roc=None,
            training_time=training_time,
            inference_time=inference_time,
            last_updated=datetime.now()
        )
        
        self.is_trained = True
        self.performance = performance
        
        logger.info(f"Model trained with accuracy: {accuracy:.3f}")
        return performance
    
    def predict(self, input_data: Dict[str, Any]) -> PredictionResponse:
        """Make prediction with TensorFlow model"""
        if not self.is_trained:
            raise ValueError("Model not trained")
        
        # Prepare input data
        processed_input = self._preprocess_input(input_data)
        
        # Make prediction
        start_time = datetime.now()
        prediction = self.model.predict(processed_input)
        prediction_time = (datetime.now() - start_time).total_seconds()
        
        # Extract prediction and confidence
        if self.config.category == ModelCategory.CLASSIFICATION:
            predicted_class = np.argmax(prediction[0])
            confidence = float(np.max(prediction[0]))
        else:
            predicted_value = float(prediction[0][0])
            confidence = 0.8  # Placeholder confidence
        
        # Generate explanation
        explanation = self.explain(input_data)
        
        # Risk assessment
        risk_assessment = self._assess_risk(predicted_value if 'predicted_value' in locals() else predicted_class, confidence)
        
        # Generate recommendations
        recommendations = self._generate_recommendations(predicted_value if 'predicted_value' in locals() else predicted_class)
        
        return PredictionResponse(
            prediction_id=f"pred_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
            model_id=self.config.model_id,
            prediction=predicted_value if 'predicted_value' in locals() else predicted_class,
            confidence=confidence,
            explanation=explanation,
            risk_assessment=risk_assessment,
            recommendations=recommendations,
            processing_time=prediction_time,
            timestamp=datetime.now()
        )
    
    def explain(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Explain prediction using SHAP or similar"""
        # Simplified explanation
        explanation = {
            "method": "feature_importance",
            "top_features": [
                {"feature": feature, "importance": np.random.random()}
                for feature in self.config.input_features[:5]
            ],
            "interpretation": "Feature importance based on model weights"
        }
        return explanation
    
    def save_model(self, filepath: str) -> bool:
        """Save TensorFlow model"""
        try:
            self.model.save(filepath)
            return True
        except Exception as e:
            logger.error(f"Failed to save model: {e}")
            return False
    
    def load_model(self, filepath: str) -> bool:
        """Load TensorFlow model"""
        try:
            self.model = keras.models.load_model(filepath)
            self.is_trained = True
            return True
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            return False
    
    def _create_model(self) -> keras.Model:
        """Create TensorFlow model architecture"""
        input_dim = len(self.config.input_features)
        
        if self.config.category == ModelCategory.CLASSIFICATION:
            # Classification model
            model = keras.Sequential([
                layers.Dense(64, activation='relu', input_shape=(input_dim,)),
                layers.Dropout(0.3),
                layers.Dense(32, activation='relu'),
                layers.Dropout(0.3),
                layers.Dense(16, activation='relu'),
                layers.Dense(1, activation='sigmoid')
            ])
            model.compile(optimizer='adam', loss='binary_crossentropy', metrics=['accuracy'])
        else:
            # Regression model
            model = keras.Sequential([
                layers.Dense(64, activation='relu', input_shape=(input_dim,)),
                layers.Dropout(0.3),
                layers.Dense(32, activation='relu'),
                layers.Dropout(0.3),
                layers.Dense(16, activation='relu'),
                layers.Dense(1)
            ])
            model.compile(optimizer='adam', loss='mse', metrics=['mae'])
        
        return model
    
    def _generate_sample_data(self, sample_size: int) -> Tuple[np.ndarray, np.ndarray]:
        """Generate sample training data"""
        X = np.random.randn(sample_size, len(self.config.input_features))
        
        if self.config.category == ModelCategory.CLASSIFICATION:
            y = np.random.randint(0, 2, sample_size)
        else:
            y = np.random.randn(sample_size)
        
        return X, y
    
    def _preprocess_input(self, input_data: Dict[str, Any]) -> np.ndarray:
        """Preprocess input data"""
        processed = []
        
        for feature in self.config.input_features:
            value = input_data.get(feature, 0.0)
            try:
                processed.append(float(value))
            except (ValueError, TypeError):
                processed.append(0.0)
        
        return np.array([processed])
    
    def _assess_risk(self, prediction: Any, confidence: float) -> str:
        """Assess risk level"""
        if confidence < 0.5:
            return "HIGH"
        elif confidence < 0.8:
            return "MEDIUM"
        else:
            return "LOW"
    
    def _generate_recommendations(self, prediction: Any) -> List[str]:
        """Generate recommendations based on prediction"""
        recommendations = [
            "Continue monitoring patient condition",
            "Consider additional diagnostic tests",
            "Review treatment plan"
        ]
        return recommendations


class ScikitLearnModelWrapper(BaseModelWrapper):
    """Scikit-learn model wrapper"""
    
    def train(self, training_data: TrainingData) -> ModelPerformance:
        """Train Scikit-learn model"""
        logger.info(f"Training Scikit-learn model: {self.config.model_id}")
        
        # Generate sample training data
        X_train, y_train = self._generate_sample_data(training_data.sample_size)
        
        # Create model based on configuration
        algorithm = self.config.hyperparameters.get('algorithm', 'random_forest')
        
        if algorithm == 'random_forest':
            if self.config.category == ModelCategory.CLASSIFICATION:
                self.model = RandomForestClassifier(
                    n_estimators=self.config.hyperparameters.get('n_estimators', 100),
                    max_depth=self.config.hyperparameters.get('max_depth', 10),
                    random_state=42
                )
            else:
                self.model = GradientBoostingRegressor(
                    n_estimators=self.config.hyperparameters.get('n_estimators', 100),
                    max_depth=self.config.hyperparameters.get('max_depth', 5),
                    random_state=42
                )
        else:
            # Default to random forest
            self.model = RandomForestClassifier(n_estimators=100, random_state=42)
        
        # Train model
        start_time = datetime.now()
        self.model.fit(X_train, y_train)
        training_time = (datetime.now() - start_time).total_seconds()
        
        # Evaluate model
        y_pred = self.model.predict(X_train)
        
        # Calculate inference time
        start_inference = datetime.now()
        self.model.predict(X_train[:1])
        inference_time = (datetime.now() - start_inference).total_seconds()
        
        # Calculate metrics
        if self.config.category == ModelCategory.CLASSIFICATION:
            accuracy = accuracy_score(y_train, y_pred)
            performance = ModelPerformance(
                model_id=self.config.model_id,
                accuracy=accuracy,
                precision=None,
                recall=None,
                f1_score=None,
                mse=None,
                mae=None,
                r2_score=None,
                auc_roc=None,
                training_time=training_time,
                inference_time=inference_time,
                last_updated=datetime.now()
            )
        else:
            mse = mean_squared_error(y_train, y_pred)
            performance = ModelPerformance(
                model_id=self.config.model_id,
                accuracy=None,
                precision=None,
                recall=None,
                f1_score=None,
                mse=mse,
                mae=None,
                r2_score=None,
                auc_roc=None,
                training_time=training_time,
                inference_time=inference_time,
                last_updated=datetime.now()
            )
        
        self.is_trained = True
        self.performance = performance
        
        logger.info(f"Model trained with accuracy: {performance.accuracy:.3f}")
        return performance
    
    def predict(self, input_data: Dict[str, Any]) -> PredictionResponse:
        """Make prediction with Scikit-learn model"""
        if not self.is_trained:
            raise ValueError("Model not trained")
        
        # Prepare input data
        processed_input = self._preprocess_input(input_data)
        
        # Make prediction
        start_time = datetime.now()
        prediction = self.model.predict(processed_input)[0]
        prediction_time = (datetime.now() - start_time).total_seconds()
        
        # Get confidence if available
        if hasattr(self.model, 'predict_proba'):
            probabilities = self.model.predict_proba(processed_input)[0]
            confidence = float(np.max(probabilities))
        else:
            confidence = 0.8  # Placeholder confidence
        
        # Generate explanation
        explanation = self.explain(input_data)
        
        # Risk assessment
        risk_assessment = self._assess_risk(prediction, confidence)
        
        # Generate recommendations
        recommendations = self._generate_recommendations(prediction)
        
        return PredictionResponse(
            prediction_id=f"pred_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
            model_id=self.config.model_id,
            prediction=prediction,
            confidence=confidence,
            explanation=explanation,
            risk_assessment=risk_assessment,
            recommendations=recommendations,
            processing_time=prediction_time,
            timestamp=datetime.now()
        )
    
    def explain(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Explain prediction using feature importance"""
        if hasattr(self.model, 'feature_importances_'):
            importance_scores = self.model.feature_importances_
            
            # Create feature importance mapping
            feature_importance = {
                feature: float(importance) 
                for feature, importance in zip(self.config.input_features, importance_scores)
            }
            
            # Sort by importance
            sorted_features = sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
            
            return {
                "method": "feature_importance",
                "top_features": sorted_features[:5],
                "feature_importance": feature_importance,
                "interpretation": "Higher values indicate greater importance in prediction"
            }
        
        return {"explanation": "Feature importance not available for this model"}
    
    def save_model(self, filepath: str) -> bool:
        """Save Scikit-learn model"""
        try:
            model_data = {
                'model': self.model,
                'config': self.config,
                'performance': self.performance,
                'is_trained': self.is_trained
            }
            
            with open(filepath, 'wb') as f:
                pickle.dump(model_data, f)
            
            return True
        except Exception as e:
            logger.error(f"Failed to save model: {e}")
            return False
    
    def load_model(self, filepath: str) -> bool:
        """Load Scikit-learn model"""
        try:
            with open(filepath, 'rb') as f:
                model_data = pickle.load(f)
            
            self.model = model_data['model']
            self.config = model_data['config']
            self.performance = model_data['performance']
            self.is_trained = model_data['is_trained']
            
            return True
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            return False
    
    def _generate_sample_data(self, sample_size: int) -> Tuple[np.ndarray, np.ndarray]:
        """Generate sample training data"""
        X = np.random.randn(sample_size, len(self.config.input_features))
        
        if self.config.category == ModelCategory.CLASSIFICATION:
            y = np.random.randint(0, 2, sample_size)
        else:
            y = np.random.randn(sample_size)
        
        return X, y
    
    def _preprocess_input(self, input_data: Dict[str, Any]) -> np.ndarray:
        """Preprocess input data"""
        processed = []
        
        for feature in self.config.input_features:
            value = input_data.get(feature, 0.0)
            try:
                processed.append(float(value))
            except (ValueError, TypeError):
                processed.append(0.0)
        
        return np.array([processed])
    
    def _assess_risk(self, prediction: Any, confidence: float) -> str:
        """Assess risk level"""
        if confidence < 0.5:
            return "HIGH"
        elif confidence < 0.8:
            return "MEDIUM"
        else:
            return "LOW"
    
    def _generate_recommendations(self, prediction: Any) -> List[str]:
        """Generate recommendations based on prediction"""
        recommendations = [
            "Continue monitoring patient condition",
            "Consider additional diagnostic tests",
            "Review treatment plan"
        ]
        return recommendations


class MLModelManager:
    """Manages ML models for digital twin platform"""
    
    def __init__(self):
        self.models: Dict[str, BaseModelWrapper] = {}
        self.model_configs: Dict[str, MLModelConfig] = {}
        self.training_data: Dict[str, TrainingData] = {}
        self.prediction_history: List[PredictionResponse] = []
        
    def register_model(self, config: MLModelConfig) -> bool:
        """Register a new ML model"""
        try:
            # Create appropriate model wrapper
            if config.framework == MLFramework.TENSORFLOW:
                wrapper = TensorFlowModelWrapper(config)
            elif config.framework == MLFramework.SCIKIT_LEARN:
                wrapper = ScikitLearnModelWrapper(config)
            else:
                logger.warning(f"Framework {config.framework} not yet supported, using Scikit-learn")
                wrapper = ScikitLearnModelWrapper(config)
            
            self.models[config.model_id] = wrapper
            self.model_configs[config.model_id] = config
            
            logger.info(f"Registered ML model: {config.model_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to register model {config.model_id}: {e}")
            return False
    
    def train_model(self, model_id: str, training_data: TrainingData) -> ModelPerformance:
        """Train a specific model"""
        if model_id not in self.models:
            raise ValueError(f"Model {model_id} not found")
        
        model = self.models[model_id]
        self.training_data[model_id] = training_data
        
        return model.train(training_data)
    
    def make_prediction(self, model_id: str, input_data: Dict[str, Any], 
                       patient_id: str) -> PredictionResponse:
        """Make prediction using specified model"""
        if model_id not in self.models:
            raise ValueError(f"Model {model_id} not found")
        
        model = self.models[model_id]
        
        if not model.is_trained:
            raise ValueError(f"Model {model_id} not trained")
        
        request = PredictionRequest(
            model_id=model_id,
            input_data=input_data,
            patient_id=patient_id,
            timestamp=datetime.now(),
            context={}
        )
        
        response = model.predict(input_data)
        self.prediction_history.append(response)
        
        return response
    
    def batch_predict(self, model_id: str, input_data_list: List[Dict[str, Any]], 
                     patient_id: str) -> List[PredictionResponse]:
        """Make batch predictions"""
        results = []
        for input_data in input_data_list:
            try:
                response = self.make_prediction(model_id, input_data, patient_id)
                results.append(response)
            except Exception as e:
                logger.error(f"Batch prediction failed: {e}")
        
        return results
    
    def get_model_performance(self, model_id: str) -> Optional[ModelPerformance]:
        """Get performance metrics for model"""
        if model_id not in self.models:
            return None
        
        return self.models[model_id].performance
    
    def retrain_model(self, model_id: str, new_training_data: TrainingData) -> ModelPerformance:
        """Retrain model with new data"""
        logger.info(f"Retraining model: {model_id}")
        
        # Update training data
        self.training_data[model_id] = new_training_data
        
        # Retrain model
        return self.train_model(model_id, new_training_data)
    
    def get_prediction_history(self, patient_id: Optional[str] = None, 
                              model_id: Optional[str] = None, 
                              limit: int = 100) -> List[PredictionResponse]:
        """Get prediction history with filters"""
        history = self.prediction_history
        
        # Apply filters
        if patient_id:
            history = [p for p in history if p.patient_id == patient_id]
        
        if model_id:
            history = [p for p in history if p.model_id == model_id]
        
        # Sort by timestamp and limit
        history.sort(key=lambda x: x.timestamp, reverse=True)
        
        return history[:limit]
    
    def evaluate_model_ensemble(self, model_ids: List[str], test_data: Dict[str, Any]) -> Dict[str, float]:
        """Evaluate ensemble of models"""
        results = {}
        
        for model_id in model_ids:
            if model_id in self.models and self.models[model_id].is_trained:
                try:
                    response = self.make_prediction(model_id, test_data, "test_patient")
                    results[model_id] = response.confidence
                except Exception as e:
                    logger.error(f"Failed to evaluate model {model_id}: {e}")
                    results[model_id] = 0.0
        
        return results
    
    def auto_retrain_if_needed(self, model_id: str, performance_threshold: float = 0.8) -> bool:
        """Automatically retrain model if performance drops below threshold"""
        if model_id not in self.models:
            return False
        
        performance = self.get_model_performance(model_id)
        if not performance:
            return False
        
        # Check if performance is below threshold
        if performance.accuracy and performance.accuracy < performance_threshold:
            logger.info(f"Model {model_id} performance ({performance.accuracy:.3f}) below threshold, retraining...")
            
            # Get training data
            training_data = self.training_data.get(model_id)
            if training_data:
                try:
                    self.retrain_model(model_id, training_data)
                    return True
                except Exception as e:
                    logger.error(f"Failed to retrain model {model_id}: {e}")
        
        return False
    
    def get_model_statistics(self) -> Dict[str, Any]:
        """Get statistics for all models"""
        stats = {
            'total_models': len(self.models),
            'trained_models': sum(1 for model in self.models.values() if model.is_trained),
            'models_by_framework': {},
            'models_by_category': {},
            'average_accuracy': 0.0,
            'total_predictions': len(self.prediction_history)
        }
        
        # Count by framework
        for config in self.model_configs.values():
            framework = config.framework.value
            stats['models_by_framework'][framework] = stats['models_by_framework'].get(framework, 0) + 1
        
        # Count by category
        for config in self.model_configs.values():
            category = config.category.value
            stats['models_by_category'][category] = stats['models_by_category'].get(category, 0) + 1
        
        # Calculate average accuracy
        accuracies = []
        for model in self.models.values():
            if model.performance and model.performance.accuracy:
                accuracies.append(model.performance.accuracy)
        
        if accuracies:
            stats['average_accuracy'] = sum(accuracies) / len(accuracies)
        
        return stats


# Example usage
def example_ml_integration():
    """Example of ML integration with digital twin"""
    
    # Create ML model manager
    manager = MLModelManager()
    
    # Create model configurations
    tensorflow_config = MLModelConfig(
        model_id="cardiovascular_risk_tf",
        model_name="Cardiovascular Risk Predictor",
        framework=MLFramework.TENSORFLOW,
        category=ModelCategory.CLASSIFICATION,
        target_variable="cardiovascular_event",
        input_features=["age", "bmi", "blood_pressure", "cholesterol", "smoking_status", "diabetes"],
        hyperparameters={"epochs": 20, "batch_size": 32},
        training_config={"validation_split": 0.2},
        performance_threshold=0.85,
        update_frequency_days=30,
        version="v1.0"
    )
    
    sklearn_config = MLModelConfig(
        model_id="treatment_response_sk",
        model_name="Treatment Response Predictor",
        framework=MLFramework.SCIKIT_LEARN,
        category=ModelCategory.REGRESSION,
        target_variable="treatment_effectiveness",
        input_features=["age", "severity", "comorbidities", "adherence", "genetic_markers"],
        hyperparameters={"algorithm": "random_forest", "n_estimators": 100},
        training_config={"test_size": 0.2},
        performance_threshold=0.80,
        update_frequency_days=14,
        version="v1.0"
    )
    
    # Register models
    manager.register_model(tensorflow_config)
    manager.register_model(sklearn_config)
    
    print(f"Registered {len(manager.models)} ML models")
    
    # Create training data
    training_data = TrainingData(
        data_source="digital_twin",
        time_range=(datetime.now() - timedelta(days=365), datetime.now()),
        features=["age", "bmi", "blood_pressure", "cholesterol", "smoking_status", "diabetes"],
        target="cardiovascular_event",
        preprocessing_steps=["normalization", "encoding"],
        sample_size=1000,
        quality_score=0.95
    )
    
    # Train models
    try:
        tf_performance = manager.train_model("cardiovascular_risk_tf", training_data)
        print(f"TensorFlow model trained with accuracy: {tf_performance.accuracy:.3f}")
        
        sklearn_performance = manager.train_model("treatment_response_sk", training_data)
        print(f"Scikit-learn model trained with MSE: {sklearn_performance.mse:.3f}")
        
    except Exception as e:
        print(f"Training failed: {e}")
    
    # Make predictions
    sample_input = {
        'age': 55,
        'bmi': 28.5,
        'blood_pressure': 135,
        'cholesterol': 220,
        'smoking_status': 1,
        'diabetes': 0
    }
    
    try:
        prediction = manager.make_prediction("cardiovascular_risk_tf", sample_input, "patient_001")
        print(f"Prediction: {prediction.prediction} with confidence: {prediction.confidence:.2f}")
        print(f"Risk assessment: {prediction.risk_assessment}")
        print(f"Recommendations: {prediction.recommendations}")
        
    except Exception as e:
        print(f"Prediction failed: {e}")
    
    # Get model statistics
    stats = manager.get_model_statistics()
    print(f"Model Statistics: {stats}")


if __name__ == "__main__":
    example_ml_integration()
