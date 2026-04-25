from typing import Dict, Iterable, List

import pandas as pd


def load_emr_dataframe(path: str) -> pd.DataFrame:
	"""
	Load EMR-like time-series counts per region and syndrome.
	Expected columns: date, region_id, syndrome, cases
	"""
	df = pd.read_csv(path, parse_dates=["date"])
	return df


def normalize_emr(df: pd.DataFrame) -> pd.DataFrame:
	df = df.copy()
	df["cases"] = df["cases"].clip(lower=0).astype(int)
	return df

