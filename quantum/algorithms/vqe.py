from typing import Tuple

import numpy as np
from qiskit_algorithms import VQE
from qiskit_algorithms.optimizers import SLSQP
from qiskit.primitives import Estimator
from qiskit.circuit.library import TwoLocal
from qiskit.quantum_info import SparsePauliOp


def toy_molecule_hamiltonian() -> SparsePauliOp:
	# Simple 2-qubit Hamiltonian example (H2 minimal basis toy)
	# H = a * (Z0) + b * (Z1) + c * (Z0Z1) + d * (X0X1)
	a, b, c, d = -1.0523732, 0.39793742, -0.0112801, 0.18093119
	return SparsePauliOp.from_list(
		[
			("ZI", a),
			("IZ", b),
			("ZZ", c),
			("XX", d),
		]
	)


def run_vqe(ansatz_layers: int = 2) -> Tuple[float, np.ndarray]:
	ham = toy_molecule_hamiltonian()
	ansatz = TwoLocal(num_qubits=2, rotation_blocks="ry", entanglement_blocks="cx", entanglement="full", reps=ansatz_layers)
	estimator = Estimator()
	optimizer = SLSQP(maxiter=200)
	vqe = VQE(estimator=estimator, ansatz=ansatz, optimizer=optimizer)
	res = vqe.compute_minimum_eigenvalue(operator=ham)
	return float(res.eigenvalue.real), res.optimal_point


if __name__ == "__main__":
	e, params = run_vqe(2)
	print("VQE energy:", e)
	print("params:", params)

