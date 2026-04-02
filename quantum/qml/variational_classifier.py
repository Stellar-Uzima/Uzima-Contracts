from typing import Tuple

import numpy as np
import pennylane as qml
from sklearn.datasets import make_classification
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score


def generate_toy_data(n_samples: int = 200, n_features: int = 2, n_informative: int = 2, random_state: int = 42):
	X, y = make_classification(
		n_samples=n_samples,
		n_features=n_features,
		n_informative=n_informative,
		n_redundant=0,
		n_clusters_per_class=1,
		class_sep=1.2,
		random_state=random_state,
	)
	return X.astype(np.float32), y.astype(np.int64)


def variational_classifier_demo() -> Tuple[float, float]:
	X, y = generate_toy_data()
	X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.25, random_state=42)

	n_qubits = 2
	dev = qml.device("default.qubit", wires=n_qubits)

	def feature_map(x):
		for i in range(n_qubits):
			qml.RX(x[i % x.shape[-1]], wires=i)
		qml.CZ(wires=[0, 1])

	weights = qml.numpy.array(np.random.uniform(-0.1, 0.1, size=(2, n_qubits)), requires_grad=True)

	@qml.qnode(dev)
	def circuit(x, w):
		feature_map(x)
		for i in range(n_qubits):
			qml.RY(w[0, i], wires=i)
		qml.CZ(wires=[0, 1])
		for i in range(n_qubits):
			qml.RY(w[1, i], wires=i)
		return qml.expval(qml.PauliZ(0))

	def predict_batch(Xb, w):
		scores = [circuit(x, w) for x in Xb]
		return (np.array(scores) > 0.0).astype(int)

	opt = qml.GradientDescentOptimizer(stepsize=0.1)
	for _ in range(50):
		idx = np.random.choice(len(X_train), size=16)
		Xb, yb = X_train[idx], y_train[idx]

		def loss(w):
			preds = predict_batch(Xb, w)
			return ((preds - yb) ** 2).mean()

		weights = opt.step(loss, weights)

	train_acc = accuracy_score(y_train, predict_batch(X_train, weights))
	test_acc = accuracy_score(y_test, predict_batch(X_test, weights))
	return float(train_acc), float(test_acc)


if __name__ == "__main__":
	train_acc, test_acc = variational_classifier_demo()
	print("Train acc:", train_acc, "Test acc:", test_acc)

