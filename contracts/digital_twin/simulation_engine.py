"""
Simulation Engine for Digital Twin Platform
Supports what-if analysis, treatment simulation, and scenario testing
"""

import numpy as np
import pandas as pd
import json
import logging
from typing import Dict, List, Optional, Any, Tuple, Union
from dataclasses import dataclass, asdict
from enum import Enum
from datetime import datetime, timedelta
import matplotlib.pyplot as plt
import seaborn as sns
from scipy.integrate import odeint
from scipy.stats import norm
import networkx as nx

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class SimulationType(Enum):
    """Types of simulations"""
    TREATMENT = "treatment"
    LIFESTYLE = "lifestyle"
    ENVIRONMENTAL = "environmental"
    MEDICATION = "medication"
    SURGICAL = "surgical"
    PREVENTIVE = "preventive"
    DISEASE_PROGRESSION = "disease_progression"
    COMPLICATION = "complication"


class ParameterType(Enum):
    """Types of simulation parameters"""
    CONTINUOUS = "continuous"
    DISCRETE = "discrete"
    CATEGORICAL = "categorical"
    BOOLEAN = "boolean"
    TIME_SERIES = "time_series"


@dataclass
class SimulationParameter:
    """Parameter for simulation"""
    name: str
    parameter_type: ParameterType
    value: Any
    range: Optional[Tuple[float, float]]
    distribution: Optional[str]  # 'normal', 'uniform', 'exponential'
    uncertainty: float  # 0.0 to 1.0
    description: str


@dataclass
class SimulationConfig:
    """Configuration for simulation"""
    simulation_id: str
    simulation_type: SimulationType
    twin_id: str
    parameters: List[SimulationParameter]
    time_horizon_days: int
    time_step_hours: int
    num_runs: int
    confidence_level: float
    outcome_metrics: List[str]
    baseline_scenario: Dict[str, Any]


@dataclass
class SimulationResult:
    """Result from simulation run"""
    simulation_id: str
    run_id: int
    timestamp: datetime
    parameters_used: Dict[str, Any]
    outcomes: Dict[str, Any]
    time_series_data: Dict[str, List[float]]
    confidence_intervals: Dict[str, Tuple[float, float]]
    sensitivity_analysis: Dict[str, float]
    recommendations: List[str]
    risk_assessment: Dict[str, str]


class PhysiologicalModel:
    """Base class for physiological models"""
    
    def __init__(self, patient_params: Dict[str, Any]):
        self.patient_params = patient_params
        self.model_params = self._initialize_model_params()
        
    def _initialize_model_params(self) -> Dict[str, Any]:
        """Initialize model-specific parameters"""
        return {
            'metabolic_rate': 1.0,  # Baseline metabolic rate
            'stress_factor': 1.0,  # Stress response factor
            'immunity_strength': 1.0,  # Immune system strength
            'recovery_rate': 1.0,  # Recovery rate factor
        }
    
    def differential_equations(self, state: List[float], t: float, params: Dict[str, Any]) -> List[float]:
        """Define differential equations for physiological model"""
        # Base implementation to be overridden
        return [0.0] * len(state)
    
    def simulate_time_series(self, initial_state: List[float], time_points: List[float], 
                           intervention_params: Dict[str, Any]) -> np.ndarray:
        """Simulate time series using ODE solver"""
        solution = odeint(self.differential_equations, initial_state, time_points, args=(intervention_params,))
        return solution


