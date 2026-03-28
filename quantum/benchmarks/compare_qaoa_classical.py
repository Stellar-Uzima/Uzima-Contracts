from typing import Tuple
import time
import networkx as nx
import numpy as np
from quantum.algorithms.qaoa import run_qaoa_maxcut


def classical_maxcut_greedy(g: nx.Graph) -> Tuple[float, np.ndarray]:
	# Simple greedy for baseline
	nodes = list(g.nodes)
	assign = {n: 0 for n in nodes}
	improved = True
	while improved:
		improved = False
		for n in nodes:
			cur = assign[n]
			assign[n] = 1 - cur
			# compute cut weight
			cw_flip = 0.0
			for u, v, d in g.edges(data=True):
				w = d.get("weight", 1.0)
				cw_flip += w if assign[u] != assign[v] else 0.0
			assign[n] = cur
			cw = 0.0
			for u, v, d in g.edges(data=True):
				w = d.get("weight", 1.0)
				cw += w if assign[u] != assign[v] else 0.0
			if cw_flip > cw:
				assign[n] = 1 - cur
				improved = True
	return float(sum(d.get("weight", 1.0) for u, v, d in g.edges(data=True) if assign[u] != assign[v])), np.array([assign[n] for n in nodes])


if __name__ == "__main__":
	# Random graph
	g = nx.gnp_random_graph(8, 0.5, seed=42)
	for u, v in g.edges:
		g[u][v]["weight"] = 1.0

	t0 = time.perf_counter()
	classical_val, _ = classical_maxcut_greedy(g)
	t_classical = (time.perf_counter() - t0) * 1000.0

	t1 = time.perf_counter()
	q_energy, _ = run_qaoa_maxcut(g, reps=1)
	t_quantum = (time.perf_counter() - t1) * 1000.0

	print("Classical greedy cut:", classical_val, "time_ms:", round(t_classical, 2))
	print("QAOA energy (proxy):", round(q_energy, 4), "time_ms:", round(t_quantum, 2))
	print("Note: Energy is not directly comparable to cut value; use as heuristic. True advantage requires hardware and larger instances.")

