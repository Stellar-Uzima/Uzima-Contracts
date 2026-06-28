"""
Data Accuracy Monitoring for Digital Twin Platform
Ensures >95% data accuracy compared to source records
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
import statistics
from sklearn.metrics import accuracy_score, mean_absolute_error, mean_squared_error
from scipy import stats
import warnings

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class AccuracyLevel(Enum):
    """Data accuracy levels"""
    EXCELLENT = "excellent"    # > 99%
    GOOD = "good"            # 95-99%
    ACCEPTABLE = "acceptable"  # 90-95%
    POOR = "poor"           # < 90%


class DiscrepancyType(Enum):
    """Types of data discrepancies"""
    MISSING_DATA = "missing_data"
    INCORRECT_VALUE = "incorrect_value"
    OUT_OF_RANGE = "out_of_range"
    TEMPORAL_MISMATCH = "temporal_mismatch"
    FORMAT_ERROR = "format_error"
    DUPLICATE_DATA = "duplicate_data"
    INCONSISTENT_UNITS = "inconsistent_units"


@dataclass
class AccuracyMetric:
    """Accuracy metric for data validation"""
    metric_name: str
    expected_value: Any
    actual_value: Any
    accuracy_score: float  # 0.0 to 1.0
    discrepancy_type: Optional[DiscrepancyType]
    confidence: float
    timestamp: datetime
    source_system: str
    data_type: str


@dataclass
class ValidationRule:
    """Rule for data validation"""
    rule_id: str
    data_type: str
    field_name: str
    validation_type: str  # 'range', 'format', 'consistency', 'completeness'
    parameters: Dict[str, Any]
    weight: float  # Importance weight for overall accuracy
    description: str


@dataclass
class AccuracyReport:
    """Comprehensive accuracy report"""
    twin_id: str
    timestamp: datetime
    overall_accuracy: float
    accuracy_level: AccuracyLevel
    data_source_accuracy: Dict[str, float]
    data_type_accuracy: Dict[str, float]
    discrepancy_summary: Dict[DiscrepancyType, int]
    critical_issues: List[str]
    recommendations: List[str]
    trend_analysis: Dict[str, str]


class DataValidator:
    """Validates data accuracy against source records"""
    
    def __init__(self):
        self.validation_rules = self._initialize_validation_rules()
        self.accuracy_history: List[AccuracyMetric] = []
        
    def _initialize_validation_rules(self) -> List[ValidationRule]:
        """Initialize validation rules for different data types"""
        rules = [
            # Vital signs validation
            ValidationRule(
                rule_id="heart_rate_range",
                data_type="vital_signs",
                field_name="heart_rate",
                validation_type="range",
                parameters={"min": 30, "max": 200, "unit": "bpm"},
                weight=1.0,
                description="Heart rate must be within normal range"
            ),
            ValidationRule(
                rule_id="blood_pressure_range",
                data_type="vital_signs",
                field_name="blood_pressure_systolic",
                validation_type="range",
                parameters={"min": 60, "max": 250, "unit": "mmHg"},
                weight=1.0,
                description="Systolic blood pressure validation"
            ),
            ValidationRule(
                rule_id="temperature_range",
                data_type="vital_signs",
                field_name="temperature",
                validation_type="range",
                parameters={"min": 35.0, "max": 42.0, "unit": "celsius"},
                weight=0.8,
                description="Body temperature validation"
            ),
            
            # Lab results validation
            ValidationRule(
                rule_id="glucose_range",
                data_type="lab_results",
                field_name="glucose",
                validation_type="range",
                parameters={"min": 20, "max": 600, "unit": "mg/dL"},
                weight=1.0,
                description="Blood glucose level validation"
            ),
            ValidationRule(
                rule_id="cholesterol_range",
                data_type="lab_results",
                field_name="cholesterol",
                validation_type="range",
                parameters={"min": 100, "max": 400, "unit": "mg/dL"},
                weight=0.9,
                description="Cholesterol level validation"
            ),
            
            # Data completeness validation
            ValidationRule(
                rule_id="required_fields",
                data_type="all",
                field_name="completeness",
                validation_type="completeness",
                parameters={"required_fields": ["timestamp", "patient_id", "value"]},
                weight=1.0,
                description="Required fields must be present"
            ),
            
            # Temporal validation
            ValidationRule(
                rule_id="timestamp_validity",
                data_type="all",
                field_name="timestamp",
                validation_type="temporal",
                parameters={"max_future_hours": 1, "max_past_years": 5},
                weight=0.7,
                description="Timestamp must be within reasonable range"
            )
        ]
        
        return rules
    
    def validate_data_point(self, data_point: Dict[str, Any], source_system: str) -> List[AccuracyMetric]:
        """Validate a single data point"""
        metrics = []
        
        for rule in self.validation_rules:
            if rule.data_type == "all" or self._matches_data_type(data_point, rule.data_type):
                metric = self._apply_validation_rule(data_point, rule, source_system)
                if metric:
                    metrics.append(metric)
        
        return metrics
    
    def _matches_data_type(self, data_point: Dict[str, Any], expected_type: str) -> bool:
        """Check if data point matches expected type"""
        data_type = data_point.get('data_type', '').lower()
        return expected_type.lower() in data_type
    
    def _apply_validation_rule(self, data_point: Dict[str, Any], rule: ValidationRule, 
                            source_system: str) -> Optional[AccuracyMetric]:
        """Apply a specific validation rule"""
        field_value = data_point.get(rule.field_name)
        
        if field_value is None:
            return AccuracyMetric(
                metric_name=rule.rule_id,
                expected_value="present",
                actual_value="missing",
                accuracy_score=0.0,
                discrepancy_type=DiscrepancyType.MISSING_DATA,
                confidence=1.0,
                timestamp=datetime.now(),
                source_system=source_system,
                data_type=rule.data_type
            )
        
        if rule.validation_type == "range":
            return self._validate_range(data_point, rule, source_system)
        elif rule.validation_type == "format":
            return self._validate_format(data_point, rule, source_system)
        elif rule.validation_type == "completeness":
            return self._validate_completeness(data_point, rule, source_system)
        elif rule.validation_type == "temporal":
            return self._validate_temporal(data_point, rule, source_system)
        
        return None
    
    def _validate_range(self, data_point: Dict[str, Any], rule: ValidationRule, 
                        source_system: str) -> AccuracyMetric:
        """Validate range constraint"""
        field_value = data_point.get(rule.field_name)
        params = rule.parameters
        
        try:
            # Convert to float for comparison
            numeric_value = float(field_value)
            
            min_val = params.get('min', float('-inf'))
            max_val = params.get('max', float('inf'))
            
            if min_val <= numeric_value <= max_val:
                accuracy_score = 1.0
                discrepancy_type = None
            else:
                accuracy_score = 0.0
                discrepancy_type = DiscrepancyType.OUT_OF_RANGE
            
            return AccuracyMetric(
                metric_name=rule.rule_id,
                expected_value=f"{min_val}-{max_val}",
                actual_value=numeric_value,
                accuracy_score=accuracy_score,
                discrepancy_type=discrepancy_type,
                confidence=0.9,
                timestamp=datetime.now(),
                source_system=source_system,
                data_type=rule.data_type
            )
            
        except (ValueError, TypeError):
            return AccuracyMetric(
                metric_name=rule.rule_id,
                expected_value="numeric",
                actual_value=str(field_value),
                accuracy_score=0.0,
                discrepancy_type=DiscrepancyType.FORMAT_ERROR,
                confidence=0.8,
                timestamp=datetime.now(),
                source_system=source_system,
                data_type=rule.data_type
            )
    
    def _validate_format(self, data_point: Dict[str, Any], rule: ValidationRule, 
                        source_system: str) -> AccuracyMetric:
        """Validate format constraint"""
        field_value = data_point.get(rule.field_name)
        
        # Simple format validation (can be extended)
        if isinstance(field_value, (int, float, str)):
            accuracy_score = 1.0
            discrepancy_type = None
        else:
            accuracy_score = 0.0
            discrepancy_type = DiscrepancyType.FORMAT_ERROR
        
        return AccuracyMetric(
            metric_name=rule.rule_id,
            expected_value="valid_format",
            actual_value=type(field_value).__name__,
            accuracy_score=accuracy_score,
            discrepancy_type=discrepancy_type,
            confidence=0.8,
            timestamp=datetime.now(),
            source_system=source_system,
            data_type=rule.data_type
        )
    
    def _validate_completeness(self, data_point: Dict[str, Any], rule: ValidationRule, 
                             source_system: str) -> AccuracyMetric:
        """Validate data completeness"""
        required_fields = rule.parameters.get('required_fields', [])
        missing_fields = [field for field in required_fields if field not in data_point]
        
        if not missing_fields:
            accuracy_score = 1.0
            discrepancy_type = None
        else:
            accuracy_score = (len(required_fields) - len(missing_fields)) / len(required_fields)
            discrepancy_type = DiscrepancyType.MISSING_DATA
        
        return AccuracyMetric(
            metric_name=rule.rule_id,
            expected_value="complete",
            actual_value=f"missing_{len(missing_fields)}",
            accuracy_score=accuracy_score,
            discrepancy_type=discrepancy_type,
            confidence=1.0,
            timestamp=datetime.now(),
            source_system=source_system,
            data_type=rule.data_type
        )
    
    def _validate_temporal(self, data_point: Dict[str, Any], rule: ValidationRule, 
                          source_system: str) -> AccuracyMetric:
        """Validate temporal constraints"""
        timestamp_str = data_point.get('timestamp')
        
        try:
            if isinstance(timestamp_str, str):
                timestamp = datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))
            elif isinstance(timestamp_str, datetime):
                timestamp = timestamp_str
            else:
                timestamp = datetime.fromtimestamp(float(timestamp_str))
            
            now = datetime.now()
            params = rule.parameters
            
            max_future_hours = params.get('max_future_hours', 1)
            max_past_years = params.get('max_past_years', 5)
            
            time_diff = now - timestamp
            
            if time_diff.total_seconds() <= max_future_hours * 3600 and \
               time_diff.total_seconds() <= max_past_years * 365 * 24 * 3600:
                accuracy_score = 1.0
                discrepancy_type = None
            else:
                accuracy_score = 0.0
                discrepancy_type = DiscrepancyType.TEMPORAL_MISMATCH
            
            return AccuracyMetric(
                metric_name=rule.rule_id,
                expected_value="valid_timestamp",
                actual_value=timestamp.isoformat(),
                accuracy_score=accuracy_score,
                discrepancy_type=discrepancy_type,
                confidence=0.9,
                timestamp=now,
                source_system=source_system,
                data_type=rule.data_type
            )
            
        except (ValueError, TypeError, OSError):
            return AccuracyMetric(
                metric_name=rule.rule_id,
                expected_value="valid_timestamp",
                actual_value=str(timestamp_str),
                accuracy_score=0.0,
                discrepancy_type=DiscrepancyType.FORMAT_ERROR,
                confidence=0.8,
                timestamp=datetime.now(),
                source_system=source_system,
                data_type=rule.data_type
            )


class AccuracyMonitor:
    """Monitors and tracks data accuracy over time"""
    
    def __init__(self, accuracy_threshold: float = 0.95):
        self.accuracy_threshold = accuracy_threshold
        self.validator = DataValidator()
        self.accuracy_history: List[AccuracyMetric] = []
        self.reports: List[AccuracyReport] = []
        
    def validate_batch(self, data_batch: List[Dict[str, Any]], source_system: str) -> List[AccuracyMetric]:
        """Validate a batch of data points"""
        all_metrics = []
        
        for data_point in data_batch:
            metrics = self.validator.validate_data_point(data_point, source_system)
            all_metrics.extend(metrics)
            self.accuracy_history.extend(metrics)
        
        return all_metrics
    
    def calculate_overall_accuracy(self, metrics: List[AccuracyMetric]) -> float:
        """Calculate weighted overall accuracy"""
        if not metrics:
            return 1.0
        
        # Get weights from validation rules
        weights = {}
        for rule in self.validator.validation_rules:
            weights[rule.rule_id] = rule.weight
        
        total_weighted_score = 0.0
        total_weight = 0.0
        
        for metric in metrics:
            weight = weights.get(metric.metric_name, 1.0)
            total_weighted_score += metric.accuracy_score * weight * metric.confidence
            total_weight += weight * metric.confidence
        
        return total_weighted_score / total_weight if total_weight > 0 else 1.0
    
    def detect_anomalies(self, recent_metrics: List[AccuracyMetric], window_size: int = 100) -> List[str]:
        """Detect anomalies in accuracy metrics"""
        anomalies = []
        
        if len(recent_metrics) < window_size:
            return anomalies
        
        # Group by metric name
        metrics_by_name = {}
        for metric in recent_metrics[-window_size:]:
            if metric.metric_name not in metrics_by_name:
                metrics_by_name[metric.metric_name] = []
            metrics_by_name[metric.metric_name].append(metric.accuracy_score)
        
        # Detect statistical anomalies
        for metric_name, scores in metrics_by_name.items():
            if len(scores) < 10:
                continue
                
            mean_score = statistics.mean(scores)
            std_score = statistics.stdev(scores) if len(scores) > 1 else 0
            
            # Check for significant drops
            recent_scores = scores[-10:]
            recent_mean = statistics.mean(recent_scores)
            
            if std_score > 0 and abs(recent_mean - mean_score) > 2 * std_score:
                if recent_mean < mean_score:
                    anomalies.append(f"Significant accuracy drop in {metric_name}: {recent_mean:.3f} vs {mean_score:.3f}")
                else:
                    anomalies.append(f"Significant accuracy improvement in {metric_name}: {recent_mean:.3f} vs {mean_mean:.3f}")
        
        return anomalies
    
    def generate_accuracy_report(self, twin_id: str, time_window_hours: int = 24) -> AccuracyReport:
        """Generate comprehensive accuracy report"""
        now = datetime.now()
        cutoff_time = now - timedelta(hours=time_window_hours)
        
        # Filter recent metrics
        recent_metrics = [
            metric for metric in self.accuracy_history 
            if metric.timestamp >= cutoff_time
        ]
        
        if not recent_metrics:
            return AccuracyReport(
                twin_id=twin_id,
                timestamp=now,
                overall_accuracy=1.0,
                accuracy_level=AccuracyLevel.EXCELLENT,
                data_source_accuracy={},
                data_type_accuracy={},
                discrepancy_summary={},
                critical_issues=["No data available for analysis"],
                recommendations=["Collect more data for accuracy analysis"],
                trend_analysis={}
            )
        
        # Calculate overall accuracy
        overall_accuracy = self.calculate_overall_accuracy(recent_metrics)
        
        # Determine accuracy level
        if overall_accuracy >= 0.99:
            accuracy_level = AccuracyLevel.EXCELLENT
        elif overall_accuracy >= 0.95:
            accuracy_level = AccuracyLevel.GOOD
        elif overall_accuracy >= 0.90:
            accuracy_level = AccuracyLevel.ACCEPTABLE
        else:
            accuracy_level = AccuracyLevel.POOR
        
        # Calculate accuracy by data source
        source_accuracy = {}
        for source in set(metric.source_system for metric in recent_metrics):
            source_metrics = [m for m in recent_metrics if m.source_system == source]
            source_accuracy[source] = self.calculate_overall_accuracy(source_metrics)
        
        # Calculate accuracy by data type
        type_accuracy = {}
        for data_type in set(metric.data_type for metric in recent_metrics):
            type_metrics = [m for m in recent_metrics if m.data_type == data_type]
            type_accuracy[data_type] = self.calculate_overall_accuracy(type_metrics)
        
        # Summarize discrepancies
        discrepancy_summary = {}
        for metric in recent_metrics:
            if metric.discrepancy_type:
                discrepancy_summary[metric.discrepancy_type] = discrepancy_summary.get(metric.discrepancy_type, 0) + 1
        
        # Identify critical issues
        critical_issues = []
        if overall_accuracy < self.accuracy_threshold:
            critical_issues.append(f"Overall accuracy ({overall_accuracy:.3f}) below threshold ({self.accuracy_threshold})")
        
        # Check for specific problematic areas
        for source, accuracy in source_accuracy.items():
            if accuracy < 0.9:
                critical_issues.append(f"Low accuracy from {source}: {accuracy:.3f}")
        
        for data_type, accuracy in type_accuracy.items():
            if accuracy < 0.9:
                critical_issues.append(f"Low accuracy for {data_type}: {accuracy:.3f}")
        
        # Check for high-frequency discrepancies
        for discrepancy_type, count in discrepancy_summary.items():
            if count > len(recent_metrics) * 0.1:  # More than 10% of all metrics
                critical_issues.append(f"High frequency of {discrepancy_type.value}: {count} occurrences")
        
        # Generate recommendations
        recommendations = self._generate_recommendations(overall_accuracy, source_accuracy, 
                                                      type_accuracy, discrepancy_summary)
        
        # Trend analysis
        trend_analysis = self._analyze_trends(recent_metrics)
        
        return AccuracyReport(
            twin_id=twin_id,
            timestamp=now,
            overall_accuracy=overall_accuracy,
            accuracy_level=accuracy_level,
            data_source_accuracy=source_accuracy,
            data_type_accuracy=type_accuracy,
            discrepancy_summary=discrepancy_summary,
            critical_issues=critical_issues,
            recommendations=recommendations,
            trend_analysis=trend_analysis
        )
    
    def _generate_recommendations(self, overall_accuracy: float, 
                                source_accuracy: Dict[str, float],
                                type_accuracy: Dict[str, float],
                                discrepancy_summary: Dict[DiscrepancyType, int]) -> List[str]:
        """Generate recommendations based on accuracy analysis"""
        recommendations = []
        
        if overall_accuracy < self.accuracy_threshold:
            recommendations.append("Overall accuracy needs improvement. Review data collection processes.")
        
        # Source-specific recommendations
        for source, accuracy in source_accuracy.items():
            if accuracy < 0.9:
                recommendations.append(f"Improve data quality from {source} source system")
        
        # Type-specific recommendations
        for data_type, accuracy in type_accuracy.items():
            if accuracy < 0.9:
                recommendations.append(f"Focus on {data_type} data quality improvements")
        
        # Discrepancy-specific recommendations
        if DiscrepancyType.MISSING_DATA in discrepancy_summary:
            recommendations.append("Address missing data issues through better data capture")
        
        if DiscrepancyType.OUT_OF_RANGE in discrepancy_summary:
            recommendations.append("Validate data ranges and implement input validation")
        
        if DiscrepancyType.FORMAT_ERROR in discrepancy_summary:
            recommendations.append("Standardize data formats across all sources")
        
        if DiscrepancyType.TEMPORAL_MISMATCH in discrepancy_summary:
            recommendations.append("Synchronize timestamps across systems")
        
        return recommendations
    
    def _analyze_trends(self, recent_metrics: List[AccuracyMetric]) -> Dict[str, str]:
        """Analyze trends in accuracy metrics"""
        trends = {}
        
        if len(recent_metrics) < 10:
            return trends
        
        # Group by metric name
        metrics_by_name = {}
        for metric in recent_metrics:
            if metric.metric_name not in metrics_by_name:
                metrics_by_name[metric.metric_name] = []
            metrics_by_name[metric.metric_name].append(metric)
        
        for metric_name, metrics in metrics_by_name.items():
            if len(metrics) < 5:
                continue
            
            # Sort by timestamp
            metrics.sort(key=lambda x: x.timestamp)
            
            # Extract accuracy scores
            scores = [m.accuracy_score for m in metrics]
            
            # Calculate trend
            if len(scores) >= 3:
                # Simple linear trend
                x = list(range(len(scores)))
                slope, intercept, r_value, p_value, std_err = stats.linregress(x, scores)
                
                if p_value < 0.05:  # Significant trend
                    if slope > 0.01:
                        trends[metric_name] = "improving"
                    elif slope < -0.01:
                        trends[metric_name] = "declining"
                    else:
                        trends[metric_name] = "stable"
                else:
                    trends[metric_name] = "stable"
        
        return trends
    
    def get_accuracy_trend(self, metric_name: str, days: int = 7) -> Dict[str, Any]:
        """Get accuracy trend for specific metric"""
        cutoff_time = datetime.now() - timedelta(days=days)
        
        metric_history = [
            metric for metric in self.accuracy_history 
            if metric.metric_name == metric_name and metric.timestamp >= cutoff_time
        ]
        
        if not metric_history:
            return {"trend": "no_data", "data_points": 0}
        
        # Sort by timestamp
        metric_history.sort(key=lambda x: x.timestamp)
        
        # Calculate trend statistics
        scores = [m.accuracy_score for m in metric_history]
        timestamps = [m.timestamp for m in metric_history]
        
        # Simple trend analysis
        if len(scores) >= 3:
            x = list(range(len(scores)))
            slope, intercept, r_value, p_value, std_err = stats.linregress(x, scores)
            
            return {
                "trend": "improving" if slope > 0.01 else "declining" if slope < -0.01 else "stable",
                "slope": slope,
                "r_squared": r_value ** 2,
                "p_value": p_value,
                "data_points": len(scores),
                "current_accuracy": scores[-1],
                "average_accuracy": statistics.mean(scores),
                "accuracy_std": statistics.stdev(scores) if len(scores) > 1 else 0.0,
                "start_date": timestamps[0].isoformat(),
                "end_date": timestamps[-1].isoformat()
            }
        
        return {
            "trend": "insufficient_data",
            "data_points": len(scores),
            "current_accuracy": scores[-1] if scores else 0.0
        }


class DataReconciliation:
    """Reconciles data between digital twin and source systems"""
    
    def __init__(self):
        self.reconciliation_history: List[Dict[str, Any]] = []
        
    def reconcile_with_source(self, twin_data: Dict[str, Any], source_data: Dict[str, Any], 
                            source_system: str) -> Dict[str, Any]:
        """Reconcile digital twin data with source system data"""
        reconciliation_result = {
            'timestamp': datetime.now().isoformat(),
            'source_system': source_system,
            'fields_compared': 0,
            'fields_matched': 0,
            'fields_mismatched': 0,
            'fields_missing_twin': 0,
            'fields_missing_source': 0,
            'overall_match_rate': 0.0,
            'discrepancies': []
        }
        
        # Get all unique field names
        all_fields = set(twin_data.keys()) | set(source_data.keys())
        
        for field in all_fields:
            twin_value = twin_data.get(field)
            source_value = source_data.get(field)
            
            reconciliation_result['fields_compared'] += 1
            
            if twin_value is None and source_value is None:
                # Both missing - not counted as discrepancy
                continue
            elif twin_value is None:
                reconciliation_result['fields_missing_twin'] += 1
                reconciliation_result['discrepancies'].append({
                    'field': field,
                    'type': 'missing_in_twin',
                    'source_value': source_value
                })
            elif source_value is None:
                reconciliation_result['fields_missing_source'] += 1
                reconciliation_result['discrepancies'].append({
                    'field': field,
                    'type': 'missing_in_source',
                    'twin_value': twin_value
                })
            else:
                # Compare values
                if self._values_match(twin_value, source_value):
                    reconciliation_result['fields_matched'] += 1
                else:
                    reconciliation_result['fields_mismatched'] += 1
                    reconciliation_result['discrepancies'].append({
                        'field': field,
                        'type': 'value_mismatch',
                        'twin_value': twin_value,
                        'source_value': source_value
                    })
        
        # Calculate overall match rate
        total_comparable = (reconciliation_result['fields_matched'] + 
                          reconciliation_result['fields_mismatched'])
        
        if total_comparable > 0:
            reconciliation_result['overall_match_rate'] = \
                reconciliation_result['fields_matched'] / total_comparable
        
        self.reconciliation_history.append(reconciliation_result)
        
        return reconciliation_result
    
    def _values_match(self, value1: Any, value2: Any, tolerance: float = 0.01) -> bool:
        """Check if two values match within tolerance"""
        if value1 == value2:
            return True
        
        # Numeric comparison with tolerance
        try:
            num1 = float(value1)
            num2 = float(value2)
            
            if num1 == 0 and num2 == 0:
                return True
            
            relative_diff = abs(num1 - num2) / max(abs(num1), abs(num2), 1.0)
            return relative_diff <= tolerance
            
        except (ValueError, TypeError):
            # String comparison
            str1 = str(value1).strip().lower()
            str2 = str(value2).strip().lower()
            return str1 == str2
    
    def get_reconciliation_summary(self, hours: int = 24) -> Dict[str, Any]:
        """Get reconciliation summary for time period"""
        cutoff_time = datetime.now() - timedelta(hours=hours)
        
        recent_reconciliations = [
            r for r in self.reconciliation_history 
            if datetime.fromisoformat(r['timestamp']) >= cutoff_time
        ]
        
        if not recent_reconciliations:
            return {"message": "No reconciliation data available"}
        
        # Aggregate statistics
        total_comparisons = sum(r['fields_compared'] for r in recent_reconciliations)
        total_matches = sum(r['fields_matched'] for r in recent_reconciliations)
        total_mismatches = sum(r['fields_mismatched'] for r in recent_reconciliations)
        
        overall_match_rate = total_matches / total_comparisons if total_comparisons > 0 else 0.0
        
        # Group by source system
        source_stats = {}
        for r in recent_reconciliations:
            source = r['source_system']
            if source not in source_stats:
                source_stats[source] = {
                    'comparisons': 0,
                    'matches': 0,
                    'mismatches': 0,
                    'match_rate': 0.0
                }
            
            source_stats[source]['comparisons'] += r['fields_compared']
            source_stats[source]['matches'] += r['fields_matched']
            source_stats[source]['mismatches'] += r['fields_mismatched']
        
        for source, stats in source_stats.items():
            stats['match_rate'] = stats['matches'] / stats['comparisons'] if stats['comparisons'] > 0 else 0.0
        
        return {
            'period_hours': hours,
            'total_reconciliations': len(recent_reconciliations),
            'overall_match_rate': overall_match_rate,
            'total_comparisons': total_comparisons,
            'total_matches': total_matches,
            'total_mismatches': total_mismatches,
            'source_system_stats': source_stats
        }


# Example usage
def example_accuracy_monitoring():
    """Example of accuracy monitoring"""
    
    # Create accuracy monitor
    monitor = AccuracyMonitor(accuracy_threshold=0.95)
    
    # Sample data points
    sample_data = [
        {
            'data_type': 'vital_signs',
            'heart_rate': 75,
            'blood_pressure_systolic': 120,
            'temperature': 36.5,
            'timestamp': '2024-01-15T10:00:00',
            'patient_id': 'patient_001'
        },
        {
            'data_type': 'lab_results',
            'glucose': 95,
            'cholesterol': 180,
            'timestamp': '2024-01-15T10:30:00',
            'patient_id': 'patient_001'
        },
        {
            'data_type': 'vital_signs',
            'heart_rate': 250,  # Out of range
            'blood_pressure_systolic': 300,  # Out of range
            'temperature': 45.0,  # Out of range
            'timestamp': '2024-01-15T11:00:00',
            'patient_id': 'patient_001'
        }
    ]
    
    # Validate data
    metrics = monitor.validate_batch(sample_data, "wearable_device")
    
    print(f"Validated {len(metrics)} metrics")
    for metric in metrics:
        print(f"  {metric.metric_name}: {metric.accuracy_score:.2f} - {metric.discrepancy_type}")
    
    # Generate accuracy report
    report = monitor.generate_accuracy_report("twin_001")
    
    print(f"\nAccuracy Report for twin_001:")
    print(f"Overall Accuracy: {report.overall_accuracy:.3f}")
    print(f"Accuracy Level: {report.accuracy_level.value}")
    print(f"Critical Issues: {report.critical_issues}")
    print(f"Recommendations: {report.recommendations}")
    
    # Data reconciliation example
    reconciler = DataReconciliation()
    
    twin_data = {
        'heart_rate': 75,
        'blood_pressure': 120,
        'temperature': 36.5
    }
    
    source_data = {
        'heart_rate': 76,
        'blood_pressure': 122,
        'temperature': 36.6,
        'oxygen_saturation': 98
    }
    
    reconciliation = reconciler.reconcile_with_source(twin_data, source_data, "emr_system")
    
    print(f"\nReconciliation Results:")
    print(f"Match Rate: {reconciliation['overall_match_rate']:.3f}")
    print(f"Fields Matched: {reconciliation['fields_matched']}")
    print(f"Fields Mismatched: {reconciliation['fields_mismatched']}")


if __name__ == "__main__":
    example_accuracy_monitoring()
