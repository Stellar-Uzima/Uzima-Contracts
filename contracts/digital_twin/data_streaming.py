"""
Real-time Data Streaming for Digital Twin Platform
Handles multi-modal data integration from wearables, EMR, genomics, and other sources
"""

import asyncio
import json
import logging
import time
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, asdict
from enum import Enum
import hashlib
import aiohttp
import websockets
from datetime import datetime, timedelta
import numpy as np

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class DataSource(Enum):
    """Data source types"""
    WEARABLE = "wearable"
    EMR = "emr"
    GENOMIC = "genomic"
    LAB_RESULTS = "lab_results"
    IMAGING = "imaging"
    PATIENT_REPORTED = "patient_reported"
    ENVIRONMENTAL = "environmental"
    MEDICATION = "medication"


class DataType(Enum):
    """Data types for different modalities"""
    VITAL_SIGNS = "vital_signs"
    ACTIVITY = "activity"
    SLEEP = "sleep"
    NUTRITION = "nutrition"
    LAB_VALUE = "lab_value"
    GENETIC_MARKER = "genetic_marker"
    IMAGING_DATA = "imaging_data"
    MEDICATION_ADHERENCE = "medication_adherence"
    SYMPTOM = "symptom"
    ENVIRONMENTAL_EXPOSURE = "environmental_exposure"


@dataclass
class DataPoint:
    """Individual data point from any source"""
    timestamp: datetime
    source: DataSource
    data_type: DataType
    patient_id: str
    value: Any
    unit: Optional[str]
    confidence: float  # 0.0 to 1.0
    metadata: Dict[str, Any]
    quality_score: float  # 0.0 to 1.0
    verification_hash: str


@dataclass
class StreamConfig:
    """Configuration for data stream"""
    stream_id: str
    source: DataSource
    data_type: DataType
    update_frequency: int  # seconds
    quality_threshold: float
    encryption_required: bool
    retention_days: int
    compression_enabled: bool


class DataValidator:
    """Validates incoming data points for quality and consistency"""
    
    def __init__(self):
        self.validation_rules = self._initialize_validation_rules()
    
    def _initialize_validation_rules(self) -> Dict[DataType, Dict]:
        """Initialize validation rules for different data types"""
        return {
            DataType.VITAL_SIGNS: {
                'heart_rate': {'min': 30, 'max': 200},
                'blood_pressure_systolic': {'min': 60, 'max': 250},
                'blood_pressure_diastolic': {'min': 30, 'max': 150},
                'temperature': {'min': 35.0, 'max': 42.0},
                'oxygen_saturation': {'min': 70, 'max': 100},
                'respiratory_rate': {'min': 8, 'max': 40}
            },
            DataType.ACTIVITY: {
                'steps': {'min': 0, 'max': 100000},
                'calories': {'min': 0, 'max': 10000},
                'distance': {'min': 0, 'max': 100},
                'active_minutes': {'min': 0, 'max': 1440}
            },
            DataType.LAB_VALUE: {
                'glucose': {'min': 20, 'max': 600},
                'cholesterol': {'min': 100, 'max': 400},
                'hemoglobin': {'min': 5, 'max': 20},
                'white_blood_cells': {'min': 2000, 'max': 50000}
            }
        }
    
    def validate_data_point(self, data_point: DataPoint) -> tuple[bool, List[str]]:
        """Validate a data point against rules"""
        errors = []
        
        # Check confidence threshold
        if data_point.confidence < 0.5:
            errors.append("Low confidence score")
        
        # Check quality score
        if data_point.quality_score < 0.7:
            errors.append("Low quality score")
        
        # Type-specific validation
        if isinstance(data_point.value, dict):
            for metric, value in data_point.value.items():
                if data_point.data_type in self.validation_rules:
                    rules = self.validation_rules[data_point.data_type]
                    if metric in rules:
                        rule = rules[metric]
                        if not (rule['min'] <= value <= rule['max']):
                            errors.append(f"{metric} out of range: {value}")
        
        return len(errors) == 0, errors
    
    def compute_quality_score(self, data_point: DataPoint) -> float:
        """Compute quality score for data point"""
        score = 1.0
        
        # Time consistency check
        now = datetime.now()
        time_diff = abs((now - data_point.timestamp).total_seconds())
        if time_diff > 3600:  # More than 1 hour old
            score -= 0.3
        
        # Source reliability
        source_reliability = {
            DataSource.EMR: 0.95,
            DataSource.GENOMIC: 0.90,
            DataSource.LAB_RESULTS: 0.95,
            DataSource.WEARABLE: 0.80,
            DataSource.PATIENT_REPORTED: 0.70
        }
        score *= source_reliability.get(data_point.source, 0.5)
        
        return max(0.0, score)


