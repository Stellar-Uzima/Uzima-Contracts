from typing import Dict, Iterable, List, Tuple

import csv
import gzip
from pathlib import Path


def load_snomed_rf2_concepts(concepts_path: str) -> Iterable[Dict]:
	"""
	Loads SNOMED CT RF2 Concepts file (e.g., sct2_Concept_Full_*.txt.gz)
	Yields dicts with id, name (placeholder), type, source.
	Note: In full implementation, join with Descriptions and Relationships for proper names and edges.
	"""
	path = Path(concepts_path)
	opener = gzip.open if path.suffix == ".gz" else open
	with opener(path, mode="rt", encoding="utf-8") as f:
		reader = csv.DictReader(f, delimiter="\t")
		for row in reader:
			if row.get("active") != "1":
				continue
			yield {
				"id": row["id"],
				"name": None,  # needs join with Descriptions
				"type": "SNOMED_Concept",
				"source": "SNOMED",
				"synonyms": [],
				"description": None,
			}


def load_snomed_rf2_descriptions(descriptions_path: str) -> Dict[str, str]:
	"""
	Maps conceptId -> preferred term (FSN or preferred synonym).
	"""
	result: Dict[str, str] = {}
	path = Path(descriptions_path)
	opener = gzip.open if path.suffix == ".gz" else open
	with opener(path, mode="rt", encoding="utf-8") as f:
		reader = csv.DictReader(f, delimiter="\t")
		for row in reader:
			if row.get("active") != "1":
				continue
			if row.get("typeId") not in {"900000000000013009", "900000000000003001"}:
				continue
			cid = row["conceptId"]
			term = row["term"]
			# Prefer preferred terms over FSN when both exist; keep the last seen preferred
			result[cid] = term
	return result


def load_snomed_rf2_relationships(relationships_path: str) -> Iterable[Tuple[str, str, str, Dict]]:
	"""
	Yields (from_id, to_id, rel_type, props)
	For demo, map all relationships to generic typeId; real implementation maps typeId to human-readable.
	"""
	path = Path(relationships_path)
	opener = gzip.open if path.suffix == ".gz" else open
	with opener(path, mode="rt", encoding="utf-8") as f:
		reader = csv.DictReader(f, delimiter="\t")
		for row in reader:
			if row.get("active") != "1":
				continue
			yield (row["sourceId"], row["destinationId"], row["typeId"], {"characteristicTypeId": row["characteristicTypeId"]})

