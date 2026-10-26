# CODE ANALYSIS — DAIS core (2026-10-12)

**Scope:** `core/src/*.rs` — logic review, contradictions, fixes.
**Result:** 102 tests green, 0 warnings (was 98 / 8 warnings).

## Found: contradictions & inconsistencies

### 1. Three divergent status ladders (deepest issue)
| Enum | Location | Variants |
|------|----------|----------|
| `ClaimStatus` | `types.rs` | Proposed, Supported, Tested, Replicated, Contested, Refuted, Superseded, Stale, Open |
| `KnowledgeStatus` | `aletheia.rs` | Proposed, Supported, Replicated, Refuted, Contested, Open, ValidInContext, Superseded |
| `ClaimStatus` (local) | `argus.rs` | Observed, Inferred, Hypothesized, Falsified — **shadowed `types::ClaimStatus`** |

The same "knowledge state" concept was modeled three ways, disagreeing on
variants (Tested/Stale only in one; ValidInContext only in another). `argus.rs`
re-declared a same-named enum, hiding the types-level one — a collision that
silently diverges.

### 2. Validator transition logic
- `is_valid_transition` was **string-based** (fragile, no type safety).
- Missing `ValidInContext`, and the ladder didn't include `Tested`/`Stale` paths.
- No enforcement of the **two-witness rule** for REPLICATED (Aletheia harvest).

### 3. OPEN without a validation rule
`KnowledgeStatus::Open` existed but OPEN ≠ denial was not representable:
no constraints / no rule that would close the question (harvest requirement).

### 4. Body Law: docs vs code
Docs canonically say **6 layers**; `body_law.rs` header claimed "7-layer" and
implemented 8 checks (0 Hardware Safety … 7 RBAC). Unclear model → drift.

### 5. Stale test counts
EVIDENCE.md said 33, README/CONCEPT/STATE said 91; actual was 98.

### 6. Warnings & logic gaps
- Unused imports (`EventType`, `Severity`, `Id`, `SafetyIntegrityLevel`).
- Deprecated `rand::thread_rng()` / `gen_range()`.
- `ros2.rs::simulate_body_law` **ignored the command entirely** — a placeholder
  that accepted any command on any registered topic (logic gap).

## Fixes applied

| # | Fix | File(s) |
|---|-----|---------|
| 1 | `KnowledgeStatus` = canonical ladder (+`Tested`, `Stale`); `ClaimStatus` → alias; `argus::ClaimStatus` → `ArgusEvidenceStatus` + `to_knowledge_status()` | `aletheia.rs`, `types.rs`, `argus.rs` |
| 2 | `TryFrom<&str>` for `KnowledgeStatus`; validator type-safe; full transition graph incl. ValidInContext/Tested/Stale; two-witness warning on REPLICATED | `aletheia.rs`, `validator.rs` |
| 3 | `OpenSpec { address, constraints, validation_rule }` | `aletheia.rs` |
| 4 | New `authority.rs`: `AuthorColumn`, `subjectivity()` = 1 − foreign-authored wakings/all, `is_subject()` (>½), `is_observer()`, `StatuteNo0` (axiom, named axiom), `observer_never_subject()` | `authority.rs` (new), `lib.rs` |
| 5 | Body Law header: 6 canonical layers + 2 boundary gates | `body_law.rs` |
| 6 | Removed unused imports; `rand::rng().random_range(...)`; `simulate_body_law` now matches `command.topic` vs safe topic | `fleet.rs`, `neural.rs`, `federated.rs`, `ros2.rs` |
| 7 | Test counts → 102; TODO Phase 4 checkboxes | `README.md`, `CONCEPT.md`, `STATE.md`, `EVIDENCE.md`, `TODO.md` |

## Completed afterwards (2026-10-12)
- REQUIRED_ROLE as a first-class relation kind: `RelationKind::RequiresRole` + `RoleRequirement` (constraints + validation rule) in `aletheia.rs` + test.
- Glossary: `docs/GLOSSARY_KNOWLEDGE_STATUS.md` — KnowledgeStatus → GRADE/Murad/CEBM/TRL mapping, independence criterion for Replication, κ guidance.
- Journal-location decision recorded: journal = inside the field; act-as-performance = outside by design (`docs/ALETHEIA_EPISTEMIC_HARVEST_2026-10-12.md` §F).
- Result: 104 tests green, 0 warnings.
