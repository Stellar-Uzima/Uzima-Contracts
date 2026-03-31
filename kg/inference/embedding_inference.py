from typing import Dict, Iterable, List, Tuple

from kg.search.semantic_search import SemanticIndex


def suggest_related_by_text(index: SemanticIndex, concept_name: str, k: int = 10) -> List[Tuple[str, float]]:
	"""
	Heuristic "inference" via embedding similarity.
	Returns list of (entity_id, score).
	"""
	return index.search(concept_name, k=k)

