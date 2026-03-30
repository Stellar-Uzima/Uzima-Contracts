from typing import Optional

import os
from qiskit import QuantumCircuit
from qiskit.primitives import StatevectorSampler, Estimator
from qiskit.quantum_info import Statevector
from qiskit_ibm_runtime import QiskitRuntimeService


class QiskitClient:
	def __init__(self, token: Optional[str] = None):
		self._service = None
		self._token = token or os.getenv("QISKIT_IBM_TOKEN")
		if self._token:
			try:
				self._service = QiskitRuntimeService(channel="ibm_quantum", token=self._token)
			except Exception:
				self._service = None
		self._sampler = StatevectorSampler()
		self._estimator = Estimator()

	def run_statevector(self, circuit: QuantumCircuit):
		sv = Statevector.from_instruction(circuit)
		return sv

	def sample(self, circuit: QuantumCircuit, shots: int = 1024):
		job = self._sampler.run([circuit], shots=shots)
		return job.result().quasi_dists[0]

	def estimator(self) -> Estimator:
		return self._estimator

	def service(self) -> Optional[QiskitRuntimeService]:
		return self._service

