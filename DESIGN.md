# DESIGN — AIS

**Version:** 1.0  
**Date:** 2026-08-01

## Design Decisions

### 1. Rust core + Python bridge + Elixir web

| Layer | Rationale |
|-------|-----------|
| **Rust** | Zero-cost safety for Body Law enforcement. Passport, Flight Recorder, and Event Store must be predictable — no GC pauses, no runtime exceptions. Rust's ownership model prevents data races in shared-memory trace buffer. |
| **Python** | LLM ecosystem lives here. OpenAI/Gemini/Ollama SDKs are Python-native. NumPy/SciPy for flight recorder analysis. The Python layer is the "soft tissue" — swap providers, add models, experiment — without touching the safety core. |
| **Elixir/Phoenix** | Real-time web UI with LiveView. Phoenix PubSub for multi-device streaming. Erlang/OTP supervision for long-running interview sessions and dashboard monitoring. |

### 2. Interview as deterministic state machine + optional LLM rephrasing

The passport interview IS a finite state machine — 6 phases, each with explicit transitions. This guarantees:
- No infinite loops
- Reproducible output
- Predictable question count

LLM is OPTIONAL — it only rephrases questions to sound more natural. Without LLM, the interview still works with template questions.

### 3. SQLite for local registry

PostgreSQL would be overkill for a single-microscope deployment. SQLite:
- Zero configuration
- Embedded in the Phoenix app
- Sufficient for hundreds of thousands of passports
- Can be upgraded to PostgreSQL when federated registry is needed

### 4. Google OAuth + anonymous mode

Google OAuth is the primary auth method (email verification, no password management). Anonymous mode (`/passport/new` without login) exists for:
- Quick demos
- Devices without internet-connected owners
- Testing

Anonymous passports are tagged `anonymous@local` and can be claimed later.

### 5. Body Law enforcement location

The 6-layer Body Law validator runs in Rust core — not in Python, not in prompts, not in web middleware. This is intentional:
- Layer 1 (firmware limits) CANNOT be bypassed by ANY software layer
- LLM prompt injection cannot disable safety
- Register-level constraints (torque, temp, laser power) are checked BEFORE any actuator moves

### 6. Energy architecture

```
LLM token:     ~1 J
SQL query:     ~1 µJ
Rust match:    ~1 pJ
```

DAIS enforces: LLM only for novel situations. 99.9% of operations are deterministic (SQL queries, Rust pattern matching, graph traversal). The LLM is called only when the flight recorder shows an anomaly that no prior trace has solved.
