from typing import Any, Dict, List

from kg.graph.neo4j_client import Neo4jClient


def create_indexes(neo: Neo4jClient) -> List[str]:
	created: List[str] = []
	stmts = [
		"CREATE INDEX concept_id IF NOT EXISTS FOR (c:Concept) ON (c.id)",
		"CREATE INDEX concept_name IF NOT EXISTS FOR (c:Concept) ON (c.name)",
		"CREATE INDEX concept_type IF NOT EXISTS FOR (c:Concept) ON (c.type)",
		"CREATE INDEX concept_source IF NOT EXISTS FOR (c:Concept) ON (c.source)",
		"CREATE INDEX concept_type_source IF NOT EXISTS FOR (c:Concept) ON (c.type, c.source)",
		"CREATE INDEX paper_pmid IF NOT EXISTS FOR (p:Paper) ON (p.pmid)",
		"CREATE INDEX paper_year IF NOT EXISTS FOR (p:Paper) ON (p.year)",
		"CREATE INDEX rel_type IF NOT EXISTS FOR ()-[r:REL]-() ON (r.type)",
	]
	with neo.session() as s:
		for q in stmts:
			s.run(q)
			created.append(q)
		try:
			s.run("CALL db.index.fulltext.createNodeIndex('concept_name_ft', ['Concept'], ['name'])")
			created.append("fulltext:concept_name_ft")
		except Exception:
			pass
	return created


def typical_query_patterns(neo: Neo4jClient) -> Dict[str, Any]:
	"""
	Warm up caches and validate query shapes.
	"""
	with neo.session() as s:
		# Concept lookup by id
		list(s.run("MATCH (c:Concept {id: $id}) RETURN c", id="D000001"))
		# Name search via fulltext
		try:
			list(s.run("CALL db.index.fulltext.queryNodes('concept_name_ft', $q) YIELD node RETURN node LIMIT 5", q="cancer"))
		except Exception:
			pass
		# Relationship traversal (bounded)
		list(s.run("MATCH (c:Concept {id: $id})-[:REL*1..2]->(d:Concept) RETURN d LIMIT 25", id="D000001"))
	return {"ok": True}


if __name__ == "__main__":
	neo = Neo4jClient()
	print("Creating indexes...")
	print(create_indexes(neo))
	print("Running warm-ups...")
	print(typical_query_patterns(neo))

