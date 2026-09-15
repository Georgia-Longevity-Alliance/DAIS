# DAIS — Autonomous Intelligence Socket

**Open-source protocol for safe embodied AI + hallucination-resistant knowledge.**

DAIS gives every autonomous device a **passport** — so any LLM can safely understand it, help it, and leave a trace that every other machine learns from. Built as the implementation layer of the AISocket + Noepedia architecture:

| Protocol | Role |
|----------|------|
| **AISocket (DAISocket)** | Safe, bounded control of physical devices — Passport, Body Law, Flight Recorder, Trace Network |
| **Proven** | Structured knowledge with provenance — claims → sources → evidence → status (delta protocol, event store) |
| **Noepedia Field** | Persistent, addressable, inspectable knowledge — objects + rule-governed networks + history (RULE_CARD, replication, coverage) |

The loop: AISocket observes/tests/acts and leaves **traces** (evidence of what happened); Proven/Noepedia consolidate those traces into **reusable knowledge** (with provenance); the next device starts where the previous one stopped instead of rediscovering from zero.

**Principle:** safety-critical decisions are deterministic (firmware, Rust, SQL, graph traversal). LLMs are confined to marked non-deterministic lanes — novelty, conflict, open questions. `Cost(anomaly) → O(1)` as the trace/network grows.

## Quick Start

```bash
# Rust core (protocol: passport, body law, flight recorder, traces, deltas, noepedia grammar)
cd core && cargo build && cargo test

# Python backend (LLM bridge, Noepedia client, passport interview)
cd py_backend && pip install -e .

# Phoenix web (dashboard, registry, knowledge browser)
cd web && mix setup && mix phx.server
```

## Architecture

| Layer | Language | Role |
|-------|----------|------|
| Core | Rust | Passport, Body Law, Flight Recorder, Event Store, Delta/Validator/Consolidator, **Noepedia grammar** (`noepedia.rs`) |
| Backend | Python | LLM bridge, HTTP client, ML integration, Noepedia API client |
| Web | Elixir/Phoenix | Dashboard, Device Registry, Knowledge Browser (claims/evidence/review) |

## Noepedia grammar (core/src/noepedia.rs)

| Object | Purpose |
|--------|---------|
| `Observation` / `Test` / `ResultStatus` | Path from observation to knowledge, kept distinct |
| `Replication` | Independent repetition — an operation, not decoration |
| `RuleCard` (+ `RuleRevision`) | A network's declared rule; rule changes are inspectable |
| `Network` / `Relation` / `RelationKind` | Objects + relations + rule card + history |
| `Placement` / `PlacementArgument` | Where an object lives in a network, and why |
| `Coverage` | Relevant vs evaluated networks — coverage ≠ confidence |
| `KnowledgeContext` | VALID-IN-CONTEXT conditions |
| `KnowledgeStatus` | PROPOSED / SUPPORTED / REPLICATED / REFUTED / CONTESTED / OPEN / VALID-IN-CONTEXT / SUPERSEDED |
| `Session` / `Outcome` | Bounded episodes of work and real-world feedback |

**Invariants:**
- A network must remain faithful to the rule by which it evaluates its objects.
- Changing a rule is allowed; changing it silently is not (RuleRevision history).
- PROPOSED ≠ FACT; OPEN and CONFLICT are legal knowledge states.
- A trace is evidence of what happened, not automatically truth.

## Status

- ✅ Rust core (v0.1): passport, body law, flight recorder, trace network, delta/event-store/validator/consolidator, **noepedia grammar** — 91 tests green.
- ✅ Python backend: `llm_bridge`, `client`, `passport_interview`, `noepedia_api` (networks, rule cards, replication, coverage, placement).
- 🟡 Phoenix web: dashboard, registry, knowledge browser (foundation).
- 🟢 **First physical instance: ARGUS-OS1** (autonomous centriole-tracking microscope, mounted as `dais/` submodule).

## License

Core (Rust): Apache 2.0 · Proven/Noepedia components: AGPL v3.0 · Web: MIT
*Born from DAISocket + Proven. Built for ARGUS-OS1. Aligned with the AISocket + Noepedia architecture (Gakely).*
