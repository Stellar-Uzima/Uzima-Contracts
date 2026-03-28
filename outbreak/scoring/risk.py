from typing import Dict

import numpy as np
import pandas as pd


def calibrate_scores(z_anomaly: pd.Series, social_signal: pd.Series | None = None, env_risk: pd.Series | None = None) -> pd.Series:
	"""
	Fuse anomaly magnitude with auxiliary signals into a [0,1] risk score.
	"""
	score = (np.tanh(z_anomaly / 3.0) + 1.0) / 2.0  # 0..1
	if social_signal is not None:
		score = 0.7 * score + 0.3 * social_signal.reindex(score.index).fillna(0.0)
	if env_risk is not None:
		score = 0.8 * score + 0.2 * env_risk.reindex(score.index).fillna(0.0)
	return score.clip(0.0, 1.0)


def threshold_alerts(score: pd.Series, threshold: float = 0.7, min_duration_days: int = 2) -> pd.Series:
	"""
	Binary alert if score above threshold for min_duration_days (rolling).
	"""
	roll = score.rolling(window=min_duration_days, min_periods=min_duration_days).mean()
	return (roll >= threshold).astype(int)

