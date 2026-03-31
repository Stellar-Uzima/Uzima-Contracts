from typing import Dict, Iterable, List, Tuple

import os
import numpy as np
import faiss
from sentence_transformers import SentenceTransformer

from kg.config import settings


class SemanticIndex:
	def __init__(self, model_name: str | None = None, index_path: str | None = None):
		self.model = SentenceTransformer(model_name or settings.embedding_model_name)
		self.dim = self.model.get_sentence_embedding_dimension()
		self.index_path = index_path or settings.faiss_index_path
		self.id_map: List[str] = []  # maps vector row -> entity id
		self._index: faiss.Index | None = None
		self._load_or_init()

	def _load_or_init(self) -> None:
		os.makedirs(os.path.dirname(self.index_path), exist_ok=True)
		if os.path.exists(self.index_path):
			self._index = faiss.read_index(self.index_path)
			# naive: expect id_map stored alongside
			map_path = self.index_path + ".ids"
			if os.path.exists(map_path):
				with open(map_path, "r", encoding="utf-8") as f:
					self.id_map = [line.strip() for line in f]
		else:
			self._index = faiss.IndexFlatIP(self.dim)

	def save(self) -> None:
		if self._index is None:
			return
		faiss.write_index(self._index, self.index_path)
		with open(self.index_path + ".ids", "w", encoding="utf-8") as f:
			for _id in self.id_map:
				f.write(_id + "\n")

	def _encode(self, texts: List[str]) -> np.ndarray:
		emb = self.model.encode(texts, normalize_embeddings=True, convert_to_numpy=True)
		return emb.astype(np.float32)

	def add_entities(self, items: List[Tuple[str, str]]) -> int:
		"""
		items: list of (entity_id, text)
		"""
		if not items:
			return 0
		entity_ids, texts = zip(*items)
		vecs = self._encode(list(texts))
		self._index.add(vecs)
		self.id_map.extend(list(entity_ids))
		self.save()
		return len(items)

	def search(self, query: str, k: int = 10) -> List[Tuple[str, float]]:
		vec = self._encode([query])
		scores, idx = self._index.search(vec, k)
		res: List[Tuple[str, float]] = []
		for i in idx[0]:
			if i < 0 or i >= len(self.id_map):
				continue
			res.append((self.id_map[i], float(scores[0][list(idx[0]).index(i)])))
		return res

