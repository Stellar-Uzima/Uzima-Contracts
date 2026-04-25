from typing import Any, Dict

from fastapi import FastAPI, Query
from pydantic import BaseModel
import pandas as pd

from outbreak.ingestion.stream import StreamBuffer
from outbreak.models.timeseries import fit_sarimax, forecast_sarimax, anomaly_scores
from outbreak.scoring.risk import calibrate_scores, threshold_alerts


app = FastAPI(title="Outbreak Detection API")
stream = StreamBuffer()


class IngestEvent(BaseModel):
	source: str
	date: str
	region_id: str
	metric: str
	value: float


@app.get("/healthz")
def healthz() -> Dict[str, Any]:
	return {"status": "ok"}


@app.post("/ingest/event")
def ingest_event(evt: IngestEvent) -> Dict[str, Any]:
	stream.add_event(evt.model_dump())
	return {"accepted": True}


@app.get("/score/region")
def score_region(region_id: str, syndrome: str = Query("all"), horizon_days: int = 14, threshold: float = 0.7) -> Dict[str, Any]:
	df = stream.snapshot()
	if df.empty:
		return {"score": [], "alerts": []}
	# Build synthetic series: sum EMR 'cases' metric if available, else aggregate any
	sub = df[(df["region_id"] == region_id)]
	if sub.empty:
		return {"score": [], "alerts": []}
	series = sub.groupby("date")["value"].sum().asfreq("D").fillna(0.0)
	model = fit_sarimax(series)
	pred = forecast_sarimax(model, steps=horizon_days)
	# align last known window for anomaly
	last_pred = model.get_prediction(start=series.index[-min(len(series), 28)]).predicted_mean
	z = anomaly_scores(series.iloc[-len(last_pred):], last_pred)
	score = calibrate_scores(z)
	alerts = threshold_alerts(score, threshold=threshold)
	return {
		"score": [{"date": d.strftime("%Y-%m-%d"), "score": float(s)} for d, s in score.items()],
		"alerts": [{"date": d.strftime("%Y-%m-%d"), "alert": int(a)} for d, a in alerts.items()],
	}

