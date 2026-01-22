# DAO Governance Framework

## 1. Governance Architecture

The DAO operates on a hybrid governance model designed to balance financial stake with active contribution, safeguarding the protocol through a judicial layer.

### Core Components
* **Governor Contract:** The central logic handler. It calculates voting power, manages proposal lifecycles, and executes passed proposals.
* **SUT Token (Plutocratic Layer):** Represents financial stake. 1 Token = 1 Vote.
* **Reputation System (Meritocratic Layer):** A non-transferable score earned by contributors. 1 Reputation Point = 1 Vote.
* **Dispute Resolution (Judicial Layer):** A council of arbiters who can veto malicious proposals that pass the vote but violate the DAO Constitution.

---

## 2. Proposal Lifecycle

### Phase 1: Proposal
* **Threshold:** A proposer must hold a combined voting power (Token + Reputation) greater than `proposal_threshold` (e.g., 100,000 units).
* **Action:** The user calls `propose()` with a description hash (IPFS CID) and the executable data (WASM calls).

### Phase 2: Voting Delay
* **Duration:** 2 Days (configurable).
* **Purpose:** A cooling-off period for the community to discuss the proposal before voting begins. Snapshotting of balances occurs here.

### Phase 3: Active Voting
* **Duration:** 5 Days (configurable).
* **Mechanism:** Users cast votes (`For`, `Against`, `Abstain`).
* **Power Calculation:** `Voting Power = SUT Balance + Reputation Score`.

### Phase 4: Resolution & Timelock
* **Success Criteria:**
    1.  `For` votes > `Against` votes.
    2.  Total votes meet the `Quorum` requirement.
* **Queuing:** Successful proposals enter a Timelock queue.

### Phase 5: Dispute Window
* **Safety Check:** During the Timelock, any Arbiter can call `dispute()` on the proposal ID.
* **Effect:** If disputed, the proposal state changes to `Disputed` and cannot be executed until resolved by the council.

### Phase 6: Execution
* **Finalization:** If the Timelock expires and no disputes exist, anyone can call `execute()`. The Governor triggers the Treasury or Contract upgrades.

---

## 3. Treasury Management

The Treasury Controller has been upgraded to support two execution paths:

1.  **Multisig Ops (Fast Track):** Routine operational expenses (e.g., server costs) can be signed by the elected multisig committee without a full DAO vote.
2.  **Governance Execution (DAO Track):** Large capital allocations require a full passed proposal. The Governor contract has special permission to bypass the multisig threshold and execute transfers directly via `governance_execute`.

---

## 4. Reputation Rules

* **Earning:** Reputation is minted by the DAO (via passed proposals) to reward code contributions, community management, or auditing.
* **Slashing:** If a contributor acts maliciously, a governance proposal can slash their reputation score.
* **Non-Transferable:** Reputation is bound to the address and cannot be sold or transferred.