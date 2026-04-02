# Outbreak Detection and Public Health Threat Prediction

This module builds multi-source, real-time outbreak detection with time-series modeling, geographical spread prediction, and a calibrated risk scoring system. It integrates with existing on-chain contracts `predictive_analytics` and `public_health_surveillance`.

## Components

- Ingestion: EMR, social media, environmental sources (+ streaming hooks)
- Modeling: time-series (SARIMAX/Prophet), anomaly scoring, spatial spread
- Scoring: unified outbreak risk score with calibration and alert thresholds
- Geo: metapopulation spread using mobility/adjacency and SEIR-like dynamics
- Retraining: batch/online retraining and scheduling
- API: endpoints for ingest, score, monitor
- Evaluation: early detection lead time and false positive rate metrics

## Quickstart

1) Python 3.10+ and virtualenv
2) `pip install -r outbreak/requirements.txt`
3) Configure `.env` for any source credentials and contract endpoints (if used)
4) Run API: `uvicorn outbreak.api.server:app --host 0.0.0.0 --port 8090`

## Acceptance Criteria Mapping

- Time-series analysis: implemented in `models/timeseries.py` with SARIMAX/Prophet and residual anomalies
- Multiple sources: ingestion adapters in `ingestion/`
- Risk scoring: `scoring/risk.py` with calibration and thresholds
- Geographical spread: `models/geo.py` spatial metapopulation projection
- Early detection: evaluation harness `evaluation/metrics.py` supports 7–14 day lead time analysis
- False positive rate: evaluation harness computes FPR and calibration curves
- Integration: `integration/onchain.py` stubs to write to `predictive_analytics` and `public_health_surveillance`
- Real-time: `ingestion/stream.py` hook + API endpoints for near-real-time updates
- Retraining: `pipeline/retrain.py` with schedule-friendly CLI

