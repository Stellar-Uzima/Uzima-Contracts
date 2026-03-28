from typing import Any, Dict, List, Optional

import os
from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel

from kg.config import settings
from kg.graph.neo4j_client import Neo4jClient
from kg.search.semantic_search import SemanticIndex


app = FastAPI(title="Healthcare Knowledge Graph API", default_response_class=None)

neo = Neo4jClient()
semantic = SemanticIndex()


class IngestOntologyRequest(BaseModel):
	source: str
	path: str


class SearchResponseItem(BaseModel):
	entity_id: str
	score: float


class SearchResponse(BaseModel):
	results: List[SearchResponseItem]


@app.on_event("startup")
def on_startup() -> None:
	neo.ensure_indexes()
	try:
		neo.setup_fulltext_indexes()
	except Exception:
		# Index may already exist
		pass


@app.get("/healthz")
def healthz() -> Dict[str, Any]:
	return {"status": "ok"}


@app.get("/query/concept_by_id")
def concept_by_id(concept_id: str) -> Dict[str, Any]:
	cypher = "MATCH (c:Concept {id: $id}) RETURN c AS c"
	with neo.session() as s:
		rec = s.run(cypher, id=concept_id).single()
		if not rec:
			raise HTTPException(status_code=404, detail="Concept not found")
		return rec["c"]


@app.get("/search/semantic", response_model=SearchResponse)
def semantic_search(q: str = Query(..., min_length=2), k: int = Query(10, ge=1, le=100)) -> SearchResponse:
	hits = semantic.search(q, k=k)
	return SearchResponse(results=[SearchResponseItem(entity_id=eid, score=score) for eid, score in hits])


@app.get("/search/hybrid")
def hybrid_search(q: str = Query(..., min_length=2), k: int = Query(10, ge=1, le=100)) -> Dict[str, Any]:
	"""
	Combines FAISS semantic search candidates with fulltext candidates, returning a de-duplicated set.
	"""
	sem_hits = semantic.search(q, k=k)
	ft_hits = neo.hybrid_search_concepts(q, limit=k)
	combined: Dict[str, Dict[str, Any]] = {}
	for eid, score in sem_hits:
		combined[eid] = {"entity_id": eid, "semantic_score": score}
	for row in ft_hits:
		eid = row.get("id")
		if not eid:
			continue
		entry = combined.get(eid, {"entity_id": eid})
		entry["fulltext_score"] = float(row.get("score", 0.0)) if "score" in row else None
		entry["name"] = row.get("name") or entry.get("name")
		entry["type"] = row.get("type") or entry.get("type")
		entry["source"] = row.get("source") or entry.get("source")
		combined[eid] = entry
	# simple ranking: prefer semantic_score, break ties with fulltext_score
	def rank_key(x: Dict[str, Any]):
		return (x.get("semantic_score") or 0.0, x.get("fulltext_score") or 0.0)
	sorted_results = sorted(combined.values(), key=rank_key, reverse=True)[:k]
	return {"results": sorted_results}

@app.post("/ingest/ontology")
def ingest_ontology(req: IngestOntologyRequest) -> Dict[str, Any]:
	source = req.source.lower()
	path = req.path
	if source == "mesh":
		from kg.ingestion.ontologies.mesh_loader import load_mesh_csv

		concepts = list(load_mesh_csv(path))
		processed = neo.upsert_concepts(concepts)
		# add to semantic index
		added = semantic.add_entities([(c["id"], c["name"]) for c in concepts if c.get("name")])
		return {"concepts": processed, "semantic_added": added}
	elif source == "snomed":
		from kg.ingestion.ontologies.snomed_loader import (
			load_snomed_rf2_concepts,
			load_snomed_rf2_descriptions,
		)

		descs = load_snomed_rf2_descriptions(path.replace("Concept", "Description"))
		concepts = []
		for c in load_snomed_rf2_concepts(path):
			c["name"] = descs.get(c["id"])
			concepts.append(c)
		processed = neo.upsert_concepts(concepts)
		added = semantic.add_entities([(c["id"], c["name"]) for c in concepts if c.get("name")])
		return {"concepts": processed, "semantic_added": added}
	else:
		raise HTTPException(status_code=400, detail="Unsupported source. Use 'snomed' or 'mesh'.")


@app.post("/ingest/pubmed")
def ingest_pubmed(q: str, retmax: int = 100) -> Dict[str, Any]:
	from kg.ingestion.papers.pubmed_ingestor import search_pubmed, fetch_pubmed_summaries

	pmids = search_pubmed(q, retmax=retmax)
	papers = list(fetch_pubmed_summaries(pmids))
	processed = neo.upsert_papers(papers)
	# Optional semantic index over titles
	added = semantic.add_entities([(p["pmid"], p["title"]) for p in papers if p.get("title")])
	return {"pmids_found": len(pmids), "papers_upserted": processed, "semantic_added": added}