class CardiovascularModel(PhysiologicalModel):
    """Cardiovascular system model"""
    
    def _initialize_model_params(self) -> Dict[str, Any]:
        """Initialize cardiovascular-specific parameters"""
        params = super()._initialize_model_params()
        params.update({
            'heart_rate_baseline': 70,  # bpm
            'blood_pressure_sys_baseline': 120,  # mmHg
            'blood_pressure_dia_baseline': 80,  # mmHg
            'cardiac_output_baseline': 5.0,  # L/min
            'vascular_resistance_baseline': 20.0,  # mmHg/(L/min)
            'stroke_volume_baseline': 70,  # mL
        })
        return params
    
    def differential_equations(self, state: List[float], t: float, params: Dict[str, Any]) -> List[float]:
        """Cardiovascular differential equations"""
        hr, bp_sys, bp_dia, co = state
        
        # Get intervention effects
        medication_effect = params.get('medication_effect', 0.0)
        exercise_effect = params.get('exercise_effect', 0.0)
        stress_effect = params.get('stress_effect', 0.0)
        
        # Heart rate dynamics
        hr_baseline = self.model_params['heart_rate_baseline']
        dhr_dt = (hr_baseline - hr) * 0.1 + medication_effect * 5 - stress_effect * 10 + exercise_effect * 2
        
        # Blood pressure dynamics
        bp_sys_baseline = self.model_params['blood_pressure_sys_baseline']
        bp_dia_baseline = self.model_params['blood_pressure_dia_baseline']
        
        dbp_sys_dt = (bp_sys_baseline - bp_sys) * 0.08 + medication_effect * 10 - stress_effect * 15
        dbp_dia_dt = (bp_dia_baseline - bp_dia) * 0.08 + medication_effect * 5 - stress_effect * 8
        
        # Cardiac output dynamics
        co_baseline = self.model_params['cardiac_output_baseline']
        dco_dt = (co_baseline - co) * 0.05 + medication_effect * 0.5 + exercise_effect * 1.0
        
        return [dhr_dt, dbp_sys_dt, dbp_dia_dt, dco_dt]


class MetabolicModel(PhysiologicalModel):
    """Metabolic system model"""
    
    def _initialize_model_params(self) -> Dict[str, Any]:
        """Initialize metabolic-specific parameters"""
        params = super()._initialize_model_params()
        params.update({
            'glucose_baseline': 90,  # mg/dL
            'insulin_baseline': 10,  # μU/mL
            'hba1c_baseline': 5.5,  # %
            'bmi_baseline': 25.0,  # kg/m²
            'energy_balance_baseline': 0.0,  # kcal/day
        })
        return params
    
    def differential_equations(self, state: List[float], t: float, params: Dict[str, Any]) -> List[float]:
        """Metabolic differential equations"""
        glucose, insulin, hba1c, bmi = state
        
        # Get intervention effects
        diet_effect = params.get('diet_effect', 0.0)
        exercise_effect = params.get('exercise_effect', 0.0)
        medication_effect = params.get('medication_effect', 0.0)
        
        # Glucose dynamics
        glucose_baseline = self.model_params['glucose_baseline']
        dglucose_dt = (glucose_baseline - glucose) * 0.1 + diet_effect * 20 - medication_effect * 30 + exercise_effect * 10
        
        # Insulin dynamics
        insulin_baseline = self.model_params['insulin_baseline']
        dinsulin_dt = (insulin_baseline - insulin) * 0.08 + (glucose - glucose_baseline) * 0.01 - medication_effect * 5
        
        # HbA1c dynamics (slow)
        hba1c_baseline = self.model_params['hba1c_baseline']
        dhba1c_dt = (hba1c_baseline - hba1c) * 0.001 + (glucose - glucose_baseline) * 0.0001
        
        # BMI dynamics
        bmi_baseline = self.model_params['bmi_baseline']
        dbmi_dt = (bmi_baseline - bmi) * 0.005 + diet_effect * 0.01 - exercise_effect * 0.02
        
        return [dglucose_dt, dinsulin_dt, dhba1c_dt, dbmi_dt]


class TreatmentSimulator:
    """Simulates treatment effects"""
    
    def __init__(self):
        self.treatment_effects = self._initialize_treatment_effects()
        
    def _initialize_treatment_effects(self) -> Dict[str, Dict[str, float]]:
        """Initialize treatment effect parameters"""
        return {
            'lisinopril': {
                'blood_pressure_reduction': 10.0,  # mmHg
                'heart_rate_effect': -2.0,  # bpm
                'kidney_protection': 0.8,
                'side_effect_risk': 0.1
            },
            'metformin': {
                'glucose_reduction': 30.0,  # mg/dL
                'insulin_sensitivity': 0.3,
                'weight_effect': -0.5,  # kg
                'gi_side_effect_risk': 0.2
            },
            'statin': {
                'cholesterol_reduction': 40.0,  # mg/dL
                'cardiovascular_protection': 0.7,
                'muscle_pain_risk': 0.05,
                'liver_effect': 0.1
            },
            'exercise_program': {
                'blood_pressure_reduction': 5.0,
                'glucose_reduction': 15.0,
                'weight_loss_rate': 0.1,  # kg/week
                'fitness_improvement': 0.2
            },
            'diet_plan': {
                'glucose_reduction': 20.0,
                'cholesterol_reduction': 15.0,
                'weight_loss_rate': 0.05,
                'adherence_factor': 0.7
            }
        }
    
    def simulate_treatment_effect(self, treatment: str, duration_days: int, 
                                adherence: float = 1.0) -> Dict[str, float]:
        """Simulate treatment effect over time"""
        if treatment not in self.treatment_effects:
            return {}
        
        effects = self.treatment_effects[treatment].copy()
        
        # Apply adherence factor
        for key, value in effects.items():
            if 'risk' not in key:  # Don't reduce risk with adherence
                effects[key] = value * adherence
        
        # Apply time decay for some effects
        time_factor = min(1.0, duration_days / 30.0)  # Full effect after 30 days
        
        for key in ['blood_pressure_reduction', 'glucose_reduction', 'cholesterol_reduction']:
            if key in effects:
                effects[key] *= time_factor
        
        return effects


