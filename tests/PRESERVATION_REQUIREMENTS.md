# Preservation Requirements for CI Merge Conflict Fix

## Overview

This document defines the preservation requirements for the CI merge conflict fix. These requirements ensure that resolving the merge conflict in the `DataKey` enum does not introduce any regressions in existing functionality.

## Property 2: Preservation - Existing DataKey Usage

**Formal Specification:**
```
FOR ANY code that uses DataKey enum variants that were NOT part of the conflict,
the fixed code SHALL produce exactly the same behavior as the original code,
preserving all existing storage access patterns and functionality.
```

## Preservation Test Coverage

The preservation property tests in `preservation_property_tests.rs` verify the following:

### 1. Genomic DataKey Variants (Requirement 3.1)

**Variants Tested:**
- `NextGenomicId` - Auto-incrementing ID for genomic datasets
- `GenomicDataset(u64)` - Storage for genomic dataset headers
- `PatientGenomic(Address)` - Index of datasets by patient
- `GeneAssociationsByGene(String)` - Gene-to-disease associations indexed by gene
- `GeneAssociationsByDisease(String)` - Gene-to-disease associations indexed by disease
- `DrugResponseKey(String, String, String)` - Drug response rules
- `Ancestry(Address)` - Ancestry profiles for patients
- `GenomicBreachCount` - Counter for genomic data breaches
- `GenomicBreach(u64)` - Individual breach records

**Expected Behavior:**
- Genomic functions (`add_genomic_dataset`, `get_genomic_dataset`, `list_patient_genomic`) must continue to work correctly
- Storage access using genomic DataKey variants must produce expected results
- Pattern matching against genomic variants must compile and execute correctly

### 2. Quantum Threat Level Variant (Requirement 3.2)

**Variant Tested:**
- `QuantumThreatLevel` - Quantum threat assessment (0-100 percentage)

**Expected Behavior:**
- Quantum threat functions (`set_quantum_threat_level`, `get_quantum_threat_level`) must continue to work correctly
- Storage access using QuantumThreatLevel variant must produce expected results

### 3. Existing Non-Conflicted DataKey Variants (Requirement 3.3)

**Variants Tested:**
- `Users` - User registry
- `IdentityRegistry` - DID registry
- `NextId` - Auto-incrementing record ID
- `RecordCount` - Total record count
- `Record(u64)` - Individual record storage
- `PatientRecords(Address)` - Index of records by patient
- `ZkVerifierContract` - ZK verifier contract address
- `RateLimitCfg(u32)` - Rate limit configuration
- `RateLimit(Address, u32)` - Rate limit tracking
- `RateLimitBypass(Address)` - Rate limit bypass flag

**Expected Behavior:**
- All existing functionality using these variants must remain unchanged
- User management, record creation, ZK verification, and rate limiting must work as before
- No behavioral changes should occur in code that doesn't interact with the conflicted lines

### 4. Existing Test Suite (Requirement 3.4)

**Test Files:**
- `tests/genomics_tests.rs` - Existing genomic functionality tests

**Expected Behavior:**
- All existing tests must continue to pass without modification
- Test behavior must remain identical before and after the fix
- No test failures should be introduced by resolving the merge conflict

## Property-Based Test Patterns

The preservation tests use property-based testing patterns to provide stronger guarantees:

### Pattern 1: Multiple Dataset Integrity
**Property:** Adding N genomic datasets should result in exactly N datasets being retrievable
**Test:** `test_preservation_property_multiple_genomic_datasets`
**Validates:** Data integrity across multiple operations

### Pattern 2: Referential Integrity
**Property:** Gene associations should be queryable by both gene and disease with consistent results
**Test:** `test_preservation_property_gene_associations`
**Validates:** Bidirectional indexing works correctly

### Pattern 3: Variant Instantiation
**Property:** All DataKey variants can be instantiated and used in storage operations
**Test:** `test_preservation_genomic_datakey_usage`, `test_preservation_existing_datakey_variants`
**Validates:** Enum variants are syntactically valid and functionally correct

## Baseline Behavior

The baseline behavior is defined by the existing test suite in `tests/genomics_tests.rs`:

1. **Genomic Dataset Storage and Queries:**
   - Adding datasets increments NextGenomicId
   - Datasets can be retrieved by ID
   - Patient datasets can be listed
   - Gene associations can be added and queried
   - Drug response rules can be set and retrieved
   - Ancestry profiles can be stored and retrieved

2. **Privacy-Preserving Research Access:**
   - ZK-based access grants work correctly
   - Genomic datasets can be accessed with valid ZK proofs
   - Access control is enforced properly

## Expected Outcome

After the merge conflict is resolved:

1. ✅ All preservation tests should PASS
2. ✅ All existing tests in `genomics_tests.rs` should PASS
3. ✅ No behavioral changes in non-conflicted code
4. ✅ Both genomic and quantum threat level variants work correctly
5. ✅ All other DataKey variants remain unaffected

## Notes

- Since the unfixed code doesn't compile due to merge conflict markers, these tests define the preservation requirements that will be verified AFTER the fix is implemented
- The tests are written based on the existing test suite and expected behavior patterns
- Property-based testing provides stronger guarantees by testing multiple scenarios
- The preservation tests complement the bug condition exploration tests by focusing on what should NOT change
