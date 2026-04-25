import os
from dataclasses import dataclass

from dotenv import load_dotenv


load_dotenv(override=True)


@dataclass
class Settings:
	neo4j_uri: str = os.getenv("NEO4J_URI", "bolt://localhost:7687")
	neo4j_user: str = os.getenv("NEO4J_USER", "neo4j")
	neo4j_password: str = os.getenv("NEO4J_PASSWORD", "neo4j")
	embedding_model_name: str = os.getenv(
		"EMBEDDING_MODEL", "sentence-transformers/all-MiniLM-L6-v2"
	)
	faiss_index_path: str = os.getenv("FAISS_INDEX_PATH", ".faiss/index.bin")


settings = Settings()

