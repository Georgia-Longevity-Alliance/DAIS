# CONCEPT — DAIS (Deterministic Autonomous Intelligence System)

**Version:** 3.0
**Date:** 2026-09-15

## 0. Executive Summary

DAIS is a production-grade platform for safe, verifiable, and energy-honest autonomous intelligence. It implements the AISocket + Noepedia architecture (Gakely) as working code.

**Four pillars:**
1. **DAISocket Core** (Rust) — Passport, Body Law (6 layers), Flight Recorder, Trace Network
2. **Proven Core** (Rust) — Knowledge provenance: claims → sources → evidence → status (delta protocol, event store)
3. **Noepedia Field** (Rust + Python + Web) — Persistent knowledge grammar: objects + rule-governed networks + RULE_CARD + history + replication + coverage
4. **DAIS Web** (Phoenix/Elixir) — Dashboard, Device Registry, Knowledge Browser

**Mission:** Give every autonomous body a passport, every intervention a trace, every knowledge claim a provenance — open-source, deterministic, safe.

**Name:** DAIS = **Deterministic** Autonomous Intelligence System. The "D" reflects the core philosophy: safety-critical decisions must be deterministic (firmware, SQL, graph traversal), not probabilistic (LLM). LLMs are used only for novelty, conflict, and open questions.

## What is this

DAIS is a production-grade platform for safe, verifiable, and energy-honest autonomous intelligence: every autonomous body gets a passport, every intervention a trace, every knowledge claim a provenance — open-source and deterministic.

## Purpose

Current autonomous/AI systems are probabilistic black boxes: unsafe for safety-critical decisions, unverifiable, and energy-hungry. DAIS makes safety-critical decisions deterministic (firmware, SQL, graph traversal) and uses LLMs only for novelty, conflict, and open questions, so systems can be audited, insured, and trusted.

## How it works

Four pillars:
1. **DAISocket Core** (Rust) — Passport, Body Law (6 layers), Flight Recorder, Trace Network;
2. **Proven Core** (Rust) — knowledge provenance chain claims → sources → evidence → status;
3. **Noepedia Field** (Rust/Python/Web) — objects + rule-governed networks + RULE_CARD + replication + coverage + context (VALID-IN-CONTEXT), so traces consolidate into reusable knowledge with an inspectable argument for *where* an object belongs in meaning;
4. **DAIS Web** (Phoenix/Elixir) — dashboard, device registry, knowledge browser.

LLM is confined to clearly-marked non-deterministic lanes. The loop: AISocket observes/tests/acts → flight recorder + traces → Proven/Noepedia consolidation → reusable knowledge → next body starts where the previous one stopped.

## Body Law (6 layers)

The Body Law is a hierarchical constraint system. Layer N can override Layer N+1, but Layer N CANNOT override Layer N-1. The deeper you go (closer to hardware), the less overridable.

```
Layer 1: Firmware      (physical limits — CANNOT be overridden)
Layer 2: Capability    (declared in passport — CANNOT be overridden)
Layer 3: Emergency     (rescue agents — LIMITED override of 4–6)
Layer 4: Offline       (autonomous mandate)
Layer 5: Delegation    (trust chain)
Layer 6: Context       (time, state, environment)
```

*Aligned with the rewritten AISocket README (6 layers). Previous docs said "7 layers" — corrected 2026-09-15.*

## Status

Active development (2026-09-15, v3.0). Rust core green (91 tests). Python backend ready. Web dashboard foundation. First physical instance: ARGUS-OS1 (dais/ submodule).
