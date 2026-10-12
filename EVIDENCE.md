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
| **Rust total** | **98** (v1.0 snapshot was 25; now 102 with authority module) | |
| Elixir: interview | 3 | Full flow, simple device, snake_case |
| Elixir: controllers | 1 | Page controller |
| Elixir: Phoenix scaffold | 4 | Default Phoenix tests |
| **Elixir total** | **8** | |
| **Grand total** | **104** (lib) + 1 doc-test | |

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

## §. Noepedia Harvest — 2026-10-12 (epistemic standard)

> Master: `docs/NOEPEDIA_HARVEST_2026-10-12.md` · Scientific confirmation: GRADE PMID 18436948; Murad New Evidence Pyramid PMID 27339128; Ioannidis PMID 16060722; Conant–Ashby DOI 10.1080/00207727008920220; Siebert MHC DOI 10.1007/s43681-022-00167-3; HART arXiv 2603.05828; REASONS arXiv 2405.02228; AgentHallu arXiv 2601.06818; Devitt ACD arXiv 2303.06813; requisite variety arXiv 1409.7475/2206.04896.

| Principle (Noepedia harvest) | DAIS evidence / target | Verified support |
|------------------------------|------------------------|------------------|
| Knowledge as status ladder, not float confidence | `KnowledgeStatus` (Proposed→Supported→Replicated/Refuted/Contested/Open/Superseded) | GRADE (PMID 18436948), Murad (PMID 27339128), TRL |
| OPEN ≠ negation; CONFLICT is legal | `KnowledgeStatus::Open` + ConflictRecord first-class | Known-unknowns (PMID 28584653); abduction survey (arXiv 2604.08016) |
| Replication = independent witness | `Replication` operation in noepedia.rs | Cross-modal confirmation; caution: cross-lingual transfer can import bias (arXiv 2305.12709) |
| Provenance = hallucination antidote | Proven chain (claims→sources→evidence→status) + traces | HART (2603.05828), REASONS (2405.02228), AgentHallu (2601.06818) |
| Tested by construction, not precedent | RuleCard invariant (network faithful to its rule) | Conant–Ashby (10.1080/00207727008920220), Ioannidis (16060722) |
| Subject = dominant signature in author column | **NEW**: author column + subjectivity criterion (§6 THEORY) | MHC Siebert (10.1007/s43681-022-00167-3); anti-rubber-stamp (2603.01202) |
| №0 = axiom, named axiom | Body Law layers 1–2 hard core; layers 3–6 revisable | Devitt ACD (2303.06813) |
