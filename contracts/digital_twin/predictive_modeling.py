"""
Predictive Modeling Engine for Digital Twin Platform
Supports various ML models for prediction, risk assessment, and treatment response
"""

import numpy as np
import pandas as pd
import json
import logging
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
from datetime import datetime, timedelta
import joblib
import pickle
from abc import ABC, abstractmethod

# ML libraries
from sklearn.ensemble import RandomForestClassifier, GradientBoostingRegressor
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.metrics import accuracy_score, mean_squared_error, roc_auc_score
import xgboost as xgb
import lightgbm as lgb

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class ModelType(Enum):
    """Types of predictive models"""
    RISK_ASSESSMENT = "risk_assessment"
    DISEASE_PROGRESSION = "disease_progression"
    TREATMENT_RESPONSE = "treatment_response"
    WELLNESS_PREDICTION = "wellness_prediction"
    READMISSION_RISK = "readmission_risk"
    MEDICATION_OPTIMIZATION = "medication_optimization"
    LIFESTYLE_RECOMMENDATION = "lifestyle_recommendation"


class PredictionHorizon(Enum):
    """Prediction time horizons"""
    IMMEDIATE = "immediate"  # < 24 hours
    SHORT_TERM = "short_term"  # 1-7 days
    MEDIUM_TERM = "medium_term"  # 1-4 weeks
    LONG_TERM = "long_term"  # 1-6 months
    EXTENDED = "extended"  # > 6 months


@dataclass
class ModelConfig:
    """Configuration for predictive model"""
    model_id: str
    model_type: ModelType
    horizon: PredictionHorizon
    target_variable: str
    features: List[str]
    algorithm: str
    hyperparameters: Dict[str, Any]
    training_window_days: int
    minimum_samples: int
    accuracy_threshold: float
    update_frequency_days: int


@dataclass
class PredictionResult:
    """Result from predictive model"""
    prediction_id: str
    model_id: str
    timestamp: datetime
    patient_id: str
    prediction_type: str
    predicted_value: Any
    confidence: float
    risk_level: str
    explanation: Dict[str, Any]
    input_features: Dict[str, float]
    model_version: str
    horizon: PredictionHorizon


@dataclass
class ModelMetrics:
    """Performance metrics for model"""
    model_id: str
    accuracy: float
    precision: float
    recall: float
    f1_score: float
    auc_roc: float
    mse: Optional[float]
    mae: Optional[float]
    training_samples: int
    validation_samples: int
    last_updated: datetime


