from typing import Callable, Dict, Any, Tuple

import numpy as np


def classical_optimizer(objective: Callable[[np.ndarray], float], x0: np.ndarray, steps: int = 100, lr: float = 0.1) -> Tuple[np.ndarray, float]:
	x = x0.copy()
	for _ in range(steps):
		# finite-diff gradient
		grad = np.zeros_like(x)
		eps = 1e-3
		fx = objective(x)
		for i in range(len(x)):
			xx = x.copy()
			xx[i] += eps
			grad[i] = (objective(xx) - fx) / eps
		x -= lr * grad
	return x, float(objective(x))


def hybrid_minimize(q_objective: Callable[[np.ndarray], float], init_params: np.ndarray) -> Dict[str, Any]:
	best_params, best_val = classical_optimizer(q_objective, init_params, steps=50, lr=0.2)
	return {"params": best_params.tolist(), "value": best_val}

