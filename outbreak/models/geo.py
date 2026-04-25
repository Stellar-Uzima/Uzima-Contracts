from typing import Dict, Tuple

import numpy as np
import pandas as pd
import networkx as nx


def simulate_spread(initial_cases: pd.Series, adjacency: nx.Graph, beta: float = 0.2, gamma: float = 0.1, days: int = 14) -> pd.DataFrame:
	"""
	Simple metapopulation SIR-like spread using adjacency as mobility proxy.
	initial_cases: index=region_id, value=cases
	Returns DataFrame [date, region_id, projected_cases]
	"""
	regions = list(initial_cases.index)
	state = initial_cases.to_dict()
	records = []
	for t in range(days):
		next_state = {r: max(0.0, float(state.get(r, 0.0))) for r in regions}
		for r in regions:
			influx = 0.0
			for nbr in adjacency.neighbors(r):
				influx += 0.05 * state.get(nbr, 0.0)  # simple mobility leakage
			new_cases = beta * (state.get(r, 0.0) + influx) - gamma * state.get(r, 0.0)
			next_state[r] = max(0.0, state.get(r, 0.0) + new_cases)
		state = next_state
		for r in regions:
			records.append({"day": t + 1, "region_id": r, "projected_cases": state[r]})
	return pd.DataFrame.from_records(records)

