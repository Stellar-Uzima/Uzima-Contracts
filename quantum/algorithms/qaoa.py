from typing import Tuple

import networkx as nx
import numpy as np
from qiskit import QuantumCircuit
from qiskit.circuit import ParameterVector
from qiskit.quantum_info import SparsePauliOp
from qiskit_algorithms import QAOA
from qiskit_algorithms.optimizers import COBYLA
from qiskit_algorithms.utils import algorithm_globals
from qiskit.primitives import Estimator


def maxcut_hamiltonian(g: nx.Graph) -> SparsePauliOp:
	n = g.number_of_nodes()
	paulis = []
	coeffs = []
	for i, j, data in g.edges(data=True):
		w = data.get("weight", 1.0)
		z = ["I"] * n
		z[i] = "Z"
		z[j] = "Z"
		paulis.append("".join(reversed(z)))
		coeffs.append(0.25 * w)
	# Constant shift and linear terms omitted as they don't affect argmin structure
	return SparsePauliOp.from_list(list(zip(paulis, coeffs)))


def run_qaoa_maxcut(g: nx.Graph, reps: int = 1, seed: int = 42) -> Tuple[float, np.ndarray]:
	algorithm_globals.random_seed = seed
	ham = maxcut_hamiltonian(g)
	estimator = Estimator()
	qaoa = QAOA(estimator=estimator, optimizer=COBYLA(maxiter=100), reps=reps)
	res = qaoa.compute_minimum_eigenvalue(operator=ham)
	energy = float(res.eigenvalue.real)
	return energy, res.optimal_point


if __name__ == "__main__":
	# Small Max-Cut instance (e.g., healthcare resource conflict graph)
	g = nx.Graph()
	g.add_weighted_edges_from([(0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0)])
	energy, params = run_qaoa_maxcut(g, reps=2)
	print("QAOA energy:", energy)
	print("params:", params)

