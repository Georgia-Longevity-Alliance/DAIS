# Glossary — KnowledgeStatus → Evidence ladders (2026-10-12)

**Purpose:** make the canonical `KnowledgeStatus` K-ladder (Noepedia harvest) comparable
with established evidence-grading systems: GRADE, the New Evidence Pyramid (Murad),
Oxford CEBM Levels of Evidence, and NASA TRL. One ladder, mapped — not a new religion.

References: GRADE Guyatt et al. *BMJ* 2008 (PMID 18436948, DOI 10.1136/bmj.39489.470347.AD);
New Evidence Pyramid Murad et al. *Evid Based Med* 2016 (PMID 27339128, DOI 10.1136/ebmed-2016-110401);
Oxford CEBM Levels of Evidence (2009); NASA TRL (Mankins 1995).

## 1. Mapping

| DAIS `KnowledgeStatus` | Meaning | GRADE | Murad pyramid | CEBM | NASA TRL (analogy) |
|---|---|---|---|---|---|
| `PROPOSED` | claimed, not yet supported | very low (expert opinion) | base (mechanistic reasoning) | 5 (mechanism) | TRL 1–2 (concept) |
| `SUPPORTED` | ≥1 piece of evidence | low | observational/case | 4–3 (case series / cohort) | TRL 3–4 (proof of concept) |
| `TESTED` | explicit procedure executed | moderate | single experiment (RCT-like, per-procedure) | 2 (non-randomized / controlled) | TRL 5–6 (validation in relevant env.) |
| `REPLICATED` | ≥2 independent replications | high | systematic review / meta-analysis tier | 1 (multiple RCTs) | TRL 7–8 (field validation) |
| `CONTESTED` | incompatible supported positions | — (conflict is first-class) | conflicting evidence | — | — |
| `REFUTED` | countered by evidence/test | evidence AGAINST | disproven | — | — |
| `OPEN` | insufficient evidence; addressable unknown + constraints + `validation_rule` | knowledge gap | knowledge gap | — | TRL 1 gap |
| `VALID_IN_CONTEXT` | true under stated conditions | conditional | context-bound | — | TRL 9 (operational, conditioned) |
| `SUPERSEDED` / `STALE` | replaced / inactive | — | — | — | — |

**Invariant (harvest):** a ladder is a verifiable ladder, not a float. `OPEN` is never
rendered as "absent/denied" — it carries a `validation_rule` (see `OpenSpec`).

## 2. Independence criterion for Replication → REPLICATED

`REPLICATED` requires **two independent witnesses** (harvest rule). Independence is met
when the two replications differ in **at least one modality/actor/source**:

- different agent (lab / device / human rater) **or**
- different method or instrumentation **or**
- different source (e.g., PMID + independent dataset + mechanism).

Not independent (rejected as a single witness):
- two replications from the same pool/run;
- two citations of the same underlying dataset;
- a cross-lingual pass of the same model (can import its own bias — arXiv 2305.12709).

The validator warns on transitions to REPLICATED with fewer than 2 sources (`validator.rs`,
two-witness rule); enforcing independence is the caller's check at publish time.

## 3. κ (inter-rater agreement) for Replication

When replications are human-annotated, record agreement:

- Cohen's κ ≥ 0.6 → acceptable for `SUPPORTED → TESTED`; κ ≥ 0.8 → strong (e.g., the
  REASONS benchmark, arXiv 2405.02228, reported κ = 0.78 at human-consensus level).
- Below κ 0.6: re-label, do not promote to REPLICATED.
- For multi-raters use Fleiss' κ.

κ is a property of the *replication record*, stored alongside `Replication.conditions`.

## 4. Where this lives in code
- Enum: `core/src/noepedia.rs` → `KnowledgeStatus` (+ `TryFrom<&str>`).
- OPEN spec: `OpenSpec` (address, constraints, validation_rule).
- Formed unknown: `RoleRequirement` + `RelationKind::RequiresRole`.
- Transition enforcement: `core/src/validator.rs` (`is_valid_transition`, two-witness warning).
