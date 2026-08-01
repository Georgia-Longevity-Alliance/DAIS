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

### 4. Noepedia = Structured Epistemology

Noepedia is the knowledge counterpart. It extends the trace network from "what happened" to "what is true." Every claim has:
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

AIS counts joules. Every operation is classified:
- **Green:** Deterministic, <1 µJ (SQL, Rust, graph traversal)
- **Yellow:** Small model inference, <1 mJ (local classifier, anomaly detector)
- **Red:** LLM inference, ~1 J/token (diagnosis, interview, synthesis)

The system minimizes Red operations by caching traces (Green lookups replace Red diagnoses) and by using Yellow pre-filters (only invoke LLM when anomaly is genuinely novel).
