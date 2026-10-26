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
| Elixir: auditor core API | 5 | status assertions (non-destructive, epochs), STOP CARD, dependency policy, composition policy |
| **Elixir total** | **13** | |
| **Grand total** | **122** (lib) + 1 doc-test + 13 (web) + 1 integration (first run) | +18 Auditor Core Harvest tests (status assertions, membrane, dependency policy, composition, journal signature, archive/compact, STOP CARD) + `tests/auditor_first_run.rs` ARGUS-OS1 end-to-end walk |

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

## §. Aletheia Harvest — 2026-10-12 (epistemic standard)

> Master: `docs/ALETHEIA_EPISTEMIC_HARVEST_2026-10-12.md` · Scientific confirmation: GRADE PMID 18436948; Murad New Evidence Pyramid PMID 27339128; Ioannidis PMID 16060722; Conant–Ashby DOI 10.1080/00207727008920220; Siebert MHC DOI 10.1007/s43681-022-00167-3; HART arXiv 2603.05828; REASONS arXiv 2405.02228; AgentHallu arXiv 2601.06818; Devitt ACD arXiv 2303.06813; requisite variety arXiv 1409.7475/2206.04896.

| Principle (Aletheia harvest) | DAIS evidence / target | Verified support |
|------------------------------|------------------------|------------------|
| Knowledge as status ladder, not float confidence | `KnowledgeStatus` (Proposed→Supported→Replicated/Refuted/Contested/Open/Superseded) | GRADE (PMID 18436948), Murad (PMID 27339128), TRL |
| OPEN ≠ negation; CONFLICT is legal | `KnowledgeStatus::Open` + ConflictRecord first-class | Known-unknowns (PMID 28584653); abduction survey (arXiv 2604.08016) |
| Replication = independent witness | `Replication` operation in aletheia.rs | Cross-modal confirmation; caution: cross-lingual transfer can import bias (arXiv 2305.12709) |
| Provenance = hallucination antidote | Proven chain (claims→sources→evidence→status) + traces | HART (2603.05828), REASONS (2405.02228), AgentHallu (2601.06818) |
| Tested by construction, not precedent | RuleCard invariant (network faithful to its rule) | Conant–Ashby (10.1080/00207727008920220), Ioannidis (16060722) |
| Subject = dominant signature in author column | **NEW**: author column + subjectivity criterion (§6 THEORY) | MHC Siebert (10.1007/s43681-022-00167-3); anti-rubber-stamp (2603.01202) |
| №0 = axiom, named axiom | Body Law layers 1–2 hard core; layers 3–6 revisable | Devitt ACD (2303.06813) |

## §. Aletheia Auditor Core Harvest — 2026-10-16 (executive discipline)

> Master: `docs/ALETHEIA_AUDITOR_CORE_HARVEST_2026-10-16.md`. Source: Aletheia ACT01–ACT20 /
> AUDITOR_CORE_SPEC (Aletheia doctrine core spec). External anchors verified via
> Crossref / PubMed / arXiv 2026-10-16. Level: **principles confirmed externally; Aletheia's concrete
> implementation is a working hypothesis, not a theorem.**

