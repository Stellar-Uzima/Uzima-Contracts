from typing import Dict

import pandas as pd


def load_social_signals(path: str) -> pd.DataFrame:
	"""
	Load social media symptom signals per region_id and date.
	Expected: date, region_id, signal_strength (0..1)
	"""
	df = pd.read_csv(path, parse_dates=["date"])
	df["signal_strength"] = df["signal_strength"].clip(0.0, 1.0)
	return df

