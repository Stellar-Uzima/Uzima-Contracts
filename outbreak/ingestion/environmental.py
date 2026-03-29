import pandas as pd


def load_environmental(path: str) -> pd.DataFrame:
	"""
	Load environmental covariates per region_id and date.
	Expected: date, region_id, temperature_c, humidity_pct, air_quality_idx
	"""
	df = pd.read_csv(path, parse_dates=["date"])
	return df

