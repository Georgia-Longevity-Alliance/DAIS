# CONCEPT — DAIS (Deterministic Autonomous Intelligence System)

**Version:** 2.0
**Date:** 2026-08-02

## 0. Executive Summary

DAIS is a production-grade platform for safe, verifiable, and energy-honest autonomous intelligence.

**Three pillars:**
1. **DAISocket Core** (Rust) — Passport, Body Law (7 layers), Flight Recorder, Trace Network
2. **Proven Core** (Rust) — Knowledge provenance: claims → sources → evidence → status
3. **DAIS Web** (Phoenix/Elixir) — Dashboard, Device Registry, Knowledge Browser

**Mission:** Give every autonomous body a passport, every intervention a trace, every knowledge claim a provenance — open-source, deterministic, safe.

**Name:** DAIS = **Deterministic** Autonomous Intelligence System. The "D" reflects the core philosophy: safety-critical decisions must be deterministic (firmware, SQL, graph traversal), not probabilistic (LLM). LLMs are used only for novelty, conflict, and open questions.

## What is this

DAIS is a production-grade platform for safe, verifiable, and energy-honest autonomous intelligence: every autonomous body gets a passport, every intervention a trace, every knowledge claim a provenance — open-source and deterministic.

## Purpose

Current autonomous/AI systems are probabilistic black boxes: unsafe for safety-critical decisions, unverifiable, and energy-hungry. DAIS makes safety-critical decisions deterministic (firmware, SQL, graph traversal) and uses LLMs only for novelty, conflict, and open questions, so systems can be audited, insured, and trusted.

## How it works

Three pillars: (1) DAISocket Core (Rust) — Passport, Body Law (7 layers), Flight Recorder, Trace Network; (2) Proven Core (Rust) — knowledge provenance chain claims → sources → evidence → status; (3) DAIS Web (Phoenix/Elixir) — dashboard, device registry, knowledge browser. LLM is confined to clearly-marked non-deterministic lanes.

## Status

Active development (2026-08-02, v2.0). Rust cores are the foundation; Web dashboard follows. Production target: embedded and server deployments with auditable traces.
