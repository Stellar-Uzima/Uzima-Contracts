# Resource Exhaustion Threat Models

## 1. Storage Exhaustion via Record Creation

**Threat**: Attackers create excessive medical records to consume storage resources and potentially cause contract failure or increased costs.

**Attack Vectors**:
- Spamming `add_record()` or `add_record_with_did()` calls to create millions of records
- Creating records with large data fields (long diagnosis/treatment strings, large tag arrays)
- Exploiting lack of limits on record creation rate
- Filling storage with redundant or meaningless records to increase query costs
- Creating records for non-existent patients to bloat patient-record mappings

**Mitigations**:
- Rate limiting on record creation operations (`OP_ADD_RECORD` with default 50 calls/hour for doctors, 10 for patients)
- Input validation on field lengths (diagnosis, treatment, category, etc.) though current validation may not limit length
- Storage costs are borne by transaction submitters on Soroban (attacker pays fees)
- Ability to pause contract via admin function during active attacks
- Record pruning/archiving strategies possible through governance (not implemented in base contract)
- Monitoring of record count growth via `get_record_count()` and patient-specific counts

**Residual Risk**:
- Determined attackers with sufficient funds could still cause storage bloat
- Legitimate users might be affected by rate limits during attacks
- Query performance degradation as storage grows (mitigated by indexing strategies)
- Requires ongoing tuning of rate limits based on observed usage patterns

## 2. Storage Exhaustion via User and Permission Proliferation

**Threat**: Attackers create excessive user profiles or permission grants to consume storage resources.

**Attack Vectors**:
- Spamming `manage_user()` to create numerous user profiles
- Excessive permission granting via `grant_permission()` to bloat permission storage
- Creating many DID associations or attribute assignments
- Exploiting role assignment to create many role-user mappings
- Creating temporary users for single-use attacks to avoid detection

**Mitigations**:
- Rate limiting on user management operations (`OP_MANAGE_USER` with same limits as record creation)
- Maximum roles per address configurable (default 10) prevents excessive role accumulation
- Permission grants can be time-bound and cleaned up automatically
- Storage efficient: Maps rather than lists for most associations
- Events emitted rather than storing large history on-chain where possible
- Ability to deactivate users rather than delete (preserves audit history)
- Regular cleanup of expired permissions and inactive users recommended

**Residual Risk**:
- Legacy data accumulation over time requires active management
- Sophisticated attackers might distribute attacks across many addresses to evade rate limits
- Requires monitoring of storage growth trends and proactive cleanup procedures

## 3. Storage Exhaustion via Cryptographic Material

**Threat**: Attackers bloat cryptographic storage through excessive key registration or large key materials.

**Attack Vectors**:
- Spamming `register_key_bundle()` in crypto registry to create many key versions
- Registering excessively large public keys (though validation limits apply)
- Creating many key bundle versions for same address through rapid rotation
- Registering malicious key materials designed to cause verification issues
- Exploiting post-quantum key fields to store large arbitrary data

**Mitigations**:
- Key registration requires self-authorization (attacker must control address)
- Public key validation includes length limits (max 1,048,576 bytes, algorithm-specific limits)
- Key bundle versioning is monotonic per address but storage grows with versions
- Events emitted for key registration and revocation
- Ability to prune old key versions through governance (not implemented in base contract)
- Monitoring of key bundle counts per address
- Separation of concerns: Different contracts for different crypto functions

**Residual Risk**:
- Long-term key accumulation requires active key lifecycle management
- Large key materials still possible within validation limits
- Requires key rotation and retirement policies as operational controls

## 4. Storage Exhaustion via Audit and Logging

**Threat**: Attackers generate excessive audit log entries to consume storage resources.

**Attack Vectors**:
- Spamming actions that generate audit logs (record access, permission changes, etc.)
- Exploiting forensic logging to create excessive entries
- Generating crypto audit entries through rapid config changes
- Creating excessive ZK audit proofs (valid or invalid)
- Spamming structured log entries through various logging functions

**Mitigations**:
- Rate limiting applies to many audit-generating operations (record creation, user management)
- Audit logs are essential for security and cannot be rate-limited to zero
- Storage efficient: Audit entries are compact structs
- Ability to prune old audit logs through governance (not implemented in base contract)
- Separate audit streams (access logs, crypto audit, ZK audit, structured logs)
- Monitoring of audit log growth rates
- Immutable design: Audit logs should generally be append-only for security

**Residual Risk**:
- Audit logs are by design accumulative and require long-term storage strategy
- Legitimate security monitoring generates necessary audit traffic
- Requires archival and retention policies as operational controls
- Balance needed between security visibility and storage costs

## 5. Computation Exhaustion via Complex Validation

**Threat**: Attackers craft inputs that cause excessive computation in validation or processing functions.