class DataStreamManager:
    """Manages real-time data streams from multiple sources"""
    
    def __init__(self):
        self.streams: Dict[str, StreamConfig] = {}
        self.active_connections: Dict[str, Any] = {}
        self.data_handlers: List[Callable] = []
        self.validator = DataValidator()
        self.buffer: Dict[str, List[DataPoint]] = {}
        self.running = False
        
    def register_stream(self, config: StreamConfig) -> bool:
        """Register a new data stream"""
        try:
            self.streams[config.stream_id] = config
            self.buffer[config.stream_id] = []
            logger.info(f"Registered stream: {config.stream_id}")
            return True
        except Exception as e:
            logger.error(f"Failed to register stream {config.stream_id}: {e}")
            return False
    
    async def start_streaming(self) -> None:
        """Start all registered data streams"""
        self.running = True
        
        # Start different stream types
        tasks = []
        
        for stream_id, config in self.streams.items():
            if config.source == DataSource.WEARABLE:
                tasks.append(asyncio.create_task(self._wearable_stream(stream_id, config)))
            elif config.source == DataSource.EMR:
                tasks.append(asyncio.create_task(self._emr_stream(stream_id, config)))
            elif config.source == DataSource.GENOMIC:
                tasks.append(asyncio.create_task(self._genomic_stream(stream_id, config)))
            elif config.source == DataSource.LAB_RESULTS:
                tasks.append(asyncio.create_task(self._lab_stream(stream_id, config)))
        
        # Wait for all streams
        await asyncio.gather(*tasks, return_exceptions=True)
    
    async def _wearable_stream(self, stream_id: str, config: StreamConfig) -> None:
        """Simulate wearable data stream"""
        logger.info(f"Starting wearable stream: {stream_id}")
        
        while self.running:
            try:
                # Simulate wearable data
                vital_signs = {
                    'heart_rate': np.random.normal(75, 10),
                    'steps': np.random.randint(0, 1000),
                    'calories': np.random.randint(100, 500),
                    'sleep_hours': np.random.normal(7.5, 1.5)
                }
                
                data_point = DataPoint(
                    timestamp=datetime.now(),
                    source=DataSource.WEARABLE,
                    data_type=DataType.VITAL_SIGNS,
                    patient_id="demo_patient",
                    value=vital_signs,
                    unit=None,
                    confidence=0.85,
                    metadata={'device': 'fitbit', 'firmware': 'v1.2.3'},
                    quality_score=0.80,
                    verification_hash=self._compute_hash(vital_signs)
                )
                
                await self._process_data_point(data_point)
                await asyncio.sleep(config.update_frequency)
                
            except Exception as e:
                logger.error(f"Error in wearable stream {stream_id}: {e}")
                await asyncio.sleep(5)
    
    async def _emr_stream(self, stream_id: str, config: StreamConfig) -> None:
        """Simulate EMR data stream"""
        logger.info(f"Starting EMR stream: {stream_id}")
        
        while self.running:
            try:
                # Simulate EMR data updates
                emr_data = {
                    'blood_pressure_systolic': np.random.normal(120, 15),
                    'blood_pressure_diastolic': np.random.normal(80, 10),
                    'temperature': np.random.normal(37.0, 0.5),
                    'medications': ['lisinopril', 'metformin'],
                    'diagnoses': ['hypertension', 'type 2 diabetes']
                }
                
                data_point = DataPoint(
                    timestamp=datetime.now(),
                    source=DataSource.EMR,
                    data_type=DataType.VITAL_SIGNS,
                    patient_id="demo_patient",
                    value=emr_data,
                    unit=None,
                    confidence=0.95,
                    metadata={'provider': 'Dr. Smith', 'facility': 'General Hospital'},
                    quality_score=0.90,
                    verification_hash=self._compute_hash(emr_data)
                )
                
                await self._process_data_point(data_point)
                await asyncio.sleep(config.update_frequency * 4)  # Less frequent updates
                
            except Exception as e:
                logger.error(f"Error in EMR stream {stream_id}: {e}")
                await asyncio.sleep(10)
    
    async def _genomic_stream(self, stream_id: str, config: StreamConfig) -> None:
        """Simulate genomic data stream (rare updates)"""
        logger.info(f"Starting genomic stream: {stream_id}")
        
        while self.running:
            try:
                # Genomic data updates are rare
                await asyncio.sleep(config.update_frequency * 100)  # Very infrequent
                
                genomic_data = {
                    'gene_variants': ['rs12345', 'rs67890'],
                    'risk_score': np.random.uniform(0.2, 0.8),
                    'pharmacogenomic_markers': ['CYP2C19*2', 'VKORC1']
                }
                
                data_point = DataPoint(
                    timestamp=datetime.now(),
                    source=DataSource.GENOMIC,
                    data_type=DataType.GENETIC_MARKER,
                    patient_id="demo_patient",
                    value=genomic_data,
                    unit=None,
                    confidence=0.99,
                    metadata={'lab': 'Genomics Lab', 'test_version': 'v2.1'},
                    quality_score=0.95,
                    verification_hash=self._compute_hash(genomic_data)
                )
                
                await self._process_data_point(data_point)
                
            except Exception as e:
                logger.error(f"Error in genomic stream {stream_id}: {e}")
                await asyncio.sleep(60)
    
    async def _lab_stream(self, stream_id: str, config: StreamConfig) -> None:
        """Simulate lab results stream"""
        logger.info(f"Starting lab results stream: {stream_id}")
        
        while self.running:
            try:
                lab_data = {
                    'glucose': np.random.normal(95, 20),
                    'cholesterol': np.random.normal(180, 30),
                    'hemoglobin': np.random.normal(14, 2),
                    'white_blood_cells': np.random.normal(7000, 2000)
                }
                
                data_point = DataPoint(
                    timestamp=datetime.now(),
                    source=DataSource.LAB_RESULTS,
                    data_type=DataType.LAB_VALUE,
                    patient_id="demo_patient",
                    value=lab_data,
                    unit='mg/dL',
                    confidence=0.95,
                    metadata={'lab': 'Clinical Lab', 'test_id': 'CBC001'},
                    quality_score=0.92,
                    verification_hash=self._compute_hash(lab_data)
                )
                
                await self._process_data_point(data_point)
                await asyncio.sleep(config.update_frequency * 8)  # Daily labs
                
            except Exception as e:
                logger.error(f"Error in lab stream {stream_id}: {e}")
                await asyncio.sleep(30)
    
    async def _process_data_point(self, data_point: DataPoint) -> None:
        """Process incoming data point"""
        try:
            # Validate data point
            is_valid, errors = self.validator.validate_data_point(data_point)
            
            if not is_valid:
                logger.warning(f"Invalid data point: {errors}")
                return
            
            # Update quality score
            data_point.quality_score = self.validator.compute_quality_score(data_point)
            
            # Add to buffer
            stream_id = self._get_stream_id_for_data(data_point)
            if stream_id and stream_id in self.buffer:
                self.buffer[stream_id].append(data_point)
                
                # Keep buffer size manageable
                if len(self.buffer[stream_id]) > 1000:
                    self.buffer[stream_id] = self.buffer[stream_id][-500:]
            
            # Notify handlers
            for handler in self.data_handlers:
                try:
                    await handler(data_point)
                except Exception as e:
                    logger.error(f"Error in data handler: {e}")
            
            logger.debug(f"Processed data point: {data_point.data_type} from {data_point.source}")
            
        except Exception as e:
            logger.error(f"Error processing data point: {e}")
    
    def _get_stream_id_for_data(self, data_point: DataPoint) -> Optional[str]:
        """Find appropriate stream ID for data point"""
        for stream_id, config in self.streams.items():
            if (config.source == data_point.source and 
                config.data_type == data_point.data_type):
                return stream_id
        return None
    
    def _compute_hash(self, data: Any) -> str:
        """Compute verification hash for data"""
        data_str = json.dumps(data, sort_keys=True, default=str)
        return hashlib.sha256(data_str.encode()).hexdigest()
    
    def add_data_handler(self, handler: Callable) -> None:
        """Add a data handler callback"""
        self.data_handlers.append(handler)
    
    def get_buffer_data(self, stream_id: str, limit: int = 100) -> List[DataPoint]:
        """Get buffered data for a stream"""
        if stream_id not in self.buffer:
            return []
        
        return self.buffer[stream_id][-limit:]
    
    def get_stream_statistics(self) -> Dict[str, Any]:
        """Get statistics for all streams"""
        stats = {}
        
        for stream_id, config in self.streams.items():
            buffer_size = len(self.buffer.get(stream_id, []))
            
            # Calculate average quality score
            if buffer_size > 0:
                avg_quality = sum(dp.quality_score for dp in self.buffer[stream_id]) / buffer_size
                last_update = self.buffer[stream_id][-1].timestamp
            else:
                avg_quality = 0.0
                last_update = None
            
            stats[stream_id] = {
                'source': config.source.value,
                'data_type': config.data_type.value,
                'buffer_size': buffer_size,
                'avg_quality_score': avg_quality,
                'last_update': last_update,
                'update_frequency': config.update_frequency
            }
        
        return stats
    
    def stop_streaming(self) -> None:
        """Stop all data streams"""
        self.running = False
        logger.info("Stopping data streaming")


