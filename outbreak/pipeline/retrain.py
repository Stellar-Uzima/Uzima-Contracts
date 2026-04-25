from typing import Dict, Any

import argparse
import pandas as pd
from outbreak.models.timeseries import fit_sarimax, fit_prophet


def retrain_series(input_csv: str, out_dir: str) -> Dict[str, Any]:
	df = pd.read_csv(input_csv, parse_dates=["date"])
	results = {}
	for (region, syndrome), sub in df.groupby(["region_id", "syndrome"]):
		series = sub.set_index("date")["cases"].asfreq("D").fillna(0)
		sarimax_model = fit_sarimax(series)
		# prophet requires 'ds','y'
		prophet_df = series.rename("y").reset_index().rename(columns={"date": "ds"})
		try:
			prophet_model = fit_prophet(prophet_df)
			results[(region, syndrome)] = {"sarimax": True, "prophet": True}
		except Exception:
			results[(region, syndrome)] = {"sarimax": True, "prophet": False}
	return {"trained": len(results), "models": results}


if __name__ == "__main__":
	parser = argparse.ArgumentParser()
	parser.add_argument("--input_csv", required=True)
	parser.add_argument("--out_dir", required=False, default="./models")
	args = parser.parse_args()
	print(retrain_series(args.input_csv, args.out_dir))

