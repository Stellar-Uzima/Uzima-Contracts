# ADR-007: On-Chain vs Off-Chain Storage of Governance Data

**Status:** Accepted  
**Date:** 2025-06-29

## Context

Governance proposals include both machine-executable calldata (which must be on-chain) and human-readable context (title, description, rationale, links to discussion threads). Storing all of this on-chain increases contract storage costs and WASM size; storing too little on-chain undermines auditability.

Options evaluated:
1. **Store everything on-chain** — maximum auditability, high storage cost
2. **Store only calldata on-chain; description off-chain (IPFS/database)** — low cost, but off-chain data can disappear or be tampered with
3. **Store calldata + content hash on-chain; full description on IPFS** — balanced: content is verifiable via hash, storage cost is bounded

Soroban contract storage is metered by ledger entry size and TTL fees; large string fields significantly increase per-entry costs and may exceed entry size limits.

## Decision

Store the following **on-chain** in the `governor` contract:

| Field | Type | On-chain |
|-------|------|:--------:|
| `proposal_id` | `u64` | ✓ |
| `proposer` | `Address` | ✓ |
| `calldata` | `Vec<ContractCall>` | ✓ |
| `description_hash` | `Bytes (SHA-256)` | ✓ |
| `created_at_ledger` | `u32` | ✓ |
| `state` | `ProposalState` | ✓ |
| `vote_tally` (yes/no/abstain weighted) | `Map<VoteOption, i128>` | ✓ |
| `execution_eta` | `u64` (Unix timestamp) | ✓ |

Store the following **off-chain on IPFS**, with the CID committed as `description_hash`:

- Proposal title
- Full rationale and background
- Links to discussion threads
- Attachments (impact assessments, code diffs)

The `description_hash` is a SHA-256 hash of the canonical JSON blob `{ cid, title, body, links }`. Any participant can verify the off-chain content by hashing the IPFS document and comparing to the on-chain hash.

## Rationale

- Calldata must be on-chain because it is what gets executed — any off-chain storage of executable data creates a manipulation risk.
- Storing a content hash on-chain provides cryptographic binding between the vote and the description, preventing post-hoc rewriting of "what was approved."
- IPFS is content-addressed — a document's CID is a hash of its content, making tampering self-evident.
- This approach is consistent with established governance systems (Compound, OpenZeppelin Governor) and is a recognized best practice.

## Consequences

- Governance tooling must pin IPFS content when proposals are created to prevent content loss (content is not guaranteed to persist on IPFS without pinning).
- The platform should run at least one IPFS pinning service node.
- Voters who want to verify proposal content must retrieve the IPFS document and verify its SHA-256 hash against the on-chain value — governance UI should automate this check.
- If IPFS content is lost (not pinned), the proposal description becomes inaccessible but the calldata and vote result remain auditable on-chain.
