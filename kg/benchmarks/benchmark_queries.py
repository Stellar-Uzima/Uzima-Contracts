import time
from statistics import mean

from kg.graph.neo4j_client import Neo4jClient
from kg.search.semantic_search import SemanticIndex


def time_it(fn, *args, runs: int = 5, **kwargs):
	latencies = []
	for _ in range(runs):
		t0 = time.perf_counter()
		fn(*args, **kwargs)
		latencies.append((time.perf_counter() - t0) * 1000.0)
	return {"p50_ms": sorted(latencies)[len(latencies) // 2], "avg_ms": mean(latencies)}


def bench_keyword_query(neo: Neo4jClient):
	def run():
		with neo.session() as s:
			list(s.run("MATCH (c:Concept) WHERE c.name CONTAINS 'cancer' RETURN c LIMIT 25"))
	return time_it(run, runs=10)


def bench_semantic_search(semantic: SemanticIndex):
	def run():
		semantic.search("breast cancer treatment", k=10)
	return time_it(run, runs=10)

def bench_hybrid(neo: Neo4jClient, semantic: SemanticIndex):
	def run():
		# combine top-k retrieval and hydrate from graph
		hits = semantic.search("breast cancer treatment", k=25)
		ids = [eid for eid, _ in hits]
		if not ids:
			return
		with neo.session() as s:
			list(s.run("MATCH (c:Concept) WHERE c.id IN $ids RETURN c LIMIT 25", ids=ids))
	return time_it(run, runs=10)

if __name__ == "__main__":
	neo = Neo4jClient()
	semantic = SemanticIndex()
	print("Keyword query:", bench_keyword_query(neo))
	print("Semantic search:", bench_semantic_search(semantic))
	print("Hybrid:", bench_hybrid(neo, semantic))