class DataIntegrator:
    """Integrates multi-modal data for digital twin synchronization"""
    
    def __init__(self, stream_manager: DataStreamManager):
        self.stream_manager = stream_manager
        self.integration_rules = self._initialize_integration_rules()
        self.sync_callbacks: List[Callable] = []
        
        # Register as data handler
        stream_manager.add_data_handler(self._on_data_received)
    
    def _initialize_integration_rules(self) -> Dict[DataType, List[DataType]]:
        """Initialize data integration rules"""
        return {
            DataType.VITAL_SIGNS: [DataType.ACTIVITY, DataType.SLEEP],
            DataType.LAB_VALUE: [DataType.NUTRITION, DataType.MEDICATION_ADHERENCE],
            DataType.GENETIC_MARKER: [DataType.LAB_VALUE, DataType.SYMPTOM],
            DataType.MEDICATION_ADHERENCE: [DataType.VITAL_SIGNS, DataType.LAB_VALUE]
        }
    
    async def _on_data_received(self, data_point: DataPoint) -> None:
        """Handle incoming data point for integration"""
        try:
            # Check if this data point triggers integration
            related_types = self.integration_rules.get(data_point.data_type, [])
            
            if related_types:
                await self._perform_integration(data_point, related_types)
            
            # Notify sync callbacks
            for callback in self.sync_callbacks:
                await callback(data_point)
                
        except Exception as e:
            logger.error(f"Error in data integration: {e}")
    
    async def _perform_integration(self, trigger_data: DataPoint, related_types: List[DataType]) -> None:
        """Perform data integration based on trigger data"""
        # Find related data points from recent history
        related_data = []
        
        for stream_id, config in self.stream_manager.streams.items():
            if config.data_type in related_types:
                recent_data = self.stream_manager.get_buffer_data(stream_id, 50)
                related_data.extend(recent_data)
        
        if related_data:
            # Perform integration logic
            integrated_result = self._integrate_data_points(trigger_data, related_data)
            logger.info(f"Integrated {len(related_data)} related data points")
            
            # Store or forward integrated result
            await self._store_integrated_result(integrated_result)
    
    def _integrate_data_points(self, primary: DataPoint, related: List[DataPoint]) -> Dict[str, Any]:
        """Integrate multiple data points"""
        result = {
            'integration_timestamp': datetime.now().isoformat(),
            'primary_data': asdict(primary),
            'related_data_count': len(related),
            'integration_confidence': 0.0,
            'correlations': {},
            'anomalies': []
        }
        
        # Calculate integration confidence
        confidences = [dp.confidence for dp in related] + [primary.confidence]
        result['integration_confidence'] = np.mean(confidences)
        
        # Find correlations (simplified)
        if primary.data_type == DataType.VITAL_SIGNS and isinstance(primary.value, dict):
            for related_dp in related:
                if related_dp.data_type == DataType.ACTIVITY and isinstance(related_dp.value, dict):
                    # Correlate heart rate with activity
                    if 'heart_rate' in primary.value and 'steps' in related_dp.value:
                        correlation = self._calculate_correlation(
                            primary.value['heart_rate'], 
                            related_dp.value['steps']
                        )
                        result['correlations'][f"{primary.data_type.value}_{related_dp.data_type.value}"] = correlation
        
        return result
    
    def _calculate_correlation(self, x: float, y: float) -> float:
        """Simple correlation calculation (placeholder)"""
        # In a real implementation, this would use historical data
        return np.random.uniform(-0.5, 0.8)
    
    async def _store_integrated_result(self, result: Dict[str, Any]) -> None:
        """Store integrated result"""
        # In a real implementation, this would store to database or send to digital twin
        logger.info(f"Storing integrated result with confidence: {result['integration_confidence']}")
    
    def add_sync_callback(self, callback: Callable) -> None:
        """Add synchronization callback"""
        self.sync_callbacks.append(callback)


