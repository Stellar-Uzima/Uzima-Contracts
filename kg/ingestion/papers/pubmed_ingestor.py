from typing import Dict, Iterable, List

import time
import requests

NCBI_EUTILS = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils"


def search_pubmed(query: str, retmax: int = 100) -> List[str]:
	params = {"db": "pubmed", "term": query, "retmax": retmax, "retmode": "json"}
	r = requests.get(f"{NCBI_EUTILS}/esearch.fcgi", params=params, timeout=30)
	r.raise_for_status()
	data = r.json()
	return data.get("esearchresult", {}).get("idlist", [])


def fetch_pubmed_summaries(pmids: List[str]) -> Iterable[Dict]:
	"""
	Yields dicts with pmid, title, abstract, year, journal, mesh_terms[]
	"""
	if not pmids:
		return []
	batch_size = 200
	for i in range(0, len(pmids), batch_size):
		batch = pmids[i : i + batch_size]
		params = {"db": "pubmed", "id": ",".join(batch), "retmode": "json"}
		r = requests.get(f"{NCBI_EUTILS}/esummary.fcgi", params=params, timeout=60)
		r.raise_for_status()
		data = r.json().get("result", {})
		for pmid in batch:
			rec = data.get(pmid)
			if not rec:
				continue
			yield {
				"pmid": pmid,
				"title": rec.get("title"),
				"abstract": None,  # needs EFetch for full abstract; keep summary minimal
				"year": rec.get("pubdate", "")[:4],
				"journal": (rec.get("fulljournalname") or rec.get("source")),
				"mesh_terms": [],
			}
		time.sleep(0.34)  # be a good API citizen (3 req/sec)