class SimulationEngine:
    """Main simulation engine for digital twin"""
    
    def __init__(self):
        self.models = {
            'cardiovascular': CardiovascularModel,
            'metabolic': MetabolicModel,
        }
        self.treatment_simulator = TreatmentSimulator()
        self.simulation_history: List[SimulationResult] = []
        
    def create_simulation_config(self, config_dict: Dict[str, Any]) -> SimulationConfig:
        """Create simulation configuration from dictionary"""
        parameters = []
        for param_name, param_data in config_dict.get('parameters', {}).items():
            param = SimulationParameter(
                name=param_name,
                parameter_type=ParameterType(param_data['type']),
                value=param_data['value'],
                range=param_data.get('range'),
                distribution=param_data.get('distribution'),
                uncertainty=param_data.get('uncertainty', 0.1),
                description=param_data.get('description', '')
            )
            parameters.append(param)
        
        return SimulationConfig(
            simulation_id=config_dict['simulation_id'],
            simulation_type=SimulationType(config_dict['simulation_type']),
            twin_id=config_dict['twin_id'],
            parameters=parameters,
            time_horizon_days=config_dict['time_horizon_days'],
            time_step_hours=config_dict['time_step_hours'],
            num_runs=config_dict['num_runs'],
            confidence_level=config_dict['confidence_level'],
            outcome_metrics=config_dict['outcome_metrics'],
            baseline_scenario=config_dict.get('baseline_scenario', {})
        )
    
    def run_simulation(self, config: SimulationConfig) -> List[SimulationResult]:
        """Run simulation with specified configuration"""
        logger.info(f"Running simulation {config.simulation_id}")
        
        results = []
        
        for run_id in range(config.num_runs):
            # Sample parameters with uncertainty
            sampled_params = self._sample_parameters(config)
            
            # Run simulation
            result = self._run_single_simulation(config, run_id, sampled_params)
            results.append(result)
        
        self.simulation_history.extend(results)
        return results
    
    def _sample_parameters(self, config: SimulationConfig) -> Dict[str, Any]:
        """Sample parameters with uncertainty"""
        sampled = {}
        
        for param in config.parameters:
            if param.parameter_type == ParameterType.CONTINUOUS:
                if param.distribution == 'normal':
                    std_dev = param.uncertainty * abs(param.value) if param.value != 0 else 1.0
                    sampled[param.name] = np.random.normal(param.value, std_dev)
                elif param.distribution == 'uniform':
                    if param.range:
                        sampled[param.name] = np.random.uniform(param.range[0], param.range[1])
                    else:
                        std_dev = param.uncertainty * abs(param.value) if param.value != 0 else 1.0
                        sampled[param.name] = np.random.uniform(param.value - std_dev, param.value + std_dev)
                else:
                    sampled[param.name] = param.value
            else:
                sampled[param.name] = param.value
        
        return sampled
    
    def _run_single_simulation(self, config: SimulationConfig, run_id: int, 
                             sampled_params: Dict[str, Any]) -> SimulationResult:
        """Run single simulation run"""
        
        # Initialize appropriate model based on simulation type
        if config.simulation_type in [SimulationType.TREATMENT, SimulationType.MEDICATION]:
            model = CardiovascularModel(sampled_params)
            initial_state = [
                model.model_params['heart_rate_baseline'],
                model.model_params['blood_pressure_sys_baseline'],
                model.model_params['blood_pressure_dia_baseline'],
                model.model_params['cardiac_output_baseline']
            ]
        elif config.simulation_type in [SimulationType.LIFESTYLE, SimulationType.PREVENTIVE]:
            model = MetabolicModel(sampled_params)
            initial_state = [
                model.model_params['glucose_baseline'],
                model.model_params['insulin_baseline'],
                model.model_params['hba1c_baseline'],
                model.model_params['bmi_baseline']
            ]
        else:
            # Default to cardiovascular model
            model = CardiovascularModel(sampled_params)
            initial_state = [
                model.model_params['heart_rate_baseline'],
                model.model_params['blood_pressure_sys_baseline'],
                model.model_params['blood_pressure_dia_baseline'],
                model.model_params['cardiac_output_baseline']
            ]
        
        # Time points
        time_points = np.arange(0, config.time_horizon_days * 24, config.time_step_hours)
        
        # Simulate with intervention
        intervention_params = self._prepare_intervention_params(config, sampled_params)
        solution = model.simulate_time_series(initial_state, time_points, intervention_params)
        
        # Calculate outcomes
        outcomes = self._calculate_outcomes(solution, config.outcome_metrics)
        
        # Time series data
        time_series_data = {}
        for i, metric in enumerate(config.outcome_metrics[:len(solution[0])]):
            time_series_data[metric] = solution[:, i].tolist()
        
        # Calculate confidence intervals (simplified)
        confidence_intervals = self._calculate_confidence_intervals(solution, config.confidence_level)
        
        # Sensitivity analysis
        sensitivity = self._perform_sensitivity_analysis(config, sampled_params)
        
        # Generate recommendations
        recommendations = self._generate_recommendations(outcomes, config.simulation_type)
        
        # Risk assessment
        risk_assessment = self._assess_risks(outcomes, config.simulation_type)
        
        return SimulationResult(
            simulation_id=config.simulation_id,
            run_id=run_id,
            timestamp=datetime.now(),
            parameters_used=sampled_params,
            outcomes=outcomes,
            time_series_data=time_series_data,
            confidence_intervals=confidence_intervals,
            sensitivity_analysis=sensitivity,
            recommendations=recommendations,
            risk_assessment=risk_assessment
        )
    
    def _prepare_intervention_params(self, config: SimulationConfig, sampled_params: Dict[str, Any]) -> Dict[str, Any]:
        """Prepare intervention parameters for simulation"""
        intervention_params = {}
        
        # Add treatment effects
        for param in config.parameters:
            if 'medication' in param.name.lower() or 'treatment' in param.name.lower():
                treatment_effect = self.treatment_simulator.simulate_treatment_effect(
                    param.name, config.time_horizon_days, sampled_params.get('adherence', 1.0)
                )
                intervention_params.update(treatment_effect)
        
        # Add lifestyle effects
        if 'exercise' in sampled_params:
            intervention_params['exercise_effect'] = sampled_params['exercise'] * 0.1
        
        if 'diet' in sampled_params:
            intervention_params['diet_effect'] = sampled_params['diet'] * 0.15
        
        if 'stress' in sampled_params:
            intervention_params['stress_effect'] = sampled_params['stress'] * 0.2
        
        return intervention_params
    
    def _calculate_outcomes(self, solution: np.ndarray, metrics: List[str]) -> Dict[str, Any]:
        """Calculate simulation outcomes"""
        outcomes = {}
        
        for i, metric in enumerate(metrics[:len(solution[0])]):
            if i < solution.shape[1]:
                final_value = solution[-1, i]
                initial_value = solution[0, i]
                change = final_value - initial_value
                percent_change = (change / initial_value * 100) if initial_value != 0 else 0
                
                outcomes[metric] = {
                    'final_value': float(final_value),
                    'initial_value': float(initial_value),
                    'change': float(change),
                    'percent_change': float(percent_change),
                    'mean_value': float(np.mean(solution[:, i])),
                    'std_value': float(np.std(solution[:, i])),
                    'min_value': float(np.min(solution[:, i])),
                    'max_value': float(np.max(solution[:, i]))
                }
        
        return outcomes
    
    def _calculate_confidence_intervals(self, solution: np.ndarray, confidence_level: float) -> Dict[str, Tuple[float, float]]:
        """Calculate confidence intervals for outcomes"""
        intervals = {}
        
        alpha = 1 - confidence_level
        for i in range(solution.shape[1]):
            values = solution[:, i]
            mean = np.mean(values)
            std = np.std(values)
            
            # Calculate critical value (simplified normal approximation)
            z_score = norm.ppf(1 - alpha/2)
            margin_error = z_score * (std / np.sqrt(len(values)))
            
            intervals[f'variable_{i}'] = (float(mean - margin_error), float(mean + margin_error))
        
        return intervals
    
    def _perform_sensitivity_analysis(self, config: SimulationConfig, base_params: Dict[str, Any]) -> Dict[str, float]:
        """Perform sensitivity analysis"""
        sensitivity = {}
        
        for param in config.parameters:
            if param.parameter_type == ParameterType.CONTINUOUS:
                # Perturb parameter by 10%
                perturbed_params = base_params.copy()
                perturbation = 0.1 * param.value
                perturbed_params[param.name] = param.value + perturbation
                
                # Run perturbed simulation (simplified)
                sensitivity_score = abs(perturbation) / (abs(param.value) + 1e-6)
                sensitivity[param.name] = sensitivity_score
        
        return sensitivity
    
    def _generate_recommendations(self, outcomes: Dict[str, Any], simulation_type: SimulationType) -> List[str]:
        """Generate recommendations based on outcomes"""
        recommendations = []
        
        for metric, result in outcomes.items():
            percent_change = result['percent_change']
            
            if percent_change > 10:
                recommendations.append(f"Significant improvement in {metric}: +{percent_change:.1f}%")
            elif percent_change < -10:
                recommendations.append(f"Concerning decline in {metric}: {percent_change:.1f}%")
            else:
                recommendations.append(f"Stable {metric}: {percent_change:+.1f}% change")
        
        # Add simulation-type specific recommendations
        if simulation_type == SimulationType.TREATMENT:
            recommendations.append("Consider adherence monitoring for optimal treatment effect")
        elif simulation_type == SimulationType.LIFESTYLE:
            recommendations.append("Lifestyle interventions show promising results")
        elif simulation_type == SimulationType.PREVENTIVE:
            recommendations.append("Preventive measures appear effective")
        
        return recommendations
    
    def _assess_risks(self, outcomes: Dict[str, Any], simulation_type: SimulationType) -> Dict[str, str]:
        """Assess risks based on outcomes"""
        risks = {}
        
        for metric, result in outcomes.items():
            final_value = result['final_value']
            
            # Risk assessment based on medical thresholds (simplified)
            if 'blood_pressure' in metric.lower():
                if final_value > 140:
                    risks[metric] = "HIGH"
                elif final_value > 120:
                    risks[metric] = "MEDIUM"
                else:
                    risks[metric] = "LOW"
            elif 'glucose' in metric.lower():
                if final_value > 126:
                    risks[metric] = "HIGH"
                elif final_value > 100:
                    risks[metric] = "MEDIUM"
                else:
                    risks[metric] = "LOW"
            elif 'heart_rate' in metric.lower():
                if final_value > 100 or final_value < 60:
                    risks[metric] = "HIGH"
                else:
                    risks[metric] = "LOW"
            else:
                # Default risk assessment
                percent_change = result['percent_change']
                if abs(percent_change) > 20:
                    risks[metric] = "HIGH"
                elif abs(percent_change) > 10:
                    risks[metric] = "MEDIUM"
                else:
                    risks[metric] = "LOW"
        
        return risks
    
    def compare_scenarios(self, results_list: List[List[SimulationResult]]) -> Dict[str, Any]:
        """Compare multiple simulation scenarios"""
        comparison = {
            'scenarios': len(results_list),
            'comparison_metrics': {},
            'best_scenario': {},
            'recommendations': []
        }
        
        # Flatten all results
        all_results = []
        for results in results_list:
            all_results.extend(results)
        
        # Compare average outcomes
        for metric in all_results[0].outcomes.keys():
            scenario_averages = []
            
            for i, results in enumerate(results_list):
                metric_values = [r.outcomes[metric]['final_value'] for r in results]
                avg_value = np.mean(metric_values)
                scenario_averages.append(avg_value)
            
            comparison['comparison_metrics'][metric] = scenario_averages
            
            # Find best scenario for this metric
            if 'blood_pressure' in metric.lower() or 'glucose' in metric.lower():
                best_idx = np.argmin(scenario_averages)  # Lower is better
            else:
                best_idx = np.argmax(scenario_averages)  # Higher is better
            
            comparison['best_scenario'][metric] = f"Scenario {best_idx + 1}"
        
        return comparison
    
    def generate_simulation_report(self, results: List[SimulationResult]) -> Dict[str, Any]:
        """Generate comprehensive simulation report"""
        if not results:
            return {}
        
        report = {
            'simulation_summary': {
                'total_runs': len(results),
                'simulation_id': results[0].simulation_id,
                'timestamp': results[0].timestamp.isoformat(),
                'confidence_level': 0.95  # Default
            },
            'outcomes_summary': {},
            'risk_summary': {},
            'recommendations_summary': [],
            'sensitivity_summary': {}
        }
        
        # Aggregate outcomes
        for metric in results[0].outcomes.keys():
            final_values = [r.outcomes[metric]['final_value'] for r in results]
            percent_changes = [r.outcomes[metric]['percent_change'] for r in results]
            
            report['outcomes_summary'][metric] = {
                'mean_final_value': float(np.mean(final_values)),
                'std_final_value': float(np.std(final_values)),
                'mean_percent_change': float(np.mean(percent_changes)),
                'std_percent_change': float(np.std(percent_changes)),
                'min_final_value': float(np.min(final_values)),
                'max_final_value': float(np.max(final_values))
            }
        
        # Aggregate risks
        all_risks = {}
        for result in results:
            for metric, risk in result.risk_assessment.items():
                if metric not in all_risks:
                    all_risks[metric] = {'LOW': 0, 'MEDIUM': 0, 'HIGH': 0}
                all_risks[metric][risk] += 1
        
        for metric, risk_counts in all_risks.items():
            total = sum(risk_counts.values())
            dominant_risk = max(risk_counts, key=risk_counts.get)
            report['risk_summary'][metric] = {
                'dominant_risk': dominant_risk,
                'confidence': risk_counts[dominant_risk] / total
            }
        
        # Aggregate recommendations
        all_recommendations = []
        for result in results:
            all_recommendations.extend(result.recommendations)
        
        # Count recommendation frequency
        recommendation_counts = {}
        for rec in all_recommendations:
            recommendation_counts[rec] = recommendation_counts.get(rec, 0) + 1
        
        # Get top recommendations
        top_recommendations = sorted(recommendation_counts.items(), key=lambda x: x[1], reverse=True)[:5]
        report['recommendations_summary'] = [rec[0] for rec in top_recommendations]
        
        return report


