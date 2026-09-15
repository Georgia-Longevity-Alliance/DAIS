"""consolidation.py — Trace → Noepedia consolidation with Vygotsky fading.

A trace is experience, not automatically knowledge. This module turns resolved,
repeated flight-recorder traces into Noepedia claims with provenance, and applies
ZPD fading so that over repetitions the same procedure needs less LLM help until
it is *internalized* (green, deterministic — cost → O(1)).

Pipeline (matches THEORY.md §10 + §11):
    flight-recorder trace (evidence of what happened)
      → group by signature (device, diagnosis, corrective action)
      → if RESOLVED and repeated independently → claim + replication records
      → ZPD fading: fraction of occurrences that still needed full LLM help
        decays; when it reaches ~0 the procedure is internalized (actual development).
"""
from dataclasses import dataclass, field
from typing import Dict, List, Optional
from uuid import UUID, uuid4

from .proven_api import ClaimData, NoepediaClient, ReplicationData


@dataclass
class TraceInput:
    """One flight-recorder trace (evidence of what happened, not truth)."""
    trace_id: str
    device_id: str
    agent_id: str                      # who/what handled it (lab, device, human)
    diagnosis: str
    action: str                        # corrective action that was taken
    outcome: str                       # resolved | mitigated | escalated | unresolved
    confidence: float = 0.5
    llm_invoked: bool = True           # whether this occurrence needed LLM help (for fading)
    timestamp: str = ""


def _norm(s: str) -> str:
    return " ".join(str(s).lower().split())


def signature(t: TraceInput) -> str:
    """Same problem + same corrective action → one reusable procedure.

    NOTE: device_id is deliberately NOT part of the grouping key — the trace
    network exists so that different devices/agents learn from each other.
    The device goes into the replication *conditions*, not the identity.
    """
    return f"{_norm(t.diagnosis)}::{_norm(t.action)}"


@dataclass
class ConsolidatedResult:
    """A procedure promoted from scattered traces to reusable knowledge."""
    claim_id: UUID
    signature: str
    subject: str                       # e.g. "photobleaching", "stage_jam"
    occurrences: int
    independent_repeats: int           # distinct agents/devices that confirmed it
    status: str                        # SUPPORTED | REPLICATED | OPEN
    replications: List[ReplicationData]
    fading: float                      # 1.0 = full LLM help needed; 0.0 = internalized
    internalized: bool


class Consolidator:
    """Group traces and consolidate resolved, repeated procedures into claims."""

    def __init__(self, client: Optional[NoepediaClient] = None,
                 min_independent_repeats: int = 2):
        self.client = client
        self.min_independent_repeats = min_independent_repeats

    def consolidate(self, traces: List[TraceInput]) -> List[ConsolidatedResult]:
        groups: Dict[str, List[TraceInput]] = {}
        for t in traces:
            groups.setdefault(signature(t), []).append(t)

        results: List[ConsolidatedResult] = []
        for sig, items in groups.items():
            resolved = [t for t in items if t.outcome == "resolved"]
            if not resolved:
                # unresolved but seen repeatedly → register OPEN
                if len(items) >= 2:
                    results.append(self._open(sig, items))
                continue
            independent = len({t.agent_id for t in resolved})
            # ZPD fading: fraction of resolved occurrences that still needed full LLM help
            llm_count = sum(1 for t in resolved if t.llm_invoked)
            fading = llm_count / len(resolved)
            internalized = fading <= 0.05
            status = (
                "REPLICATED"
                if independent >= self.min_independent_repeats
                else "SUPPORTED"
            )
            reps = [
                ReplicationData(
                    claim_id=uuid4(),
                    agent_id=uuid4(),
                    method=t.action,
                    outcome="supports",
                    conditions={"device": t.device_id, "diagnosis": t.diagnosis},
                )
                for t in resolved
            ]
            res = ConsolidatedResult(
                claim_id=uuid4(),
                signature=sig,
                subject=_norm(items[0].diagnosis),
                occurrences=len(items),
                independent_repeats=independent,
                status=status,
                replications=reps,
                fading=fading,
                internalized=internalized,
            )
            if self.client:
                self._submit(res)
            results.append(res)
        return results

    def _open(self, sig: str, items: List[TraceInput]) -> ConsolidatedResult:
        return ConsolidatedResult(
            claim_id=uuid4(),
            signature=sig,
            subject=_norm(items[0].diagnosis),
            occurrences=len(items),
            independent_repeats=0,
            status="OPEN",
            replications=[],
            fading=1.0,
            internalized=False,
        )

    def _submit(self, res: ConsolidatedResult) -> None:
        if self.client is None:
            return
        claim = ClaimData(
            subject=res.subject,
            predicate="resolved_by",
            object=res.signature.split("::")[-1],
            status=res.status,
        )
        # one call per consolidated procedure (client.create_publication first)
        for rep in res.replications:
            self.client.submit_replication(uuid4(), rep)


def load_traces(path: str) -> List[TraceInput]:
    """Load traces from a JSON list of trace objects."""
    import json

    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    return [TraceInput(**d) for d in data]
