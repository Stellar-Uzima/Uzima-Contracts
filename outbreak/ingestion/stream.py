from typing import Dict, Any

import datetime as dt
import pandas as pd


class StreamBuffer:
	def __init__(self):
		self.frames: list[pd.DataFrame] = []

	def add_event(self, payload: Dict[str, Any]) -> None:
		"""
		payload: {source: str, date: ISO, region_id: str, metric: str, value: float}
		"""
		df = pd.DataFrame([payload])
		df["date"] = pd.to_datetime(df["date"])
		self.frames.append(df)

	def snapshot(self) -> pd.DataFrame:
		if not self.frames:
			return pd.DataFrame(columns=["source", "date", "region_id", "metric", "value"])
		return pd.concat(self.frames, ignore_index=True)

