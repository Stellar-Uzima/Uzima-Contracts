# Multi-Region Deployment Simulation and Failover Tests

This document describes the multi-region deployment simulation and failover testing framework for Uzima DR contracts.

## Overview

The simulation framework provides offline testing capabilities for multi-region deployment scenarios, failover behavior, and recovery procedures without requiring actual network deployment.

## Components

### 1. Simulation Script (`scripts/simulate_multi_region_deploy.sh`)

A bash-based simulation runner that tests deployment scenarios and failover behavior:

```bash
./scripts/simulate_multi_region_deploy.sh
```

**Test categories:**
- Contract deployment order and initialization
- Region health monitoring
- Single region failure and failover
- Cascading failure scenarios
- RTO compliance verification
- Data replication simulation
- Replica lag detection

### 2. Contract Simulation Modules

#### `contracts/multi_region_orchestrator/src/simulation.rs`

Provides simulation types and functions for the orchestrator:

- **`DeploymentScenario`** - Defines a deployment simulation scenario
- **`FailoverScenario`** - Defines a failover test scenario
- **`SimulationResult`** - Records simulation execution results
- **`DeploymentSimulator`** - Core simulation engine

#### `contracts/regional_node_manager/src/simulation.rs`

Provides simulation types and functions for node management:

- **`NodeFailureScenario`** - Defines node failure test scenarios
- **`HealthCheckSimulation`** - Simulates health check execution
- **`ReplicationSimulation`** - Simulates data replication
- **`NodeSimulator`** - Core node simulation engine

## Running Simulations

### Quick Simulation
```bash
./scripts/simulate_multi_region_deploy.sh
```

### Contract Tests
```bash
cargo test -p multi-region-orchestrator -- simulation
cargo test -p regional-node-manager -- simulation
```

## CI

`.github/workflows/multi-region-dr-simulation.yml` runs
`./scripts/simulate_multi_region_deploy.sh` on every push/PR touching the
simulation script, `config/multi_region_dr.json`, or the
`multi_region_orchestrator`/`regional_node_manager`/`failover_detector`/
`sync_manager` contracts. A failing assertion fails the job (the script's
own `TESTS_FAILED` exit-1 behavior — nothing swallows it), and
`reports/multi_region_simulation/` is uploaded as a build artifact.

## Output

Simulation results are saved to `reports/multi_region_simulation/`:
- `simulation_report.json` - Structured test results
- Console output with pass/fail status for each scenario
