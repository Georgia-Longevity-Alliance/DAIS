# CONCEPT — DAIS (Autonomous Intelligence Socket)

**Version:** 1.0  
**Date:** 2026-07-31  
**Parent:** ~/Desktop/Marketing/

---

## 0. Executive Summary

DAIS is the reference implementation merging **DAISocket** (embodied AI safety protocol) and **Proven** (hallucination-resistant knowledge system) into a single, production-grade platform.

**Three pillars:**
1. **DAISocket Core** (Rust) — Passport, Body Law, Flight Recorder, Trace Network
2. **Proven Core** (Rust) — Delta Protocol, Event Store, Validator, Consolidator
3. **DAIS Web** (Phoenix/Elixir) — Dashboard, Device Registry, Knowledge Browser, LiveView

**Mission:** Give every autonomous body a passport, every intervention a trace, every knowledge claim a provenance — and make it all open-source, energy-honest, and safe.

---

## 1. Why DAIS Exists

### 1.1 The Two Missing Pieces

Current AI landscape has two structural gaps:

| Gap | Problem | DAIS Solution |
|-----|---------|--------------|
| **Embodied safety** | LLMs can't safely control physical devices — safety rules in prompts are unreliable | Body Law in firmware, `forbidden_always` enforced at ALU cost |
| **Knowledge hallucination** | LLM answers lack provenance — `PROPOSED` masked as `FACT`, conflicts disappear in fluent text | Structured knowledge field: claims → sources → evidence → status |

DAISocket + Proven were born as separate projects. DAIS unifies them because they share the same architectural DNA: **addressable, verifiable, append-only structures with deterministic safety boundaries.**

### 1.2 The Energy Argument

```
LLM token: ~1 joule
Microcontroller addition: ~1 picojoule
Ratio: 10¹² : 1
```

Using an LLM as a continuous controller is like running to the equator to take a single step. DAIS enforces:
- **Deterministic execution** (firmware, SQL, graph traversal) for routine operations
- **LLM intervention** only for novelty, conflict, and open questions
- **Trace preservation** so solved problems are never paid for twice

---

## 2. Architecture

### 2.1 Layer Model

```
┌─────────────────────────────────────────────┐
│              WEB LAYER (Phoenix)             │
│  Dashboard │ Registry │ Knowledge Browser    │
├─────────────────────────────────────────────┤
│            PYTHON LAYER (py_backend)         │
│  LLM Bridge │ HTTP Client │ ML Integration   │
├─────────────────────────────────────────────┤
│              RUST CORE (core/)               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ DAISocket  │  │ Proven │  │  Shared  │   │
│  │ Passport  │  │  Delta   │  │  Event   │   │
│  │ Body Law  │  │Validator │  │  Store   │   │
│  │ FlightRec │  │Consolid. │  │  Types   │   │
│  │ TraceNet  │  │ Renderer │  │  Crypto  │   │
│  └──────────┘  └──────────┘  └──────────┘   │
├─────────────────────────────────────────────┤
│           DEVICE LAYER (firmware)            │
│  ESP32 │ Arduino │ Raspberry Pi │ Jetson     │
└─────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
DEVICE                    DAIS CORE                   USER/LLM
  │                          │                          │
  ├─ heartbeat ─────────────→│                          │
  │                          ├─ registry update         │
  │                          │                          │
  ├─ event ─────────────────→│                          │
  │                          ├─ flight_recorder.write() │
  │                          │                          │
  │                    ┌─────┤                          │
  │                    │BLOCKED?                        │
  │                    └─────┤                          │
  │                          ├─ LLM invoked ←───────────┤
  │                          ├─ reads passport          │
  │                          ├─ reads flight recorder   │
  │                          ├─ diagnoses               │
  │                          ├─ acts (within mandate)   │
  │                          ├─ writes trace            │
  │                          │                          │
  │                          ├─ knowledge → Proven ───→│
  │                          │                          │
```

---

## 3. Component Specifications

### 3.1 DAISocket Core (Rust)

#### Passport (`core/src/passport.rs`)
```rust
struct Passport {
    device_id: Uuid,
    name: String,
    description: String,
    capabilities: Vec<Capability>,
    forbidden_always: Vec<ForbiddenAction>,  // Enforced in firmware
    risk_class: RiskClass,
    autonomous_mandate: Option<AutonomousMandate>,
    emergency_contacts: Vec<EmergencyContact>,
    platform: Platform,
    version: SemVer,
    signature: Option<Ed25519Signature>,     // Roadmap: cryptographic
}
```

#### Body Law (`core/src/body_law.rs`)
6-layer validation pipeline:
1. **Firmware rules** — hardware-enforced constraints (torque, temp, laser power)
2. **Capability check** — is this action in the passport?
3. **Emergency override** — rescue agents with limited access
4. **Offline mandate** — what's allowed when connection drops
5. **Delegation chain** — who authorized whom
6. **Context validation** — time, environment, state preconditions

#### Flight Recorder (`core/src/flight_recorder.rs`)
```rust
struct FlightRecorder {
    buffer: RingBuffer<FlightEvent>,
    capacity: usize,  // ESP32: 256, Android: 65536
    event_types: HashSet<EventType>,
}
```

