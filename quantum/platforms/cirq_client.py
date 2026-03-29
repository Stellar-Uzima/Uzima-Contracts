import cirq
from typing import Dict


class CirqClient:
	def __init__(self):
		self.sim = cirq.Simulator()

	def run(self, circuit: cirq.Circuit, repetitions: int = 1000) -> Dict[str, int]:
		result = self.sim.run(circuit, repetitions=repetitions)
		# Flatten to counts
		# Note: for multi-qubit, collect bitstrings
		hist = result.histogram(key=list(result.measurements.keys())[0]) if result.measurements else {}
		return dict(hist)

