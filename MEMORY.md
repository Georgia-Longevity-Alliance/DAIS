# MEMORY — DAIS

**Created:** 2026-07-31  
**Updated:** 2026-09-15

## 2026-10-16 — Demon Core Harvest IMPLEMENTED (P0–P2; 104 → 122 tests)

### Decisions
1. **P0–P2 of the Demon Core roadmap implemented in Rust core + Python.** All new behavior is
   additive; the canonical `KnowledgeStatus` ladder and wire format are unchanged.

### What was built (Rust core)
- **Non-destructive status (DCT11–12):** `HistoryStatus` + `StatusAssertion` (epoch-scoped) in
  `noepedia.rs`; `event_store.rs::replay` now APPENDS a status assertion instead of overwriting
  (`current_epistemic_status` = last assertion; `status` kept for compatibility; `history_status`
  = committed-fact dimension). Relate now populates the relations projection.
- **STOP/NEED (DEMON §7–8):** `RegimeState` (GO/ASK/BLOCK/NEED), `NeedClass` (12-class taxonomy),
  `StopCard`; `Session` carries `regime` + `stop_cards`; `OutcomeKind::Blocked/Need`.
- **Transaction membrane (DCT08–09):** `validator.rs::membrane_check` — a delta cannot use rules
  it introduces to authorize itself (SELF_REFERENCE_BAN; rule-set from committed N-1 snapshot).
- **Dependency policy (DCT05–07):** `DependencyPolicy` (ROLE=POLICY) + `dependency_policy_check`
  + `find_applicable_policy` — upstream transition with dependents and no policy → lawful BLOCK
  (MISSING_DEPENDENCY_POLICY / NEED / MISSING_RULE).
- **Composition semantics (DCT19–20):** `JustificationComposition` (ALTERNATIVE/JOINT),
  `CompositionPolicy`, `evaluate_composition` → `CompositionVerdict`.
- **Archive/Compact (TRACE CONTINUITY):** `event_store.rs::archive`/`compact`/`check_trace_continuity`
  — addresses preserved via `compacted_into` + summary `compacted_ids`.
- **Ed25519 journal signing (TRACE CONTINUITY, tamper-evident):** `JournalGuard` signs the canonical
  log digest; `EventStore::sign_log`/`verify_log`; compaction/archiving are re-signable events, silent
  rewrites fail verification.

### What was built (Python)
- `llm_bridge.py`: `classify_regime(text)` → GO/ASK/BLOCK/NEED + `StopCard`/`RegimeVerdict`;
  AMBIGUOUS_MATCH handled as NEED (can ask a disambiguating question).
- `proven_api.py`: `submit_status_assertion`, `add_dependency_policy`, `register_composition_policy`,
  `record_stop_card`.

### Key insight
A status transition is now **an addressable event, not a field overwrite** — this is the single
change that makes DAIS's append-only log actually trace-continuous. The membrane + dependency-policy
checks turn "the LLM proposed a status change" into a lawfully bounded operation: no self-serving
rules, no undefined downstream consequences.

## 2026-10-16 — Demon Core Harvest (wave 3: DCT01–20) — analysis only, code P0 not yet done

### Decisions
1. **Third Noepedia wave (DCT01–DCT20 / DEMON_CORE_SPEC, 2026-10-16) analyzed & harvested into DAIS** —
   master `docs/NOEPEDIA_DEMON_CORE_HARVEST_2026-10-16.md`. The wave-2 harvest (author column, №0,
   OpenSpec) covered *epistemics*; wave 3 covers *executive discipline*: how the engine stops, does not
   invent, does not erase history, and cannot self-legitimize.
2. **TOP-3 for DAIS** (priority order): (1) TRACE CONTINUITY + HISTORY/EPISTEMIC status separation +
   non-destructive MARK; (2) STOP/NEED regime + STOP CARD as first-class session outcome and LLM-lane
   protocol; (3) transaction membrane (self-reference ban: delta N vs rule-set N-1) + dependency-policy
   in the network + rule-level conflict containment.
3. **Scientific confirmation performed** (Crossref/PubMed/arXiv 2026-10-16): abstention/selective
   prediction, OWA, authenticated append-only logs, Dung argumentation, term-rewriting termination≠confluence,
   AGM belief revision, PROV-AGENT — anchors recorded in EVIDENCE.md + master doc §4.
4. **Weak spots recorded honestly** (master §5): abstention calibration trade-off, STOP-taxonomy
   inflation, membrane snapshot cost, Ed25519 needed for real TRACE CONTINUITY (moved from optional to
   hard condition), NETWORK RULES vs EXECUTOR INVARIANTS boundary still OPEN. Transfer to deterministic
   Rust on real ARGUS-OS1 traces NOT yet verified.
5. **Occam applies to DAIS too**: do not port all 12 findings; port only what closes a real gap on
   ARGUS-OS1. P0 = status separation + STOP CARD; P1 = membrane + dependency-policy.