#### Trace Network (`core/src/trace.rs`)
```rust
struct InterventionTrace {
    trace_id: Uuid,
    device_id: Uuid,
    agent_id: Uuid,
    diagnosis: String,
    actions_taken: Vec<Action>,
    outcome: Outcome,
    timestamp: DateTime<Utc>,
    references: Vec<Uuid>,  // Links to prior related traces
}
```

### 3.2 Proven Core (Rust)

#### Delta Protocol (`core/src/delta.rs`)
Legal operations: CREATE, UPDATE, RELATE, DEPRECATE, CONSOLIDATE.
Every delta is a proposed change — the validator decides acceptance.

#### Event Store (`core/src/event_store.rs`)
Append-only log. SQLite for embedded, PostgreSQL for server.
`(event_id, timestamp, actor, operation, payload, signature)`

#### Validator (`core/src/validator.rs`)
Checks: schema conformance, referential integrity, status transitions, permission boundaries, consistency rules.

### 3.3 Python Backend (`py_backend/`)

- **LLM Bridge:** Standard prompt + tool interface for any LLM (OpenAI, Anthropic, Gemini, local Ollama)
- **HTTP Client:** Connect to googuly.online registry + Proven API
- **Scientific:** NumPy/SciPy for flight recorder analysis, scikit-learn for anomaly detection

### 3.4 Phoenix Web (`web/`)

- **Dashboard:** Device status, recent traces, active sessions
- **Registry:** Device search, passport viewer, permission management
- **Knowledge Browser:** Proven publication explorer, claim graph visualisation
- **LiveView:** Real-time flight recorder streaming

---

## 4. First MVP — ARGUS-OS1 Integration

The first real-world deployment target is **ARGUS-OS1** (automated centriole tracking microscope).

| DAIS Component | ARGUS-OS1 Instantiation |
|---------------|------------------------|
| Passport | Microscope identity: 488/561/640nm lasers, Sangaboard stage, microfluidic |
| Body Law | Laser safety, temp ≤37°C, phototoxicity ceiling (div rate >90%) |
| Flight Recorder | Centriole tracking log: coordinates, intensities, division events |
| LLM Bridge | Anomaly detection → Gemini Flash diagnosis → safe restart or pause |
| Trace Network | Solved anomalies recorded → all ARGUS devices learn |
| Knowledge Field | Published findings → Proven claims with provenance |

### V9 — Robot Hands + Shared Local LLM Brain (2026-08-17)

V9 is an autonomy layer over every OS stage (OS1/OS2/OS3): robot hands operate through the glove ports instead of human hands (24/7 servicing), and an external LLM brain runs on the same local host that controls the micromanipulators and micro-robots inside the enclosure.

| DAIS Component | V9 Instantiation |
|---------------|------------------|
| Passport | +15 capabilities: pick-and-place, pipette, wipe, UV, capillary, rake, charge, calibrate, transfer-in, transfer-out, door-interlock |
| Body Law | force ≤5 N, speed ≤200 mm/s, no-touch zones around the objective during fs-laser |
| Flight Recorder | every hand action: pose (x,y,z,theta), force, timestamp, camera frame |
| LLM Bridge | arm error diagnosis → safe restart; escalation to human at confidence <0.7 |
| Trace Network | servicing procedures (objective cleaning, capillary replacement) in the shared registry |
| Knowledge Field | servicing procedures → Proven claims with verification |

Design: [ARGUS-OS1/docs/V9_PROTOTYPE.md](https://github.com/Georgia-Longevity-Alliance/ARGUS-OS1/blob/main/docs/V9_PROTOTYPE.md) | [ARGUS-OS1/docs/STERILIZATION_TRANSFER.md](https://github.com/Georgia-Longevity-Alliance/ARGUS-OS1/blob/main/docs/STERILIZATION_TRANSFER.md)

---

## 5. Success Criteria

1. ✅ Passport generated for ARGUS-OS1 V6
2. ✅ Flight Recorder captures 100-embryo run
3. ✅ Body Law prevents laser over-power in firmware
4. ✅ LLM successfully diagnoses 3 simulated anomalies
5. ✅ Trace recorded and retrievable by second device
6. ✅ Proven publication created with ≥5 claims, each with source + evidence
7. ✅ Phoenix dashboard shows live device status

---

## 6. Infrastructure

### AIS Global Server

| Parameter | Value |
|-----------|-------|
| **Type** | Dedicated server |
| **Specs** | 8+ vCPU, 16+ GB RAM, SSD |
| **Cost** | ~$150/мес |
| **Period** | 10 years |
| **Total** | **$18,000** |

**Components hosted:**
- AIS Dashboard + Device Registry (Phoenix)
- Noepedia Knowledge Browser + API
- Event Store (PostgreSQL)
- Trace Network (global trace replication)
- LLM Bridge endpoint

**Shared across all AIS-based robots:** ARGUS-OS1, OS2, OS3, and future longevity robots.

---

## 7. References

- DAISocket: https://github.com/gakelytemp-creator/DAISocket
- Proven: https://github.com/gakelytemp-creator/Proven
- ARGUS-OS1: ~/Desktop/Marketing/ARGUS-OS1/
- MCP Spec: https://modelcontextprotocol.io/
