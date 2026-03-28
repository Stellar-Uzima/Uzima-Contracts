from typing import Any, Dict, List

from neo4j import Session


class Rule:
	def __init__(self, name: str, cypher: str, params: Dict[str, Any] | None = None):
		self.name = name
		self.cypher = cypher
		self.params = params or {}


class RulesEngine:
	def __init__(self, session: Session):
		self.session = session
		self.rules: List[Rule] = []

	def add_rule(self, rule: Rule) -> None:
		self.rules.append(rule)

	def run(self) -> List[Dict[str, Any]]:
		results: List[Dict[str, Any]] = []
		for rule in self.rules:
			res = self.session.run(rule.cypher, **rule.params)
			results.append({"rule": rule.name, "count": res.consume().counters.nodes_created + res.consume().counters.relationships_created})
		return results

