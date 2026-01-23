# Regulatory Compliance Framework

## Overview

Stellar Uzima implements a "Compliance-First" architecture designed to satisfy requirements for:

- **HIPAA** (Health Insurance Portability and Accountability Act) - USA
- **GDPR** (General Data Protection Regulation) - EU
- **POPIA** (Protection of Personal Information Act) - South Africa

## Features

### 1. Right to be Forgotten (GDPR Art. 17)

- **Implementation:** `purge_record` function in `medical_records` contract.
- **Mechanism:** Permanently removes the record data mapping from the current ledger state. While historical blocks remain on archival nodes, the smart contract state effectively "forgets" the data, rendering it inaccessible via standard DApp interfaces.
- **Audit:** The purge action itself is logged for accountability (GDPR Art. 30).

### 2. Data Sovereignty & Residency

- **Implementation:** `region` tagging on all MedicalRecords.
- **Enforcement:** The `ComplianceRules` contract validates that access requests originate from authorized jurisdictions before releasing data.

### 3. Audit Trails (HIPAA § 164.312(b))

- **Implementation:** Immutable `AccessLog` on-chain.
- **Scope:** Tracks User ID, Patient ID, Record ID, Timestamp, Purpose, and Outcome (Grant/Deny) for _every_ interaction.

### 4. Rule Engine

- **Implementation:** `ComplianceRulesContract`.
- **Configurability:** Admins can adjust retention periods and consent requirements dynamically without upgrading the core storage contract.

## Configuration Guide

### Setting a GDPR Rule (EU)

```bash
./scripts/interact.sh compliance_rules set_rule \
  --region "EU" \
  --category "General" \
  --requires_consent true \
  --data_residency true
```
