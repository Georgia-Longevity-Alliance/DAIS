# ARGUS Centriole Knowledge Field — MVP (Noepedia)

**Date:** 2026-09-15
**Status:** Seed (loadable) + reference
**Files:** `py_backend/daisocket/seed_centriole_field.json` · `core/src/noepedia.rs` · `core/src/zpd.rs`
**Sources:** ARGUS-OS1 `CONCEPT.md` (pilots P0–P8, H1), Noepedia grammar.

## 1. What this is

The first domain MVP of the Noepedia field, aligned with ARGUS. Pilots P0–P8 are represented as CLAIM nodes with an evidence layer, a Go/No-Go RULE_CARD per protocol, and COVERAGE (relevant vs evaluated networks). This turns scattered experiment logs into **inspectable reusable knowledge**: the upper layer (claim) for fast use, the lower layer (evidence/context) for inspection.

## 2. Pilots as CLAIM nodes (from `seed_centriole_field.json`)

| Claim | Subject | Status | Valid-in | Evidence |
|-------|---------|:---:|----------|----------|
| `claim-p2-phototoxicity` | P2: division rate >90% of dark control | PROPOSED | embryo 25C, 488/561/405nm | Go/No-Go threshold |
| `claim-p6-sas1-latency` | P6: SAS-1→SAS-4 latency 3 criteria | OPEN | somatic; oogenesis pending | 3 criteria required else descriptive |
| `claim-h1-pedigree-fate` | H1: Pedigree Score predicts fate | **OPEN** | 0→100 cells, 25C | Erpf 2020 (supporting, small n) + stochasticity (inconclusive) |
| `claim-p5-sister-pairs` | P5: ≥40 pairs → sensitivity | SUPPORTED | Sulston 1983 | ~500 pairs, power >80% OR≥1.5 |

**RULE_CARD examples:**
- `protocol_validity`: *"all stated thresholds must be satisfied; any failure → descriptive only"* — e.g. P6 is valid only if all 3 latency criteria met.
- `lineage_centriole_fate`: *"sister-pair design (ICC eliminated); joint model BF>10; fixed N=100; no intermediate stopping"* — this is the preregistered rule.

## 3. Coverage (≠ confidence)

Coverage records how many relevant networks have been *evaluated* vs merely *identified*:

| Conclusion | relevant | evaluated | not yet | supporting | conflicting | status |
|-----------|:---:|:---:|:---:|:---:|:---:|:---:|
| H1: Pedigree Score predicts fate | 6 | 2 | 4 | 1 | 0 | OPEN |
| P6: SAS-1 valid marker | 3 | 0 | 3 | 0 | 0 | OPEN |

> "The evaluated networks agree" is **not** the same as "the relevant networks have been sufficiently surveyed." OPEN is legal until coverage closes.

## 4. COVERAGE table for the review «Asymmetric Divisions and Adult Stem Cells»

Direct application of the Noepedia coverage discipline to the manuscript meta-analysis:

| Evaluative network | relevant systems | evaluated | supporting (direct) | conflicting | not yet | status |
|--------------------|:---:|:---:|:---:|:---:|:---:|:---:|
| Non-random chromosome segregation (Cairns) | 12 | 9 | 3 (minority) | 6 | 3 | **CONTESTED / OPEN** |
| Asymmetric epigenetic partitioning (histones, bookmarking) | 10 | 8 | 7 | 1 | 2 | SUPPORTED → REPLICATED |
| Centriolar theory of differentiation (incl. own 2005) | 7 | 4 | 3 | 2 | 3 | CONTESTED |
| Age-selective organelle partitioning | 9 | 6 | 5 | 1 | 3 | SUPPORTED |
| Division-mode plasticity | 8 | 5 | 4 | 1 | 3 | SUPPORTED |

**Reading for the manuscript:** the claim "direct support for non-random chromosome segregation is confined to a minority" is backed by a coverage count (3/9 supporting, 6/9 conflicting), and the "asymmetric epigenetic partitioning is more widespread" claim by 7/8 supporting. Both are reported as *coverage*, not as a single confidence number — and the Cairns question stays OPEN/CONTESTED rather than silently closed.

## 5. Next steps

- [ ] Load `seed_centriole_field.json` into the Phoenix knowledge browser (networks, claims, evidence).
- [ ] Wire `consolidation.py` to the ARGUS flight recorder (P1): resolved repeated anomalies → REPLICATED claims.
- [ ] Fold the COVERAGE table (§4) into the «Asymmetric Divisions» manuscript methods (COVERAGE + statuses).
- [ ] After pilot data: promote P-claims per their Go/No-Go RULE_CARDs.
