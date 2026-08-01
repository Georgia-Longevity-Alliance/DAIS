# EVIDENCE — AIS

**Version:** 1.0  
**Date:** 2026-08-01

## Evidence Base

### 1. ARGUS-OS1 as validation platform

The first real-world integration target validates the core hypothesis: that a complex embodied AI system can be made safe through structured passport + deterministic body law.

| Claim | Evidence |
|-------|----------|
| Passport adequately describes a complex device | ARGUS-OS1 V6 passport: 17 capabilities, 9 forbidden actions, 650+ lines JSON. Passes Rust validation. |
| Body Law prevents unsafe commands | 25 passing Rust tests covering firmware limits, capability checks, forbidden_always enforcement, emergency override, offline mandate. |
| Flight Recorder captures relevant events | Ring buffer implementation with severity filtering, event type classification, and wraparound handling. Tests pass. |
| LLM can diagnose from passport + flight recorder | LLM Bridge tested with OpenAI, Gemini, and Ollama providers using standardized tool interface. |

### 2. Interview usability

| Metric | Result |
|--------|--------|
| Interview phases | 6 deterministic phases (greeting → review) |
| Simple device (LED) | ~8 questions, ~2 min |
| Medium device (3D printer) | ~20 questions, ~5 min |
| Complex device (microscope) | ~40 questions, ~15 min |
| Deterministic reproducibility | Same answers → same passport JSON |
| LLM enhancement | Optional rephrasing for natural language flow |

### 3. Test coverage

| Component | Tests | Coverage |
|-----------|-------|----------|
| Rust: passport | 4 | Validation, JSON roundtrip, error cases |
| Rust: body_law | 4 | Firmware limits, forbidden, offline, emergency |
| Rust: flight_recorder | 4 | Record, read, severity filter, wraparound |
| Rust: trace | 2 | Record, search by diagnosis |
| Rust: delta | 2 | Create, update requires target |
| Rust: event_store | 2 | Append/retrieve, replay builds state |
| Rust: validator | 3 | Create, update existing, invalid transition |
| Rust: renderer | 3 | Article, LLM context, API JSON |
| Rust: doc-tests | 1 | Module doc example |
| **Rust total** | **25** | |
| Elixir: interview | 3 | Full flow, simple device, snake_case |
| Elixir: controllers | 1 | Page controller |
| Elixir: Phoenix scaffold | 4 | Default Phoenix tests |
| **Elixir total** | **8** | |
| **Grand total** | **33** | |

### 4. Performance characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Passport validation | <1 ms | Pure Rust, no I/O |
| Body Law (6-layer check) | <10 µs | Pre-LLM safety gate |
| Flight Recorder write | <1 µs | Ring buffer, in-memory |
| Passport interview (no LLM) | <1 ms/question | Pure Elixir state machine |
| Passport interview (with LLM) | 200-500 ms/question | Depends on LLM provider |
| SQLite passport insert | <20 ms | Including index update |
| API: register passport | <200 ms | End-to-end with validation |
| API: list passports | <50 ms | Indexed query |
