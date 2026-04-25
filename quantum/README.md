# Quantum Computing for Healthcare Optimization and Drug Discovery

This module provides a practical scaffold to explore and integrate quantum algorithms for healthcare and drug discovery, including circuit simulation, quantum optimization (QAOA, VQE), quantum machine learning, basic quantum cryptography (BB84), hybrid classical-quantum workflows, and platform integrations (IBM Q via Qiskit, Google Quantum AI via Cirq).

## Components

- Platforms: `qiskit_client.py`, `cirq_client.py`
- Simulation: statevector and sampler runners
- Optimization: QAOA for QUBO/Max-Cut, VQE for small molecular Hamiltonians
- QML: variational classifier and kernel methods (via PennyLane and/or Qiskit ML)
- Cryptography: BB84 protocol simulation and simple QRNG
- Workflows: hybrid pipelines combining classical optimizers with quantum circuits
- Benchmarks: side-by-side comparisons against classical baselines
- Demos: healthcare scheduling (QUBO), molecular toy model (VQE), toxicity classification (QML)

## Quickstart

1) Python 3.10+ and a virtualenv
2) `pip install -r quantum/requirements.txt`
3) Optional: set tokens in environment for cloud access:
   - IBM: `QISKIT_IBM_TOKEN=...`
   - Google: see Cirq/AQT/Cloud doc as applicable

### Run Local Simulations
- QAOA Max-Cut demo:
  - `python -m quantum.algorithms.qaoa`
- VQE toy molecule demo:
  - `python -m quantum.algorithms.vqe`
- QML variational classifier:
  - `python -m quantum.qml.variational_classifier`
- BB84 protocol:
  - `python -m quantum.crypto.bb84`

### Benchmarks and Comparisons
- `python -m quantum.benchmarks.compare_qaoa_classical`

## Notes on Quantum Advantage
True quantum advantage is hardware and problem-size dependent. We include comparison harnesses and identify regimes where quantum approaches may have structural benefits, but on small simulators this often does not show runtime advantage. Use this as a research and integration scaffold.

