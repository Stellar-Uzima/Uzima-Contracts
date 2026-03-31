from typing import Any, Dict, Iterable, List, Optional, Tuple

from neo4j import GraphDatabase, Session

from kg.config import settings


class Neo4jClient:
	"""
	Thin wrapper around the Neo4j driver with convenience helpers for idempotent upserts
	and batched writes suitable for large ontology ingest.
	"""

	def __init__(self, uri: Optional[str] = None, user: Optional[str] = None, password: Optional[str] = None):
		self._driver = GraphDatabase.driver(
			uri or settings.neo4j_uri,
			auth=(user or settings.neo4j_user, password or settings.neo4j_password),
			max_connection_lifetime=3600,
		)

	def close(self) -> None:
		self._driver.close()

	def session(self) -> Session:
		return self._driver.session()

	def ensure_indexes(self) -> None:
		queries = [
			"CREATE INDEX concept_id IF NOT EXISTS FOR (c:Concept) ON (c.id)",
			"CREATE INDEX concept_name IF NOT EXISTS FOR (c:Concept) ON (c.name)",
			"CREATE INDEX concept_type IF NOT EXISTS FOR (c:Concept) ON (c.type)",
			"CREATE INDEX concept_source IF NOT EXISTS FOR (c:Concept) ON (c.source)",
			"CREATE INDEX concept_type_source IF NOT EXISTS FOR (c:Concept) ON (c.type, c.source)",
			"CREATE INDEX paper_pmid IF NOT EXISTS FOR (p:Paper) ON (p.pmid)",
			"CREATE INDEX paper_year IF NOT EXISTS FOR (p:Paper) ON (p.year)",
			"CREATE INDEX rel_type IF NOT EXISTS FOR ()-[r:REL]-() ON (r.type)",
		]
		with self.session() as s:
			for q in queries:
				s.run(q)

	def upsert_concepts(self, concepts: Iterable[Dict[str, Any]]) -> int:
		"""
		concepts: iterable of dicts with keys: id, name, type, source, synonyms, description
		Returns number of concepts processed.
		"""
		cypher = """
		UNWIND $rows AS row
		MERGE (c:Concept {id: row.id})
		ON CREATE SET c.name = row.name,
		              c.type = row.type,
		              c.source = row.source,
		              c.synonyms = coalesce(row.synonyms, []),
		              c.description = row.description
		ON MATCH SET  c.name = coalesce(row.name, c.name),
		              c.type = coalesce(row.type, c.type),
		              c.source = coalesce(row.source, c.source),
		              c.synonyms = coalesce(row.synonyms, c.synonyms),
		              c.description = coalesce(row.description, c.description)
		RETURN count(*) AS processed
		"""
		with self.session() as s:
			res = s.run(cypher, rows=list(concepts))
			return res.single()["processed"]

	def upsert_papers(self, papers: Iterable[Dict[str, Any]]) -> int:
		cypher = """
		UNWIND $rows AS row
		MERGE (p:Paper {pmid: row.pmid})
		ON CREATE SET p.title = row.title,
		              p.abstract = row.abstract,
		              p.year = row.year,
		              p.journal = row.journal,
		              p.mesh_terms = coalesce(row.mesh_terms, [])
		ON MATCH SET  p.title = coalesce(row.title, p.title),
		              p.abstract = coalesce(row.abstract, p.abstract),
		              p.year = coalesce(row.year, p.year),
		              p.journal = coalesce(row.journal, p.journal),
		              p.mesh_terms = coalesce(row.mesh_terms, p.mesh_terms)
		RETURN count(*) AS processed
		"""
		with self.session() as s:
			res = s.run(cypher, rows=list(papers))
			return res.single()["processed"]

	def upsert_relationships(self, rels: Iterable[Tuple[str, str, str, Dict[str, Any]]]) -> int:
		"""
		rels: iterable of tuples (from_id, to_id, rel_type, properties_dict)
		Nodes assumed to be :Concept
		"""
		cypher = """
		UNWIND $rows AS row
		MATCH (a:Concept {id: row.from_id})
		MATCH (b:Concept {id: row.to_id})
		MERGE (a)-[r:REL {type: row.rel_type}]->(b)
		SET r += coalesce(row.props, {})
		RETURN count(*) AS processed
		"""
		payload = [{"from_id": f, "to_id": t, "rel_type": rt, "props": p} for f, t, rt, p in rels]
		with self.session() as s:
			res = s.run(cypher, rows=payload)
			return res.single()["processed"]

	def link_paper_mentions(self, mentions: Iterable[Tuple[str, str]]) -> int:
		"""
		mentions: (pmid, concept_id)
		"""
		cypher = """
		UNWIND $rows AS row
		MATCH (p:Paper {pmid: row.pmid})
		MATCH (c:Concept {id: row.cid})
		MERGE (p)-[:MENTIONS]->(c)
		RETURN count(*) AS processed
		"""
		payload = [{"pmid": pmid, "cid": cid} for pmid, cid in mentions]
		with self.session() as s:
			res = s.run(cypher, rows=payload)
			return res.single()["processed"]

	def keyword_query_concepts(self, text: str, limit: int = 10) -> List[Dict[str, Any]]:
		cypher = """
		CALL db.index.fulltext.queryNodes('concept_name_ft', $q)
		YIELD node, score
		RETURN node{.*, score: score}
		ORDER BY score DESC
		LIMIT $limit
		"""
		with self.session() as s:
			res = s.run(cypher, q=text, limit=limit)
			return [r["node"] for r in res]

	def setup_fulltext_indexes(self) -> None:
		with self.session() as s:
			s.run("CALL db.index.fulltext.createNodeIndex('concept_name_ft', ['Concept'], ['name'])")

	def hybrid_search_concepts(self, query: str, limit: int = 10) -> List[Dict[str, Any]]:
		"""
		Hybrid keyword search leveraging fulltext index with a fallback to contains.
		"""
		with self.session() as s:
			try:
				cypher = """
				CALL db.index.fulltext.queryNodes('concept_name_ft', $q)
				YIELD node, score
				RETURN node{.*, score: score}
				ORDER BY score DESC
				LIMIT $limit
				"""
				res = s.run(cypher, q=query, limit=limit)
				rows = [r["node"] for r in res]
				if rows:
					return rows
			except Exception:
				pass
			# Fallback simple contains (slower)
			cypher2 = """
			MATCH (c:Concept)
			WHERE toLower(c.name) CONTAINS toLower($q)
			RETURN c AS node
			LIMIT $limit
			"""
			res2 = s.run(cypher2, q=query, limit=limit)
			return [r["node"] for r in res2]