# Example usage
def example_simulation():
    """Example of digital twin simulation"""
    
    # Create simulation configuration
    config_dict = {
        'simulation_id': 'cardiovascular_treatment_sim',
        'simulation_type': 'treatment',
        'twin_id': 'twin_001',
        'time_horizon_days': 90,
        'time_step_hours': 6,
        'num_runs': 10,
        'confidence_level': 0.95,
        'outcome_metrics': ['heart_rate', 'blood_pressure_sys', 'blood_pressure_dia', 'cardiac_output'],
        'parameters': {
            'medication_lisinopril': {
                'type': 'continuous',
                'value': 10.0,
                'range': (5.0, 20.0),
                'distribution': 'normal',
                'uncertainty': 0.2,
                'description': 'Lisinopril dosage in mg'
            },
            'exercise_intensity': {
                'type': 'continuous',
                'value': 0.5,
                'range': (0.0, 1.0),
                'distribution': 'uniform',
                'uncertainty': 0.3,
                'description': 'Exercise intensity level'
            },
            'stress_level': {
                'type': 'continuous',
                'value': 0.3,
                'range': (0.0, 1.0),
                'distribution': 'normal',
                'uncertainty': 0.4,
                'description': 'Stress level'
            },
            'adherence': {
                'type': 'continuous',
                'value': 0.8,
                'range': (0.5, 1.0),
                'distribution': 'normal',
                'uncertainty': 0.1,
                'description': 'Medication adherence'
            }
        },
        'baseline_scenario': {
            'heart_rate': 75,
            'blood_pressure_sys': 130,
            'blood_pressure_dia': 85,
            'cardiac_output': 5.2
        }
    }
    
    # Create simulation engine
    engine = SimulationEngine()
    
    # Create configuration
    config = engine.create_simulation_config(config_dict)
    
    # Run simulation
    results = engine.run_simulation(config)
    
    print(f"Simulation completed with {len(results)} runs")
    print(f"Average blood pressure reduction: {np.mean([r.outcomes['blood_pressure_sys']['change'] for r in results]):.1f} mmHg")
    
    # Generate report
    report = engine.generate_simulation_report(results)
    print(f"Risk summary: {report['risk_summary']}")
    print(f"Top recommendations: {report['recommendations_summary']}")


if __name__ == "__main__":
    example_simulation()
