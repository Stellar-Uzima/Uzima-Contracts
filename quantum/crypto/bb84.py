from typing import List, Tuple

import numpy as np


def bb84_simulate(n_bits: int = 128, eavesdrop_rate: float = 0.0, seed: int = 42) -> Tuple[List[int], List[int], float]:
	"""
	Simulate BB84 key exchange and return (alice_key, bob_key, qber).
	"""
	rng = np.random.default_rng(seed)
	alice_bits = rng.integers(0, 2, size=n_bits)
	alice_bases = rng.integers(0, 2, size=n_bits)  # 0=rectilinear, 1=diagonal
	bob_bases = rng.integers(0, 2, size=n_bits)

	# Eavesdropper Eve measures with probability eavesdrop_rate using random bases
	eve_bases = rng.integers(0, 2, size=n_bits)
	eve_active = rng.random(n_bits) < eavesdrop_rate

	def transmit(bit, basis, eve_act, eve_basis, bob_basis):
		# If Eve measures in wrong basis, she randomizes the bit for Bob
		if eve_act:
			if eve_basis == basis:
				measured = bit
			else:
				measured = rng.integers(0, 2)
			# Eve resends measured in her basis; Bob receives accordingly
			prep_bit = measured
			prep_basis = eve_basis
		else:
			prep_bit = bit
			prep_basis = basis
		# Bob measures
		if bob_basis == prep_basis:
			return prep_bit
		else:
			return rng.integers(0, 2)

	bob_bits = [
		transmit(alice_bits[i], alice_bases[i], bool(eve_active[i]), eve_bases[i], bob_bases[i]) for i in range(n_bits)
	]

	# Sifting: keep positions where bases match
	mask = alice_bases == bob_bases
	alice_key = alice_bits[mask]
	bob_key = np.array(bob_bits)[mask]
	if len(alice_key) == 0:
		return [], [], 0.0
	qber = float((alice_key != bob_key).mean())
	return alice_key.tolist(), bob_key.tolist(), qber


if __name__ == "__main__":
	a, b, qber = bb84_simulate(256, eavesdrop_rate=0.1)
	print("Key length:", len(a), "QBER:", qber)