### What was written
- `docs/NOEPEDIA_DEMON_CORE_HARVEST_2026-10-16.md` (new master: 15 findings, top-3, confirmation table,
  weak spots, meta-analysis, roadmap P0–P3)
- `THEORY.md` §8–11 (trace continuity, STOP/NEED, membrane, termination≠confluence/composition)
- `EVIDENCE.md` §Demon Core Harvest (confirmation table + weak spots)
- `TODO.md` Phase 5 (P0–P3 tasks)
- `docs/SIMILAR_IDEAS.md` (new similar-ideas additions)
- `docs/GLOSSARY_KNOWLEDGE_STATUS.md` (history/epistemic split + composition semantics)

### Key insight
DAIS already had 80% of the append-only foundation (event store + delta + validator). The real gap is
**in-place status mutation** — it silently violates TRACE CONTINUITY. Separating
`HistoryStatus`/`CurrentEpistemicStatus` + epoch is the single highest-value code change.

## 2026-09-15 — Noepedia grammar integrated (v3.0)

### Decisions
1. **Noepedia grammar as a Rust module** (`core/src/noepedia.rs`) — objects, networks, RULE_CARD, replication, coverage, context, placement, session/outcome as first-class types, exported in prelude. Aligns with the rewritten Gakely Noepedia README (1029 lines) and AISocket README.
2. **Body Law fixed to 6 layers** (firmware, capability, emergency, offline, delegation, context) per the rewritten AISocket README; earlier docs said "7 layers".
3. **Four pillars** — DAISocket Core, Proven Core, Noepedia Field, DAIS Web.
4. **Python Noepedia client extended** — `define_network`, `revise_rule_card`, `submit_replication`, `record_coverage`, `add_placement`.

### What was built
- `core/src/noepedia.rs`: Network+RuleCard+RuleRevision, Relation/RelationKind, Replication, Coverage, KnowledgeContext, Observation/Test, Placement/PlacementArgument, Session/Outcome, KnowledgeStatus (OPEN/CONFLICT/VALID-IN-CONTEXT). 4 new unit tests; full suite 91 green.
- README/CONCEPT/THEORY/STATE updated to v3.0.

### Key insight
DAIS already had the Proven half (claims→sources→evidence→status, delta/event-store/consolidator with OPEN/CONFLICT). The gap was Noepedia's *semiotic layer*: explicit networks with RULE_CARD, coverage (≠ confidence), and replication as a first-class operation. That is what turns traces into reusable, inspectable knowledge rather than an ever-larger prompt.

## 2026-08-01 — Full Implementation Day

### Decisions
1. **Local SQLite registry** instead of googuly.online dependency. System must be self-contained. API: POST/GET /api/passports.
2. **Elixir interview port** — Interview state machine rewritten in pure Elixir for Phoenix integration. Python version kept for CLI/API use.
3. **Anonymous mode** — `/passport/new` works without Google login. Anonymous passports tagged `anonymous@local`.
4. **Core files** — Added DESIGN.md, THEORY.md, EVIDENCE.md. 11/11 complete.

### What was built
- PassportInterview: 6-phase Socratic interview (Python 550+ lines, Elixir 650+ lines)
- ARGUS-OS1 V6 passport example: 17 capabilities, 9 forbidden actions, 650+ lines JSON
- Phoenix LiveView: chat interface for passport creation
- Dashboard LiveView: registered device table with detail view
- REST API: POST/GET /api/passports, GET /api/passports/:id
- Google OAuth via ueberauth_google
- SQLite database: users + passports tables

### Issues resolved
- Fixed capability_detail phase — was stuck in parameter loop → restructured with sub-step counter (0→5)
- Fixed forbidden phase — "yes" to "more?" was creating a forbidden named "yes" → added explicit "yes" handler
- Fixed parse_param — `%{param | "constraints" => ...}` KeyError when key missing → Map.put
- Fixed clause ordering warning → moved handle_auto_step after all handle_phase clauses
- Fixed unused variable warning → restructured param building without shadow binding

### Key insight
The passport interview IS the product, not the data structure. The Rust Passport struct defines the TARGET. The interview is the PROCESS. Without the interview, the passport is just a struct. With the interview, it's a Socratic method for extracting tacit device knowledge into formal safety constraints.

## 2026-07-31 — Project Creation
Decision: Created DAIS as umbrella project under Marketing/ — merging DAISocket + Proven into single implementation.
Rationale:
- Gakely's DAISocket and Proven are conceptually strong but lacked code
- ARGUS-OS1 is the perfect first integration target
- Rust for core protocol (safety + performance), Python for AI/ML, Phoenix for web
Key insights from code review:
- DAISocket: Python "Ready" but no Python code in repo — need to build it
- Proven: pure concept paper, no implementation — need MVP with 3-4 object types
- Both share architectural DNA: addressable, append-only, deterministic safety boundaries

## 2026-10-12 — Noepedia Harvest: author column + №0 axiom