**Attack Vectors**:
- Creating records with extremely large arrays (tags, custom fields) to strain validation
- Crafting ZK proofs with complex public inputs to strain verification
- Creating permission grant lists with many entries to strain iteration
- Exploiting nested data structures to cause deep recursion or iteration
- Creating medical records with many custom fields to strain validation loops

**Mitigations**:
- Input validation on array sizes and map sizes (though current validation may not limit quantity)
- Reasonable limits on field sizes and complexity in validation functions
- Storage costs naturally limit excessive data submission (attacker pays fees)
- Iteration bounds based on actual data size, not attacker-controlled limits
- Use of efficient data structures (maps, vectors) with known complexity characteristics
- Events emitted rather than complex on-chain computation where possible
- Ability to monitor transaction resource usage through Soroban metrics

**Residual Risk**:
- Complex validation logic could still be exploited for DoS
- Requires profiling of validation functions to identify potential bottlenecks
- Need for input size limits on variable-length fields
- Trade-off between functionality and security in validation complexity

## 6. State Reading Exhaustion via Expensive Queries

**Threat**: Attackers execute expensive read operations to consume computational resources and potentially cause denial of service.

**Attack Vectors**:
- Requesting large page sizes in paginated queries (`get_history()`)
- Iterating through large datasets via sequential queries
- Exploiting lack of limits on query complexity or result size
- Requesting records for non-existent patients to waste computation
- Creating complex query patterns that trigger expensive computation paths

**Mitigations**:
- Pagination limits: Page size validation in `get_history()` and similar functions
- Query complexity naturally limited by actual data size (must iterate through real data)
- Access controls prevent unauthorized querying of sensitive data
- Costs borne by query submitters on Soroban (attacker pays fees)
- Ability to monitor and alert on expensive query patterns
- Indexing strategies to improve query performance (patient-record mappings)
- Default reasonable limits where explicit limits not present

**Residual Risk**:
- Legitimate users may need to query large datasets (requires reasonable limits)
- Query optimization requires ongoing effort as data patterns evolve
- Balance needed between query functionality and resource protection
- Requires monitoring of query performance and abuse patterns

## 7. Event Generation Exhaustion

**Threat**: Attackers cause excessive event generation to consume network resources or potentially cause indexing issues.

**Attack Vectors**:
- Spamming operations that generate many events (record creation, user management, etc.)
- Exploiting loops or recursive patterns to generate multiple events per transaction
- Creating events with large data payloads (though event data is limited)
- Generating events designed to cause issues in off-chain indexers or processors
- Spamming events that trigger expensive off-chain processing

**Mitigations**:
- Rate limiting applies to many event-generating operations
- Event data size naturally limited by transaction size limits on Soroban
- Events are essential for transparency and auditability
- Ability to monitor event rates and patterns
- Off-chain systems should implement their own rate limiting and abuse protection
- Events designed to be minimal and informative rather than bulky
- Separation of concerns: Contract generates events, off-chain systems process them

**Residual Risk**:
- Events are by design informative and some volume is expected
- Off-chain systems must be designed to handle expected event volumes
- Requires monitoring of event processing lag and backlogs
- Balance needed between event informativeness and processing efficiency

## 8. Cross-Chain Resource Exhaustion

**Threat**: Attackers exploit cross-chain mechanisms to consume resources on connected chains or the Uzima contract.

**Attack Vectors**:
- Spamming cross-chain record synchronization requests
- Creating excessive cross-chain references to bloat storage
- Exploiting cross-chain bridges to transfer assets or data maliciously
- Generating excessive cross-chain events or proofs
- Manipulating cross-chain state to cause reprocessing loops

**Mitigations**:
- Cross-chain operations require admin authorization for configuration changes
- Individual cross-chain actions may have their own limits and validation
- Storage efficient: Cross-chain references include minimal necessary data
- Events emitted for cross-chain activities for monitoring
- Ability to pause or disable cross-chain functionality via governance
- Monitoring of cross-chain activity volumes and patterns
- Time delays and confirmation requirements in cross-chain transfers
- Validation of external chain identifiers and record hashes

**Residual Risk**:
- Security depends on weakest link in connected chains
- Requires monitoring of all connected chains for resource exhaustion attacks
- Cross-chain complexity increases attack surface and potential failure modes
- Requires robust error handling and circuit breaker patterns

## 9. Governance Resource Exhaustion

**Threat**: Attackers exploit governance mechanisms to consume resources through proposal spam or voting manipulation.

**Attack Vectors**:
- Spamming proposal creation to bloat proposal storage
- Excessive voting on proposals to consume computational resources
- Creating complex proposals with large execution data
- Exploiting voting mechanisms to cause repeated computation
- Generating excessive governance events for monitoring systems

**Mitigations**:
- Proposal creation requires voting power (token balance or reputation)
- Proposal storage efficient: Only essential data stored (ID, proposer, hashes, etc.)
- Voting requires authentication and has computational cost based on voting power
- Events emitted for governance actions for transparency
- Ability to monitor proposal and voting rates
- Dispute mechanisms to challenge malicious proposals
- Time delays in governance process provide reaction window
- Minimum proposal thresholds prevent spam by powerless accounts

