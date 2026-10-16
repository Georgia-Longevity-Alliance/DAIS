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

## 5. History vs Epistemic status (Demon Core Harvest — 2026-10-16)

A record carries two independent status dimensions (DCT11–DCT12):

| Dimension | Meaning | Mutability |
|-----------|---------|------------|
| `HISTORY_STATUS` | the committed fact — was this transition recorded? | stable; may be COMPACTED/ARCHIVED/SUPERSEDED but never made unaddressable |
| `CURRENT_EPISTEMIC_STATUS` | current trust — is the claim supported/refuted/contested/open now? | revisable; time-, scope-, audit-dependent |

**Non-destructive MARK:** a status transition creates a *new* addressable status assertion
(scope/time-stamped) and keeps prior assertions as historical trace. Never an overwrite.
Target code change: `HistoryStatus`/`CurrentEpistemicStatus` fields + `epoch` on status assertions;
validator emits a new assertion instead of mutating state in place.

## 6. Justification composition (Demon Core Harvest — 2026-10-16)

`REPLICATED` (and any multi-witness state) requires an explicit composition policy (DCT19–DCT20):

- **ALTERNATIVE SUPPORT** — any one complete supported justification may suffice (e.g., independent
  replications; one surviving path keeps the result supported).
- **JOINT SUPPORT** — all required components must remain supported (e.g., several constraints needed
  together); a challenge to one component challenges the result.

> Multiple justifications ≠ redundancy. `same result ≠ same provenance`; convergent paths accumulate
> provenance, not truth-value.

The composition policy lives in the network (addressable), not in the demon core. If missing →
`NEED / MISSING_RULE`.

## 7. Applicability qualifiers (Demon Core Harvest — 2026-10-16)

`VALID_IN_CONTEXT` is not a single scalar field; applicability is an addressable assertion with
qualifiers (DCT13–DCT16). Confirmed dimensions: SCOPE, TIME. Disjoint scope/time can host
incompatible states without conflict.

> Conflict requires simultaneous applicability plus incompatibility. Applicability conflict is
> resolved **before** semantic/status conflict. `NOT_APPLICABLE ≠ FALSE ≠ CHALLENGED`.

Generic qualifier framework: `ASSERTION → QUALIFIER (ROLE=QUALIFIER)`, `QUALIFIER → MATCH_POLICY
(ROLE=MATCH_POLICY)`; if a qualifier has no matching policy → NEED (`MISSING_RULE`).
