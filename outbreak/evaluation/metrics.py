from typing import Dict, Tuple

import numpy as np
import pandas as pd


def compute_false_positive_rate(y_true_alert: pd.Series, y_pred_alert: pd.Series) -> float:
	"""
	Both series indexed by date with 0/1 values.
	"""
	y_true = y_true_alert.reindex(y_pred_alert.index).fillna(0).astype(int)
	y_pred = y_pred_alert.fillna(0).astype(int)
	fp = int(((y_pred == 1) & (y_true == 0)).sum())
	tn = int(((y_pred == 0) & (y_true == 0)).sum())
	if fp + tn == 0:
		return 0.0
	return float(fp / (fp + tn))


def lead_time_days(y_true_onset: pd.Series, y_pred_alert: pd.Series) -> float:
	"""
	Compute mean lead time (days) for detected events: difference between first true onset and first predicted alert prior to onset.
	Assumes y_true_onset has value 1 on first onset day only.
	"""
	idx = y_true_onset.index
	try:
		onset_day = idx[y_true_onset == 1][0]
	except IndexError:
		return 0.0
	alert_days = idx[y_pred_alert == 1]
	if len(alert_days) == 0:
		return 0.0
	prior_alerts = alert_days[alert_days <= onset_day]
	if len(prior_alerts) == 0:
		return 0.0
	return float((onset_day - prior_alerts[0]).days)

