from typing import Dict

import numpy as np
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector


def simulate_statevector(circuit: QuantumCircuit) -> np.ndarray:
	sv = Statevector.from_instruction(circuit)
	return sv.data


def measure_counts(circuit: QuantumCircuit, shots: int = 1024) -> Dict[str, int]:
	circ = circuit.copy()
	if not circ.num_clbits:
		circ.measure_all()
	from qiskit.primitives import StatevectorSampler
	sampler = StatevectorSampler()
	res = sampler.run([circ], shots=shots).result().quasi_dists[0]
	# quasi probs to counts (approx)
	counts = {k: int(v * shots) for k, v in res.items()}
	return counts

