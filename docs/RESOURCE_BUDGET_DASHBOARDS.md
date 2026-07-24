# Resource Budget Dashboards and Alerting

This document describes the per-contract resource budget dashboard and alerting system for Uzima contracts.

## Overview

The budget dashboard system generates HTML dashboards showing per-contract resource utilization against defined budgets, with alerting for budget breaches.

## Components

### 1. Dashboard Generation (`scripts/generate_budget_dashboard.mjs`)

Generates an HTML dashboard from budget and baseline data:

```bash
node scripts/generate_budget_dashboard.mjs
```

### 2. Shell Wrapper (`scripts/generate_budget_dashboard.sh`)

```bash
./scripts/generate_budget_dashboard.sh
./scripts/generate_budget_dashboard.sh --open
./scripts/generate_budget_dashboard.sh --check-alerts
```

### 3. Alerting Configuration (`resource-budgets/alerting.json`)

Defines alert rules and notification channels for budget breaches.

## Alert Rules

| Rule | Metric | Condition | Severity | Action |
|------|--------|-----------|----------|--------|
| WASM Size Critical | wasm_bytes | Exceeds budget | Critical | Block merge |
| WASM Size Warning | wasm_bytes | >80% of budget | Warning | Notify |
| WASM Regression | wasm_regression | >10% from baseline | Warning | Notify |
| WASM Regression Critical | wasm_regression | >25% from baseline | Critical | Block merge |

## Dashboard Features

- Per-contract WASM size utilization with progress bars
- Storage and CPU budget limits
- Regression tolerance display
- Color-coded status badges (OK/WARN/CRIT)
- Summary statistics

## Data Sources

- `resource-budgets/budgets.json` - Per-contract budget limits
- `resource-budgets/baselines.json` - Recorded WASM sizes from builds
- `resource-budgets/alerting.json` - Alert rules and thresholds
