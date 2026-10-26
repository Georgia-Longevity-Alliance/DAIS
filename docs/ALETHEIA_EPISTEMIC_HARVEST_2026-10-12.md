# ALETHEIA EPISTEMIC STANDARD — DAIS application (2026-10-12)

**Source:** Correspondence (2026-10-12), Aletheia doctrine core spec,
second wave of external review. **Master analysis:** `~/Desktop/Services/docs/ALETHEIA_EPISTEMIC_HARVEST_2026-10-12.md`.
**Companion (ready to send):** `~/Desktop/Services/docs/ALETHEIA_AUTHORSHIP_LEDGER_2026-10-12.md`.

**Why this belongs in DAIS:** DAIS implements the AISocket + Aletheia architecture as working code.
The review produced two deliverables DAIS should carry as first-class theory + code:
**(1) the author-column transition rule (v0.1)** — who is actually in control of an autonomous body;
**(2) the №0 axiom (option B)** plus two corollaries (reactivity, read-only observer).

---

## A. Eight principles → DAIS mapping

| # | Principle | DAIS component (already present) | Action |
|---|-----------|----------------------------------|--------|
| 1 | **Hallucination = provenance/attribution failure** (record that lost its label and was taken as sensory) | Proven (claims→sources→evidence→status); Flight Recorder traces; invariant "a trace is evidence of what happened, not automatically truth" | Enforce: any LLM output entering the field carries provenance; a record without a label is a **defect**, not "no data". Attribution in agent steps is hard — do not trust "the model will figure it out" |
| 2 | **K-ladder (verifiable steps) instead of float confidence** | `KnowledgeStatus` = Proposed/Supported/Replicated/Refuted/Contested/Open/Superseded — **already a K-ladder** | Map explicitly to GRADE (PMID 18436948), Murad pyramid (PMID 27339128), Oxford CEBM, NASA TRL; document the mapping in the glossary |
| 3 | **Two independent witnesses / replication** | Aletheia `Replication` ("an operation, not decoration") | Require ≥2 independent replications/sources for Supported→Replicated; define *independence* criterion + inter-rater κ; cross-modal, not same-pool (cross-lingual control can import bias — arXiv 2305.12709) |
| 4 | **OPEN ≠ negation** | `KnowledgeStatus::Open`; CONFLICT is first-class | Ensure Open is never rendered as "absent/denied". Open = addressable unknown + **constraints + validation rule** (add a `validation_rule` field to Open records) |
| 5 | **Formed unknown / REQUIRED_ROLE** (relation requires a participant; identity unknown) | Aletheia relation/placement grammar | Add REQUIRED_ROLE as a first-class relation kind (role known, identity unknown) with constraints; without a validation rule it becomes apophenia (false role projection) |
| 6 | **Knowledge tested by construction, not precedent** | Invariant "a network must remain faithful to the rule by which it evaluates its objects" | Validation = build the scene (replicate/test), not "someone wrote it". Every good regulator must be a model (Conant–Ashby, DOI 10.1080/00207727008920220); precedent is systematically unreliable (Ioannidis, PMID 16060722) |
| 7 | **Inventory = exorcism** (no address → cannot be rewritten) | Addressable objects are core; observability | Audit = full inventory first (addresses), then fixes; you can only regulate what you can distinguish (Ashby requisite variety — arXiv 1409.7475, 2206.04896) |
| 8 | **Author column / Meaningful Human Control** (subject = dominant signature in own decision vector; AGI threshold = right to write, not model power) | **NEW — to implement** (see §B) | Add author column to Proven/Aletheia; who writes the decision vector = who controls (Siebert MHC, DOI 10.1007/s43681-022-00167-3; Devitt ACD, arXiv 2303.06813; anti-rubber-stamp, arXiv 2603.01202) |

---

## B. Author-column transition rule (v0.1) — to implement

```
RULE (v0.1) — author-column transition
· Column of record R: signature S (provenance object) · door D · time T (F1).
· The column records the WRITING event (who placed the knot / issued the mandate), not the waking event.
· Subjectivity of knot K over window W = 1 − (foreign signatures in waking records / all).
· Threshold v0: > ½ over W, by count; weights after bench.
· Meta: this rule is itself a knot; its authors are recorded in column №0.
```

Example: Vano wakes three times — A (placed by Vano), B (placed by Vano yesterday), C (placed by Niko).
Subjectivity = 1 − 1/3 = 2/3 > ½ → Vano is the subject of this window.

**DAIS mapping:**
- `signature S` = provenance object (already in Proven)
- `door D` = modality lane (Observation / Test / LLM non-deterministic lane)
- `time T` = F1 order (already in event store)
- Column records the **writing** event (who issued the mandate), not who merely reacted
- Subjectivity = non-share of foreign mandates in the window → **measure of self-direction of an autonomous body**
- Meta: the rule is itself a knot; its authors recorded in №0 column

---

## C. №0 axiom (option B) — to implement

```
№0 (axiom, named axiom):
"Nothing derived becomes quiet truth. The root is declared and revised by sessions,
 with a quorum. The content of №0 = the label of №0."
```