### Decisions
1. **Author column as first-class theory+code** — the second-wave external review of the Noepedia philosophy produced two deliverables DAIS adopts: (a) author-column transition rule v0.1 (subjectivity = 1 − foreign share over window W, threshold >½ by count), and (b) №0 axiom (option B, "axiom named axiom").
2. **№0 = root record; Body Law split** — layers 1–2 (Firmware, Capability, CANNOT be overridden) are the axiom's hard core; layers 3–6 are the revisable part, amended by sessions with a quorum.
3. **Observer = read-only, never a subject** — RBAC Observer reads without writing the column; observer and subject are non-overlapping classes; "the field is honest because its guarantees live outside the field."
4. **Journal is inside the field** — provenance records are SPO triplets (noepedia.rs), so the "two floors" (content vs act-of-recording) collapse for labels; act-as-performance stays outside by design (OPEN-BY-DESIGN).
5. **Open = unknown + constraints + validation rule** — add `validation_rule` to `KnowledgeStatus::Open`; OPEN is never rendered as "absent/denied".

### What was built (docs)
- `docs/NOEPEDIA_HARVEST_2026-10-12.md` (8 principles → DAIS mapping, rule, axiom, corollaries, weaknesses, refs)
- `THEORY.md` §6 (author column & subjectivity) + §7 (№0 axiom)
- `EVIDENCE.md` §Noepedia Harvest (epistemic standard + verified refs)
- `TODO.md` Phase 4 (implementation tasks)

## 2026-10-12 (2) — Code analysis & fixes: contradictions resolved

### Analysis (found)
1. **Three divergent status ladders** — `types::ClaimStatus` (9 variants), `noepedia::KnowledgeStatus` (8 variants), `argus::ClaimStatus` (Observed/Inferred/Hypothesized/Falsified, name collision shadowing `types::ClaimStatus`). Same concept modeled 3 ways → inconsistency.
2. **Validator transitions were string-based** and lacked ValidInContext/Tested/Stale paths; no two-witness rule for REPLICATED.
3. **OPEN had no validation rule** (Noepedia harvest: OPEN = addressable unknown + constraints + validation rule).
4. **Body Law code claimed "7-layer"** (0–7) while docs canonically say 6 layers + 2 boundary gates.
5. **Stale test counts** in docs (EVIDENCE 33 / README·CONCEPT·STATE 91 vs actual 98).
6. **8 build warnings** (unused imports, deprecated rand::thread_rng/gen_range, unused command in ros2 simulated body law that ignored the command — a logic gap).

### Fixes
1. **Unified the ladder:** `KnowledgeStatus` is now the canonical K-ladder (added `Tested`, `Stale`); `types::ClaimStatus` is an alias; `argus::ClaimStatus` renamed to `ArgusEvidenceStatus` with `to_knowledge_status()` mapping. Wire format unchanged (UPPERCASE).
2. **Validator now type-safe** on `KnowledgeStatus` (via `TryFrom<&str>`), full transition graph incl. ValidInContext/Tested/Stale, plus a two-witness warning for transitions to REPLICATED.
3. **`OpenSpec`** (address + constraints + validation_rule) added to `noepedia.rs`.
4. **New module `core/src/authority.rs`** — author column v0.1 (`AuthorColumn`), subjectivity criterion `1 − foreign-authored wakings/all`, `is_subject` (>½), `is_observer`, `StatuteNo0` (axiom named axiom), `observer_never_subject` theorem. Wired into lib.rs + prelude.
5. **Body Law model aligned:** 6 canonical layers + 2 boundary gates (Hardware Safety, RBAC) documented in code header.
6. **Warnings fixed:** unused imports, `rand::rng().random_range(...)`, `simulate_body_law` now uses the command (topic must match the safe topic).
7. **Docs updated:** test counts → 102; TODO Phase 4 checkboxes.

### Tests
102 passed, 0 failed, 0 warnings (was 98, 8 warnings).

## 2026-10-12 (3) — Phase 4 completed: REQUIRED_ROLE, glossary, journal decision

### Decisions
1. **REQUIRED_ROLE as first-class relation kind** — `RelationKind::RequiresRole` + `RoleRequirement { role, constraints, validation_rule }` in `noepedia.rs`. A formed unknown (role known, identity unknown) must carry a validation rule, or it is apophenia.
2. **Glossary mapping adopted** — `docs/GLOSSARY_KNOWLEDGE_STATUS.md`: KnowledgeStatus → GRADE (PMID 18436948) / Murad pyramid (PMID 27339128) / Oxford CEBM / NASA TRL; independence criterion for Replication→REPLICATED (≥2 witnesses differing in agent/method/source); κ ≥ 0.6 accept, κ ≥ 0.8 strong, < 0.6 do not promote.
3. **Journal location — decision made:** journal = inside the field (provenance objects are SPO triplets per README 127/153/484/701); act-as-performance = outside by design (the same residue as the story's smile, OPEN-BY-DESIGN). No further README change needed.

### Result
- 104 tests green, 0 warnings (was 102).
- TODO Phase 4 fully checked; CODE_ANALYSIS "Remaining" closed.
