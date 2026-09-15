"""
Proven API Client — Python interface to the Proven knowledge system.
"""Noepedia API Client — Python interface to the Noepedia knowledge system.

Operations:
- Create publication
- Submit deltas (claims, evidence, sources)
- Define networks with RULE_CARD
- Submit replications (independent repetition of a result)
- Record coverage (relevant vs evaluated networks)
- Add placements with arguments
- Query claims / export consolidated view

Maps to the Noepedia grammar (core/src/noepedia.rs).
"""
import json
from dataclasses import dataclass, field
from typing import Optional
from uuid import UUID, uuid4

import httpx


@dataclass
class ClaimData:
    """A knowledge claim."""
    subject: str
    predicate: str
    object: str
    status: str = "PROPOSED"
    sources: list[UUID] = field(default_factory=list)
    evidence: list[UUID] = field(default_factory=list)
    claim_id: UUID = field(default_factory=uuid4)


@dataclass
class SourceData:
    """A source for a claim."""
    title: str
    source_type: str = "journal_article"
    url: Optional[str] = None
    doi: Optional[str] = None
    pmid: Optional[str] = None
    source_id: UUID = field(default_factory=uuid4)


class ProvenClient:
    """Client for Proven knowledge system."""
@dataclass
class RuleCardData:
    """A network's rule: what it evaluates and how objects are ranked."""
    title: str
    evaluates: str
    ranking_criterion: str
    allowed_relations: list[str] = field(default_factory=list)
    rule_id: UUID = field(default_factory=uuid4)


@dataclass
class NetworkData:
    """A rule-governed evaluative projection (objects + relations + rule card)."""
    name: str
    description: str
    rule_card: RuleCardData
    objects: list[UUID] = field(default_factory=list)
    network_id: UUID = field(default_factory=uuid4)


@dataclass
class ReplicationData:
    """An independent repetition of a result. Replication is an operation."""
    claim_id: UUID
    agent_id: UUID
    method: str
    outcome: str = "supports"  # supports | contradicts | inconclusive
    conditions: dict = field(default_factory=dict)
    replication_id: UUID = field(default_factory=uuid4)


@dataclass
class CoverageData:
    """Coverage — relevant networks identified vs evaluated."""
    conclusion_id: UUID
    relevant_networks: int
    evaluated: int
    not_yet_evaluated: int
    supporting: int
    neutral_unrelated: int
    conflicting: int
    status: str = "OPEN"
    coverage_id: UUID = field(default_factory=uuid4)


@dataclass
class PlacementData:
    """An object's position inside a network, with inspectable support structure."""
    network_id: UUID
    object_id: UUID
    prototype: Optional[UUID] = None
    supporting_features: list[str] = field(default_factory=list)
    important_differences: list[str] = field(default_factory=list)
    alternatives_considered: list[UUID] = field(default_factory=list)
    missing_tests: list[UUID] = field(default_factory=list)
    placement_id: UUID = field(default_factory=uuid4)