**DAIS mapping:** the Body Law hierarchy IS the statute. №0 = the root record (the only record
outside a session). In DAIS terms: layers 1–2 (Firmware, Capability — "CANNOT be overridden") are
the axiom's hard core; layers 3–6 (Emergency, Offline, Delegation, Context) are the revisable part,
amended by sessions with a quorum. Name the root axiom; declare its revision procedure.

---

## D. Corollary 1 — reactivity, two-axis (the animal counterexample, 2026-10-12)

Subjectivity is TWO axes, not one: `1 − foreign` measures FREEDOM (non-domination); the share of
own measures AUTHORSHIP (whether the actor writes at all); quality of control is a third, orthogonal
axis. The animal: the jungle signs nothing → freedom 1, authorship 0 → not a subject (our paradigm
non-subject). The agenda-slave: neither. The anxious planner: an author but a poor one (quality
axis). Both numbers enter rule v0.1; `authority.rs` exposes `freedom`, `authorship`,
`control_quality` (bench), and `is_subject` = free AND author.

---

## E. Corollary 2 — read-only observer (the third root)

If observation = reading without writing, and power = writing, then the read-only observer never
appears in the column → it is never a subject. **Observer and subject are non-overlapping classes.**

**DAIS mapping:** the RBAC **Observer** role already exists. Enforce: Observer reads without writing
the column. Consequence: the guarantee "nothing becomes quiet truth" rests on readers excluded from
authorship — the guarantee lives outside the field. This is not a bug; name it as architecture.
("The field is honest because its guarantees live outside the field.")

---

## F. Open (OPEN) — where the journal lives

Provenance objects are field objects (SPO triplets; Aletheia README lines 127, 153, 701; line 484:
*provenance of confidence should not remain a decorative label outside the working network*), while
the exchange phrasing said "journal outside". In DAIS this resolves cleanly: `aletheia.rs` already
treats provenance records as first-class objects → **the journal is inside** → the "two floors"
(content vs act-of-recording) collapse for the labels; what stays outside is the act-as-performance
(irreducible, by design — like the story's smile). **DECISION (2026-10-12):** journal = inside the field; act-as-performance = outside by design.
Provenance records are first-class SPO objects (`aletheia.rs`); the two floors (content vs
act-of-recording) collapse for labels; the irreducible residue (act-as-performance) is the
same residue already scoped out as the story's smile (OPEN-BY-DESIGN). No further README change
needed — the architecture already states it (lines 127/153/484/701).

---

## G. Weaknesses / meta-analysis

| # | Weakness | Mitigation |
|---|----------|-----------|
| 1 | K-ladder (KnowledgeStatus) not calibrated; "independent witnesses" subjective | Define independence criterion + inter-rater κ; map to GRADE/Murad/CEBM/TRL |
| 2 | Cross-lingual/independent control can import bias (arXiv 2305.12709) | Use different modalities/methods/actors, not two passes of one model |
| 3 | Provenance label can be forged; agent-step attribution is hard (AgentHallu: best 41.1% step localization) | Verify the label (double gate); "who verifies the verifier" regress terminates at №0 |
| 4 | Abduction / REQUIRED_ROLE without validation rule → hypothesis explosion, apophenia | Every REQUIRED_ROLE carries constraints + validation rule (CtrlHGen, arXiv 2505.20948) |
| 5 | Human-in-the-loop rubber stamp (automation bias, arXiv 2603.01202) | Forced justification, surprise checks, stratified audit of signatures |
| 6 | "Tested by construction" — the model itself can be wrong the same way | Independently validate the model (two witnesses applied to the model; Ioannidis discipline) |
| 7 | №0 as static root vs evolving field | Amendment by sessions with quorum; content = label (self-consistent) |

**Meta:** DAIS already encodes most of this (status ladder, OPEN, replication, RuleCard history,
provenance). The genuinely new deliverables are the **author column + subjectivity criterion** (§B–§E):
they turn "safe autonomy" from a safety-check property into a measurable self-direction property.
This is the highest-value addition of this harvest for DAIS.

---

## H. References (verified)

**arXiv:** 2603.05828 (HART) · 2405.02228 (REASONS) · 2601.19927 (RAG-attribution survey) ·
2601.06818 (AgentHallu) · 2608.18115 (token-level detect) · 2604.08016 (abduction survey) ·
2510.11462 (DARK) · 2607.18564 (HALO) · 2505.20948 (CtrlHGen) · 2305.14857 (BUFFET) ·
2305.12709 (cross-lingual bias) · 2112.01298 (MHC properties) · 2502.08255 (MHC framework) ·
2303.06813 (ACD/Devitt) · 2504.11486 (cognitive warfare MHC) · 2603.01202 (FSD vulnerability) ·
1409.7475 (requisite variety) · 2206.04896 (multi-scale RV).

**PubMed:** 18436948 (GRADE) · 27339128 (New Evidence Pyramid) · 16060722 (Ioannidis) · 28584653 (known unknowns).

**DOI:** 10.1080/00207727008920220 (Conant–Ashby) · 10.1007/s43681-022-00167-3 (MHC, Siebert).

**Classics:** Ashby 1957; Oxford CEBM 2009; NASA TRL (Mankins 1995); W3C PROV; Lyons 1967; Verhaar 1967/68; Denzin 1978.
