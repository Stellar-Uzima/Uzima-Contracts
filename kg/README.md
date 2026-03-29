# Healthcare Knowledge Graph Service

This service provides a scalable healthcare knowledge graph integrating medical ontologies (SNOMED CT, MeSH), research papers (PubMed), semantic search, and basic inference. It targets fast query responses (<1s on indexed lookups) and a growth path to 10M+ concepts.

## Architecture Overview

- Graph database: Neo4j (pluggable; can evolve to JanusGraph + Elasticsearch/OpenSearch for horizontal scale)
- Ingestion:
  - Ontologies: SNOMED CT, MeSH
  - Research: PubMed (E-utilities)
- Search: Semantic search via Sentence-Transformers embeddings and FAISS index; hybrid with graph metadata
- Inference: Rule-based pattern inference; embedding similarity as heuristic discovery
- API: FastAPI for ingestion, search, and query

## Getting Started

1) Environment
   - Python 3.10+
   - Neo4j 5.x (local or remote)
   - Optional GPU accelerators for faster embedding

2) Install
   - Create and activate a virtualenv
   - `pip install -r kg/requirements.txt`

3) Configure
   - Copy `.env.example` to `.env` and set:
     - `NEO4J_URI=bolt://localhost:7687`
     - `NEO4J_USER=neo4j`
     - `NEO4J_PASSWORD=...`
     - `EMBEDDING_MODEL=sentence-transformers/all-MiniLM-L6-v2`
     - `FAISS_INDEX_PATH=.faiss/index.bin`

4) Run API
   - `uvicorn kg.api.server:app --host 0.0.0.0 --port 8080`

## Data Model (Simplified)

- Nodes:
  - Concept {id, name, type, source, synonyms[], description}
  - Paper {pmid, title, abstract, year, journal, mesh_terms[]}
- Relationships:
  - CONCEPT_REL: Concept-Concept (e.g., is_a, associated_with, treats, causes)
  - MENTIONS: Paper->Concept

Indexes (Neo4j):
- `Concept(id)`, `Concept(name)`
- `Paper(pmid)`

## Scaling to 10M+ Concepts

- Use batch ingestion with periodic commits and idempotent upserts
- Maintain compound indexes for frequent filters (type, source)
- Prefer relationship directions and query patterns that leverage indexes
- Use FAISS IVF or HNSW for vector search; for distributed scale, switch to OpenSearch k-NN or Elasticsearch vector fields
- Optionally migrate to JanusGraph (Cassandra/Scylla + ES/OS) for larger clusters

## Benchmarks

- `python -m kg.benchmarks.benchmark_queries` runs sample latency checks for key query patterns

## Security and Compliance

- Do not ingest PHI/PII into this graph
- Sources: SNOMED CT requires appropriate licensing; MeSH is open; PubMed metadata usage per NCBI T&Cs

