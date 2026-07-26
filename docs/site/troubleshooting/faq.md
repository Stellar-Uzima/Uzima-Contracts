# FAQ

**Q: What programming languages are used?**
Smart contracts are written in Rust using the Soroban framework.

**Q: Is this HIPAA compliant?**
The system is designed with HIPAA-aligned principles. Compliance depends on your specific implementation and deployment.

**Q: How are medical records encrypted?**
Records are encrypted client-side using the patient's public key before submission. The contract stores only encrypted blobs.

**Q: Can I run this on my own infrastructure?**
Yes. Deploy to your own Stellar node or use the public testnet/mainnet.

**Q: Why exact SDK version pinning?**
The `=21.7.7` prefix in `Cargo.toml` ensures all developers and CI use exactly the same SDK version, preventing subtle incompatibilities from patch releases.

**Q: What is the CEI pattern?**
Checks-Effects-Interactions: validate inputs first, update state second, make external calls last. This prevents reentrancy attacks.

**Q: How do I contribute?**
See [CONTRIBUTING.md](../../../CONTRIBUTING.md) and the [Code Review Process](../../CODE_REVIEW_PROCESS.md).
