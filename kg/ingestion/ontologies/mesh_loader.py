from typing import Dict, Iterable

import csv
from pathlib import Path


def load_mesh_csv(mesh_path: str) -> Iterable[Dict]:
	"""
	Loads a simplified MeSH CSV with columns: DescriptorUI, DescriptorName, TreeNumbers (optional)
	For official MeSH, convert XML to CSV beforehand or parse XML directly.
	"""
	path = Path(mesh_path)
	with open(path, mode="rt", encoding="utf-8") as f:
		reader = csv.DictReader(f)
		for row in reader:
			yield {
				"id": row["DescriptorUI"],
				"name": row["DescriptorName"],
				"type": "MeSH_Descriptor",
				"source": "MeSH",
				"synonyms": [],
				"description": None,
			}