**Residual Risk**:
- Determined attackers with sufficient resources could still spam governance
- Requires ongoing monitoring of governance participation and proposal quality
- Balance needed between accessibility and spam prevention in governance
- Requires active community participation to counter malicious governance attempts

## Security Controls Mapping

| Threat | Primary Security Controls | Secondary Controls | Monitoring/Detection |
|--------|--------------------------|-------------------|----------------------|
| Storage Exhaustion via Records | Rate limits, field validation, payer-pays model | Pause function, monitoring | Record count growth, storage usage metrics |
| Storage Exhaustion via Users/Permissions | Rate limits, role limits, time-bounded grants | Storage efficiency, events | User/profile growth, permission grant trends |
| Storage Exhaustion via Crypto | Self-authentication, key validation, versioning | Events, pruning capability | Key bundle counts, storage growth monitoring |
| Storage Exhaustion via Audit/Logs | Rate limits on gen ops, storage efficiency | Pruning capability, separate streams | Audit log growth rates, stream monitoring |
| Computation Exhaustion | Input validation, storage cost limits, efficient algorithms | Monitoring, complexity profiling | Transaction resource usage, validation timing |
| State Reading Exhaustion | Pagination limits, access controls, payer-pays | Indexing, monitoring | Query performance metrics, abuse pattern detection |
| Event Generation Exhaustion | Rate limits, event size limits, essential design | Off-chain protection, monitoring | Event rates, processing lag, backlog monitoring |
| Cross-Chain Resource Exhaustion | Admin auth for config, validation, time delays | Pausing, monitoring | Cross-chain activity volumes, anomaly detection |
| Governance Resource Exhaustion | Voting power requirement, storage efficiency | Dispute mechanisms, timing | Proposal/voting rates, governance participation |

## Recommended Operational Controls

1. **Resource Monitoring Dashboard**: Track key metrics (record counts, user counts, storage usage, event rates)
2. **Rate Limit Tuning**: Regular adjustment of limits based on observed legitimate usage and attack patterns
3. **Storage Hygiene Procedures**: Regular cleanup of expired data, inactive users, old key versions
4. **Query Performance Monitoring**: Track and optimize expensive query patterns
5. **Event Processing Health**: Monitor off-chain event processing lag and backlogs
6. **Governance Participation Analysis**: Track voting patterns and proposal quality
7. **Cross-Chain Activity Monitoring**: Monitor volumes and patterns across all connected chains
8. **Resource Exhaustion Playbooks**: Specific procedures for different types of resource attacks
9. **Stress Testing**: Regular load testing to identify breaking points and tuning opportunities
10. **Capacity Planning**: Forecast resource needs based on growth trends and usage patterns
11. **Alerting Thresholds**: Set alerts for abnormal resource consumption patterns
12. **Emergency Response Procedures**: Define steps for responding to active resource exhaustion attacks
13. **Vendor and Dependency Monitoring**: Track resource usage patterns of dependencies and integrations
14. **Benchmarking**: Establish baseline performance metrics for normal operations
15. **Continuous Improvement**: Regular review and update of resource protection measures based on lessons learned

## Specific Implementation Recommendations

1. **Add explicit limits to variable-length fields**:
   - Diagnosis/treatment strings: Reasonable maximum length (e.g., 10,000 characters)
   - Tag arrays: Maximum number of tags per record (e.g., 50 tags)
   - Custom fields maps: Maximum number of custom fields (e.g., 20 fields)

2. **Enhance rate limiting**:
   - Consider different limits for different operation types
   - Implement burst protection alongside rate limits
   - Consider address-based rate limiting in addition to role-based

3. **Implement storage pruning capabilities**:
   - Governance-controlled pruning of old audit logs
   - Key version retirement policies
   - Inactive user cleanup procedures (with data export options)

4. **Enhance monitoring capabilities**:
   - Add view functions for key resource metrics
   - Consider emitting summary metrics periodically
   - Integrate with external monitoring systems

5. **Improve pagination controls**:
   - Enforce maximum page sizes in all paginated queries
   - Consider cursor-based pagination for large datasets
   - Add query timeout or cost estimation where possible

6. **Strengthen cross-chain protections**:
   - Implement circuit breaker patterns for cross-chain operations
   - Add volume limits and anomaly detection
   - Implement robust error handling and retry limits

7. **Enhance governance protections**:
   - Consider minimum time between proposals from same address
   - Implement proposal size limits
   - Add quorum requirements for different proposal types

8. **Develop resource exhaustion response playbooks**:
   - Specific procedures for storage exhaustion attacks
   - Response plans for rate limit evasion attempts
   - Governance attack mitigation strategies
   - Cross-chain incident response procedures

These controls should be implemented as a defense-in-depth strategy, with multiple layers working together to prevent, detect, and respond to resource exhaustion attacks.