| Principle (auditor core) | DAIS evidence / target | Verified external support |
|------------------------|------------------------|---------------------------|
| TRACE CONTINUITY: committed transition may be compacted/superseded but never made historically unaddressable | event store + delta protocol; target: `Archive`/`Compact` with address preservation | Authenticated append-only logs: SoK Authenticated Prefix Relations (arXiv); 2P-BFT-Log (arXiv); Transparency Logs via Append-Only Auth. Dictionaries, 10.1145/3319535.3345652; Certificate Transparency, 10.1145/2668152.2668154 |
| HISTORY_STATUS ≠ EPISTEMIC_STATUS; non-destructive MARK (new assertion + epoch, old kept) | target: `HistoryStatus`/`CurrentEpistemicStatus` separation; validator emits new status assertion instead of in-place mutation | AGM belief revision: AGM Belief Revision, Semantically (arXiv); modal-logic translation of AGM axioms (arXiv); event-sourced agent audits: ESAA-Security (arXiv); The Log is the Agent (arXiv) |
| STOP/NEED regime (GO/ASK/BLOCK/NEED + STOP CARD); stopping at the right place is success | target: `StopCard` + `NeedClass` in `SessionOutcome`; LLM bridge maps GO/ASK/BLOCK/NEED | Selective prediction / abstention: Learning to Say "I Don't Know" (TechRxiv 10.36227/techrxiv.175682660.02761872/v1); Selective "Selective Prediction" (ACL 2024, 10.18653/v1/2024.findings-acl.767); Self-Trust and Epistemic Humility, 10.1093/oso/9780190864873.003.0014 |
| ASK ≠ NEED; NOT_FOUND ≠ absent (open-world) | `KnowledgeStatus::Open` + OpenSpec (existing); STOP CARD protocol for LLM lane | Open-world assumption: Hallucination Is Inevitable for LLMs with the OWA (10.20944/preprints202510.0689.v1); OWA in KR (10.1007/978-1-4419-9863-7_734) |
| Silence ≠ support (`NO VETO ≠ SUPPORTED`) | validator requires explicit support for promotion (existing two-witness warning) | Absence of evidence ≠ evidence of absence: null-result replication (arXiv); PubMed evidence-of-absence (42884584…) |
| Transaction membrane: delta cannot use rules it introduces to authorize itself; rule-set from committed state N-1 | target: snapshot-isolation of rules in `validator.rs`; self-reference ban | Reflective authorization / policy layers: Authorization Policy (10.1007/0-387-23483-7_20); policy validation framework (10.1109/csac.2003.1254336); Datalog stratified negation: Stratified Negation in Limit Datalog (10.24963/ijcai.2018/259) |
| Dependency policy lives in the network; rule-level conflict → addressable CONFLICT, no fabricated result | target: `DependencyPolicy` as addressable object; rule-level conflict containment | Dung abstract argumentation (arXiv: On the Semantics of AAF; Semi-Abstract Value-Based AF); multi-context reasoning (10.1016/j.datak.2022.102044) |
| Termination ≠ Confluence; cycle → BLOCK, no fabricated outcome | target: finite justification graphs in consolidator | Term rewriting theory: Simplifying Conditional TRS (10.1016/S0747-7171(87)80010-X); Fair Conditional TRS (10.1007/978-3-662-09691-8_11); confluence non-terminating (arXiv 1203.5121) |
| Same result ≠ same provenance; multiple justifications ≠ redundancy (ALTERNATIVE vs JOINT composition policy) | target: composition policy for multi-witness REPLICATED | Justification logic (arXiv: strong finite model property); Alternative Pathways to Proof (10.1093/he/9780198979340.003.0014); Minimal Sufficient Projections of Proof Certificates (10.2139/ssrn.7366621) |
| Addressable triplets & patterns; everything is a record that can be a target | RDF-based aletheia grammar (aletheia.rs) | Named graphs / RDF-star: Easy and Complex (10.1007/978-3-031-21422-6_18); RDF named graphs (10.59350/n84f2-8q140) |
| Provenance as agentic prerequisite (cross-cutting) | Proven chain + traces (existing) | PROV-AGENT (10.1109/escience65000.2025.00093); Responsible Agentic AI Requires Explicit Provenance (SSRN 10.2139/ssrn.6518458); PROV-O agent simulation (10.1145/2457317.2457377) |

**Weak spots (recorded in master doc §5):** abstention calibration trade-off (over-/under-abstention);
STOP-card taxonomy inflation risk; `CAN_ASK_OUTSIDE` budget control; membrane snapshot-isolation cost;
TRACE CONTINUITY needs Ed25519 signature to be a guarantee (currently discipline only — Phase 2 TODO
becomes a hard condition); NETWORK RULES vs EXECUTOR INVARIANTS boundary is itself OPEN.
**Not yet verified:** transfer of these principles to deterministic Rust on real ARGUS-OS1 traces.