class BaseModel(ABC):
    """Abstract base class for predictive models"""
    
    def __init__(self, config: ModelConfig):
        self.config = config
        self.model = None
        self.scaler = None
        self.label_encoders = {}
        self.is_trained = False
        self.metrics = None
        
    @abstractmethod
    def train(self, data: pd.DataFrame) -> ModelMetrics:
        """Train the model"""
        pass
    
    @abstractmethod
    def predict(self, data: Dict[str, Any]) -> PredictionResult:
        """Make prediction"""
        pass
    
    @abstractmethod
    def explain(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Explain prediction"""
        pass
    
    def preprocess_data(self, data: pd.DataFrame, fit_transformers: bool = False) -> np.ndarray:
        """Preprocess data for model"""
        processed_data = data.copy()
        
        # Handle missing values
        processed_data = processed_data.fillna(processed_data.mean())
        
        # Encode categorical variables
        categorical_columns = processed_data.select_dtypes(include=['object']).columns
        for col in categorical_columns:
            if col not in self.label_encoders:
                self.label_encoders[col] = LabelEncoder()
            
            if fit_transformers:
                processed_data[col] = self.label_encoders[col].fit_transform(processed_data[col])
            else:
                # Handle unseen labels
                try:
                    processed_data[col] = self.label_encoders[col].transform(processed_data[col])
                except ValueError:
                    processed_data[col] = 0  # Default value for unseen labels
        
        # Scale features
        if self.scaler is None:
            self.scaler = StandardScaler()
        
        if fit_transformers:
            scaled_data = self.scaler.fit_transform(processed_data)
        else:
            scaled_data = self.scaler.transform(processed_data)
        
        return scaled_data


class RiskAssessmentModel(BaseModel):
    """Risk assessment model for various health conditions"""
    
    def train(self, data: pd.DataFrame) -> ModelMetrics:
        """Train risk assessment model"""
        logger.info(f"Training risk assessment model: {self.config.model_id}")
        
        # Prepare features and target
        X = data[self.config.features]
        y = data[self.config.target_variable]
        
        # Preprocess data
        X_processed = self.preprocess_data(data, fit_transformers=True)
        
        # Split data
        X_train, X_val, y_train, y_val = train_test_split(
            X_processed, y, test_size=0.2, random_state=42
        )
        
        # Initialize model based on algorithm
        if self.config.algorithm == 'random_forest':
            self.model = RandomForestClassifier(**self.config.hyperparameters)
        elif self.config.algorithm == 'xgboost':
            self.model = xgb.XGBClassifier(**self.config.hyperparameters)
        elif self.config.algorithm == 'logistic_regression':
            self.model = LogisticRegression(**self.config.hyperparameters)
        else:
            self.model = RandomForestClassifier(n_estimators=100, random_state=42)
        
        # Train model
        self.model.fit(X_train, y_train)
        self.is_trained = True
        
        # Calculate metrics
        y_pred = self.model.predict(X_val)
        y_pred_proba = self.model.predict_proba(X_val)[:, 1]
        
        metrics = ModelMetrics(
            model_id=self.config.model_id,
            accuracy=accuracy_score(y_val, y_pred),
            precision=self._calculate_precision(y_val, y_pred),
            recall=self._calculate_recall(y_val, y_pred),
            f1_score=self._calculate_f1(y_val, y_pred),
            auc_roc=roc_auc_score(y_val, y_pred_proba),
            mse=None,
            mae=None,
            training_samples=len(X_train),
            validation_samples=len(X_val),
            last_updated=datetime.now()
        )
        
        self.metrics = metrics
        logger.info(f"Model trained with accuracy: {metrics.accuracy:.3f}")
        
        return metrics
    
    def predict(self, data: Dict[str, Any]) -> PredictionResult:
        """Make risk prediction"""
        if not self.is_trained:
            raise ValueError("Model not trained")
        
        # Convert to DataFrame
        df = pd.DataFrame([data])
        
        # Preprocess
        X_processed = self.preprocess_data(df[self.config.features])
        
        # Make prediction
        prediction_proba = self.model.predict_proba(X_processed)[0]
        prediction = self.model.predict(X_processed)[0]
        
        # Determine risk level
        if prediction_proba[1] >= 0.8:
            risk_level = "HIGH"
        elif prediction_proba[1] >= 0.5:
            risk_level = "MEDIUM"
        else:
            risk_level = "LOW"
        
        # Generate explanation
        explanation = self.explain(data)
        
        return PredictionResult(
            prediction_id=f"pred_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
            model_id=self.config.model_id,
            timestamp=datetime.now(),
            patient_id=data.get('patient_id', 'unknown'),
            prediction_type='risk_assessment',
            predicted_value=int(prediction),
            confidence=float(prediction_proba[1]),
            risk_level=risk_level,
            explanation=explanation,
            input_features=data,
            model_version="v1.0",
            horizon=self.config.horizon
        )
    
    def explain(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Explain prediction using feature importance"""
        if not self.is_trained or not hasattr(self.model, 'feature_importances_'):
            return {"explanation": "Feature importance not available"}
        
        # Get feature importance
        feature_importance = dict(zip(self.config.features, self.model.feature_importances_))
        
        # Sort by importance
        sorted_features = sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
        
        # Generate explanation
        explanation = {
            "method": "feature_importance",
            "top_features": sorted_features[:5],
            "feature_importance": feature_importance,
            "interpretation": "Higher values indicate greater importance in risk prediction"
        }
        
        return explanation
    
    def _calculate_precision(self, y_true, y_pred) -> float:
        """Calculate precision"""
        from sklearn.metrics import precision_score
        return precision_score(y_true, y_pred, average='weighted')
    
    def _calculate_recall(self, y_true, y_pred) -> float:
        """Calculate recall"""
        from sklearn.metrics import recall_score
        return recall_score(y_true, y_pred, average='weighted')
    
    def _calculate_f1(self, y_true, y_pred) -> float:
        """Calculate F1 score"""
        from sklearn.metrics import f1_score
        return f1_score(y_true, y_pred, average='weighted')


class TreatmentResponseModel(BaseModel):
    """Model for predicting treatment response"""
    
    def train(self, data: pd.DataFrame) -> ModelMetrics:
        """Train treatment response model"""
        logger.info(f"Training treatment response model: {self.config.model_id}")
        
        X = data[self.config.features]
        y = data[self.config.target_variable]
        
        X_processed = self.preprocess_data(data, fit_transformers=True)
        
        X_train, X_val, y_train, y_val = train_test_split(
            X_processed, y, test_size=0.2, random_state=42
        )
        
        # Use gradient boosting for treatment response
        self.model = GradientBoostingRegressor(**self.config.hyperparameters)
        self.model.fit(X_train, y_train)
        self.is_trained = True
        
        # Calculate metrics
        y_pred = self.model.predict(X_val)
        
        metrics = ModelMetrics(
            model_id=self.config.model_id,
            accuracy=0.0,  # Not applicable for regression
            precision=0.0,
            recall=0.0,
            f1_score=0.0,
            auc_roc=0.0,
            mse=mean_squared_error(y_val, y_pred),
            mae=np.mean(np.abs(y_val - y_pred)),
            training_samples=len(X_train),
            validation_samples=len(X_val),
            last_updated=datetime.now()
        )
        
        self.metrics = metrics
        logger.info(f"Model trained with MSE: {metrics.mse:.3f}")
        
        return metrics
    
    def predict(self, data: Dict[str, Any]) -> PredictionResult:
        """Make treatment response prediction"""
        if not self.is_trained:
            raise ValueError("Model not trained")
        
        df = pd.DataFrame([data])
        X_processed = self.preprocess_data(df[self.config.features])
        
        prediction = self.model.predict(X_processed)[0]
        
        # Determine response level
        if prediction >= 0.7:
            response_level = "EXCELLENT"
        elif prediction >= 0.4:
            response_level = "MODERATE"
        else:
            response_level = "POOR"
        
        explanation = self.explain(data)
        
        return PredictionResult(
            prediction_id=f"pred_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
            model_id=self.config.model_id,
            timestamp=datetime.now(),
            patient_id=data.get('patient_id', 'unknown'),
            prediction_type='treatment_response',
            predicted_value=float(prediction),
            confidence=0.85,  # Placeholder
            risk_level=response_level,
            explanation=explanation,
            input_features=data,
            model_version="v1.0",
            horizon=self.config.horizon
        )
    
    def explain(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Explain treatment response prediction"""
        if not self.is_trained:
            return {"explanation": "Model not trained"}
        
        # Simplified explanation for treatment response
        explanation = {
            "method": "regression_feature_importance",
            "key_factors": [
                "Patient age and weight",
                "Genetic markers",
                "Previous treatment history",
                "Current medications",
                "Lifestyle factors"
            ],
            "interpretation": "Response prediction based on multi-factor analysis"
        }
        
        return explanation


class PredictiveModelManager:
    """Manages multiple predictive models for digital twin"""
    
    def __init__(self):
        self.models: Dict[str, BaseModel] = {}
        self.model_configs: Dict[str, ModelConfig] = {}
        self.prediction_history: List[PredictionResult] = []
        
    def register_model(self, config: ModelConfig) -> bool:
        """Register a new predictive model"""
        try:
            # Create appropriate model instance
            if config.model_type == ModelType.RISK_ASSESSMENT:
                model = RiskAssessmentModel(config)
            elif config.model_type == ModelType.TREATMENT_RESPONSE:
                model = TreatmentResponseModel(config)
            else:
                model = RiskAssessmentModel(config)  # Default to risk assessment
            
            self.models[config.model_id] = model
            self.model_configs[config.model_id] = config
            
            logger.info(f"Registered model: {config.model_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to register model {config.model_id}: {e}")
            return False
    
    def train_model(self, model_id: str, training_data: pd.DataFrame) -> ModelMetrics:
        """Train a specific model"""
        if model_id not in self.models:
            raise ValueError(f"Model {model_id} not found")
        
        model = self.models[model_id]
        
        # Validate training data
        if len(training_data) < model.config.minimum_samples:
            raise ValueError(f"Insufficient training data: {len(training_data)} < {model.config.minimum_samples}")
        
        return model.train(training_data)
    
    def make_prediction(self, model_id: str, input_data: Dict[str, Any]) -> PredictionResult:
        """Make prediction using specified model"""
        if model_id not in self.models:
            raise ValueError(f"Model {model_id} not found")
        
        model = self.models[model_id]
        
        if not model.is_trained:
            raise ValueError(f"Model {model_id} not trained")
        
        prediction = model.predict(input_data)
        self.prediction_history.append(prediction)
        
        return prediction
    
    def batch_predict(self, model_id: str, input_data_list: List[Dict[str, Any]]) -> List[PredictionResult]:
        """Make batch predictions"""
        results = []
        for input_data in input_data_list:
            try:
                prediction = self.make_prediction(model_id, input_data)
                results.append(prediction)
            except Exception as e:
                logger.error(f"Batch prediction failed: {e}")
        
        return results
    
    def get_model_metrics(self, model_id: str) -> Optional[ModelMetrics]:
        """Get performance metrics for model"""
        if model_id not in self.models:
            return None
        
        return self.models[model_id].metrics
    
    def retrain_model(self, model_id: str, new_data: pd.DataFrame) -> ModelMetrics:
        """Retrain model with new data"""
        logger.info(f"Retraining model: {model_id}")
        
        # Combine with existing training data if available
        # In a real implementation, you'd maintain a training data repository
        
        return self.train_model(model_id, new_data)
    
    def get_prediction_history(
        self, 
        patient_id: Optional[str] = None,
        model_id: Optional[str] = None,
        limit: int = 100
    ) -> List[PredictionResult]:
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
    
    def evaluate_model_performance(self, model_id: str, test_data: pd.DataFrame) -> Dict[str, float]:
        """Evaluate model performance on test data"""
        if model_id not in self.models:
            raise ValueError(f"Model {model_id} not found")
        
        model = self.models[model_id]
        
        # Make predictions on test data
        X = test_data[model.config.features]
        y_true = test_data[model.config.target_variable]
        
        predictions = []
        for _, row in test_data.iterrows():
            input_data = dict(row[model.config.features])
            prediction = model.predict(input_data)
            predictions.append(prediction.predicted_value)
        
        # Calculate metrics
        if model.config.model_type == ModelType.RISK_ASSESSMENT:
            accuracy = accuracy_score(y_true, predictions)
            auc = roc_auc_score(y_true, predictions)
            return {"accuracy": accuracy, "auc_roc": auc}
        else:
            mse = mean_squared_error(y_true, predictions)
            mae = np.mean(np.abs(y_true - predictions))
            return {"mse": mse, "mae": mae}
    
    def save_model(self, model_id: str, filepath: str) -> bool:
        """Save trained model to file"""
        if model_id not in self.models:
            return False
        
        try:
            model_data = {
                'model': self.models[model_id].model,
                'scaler': self.models[model_id].scaler,
                'label_encoders': self.models[model_id].label_encoders,
                'config': self.model_configs[model_id],
                'metrics': self.models[model_id].metrics
            }
            
            with open(filepath, 'wb') as f:
                pickle.dump(model_data, f)
            
            logger.info(f"Model {model_id} saved to {filepath}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to save model {model_id}: {e}")
            return False
    
    def load_model(self, model_id: str, filepath: str) -> bool:
        """Load trained model from file"""
        try:
            with open(filepath, 'rb') as f:
                model_data = pickle.load(f)
            
            # Recreate model instance
            config = model_data['config']
            if config.model_type == ModelType.RISK_ASSESSMENT:
                model = RiskAssessmentModel(config)
            elif config.model_type == ModelType.TREATMENT_RESPONSE:
                model = TreatmentResponseModel(config)
            else:
                model = RiskAssessmentModel(config)
            
            model.model = model_data['model']
            model.scaler = model_data['scaler']
            model.label_encoders = model_data['label_encoders']
            model.metrics = model_data['metrics']
            model.is_trained = True
            
            self.models[model_id] = model
            self.model_configs[model_id] = config
            
            logger.info(f"Model {model_id} loaded from {filepath}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to load model {model_id}: {e}")
            return False


# Example usage
def create_sample_models():
    """Create sample predictive models"""
    manager = PredictiveModelManager()
    
    # Risk assessment model
    risk_config = ModelConfig(
        model_id="cardiovascular_risk",
        model_type=ModelType.RISK_ASSESSMENT,
        horizon=PredictionHorizon.MEDIUM_TERM,
        target_variable="cardiovascular_event",
        features=["age", "bmi", "blood_pressure", "cholesterol", "smoking_status", "diabetes"],
        algorithm="random_forest",
        hyperparameters={"n_estimators": 100, "max_depth": 10, "random_state": 42},
        training_window_days=365,
        minimum_samples=1000,
        accuracy_threshold=0.8,
        update_frequency_days=30
    )
    
    # Treatment response model
    treatment_config = ModelConfig(
        model_id="diabetes_treatment_response",
        model_type=ModelType.TREATMENT_RESPONSE,
        horizon=PredictionHorizon.SHORT_TERM,
        target_variable="hba1c_reduction",
        features=["age", "bmi", "baseline_hba1c", "medication_type", "adherence_score", "diet_compliance"],
        algorithm="gradient_boosting",
        hyperparameters={"n_estimators": 100, "learning_rate": 0.1, "max_depth": 5},
        training_window_days=180,
        minimum_samples=500,
        accuracy_threshold=0.7,
        update_frequency_days=14
    )
    
    manager.register_model(risk_config)
    manager.register_model(treatment_config)
    
    return manager


if __name__ == "__main__":
    # Example usage
    manager = create_sample_models()
    
    print("Digital Twin Predictive Modeling Engine")
    print(f"Registered models: {list(manager.models.keys())}")
    
    # Create sample training data
    np.random.seed(42)
    n_samples = 1000
    
    training_data = pd.DataFrame({
        'age': np.random.randint(18, 80, n_samples),
        'bmi': np.random.normal(28, 5, n_samples),
        'blood_pressure': np.random.normal(120, 15, n_samples),
        'cholesterol': np.random.normal(200, 40, n_samples),
        'smoking_status': np.random.choice([0, 1], n_samples),
        'diabetes': np.random.choice([0, 1], n_samples),
        'cardiovascular_event': np.random.choice([0, 1], n_samples, p=[0.9, 0.1])
    })
    
    # Train risk assessment model
    try:
        metrics = manager.train_model("cardiovascular_risk", training_data)
        print(f"Model trained with accuracy: {metrics.accuracy:.3f}")
        
        # Make prediction
        sample_input = {
            'patient_id': 'demo_patient',
            'age': 55,
            'bmi': 30.5,
            'blood_pressure': 135,
            'cholesterol': 220,
            'smoking_status': 1,
            'diabetes': 0
        }
        
        prediction = manager.make_prediction("cardiovascular_risk", sample_input)
        print(f"Prediction: {prediction.risk_level} risk (confidence: {prediction.confidence:.2f})")
        
    except Exception as e:
        print(f"Error in demo: {e}")
