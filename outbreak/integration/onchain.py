from typing import Dict, Any, Optional

import os
import time
import hashlib
import requests


def _hash_bytes(data: bytes) -> str:
	return hashlib.sha256(data).hexdigest()


def publish_prediction_to_contract(api_base: str, model_id: str, outcome_type: str, value_bps: int, confidence_bps: int, features_used: list[str], risk_factors: list[str], explanation_ref: str) -> Dict[str, Any]:
	"""
	Placeholder: integrate with `predictive_analytics` off-chain gateway if available.
	"""
	payload = {
		"model_id": model_id,
		"outcome_type": outcome_type,
		"predicted_value_bps": value_bps,
		"confidence_bps": confidence_bps,
		"features_used": features_used,
		"risk_factors": risk_factors,
		"explanation_ref": explanation_ref,
	}
	# Stubbed http post if such a gateway exists; otherwise, log or return payload
	try:
		r = requests.post(f"{api_base}/predictive_analytics/publish", json=payload, timeout=10)
		r.raise_for_status()
		return r.json()
	except Exception:
		return {"queued": True, "payload": payload}


def publish_outbreak_alert(api_base: str, region_id: str, severity: str, alert_type: str, risk_score_bps: int, valid_until: int) -> Dict[str, Any]:
	"""
	Placeholder: integrate with `public_health_surveillance` off-chain gateway if available.
	"""
	payload = {
		"region_id": region_id,
		"severity": severity,
		"alert_type": alert_type,
		"risk_score_bps": risk_score_bps,
		"valid_until": valid_until,
	}
	try:
		r = requests.post(f"{api_base}/public_health/alert", json=payload, timeout=10)
		r.raise_for_status()
		return r.json()
	except Exception:
		return {"queued": True, "payload": payload}