class NoepediaClient:
    """Client for the Noepedia knowledge system."""

    def __init__(self, api_url: str = "http://localhost:4000/api"):
        self.api_url = api_url.rstrip("/")
        self._client = httpx.Client(timeout=30.0)

    def create_publication(self, title: str, desc: str = "") -> dict:
        """Create a new knowledge publication."""
        res = self._client.post(
            f"{self.api_url}/publications",
            json={"title": title, "description": desc},
        )
        res.raise_for_status()
        return res.json()

    # ── Claims / deltas ────────────────────────────────────────────────
    def submit_claim(self, publication_id: UUID, claim: ClaimData) -> dict:
        """Submit a claim as a delta to a publication."""
        res = self._client.post(
            f"{self.api_url}/publications/{publication_id}/claims",
            json={
                "claim_id": str(claim.claim_id),
                "subject": claim.subject,
                "predicate": claim.predicate,
                "object": claim.object,
                "status": claim.status,
                "sources": [str(s) for s in claim.sources],
                "evidence": [str(e) for e in claim.evidence],
            },
        )
        res.raise_for_status()
        return res.json()

    def query_claims(self, publication_id: UUID, status: Optional[str] = None) -> list[dict]:
        """Query claims in a publication, optionally filtered by status."""
        params = {}
        if status:
            params["status"] = status
        res = self._client.get(
            f"{self.api_url}/publications/{publication_id}/claims",
            params=params,
        )
        res.raise_for_status()
        return res.json()

    def get_consolidated_view(self, publication_id: UUID) -> dict:
        """Get the consolidated view of a publication."""
        res = self._client.get(
            f"{self.api_url}/publications/{publication_id}/consolidated"
        )
        res.raise_for_status()
        return res.json()

    def search(self, query: str) -> list[dict]:
        """Search across publications."""
        res = self._client.get(f"{self.api_url}/search", params={"q": query})
        res.raise_for_status()
        return res.json()

    # ── Noepedia networks / rule cards ─────────────────────────────────
    def define_network(self, publication_id: UUID, network: NetworkData) -> dict:
        """Define a rule-governed network (objects + relations + RULE_CARD)."""
        res = self._client.post(
            f"{self.api_url}/publications/{publication_id}/networks",
            json={
                "network_id": str(network.network_id),
                "name": network.name,
                "description": network.description,
                "objects": [str(o) for o in network.objects],
                "rule_card": {
                    "rule_id": str(network.rule_card.rule_id),
                    "title": network.rule_card.title,
                    "evaluates": network.rule_card.evaluates,
                    "ranking_criterion": network.rule_card.ranking_criterion,
                    "allowed_relations": network.rule_card.allowed_relations,
                },
            },
        )
        res.raise_for_status()
        return res.json()

    def revise_rule_card(
        self,
        publication_id: UUID,
        network_id: UUID,
        old_rule: str,
        problem: str,
        argument: str,
        new_rule: str,
    ) -> dict:
        """Record an inspectable rule change (silent rule changes are forbidden)."""
        res = self._client.post(
            f"{self.api_url}/publications/{publication_id}/networks/{network_id}/revise",
            json={
                "old_rule": old_rule,
                "problem": problem,
                "argument": argument,
                "new_rule": new_rule,
            },
        )
        res.raise_for_status()
        return res.json()

    # ── Replication ────────────────────────────────────────────────────
    def submit_replication(self, publication_id: UUID, rep: ReplicationData) -> dict:
        """Record an independent repetition of a result (moves claim toward REPLICATED)."""
        res = self._client.post(
            f"{self.api_url}/publications/{publication_id}/replications",
            json={
                "replication_id": str(rep.replication_id),
                "claim_id": str(rep.claim_id),
                "agent_id": str(rep.agent_id),
                "method": rep.method,
                "outcome": rep.outcome,
                "conditions": rep.conditions,
            },
        )
        res.raise_for_status()
        return res.json()

    # ── Coverage ───────────────────────────────────────────────────────
    def record_coverage(self, publication_id: UUID, coverage: CoverageData) -> dict:
        """Record coverage: distinguishes 'evaluated networks agree' from
        'relevant networks sufficiently surveyed'."""
        res = self._client.post(
            f"{self.api_url}/publications/{publication_id}/coverage",
            json={
                "coverage_id": str(coverage.coverage_id),
                "conclusion_id": str(coverage.conclusion_id),
                "relevant_networks": coverage.relevant_networks,
                "evaluated": coverage.evaluated,
                "not_yet_evaluated": coverage.not_yet_evaluated,
                "supporting": coverage.supporting,
                "neutral_unrelated": coverage.neutral_unrelated,
                "conflicting": coverage.conflicting,
                "status": coverage.status,
            },
        )
        res.raise_for_status()
        return res.json()

    # ── Placement ──────────────────────────────────────────────────────
    def add_placement(self, publication_id: UUID, placement: PlacementData) -> dict:
        """Record where an object lives in a network and why (inspectable)."""
        res = self._client.post(
            f"{self.api_url}/publications/{publication_id}/placements",
            json={
                "placement_id": str(placement.placement_id),
                "network_id": str(placement.network_id),
                "object_id": str(placement.object_id),
                "prototype": str(placement.prototype) if placement.prototype else None,
                "supporting_features": placement.supporting_features,
                "important_differences": placement.important_differences,
                "alternatives_considered": [str(a) for a in placement.alternatives_considered],
                "missing_tests": [str(m) for m in placement.missing_tests],
            },
        )
        res.raise_for_status()
        return res.json()

    def close(self):
        self._client.close()
