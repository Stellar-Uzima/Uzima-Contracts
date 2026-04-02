from typing import Dict, Tuple, Optional

import pandas as pd
import numpy as np
from statsmodels.tsa.statespace.sarimax import SARIMAX
from prophet import Prophet


def fit_sarimax(series: pd.Series, order=(1, 1, 1), seasonal_order=(0, 1, 1, 7)) -> object:
	model = SARIMAX(series, order=order, seasonal_order=seasonal_order, enforce_stationarity=False, enforce_invertibility=False)
	return model.fit(disp=False)


def forecast_sarimax(model: object, steps: int = 14) -> pd.Series:
	return model.forecast(steps=steps)


def fit_prophet(df: pd.DataFrame) -> Prophet:
	"""
	df with columns: ds, y
	"""
	m = Prophet(weekly_seasonality=True, yearly_seasonality=True, daily_seasonality=False)
	m.fit(df)
	return m


def forecast_prophet(model: Prophet, periods: int = 14, freq: str = "D") -> pd.DataFrame:
	future = model.make_future_dataframe(periods=periods, freq=freq)
	return model.predict(future)


def anomaly_scores(actual: pd.Series, predicted: pd.Series) -> pd.Series:
	"""
	Compute standardized residuals as anomaly score.
	"""
	resid = actual.reindex(predicted.index) - predicted
	std = resid.rolling(window=14, min_periods=7).std().replace(0, np.nan)
	z = resid / std
	return z.fillna(0.0).abs()

