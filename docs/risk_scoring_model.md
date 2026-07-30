# 🛡️ Contract Risk Scoring Model Architecture

## 📐 Scoring Formula
Contract risk is determined using a weighted multi-factor calculation:

$$\text{Composite Risk} = (0.30 \times \text{Complexity}) + (0.20 \times \text{History}) + (0.50 \times \text{Blast Radius})$$

### 1. Complexity (Weight: 30%)
* **Source Lines of Code (SLOC):** Total executable rust code count.
* **Control Flow:** Branching structures (`if`, `match`, `loop`, `while`).

### 2. History (Weight: 20%)
* **Commit Churn:** Frequency of file updates in Git history.
* **Security Revisions:** Commits tagged with bug/fix/patch keywords.

### 3. Blast Radius (Weight: 50%)
* **Authorization Depth:** Explicit `require_auth()` checks.
* **Cross-Contract Invocations:** Invocations targeting external Soroban contracts.
* **State Footprint:** Storage read/write operations on ledger keys.

---

## 🚦 Risk Tiering Thresholds

| Risk Tier | Score Range | Action Required |
| :--- | :--- | :--- |
| **CRITICAL** | $70.0 - 100.0$ | Mandatory multi-party security sign-off before deployment. |
| **HIGH** | $40.0 - 69.9$ | Detailed unit & integration test coverage required ($>90\%$). |
| **MEDIUM** | $20.0 - 39.9$ | Standard peer code review. |
| **LOW** | $0.0 - 19.9$ | Standard CI/CD automated validation. |