# Example usage
async def main():
    """Example of digital twin data streaming"""
    
    # Create stream manager
    stream_manager = DataStreamManager()
    
    # Register data streams
    streams = [
        StreamConfig(
            stream_id="wearable_vitals",
            source=DataSource.WEARABLE,
            data_type=DataType.VITAL_SIGNS,
            update_frequency=60,  # Every minute
            quality_threshold=0.7,
            encryption_required=True,
            retention_days=30,
            compression_enabled=True
        ),
        StreamConfig(
            stream_id="emr_data",
            source=DataSource.EMR,
            data_type=DataType.VITAL_SIGNS,
            update_frequency=300,  # Every 5 minutes
            quality_threshold=0.8,
            encryption_required=True,
            retention_days=365,
            compression_enabled=True
        ),
        StreamConfig(
            stream_id="lab_results",
            source=DataSource.LAB_RESULTS,
            data_type=DataType.LAB_VALUE,
            update_frequency=86400,  # Daily
            quality_threshold=0.9,
            encryption_required=True,
            retention_days=730,
            compression_enabled=True
        )
    ]
    
    for stream in streams:
        stream_manager.register_stream(stream)
    
    # Create data integrator
    integrator = DataIntegrator(stream_manager)
    
    # Add custom data handler
    async def custom_handler(data_point: DataPoint):
        print(f"Received {data_point.data_type.value} from {data_point.source.value}: {data_point.value}")
    
    stream_manager.add_data_handler(custom_handler)
    
    # Start streaming
    print("Starting digital twin data streaming...")
    streaming_task = asyncio.create_task(stream_manager.start_streaming())
    
    # Run for demonstration
    await asyncio.sleep(30)  # Run for 30 seconds
    
    # Stop streaming
    stream_manager.stop_streaming()
    await streaming_task
    
    # Show statistics
    stats = stream_manager.get_stream_statistics()
    print("\nStream Statistics:")
    for stream_id, stat in stats.items():
        print(f"{stream_id}: {stat['buffer_size']} points, avg quality: {stat['avg_quality_score']:.2f}")


if __name__ == "__main__":
    asyncio.run(main())
