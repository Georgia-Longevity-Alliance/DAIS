# MEMORY — DAIS

**Created:** 2026-07-31  
**Updated:** 2026-08-01

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
