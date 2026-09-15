# THEORY — AIS

**Version:** 1.0  
**Date:** 2026-08-01

## Theoretical Foundations

### 1. Passport = Socratic elicitation

The device passport is NOT filled by a form. It is elicited through dialogue. This is Socrates's method applied to device ontology:

> The owner KNOWS the device — its limits, its dangers, its quirks — but this knowledge is tacit, unstructured, and distributed across experience. The LLM interviewer, like Socrates, does not TEACH the owner anything. It EXTRACTS what the owner already knows into a formal structure.

**Formal claim:** For any device D with owner O, there exists a sequence of questions Q₁...Qₙ such that the passport P generated from answers A₁...Aₙ contains all safety-relevant information that O knows about D, with n proportional to device complexity.

### 2. Body Law = Hierarchical Safety

The 6-layer Body Law is an instance of hierarchical constraint systems:

```
Layer 1: Firmware (physical limits — CANNOT be overridden)
Layer 2: Capability (declared in passport — CANNOT be overridden)
Layer 3: Emergency (rescue agents — LIMITED override of 4-6)
Layer 4: Offline (autonomous mandate)
Layer 5: Delegation (trust chain)
Layer 6: Context (time, state, environment)
```

**Invariant:** Layer N can override Layer N+1, but Layer N CANNOT override Layer N-1. This is the opposite of typical security models (where higher layers have MORE power). In AIS, the DEEPER you go (closer to hardware), the LESS overridable.

### 3. Trace Network = Collective Learning

Every intervention leaves a trace: (diagnosis, actions, outcome, timestamp, references). When device B encounters anomaly X, it queries the trace network: "Has any device encountered X before?"

**Formula:**

```
Cost(anomaly X) = 
  O(1)           if ∃ trace with outcome=RESOLVED matching X
  O(LLM tokens)  if novel — LLM diagnoses, acts, records trace
  O(0)           subsequent — trace found, no LLM needed
```

The energy cost of solving X decreases to zero over time, asymptotically approaching O(1) as the trace network grows.

### 4. Proven = Structured Epistemology

Proven is the knowledge counterpart. It extends the trace network from "what happened" to "what is true." Every claim has:
- Status: PROPOSED → SUPPORTED → TESTED → REPLICATED (or: CONTESTED, REFUTED, SUPERSEDED)
- Sources: DOI, PMID, experiment ID, personal communication
- Evidence: supporting, contradicting, inconclusive, with strength 0.0–1.0

**Invariant:** PROPOSED ≠ FACT. Status is NEVER implicit. An LLM cannot mask a PROPOSED claim as established knowledge — the status field is structurally enforced.

### 5. Energy Honesty

```
LLM inference: ~1 J/token
SQL SELECT:    ~1 µJ
Rust match:    ~1 pJ
```

DAIS counts joules. Every operation is classified:
- **Green:** Deterministic, <1 µJ (SQL, Rust, graph traversal)
- **Yellow:** Small model inference, <1 mJ (local classifier, anomaly detector)
- **Red:** LLM inference, ~1 J/token (diagnosis, interview, synthesis)

The system minimizes Red operations by caching traces (Green lookups replace Red diagnoses) and by using Yellow pre-filters (only invoke LLM when anomaly is genuinely novel).

### 6. Noepedia Grammar (v3.0)

Noepedia stores knowledge OUTSIDE any model's weights as a persistent, addressable, inspectable field. Implemented in `core/src/noepedia.rs`.

**Three layers:**
1. **Object field** — an object (object/doc/person/component/source) enters with minimal interpretation.
2. **Semiotic/evaluative networks** — the same object participates in many independent projections: IS-PART-OF, IS-INSTANCE-OF, DIFFERS-FROM, SUPPORTS, CONTRADICTS, FUNCTIONS-AS, VALID-IN-CONTEXT, BELONGS-TO-PROTOTYPE, RANKED-BY-RELIABILITY.
3. **Evaluation process** — why an object is placed there, why the relation is trusted, what would move it.

**RULE_CARD:** every network has a declared rule (what it evaluates, how it ranks). The navigator reads the rule; it does not hold it in weights. Invariant: a network must stay faithful to its rule. Changing a rule is allowed; changing it silently is not — every change is an inspectable `RuleRevision` (old rule → problem → argument → new rule → re-evaluation of affected placements).

### 7. Replication Is an Operation, Not Decoration

A claim moves through states PROPOSED → SUPPORTED → REPLICATED ×1 … ×N → reusable knowledge. `Replication` records who/what repeated the result, under which conditions, and whether it supports/contradicts. Thresholds depend on domain and consequence. Popularity, authority, confidence, and eloquence are not substitutes for repetition.

### 8. Coverage ≠ Confidence

"The evaluated networks agree" is not the same as "the relevant networks have been sufficiently surveyed." `Coverage` records relevant vs evaluated vs not-yet-evaluated networks, and supporting vs neutral vs conflicting. A conclusion can look strong simply because only one relevant perspective has been consulted.

### 9. Observation / Inference / Knowledge Must Not Collapse

OBSERVATION ≠ HYPOTHESIS ≠ RESULT ≠ REPLICATION ≠ current KNOWLEDGE. OPEN and CONFLICT are legal knowledge states; the system must never silently convert PROPOSED→FACT, UNKNOWN→CONFIDENT ANSWER, or PARTIAL COVERAGE→COMPLETE UNDERSTANDING.

### 10. The Trace → Knowledge Loop (AISocket + Noepedia)

```text
AISocket
  observe / test / act → flight recorder + trace
Noepedia/Proven
  consolidate trace → claim with evidence + replication + coverage
next body
  starts where the previous one stopped (Green lookup, not Red LLM)
```

A trace is experience, not automatically knowledge. It becomes knowledge through comparison, replication, and consolidation under a RULE_CARD — never by being appended to an ever-larger prompt.

### 11. Zone of Proximal Development (Vygotsky) — implemented in `core/src/zpd.rs`

Vygotsky: higher functions form from the outside, through interaction. Learning happens in the **zone of proximal development (ZPD)** — between what the learner can do independently (*actual development*) and what it can do with a *more competent other*. Every ability passes three stages: **interpsychic** (other helps) → **extragsychic** (self-guided aloud) → **intrapsychic** (internalized, automatic).

DAIS maps this onto bounded autonomy:
- The **more competent other** is the LLM; it scaffolds *just at the boundary*, never doing what the body already knows.
- **Three zones:** `ActualDevelopment` (green, deterministic, cached) / `ProximalDevelopment` (LLM scaffolds at the edge) / `Inaccessible` (human or OPEN).
- **Scaffolding levels:** ObserveOnly → Hint → GuidedSteps → FullHelp → HumanRequired.
- **Fading:** after each successful scaffolded resolution the help decays geometrically (1.0 → 0.5 → 0.25 …). When fading reaches ~0 the skill is **internalized** — moved to actual development.
- **Internalization = Noepedia consolidation:** when the result is consolidated as a REPLICATED claim with sufficient coverage, the next occurrence is Green. This is the energy-honest asymptote `Cost(anomaly) → O(1)`.

```text
interpsychic  →  LLM scaffolds (red, ~1 J/token)
                 trace recorded
                 consolidated into Noepedia (REPLICATED)
extragsychic   →  fading: hint instead of full help
intrapsychic   →  internalized: green, deterministic, ~pJ
```
