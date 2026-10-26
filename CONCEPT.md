# CONCEPT — DAIS (Deterministic Autonomous Intelligence System)

**Version:** 3.0
**Date:** 2026-09-15

## 0. Executive Summary

DAIS is a production-grade platform for safe, verifiable, and energy-honest autonomous intelligence. It implements the AISocket + Aletheia architecture (Aletheia team) as working code.

**Four pillars:**
1. **DAISocket Core** (Rust) — Passport, Body Law (6 layers), Flight Recorder, Trace Network
2. **Proven Core** (Rust) — Knowledge provenance: claims → sources → evidence → status (delta protocol, event store)
3. **Aletheia Field** (Rust + Python + Web) — Persistent knowledge grammar: objects + rule-governed networks + RULE_CARD + history + replication + coverage
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
3. **Aletheia Field** (Rust/Python/Web) — objects + rule-governed networks + RULE_CARD + replication + coverage + context (VALID-IN-CONTEXT), so traces consolidate into reusable knowledge with an inspectable argument for *where* an object belongs in meaning;
4. **DAIS Web** (Phoenix/Elixir) — dashboard, device registry, knowledge browser.

LLM is confined to clearly-marked non-deterministic lanes. The loop: AISocket observes/tests/acts → flight recorder + traces → Proven/Aletheia consolidation → reusable knowledge → next body starts where the previous one stopped.

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

## Closed-loop laser cell-disruption capability (2026-10-20)

> Added from the Nikon GA+JOBS benchmark analysis (Kaneshiro et al. 2022, Quant Plant Biol 3:e30, DOI 10.1017/qpb.2022.24) — fully automated in-situ elimination of non-target cells on an inverted microscope. Master: `../docs/NIKON_AUTOMATED_LASER_SORTING_ANALYSIS_2026-10-20.md`.

### Canonical Passport capability: `targeted_cell_disruption_automated`

Builds on the existing OS1 V6 `ablation_laser` + `ablate_daughter_cells` (manual/scripted) by making the disruption **closed-loop and fully automated**:

| Field | Value |
|-------|-------|
| name | `targeted_cell_disruption_automated` |
| description | Automated closed-loop laser disruption of a defined set of non-target cells in situ: segment → classify → pre-ablation viability gate → keep-out masking → targeted irradiation → in-loop verification → per-cell QC. Preserves positional/lineage info (in-situ, not sorting). |
| risk | high |
| parameters | `target_class` (segment/class labels), `keep_out_radius_um` (mask, e.g. 60), `power_mw`, `duration_ms`, `verify_window_min`, `verify_markers` (e.g. H2B fragmentation, Centrin1 loss), `max_retries` (1; each retry +10% dose, capped by Body Law) |
| body-law constraints | phototoxicity ceiling (dose cap per Body Law L1/L2), no-fire on protected cells, no-touch zones during laser |
| verification | per-cell success + wrong-cell-kill → Flight Recorder; ablation pass/fail is a Flight-Recorder log entry |

### Loop primitives (first-class V9-BRAIN building blocks)

V9-BRAIN should expose **reusable loop primitives** (JOBS-like, from the analyzed Nikon workflow) so the same loop runs ablation (OS3), in-situ isolation (OS4), and routine acquisition (OS1):

```
acquire(tile) -> analyze(seg+class) -> gate(viability) -> mask(keep-out) -> act(laser/move) -> verify(pass/fail) -> qc(log)
```

Each primitive is: deterministic, schema-typed, logged to Flight Recorder, bounded by Body Law. The LLM assembles primitives; it never implements their internals (deterministic lane).

## Status

Active development (2026-09-15, v3.0). Rust core green (104 tests). Python backend ready. Web dashboard foundation. First physical instance: ARGUS-OS1 (dais/ submodule).
