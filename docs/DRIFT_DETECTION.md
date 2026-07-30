# Drift Detection for Generated Artifacts

## Overview

The `scripts/check_drift.sh` script detects drift between generated artifacts
(schemas, documentation) and their committed versions. It validates that generated
files are up to date with their source data.

## What It Checks

### Schema Drift
- Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`)
- Checks that event schema files referenced in the registry actually exist
- Detects invalid JSON that would break tooling

### Documentation Drift
- Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs`
- Detects TODO/FIXME/PLACEHOLDER markers in generated docs
- Checks contract READMEs reference their public struct names

## Usage

`ash
# Full drift check (schemas + docs)
./scripts/check_drift.sh --all

# Schema drift only
./scripts/check_drift.sh --schema

# Docs drift only
./scripts/check_drift.sh --docs

# Via npm
npm run drift:check
npm run drift:check:schema
npm run drift:check:docs
`"
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs += "
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs += 

Add to CI pipeline to catch drift before merge:

`yaml
- name: Check artifact drift
  run: ./scripts/check_drift.sh --all
`"
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs `"
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs += "
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs +=   Add to CI pipeline to catch drift before merge:  `yaml - name: Check artifact drift   run: ./scripts/check_drift.sh --all += "
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs `"
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs += "
# Drift Detection for Generated Artifacts  ## Overview  The `scripts/check_drift.sh` script detects drift between generated artifacts (schemas, documentation) and their committed versions. It validates that generated files are up to date with their source data.  ## What It Checks  ### Schema Drift - Validates JSON structure of schema registries (`schemas/events/event-schema-registry.json`, `schemas/interface-registry/registry.json`) - Checks that event schema files referenced in the registry actually exist - Detects invalid JSON that would break tooling  ### Documentation Drift - Verifies `docs/ERROR_CODES.md` covers all contracts with `src/errors.rs` - Detects TODO/FIXME/PLACEHOLDER markers in generated docs - Checks contract READMEs reference their public struct names  ## Usage  `ash # Full drift check (schemas + docs) ./scripts/check_drift.sh --all  # Schema drift only ./scripts/check_drift.sh --schema  # Docs drift only ./scripts/check_drift.sh --docs  # Via npm npm run drift:check npm run drift:check:schema npm run drift:check:docs +=   Add to CI pipeline to catch drift before merge:  `yaml - name: Check artifact drift   run: ./scripts/check_drift.sh --all += 

When drift is detected:
1. Regenerate the affected artifacts using the appropriate generation script
2. Verify the regenerated content is correct
3. Commit the updated artifacts
