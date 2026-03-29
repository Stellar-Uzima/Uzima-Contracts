# Medical Record Search and Indexing

Contract: `contracts/medical_record_search`

## Core capabilities
- Privacy-preserving index ingestion (`index_record`, `batch_index_records`) using hashed search tokens
- Filtered search with relevance scoring (`search`)
- Query caching with TTL and eviction (`set_cache_policy`, `get_cache_entry`, `invalidate_cache`)
- Query ranking controls (`set_ranking`)
- Search audit logging (`get_audit`) and query hash preview (`preview_query_hash`)
- Role-based permissions (indexer, searcher, auditor, confidential-search)

## Design notes
- Search input is hash-based (`BytesN<32>`) for tokens, attributes, categories, and record references.
- Complex filters support:
  - required + optional token sets
  - category and attribute filters
  - network scope
  - patient scope
  - date range and minimum quality threshold
- Confidential entries require both `include_confidential=true` and confidential search role.
