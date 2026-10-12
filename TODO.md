# TODO — DAIS

**Updated:** 2026-08-02

## Phase 0: Foundation ✅ COMPLETE
- [x] Create core Rust project
- [x] Passport + serde
- [x] Body Law — 6 canonical layers (Firmware, Capability, Emergency, Offline, Delegation, Context) + 2 boundary gates (Hardware Safety, RBAC)
- [x] Flight Recorder — ring buffer
- [x] Trace Network — intervention traces + publish/remix/community
- [x] Proven: Delta Protocol, Event Store, Validator, Consolidator, Renderer
- [x] Python backend — LLM Bridge, HTTP Client, PassportInterview
- [x] Phoenix web — Dashboard, Passport Interview, Registry, OAuth
- [x] ARGUS-OS1 Passport example (650+ lines JSON)
- [x] All core files (11/11)
- [x] 102 tests passing, 0 warnings (2026-10-12: unified status ladder, authority module)

## Phase 0.5: LEGO Proposal Integration ✅ COMPLETE (2026-08-02)
- [x] Multi-Body / Device Swarm (Passport::change_body, body_history)
- [x] Hardware Safety (SafetyHardware: physical STOP, deadman switch, key-lock)
- [x] RBAC — 5 roles (Owner, Operator, Observer, Parent, EmergencyRescue)
- [x] Challenge Framework — 5 standard benchmarks, compliance scoring
- [x] Prizes/Unlockables — requirements → rewards system
- [x] Community Trace Library — publish, tags, discover
- [x] Improvement Loops — challenge→construct→test→result→new_parts cycle
- [x] Public Remixes — remix_of, find_remixes_of, remix_count
- [x] Pilot Success Criterion — multi-body evaluation
- [x] Save LEGO analysis to docs/

## Phase 1: ARGUS-OS1 Integration
- [ ] Deploy Flight Recorder on Jetson Orin NX (⏳ needs hardware)
- [x] Connect local registry to ARGUS-OS1 ✅ (ArgusRegistry + cross-session queries)
- [x] LLM anomaly diagnosis test (3 simulated failures) ✅
- [x] Trace Network: record + retrieve across sessions ✅
- [x] Proven: publish ARGUS-OS1 findings as knowledge field ✅

## Phase 2: Production Hardening
- [ ] Google OAuth credentials (GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET)
- [ ] Deploy to jaba@server
- [ ] Cryptographic passport signatures (Ed25519)
- [ ] Federation between registry servers
- [ ] PostgreSQL event store for server deployment
- [ ] WebRTC direct device connection
- [ ] ROS2 bridge
- [ ] Matter/WoT compatibility layer

## Phase 3: Community
- [ ] Open source release (Apache 2.0 + AGPL for Proven)
- [ ] Documentation site
- [ ] Contributor guide
- [ ] Demo video

## Phase 4: Noepedia Harvest — Author Column & №0 (2026-10-12)
> Мастер: `docs/NOEPEDIA_HARVEST_2026-10-12.md` · companion: `~/Desktop/Services/docs/NOEPEDIA_COLUMN_RULE_AND_AXIOM_2026-10-12.md`
- [x] Implement author column in `core/src/authority.rs` (`AuthorColumn`): record R carries signature S · door D · time T; column records the WRITING event, not the waking event
- [x] Implement subjectivity criterion v0 in `authority.rs`: `1 − (foreign-authored wakings / all wakings over window W)`; threshold > ½ by count; weights after bench
- [x] Add `OpenSpec` (address + constraints + `validation_rule`) to `noepedia.rs` — OPEN = addressable unknown + validation rule; never rendered as "absent/denied"
- [x] Enforce RBAC Observer as read-only (already in `body_law.rs` layer 7, verified) + `authority.rs::is_observer`; document "guarantees live outside the field"
- [x] Record №0 axiom as `StatuteNo0` (content = label; hard core = Body Law layers 1–2; layers 3–6 revisable by sessions with quorum)
- [x] Add REQUIRED_ROLE as first-class relation kind (`RelationKind::RequiresRole` + `RoleRequirement` with constraints + validation rule) in `noepedia.rs`
- [x] Map `KnowledgeStatus` explicitly to GRADE/Murad/CEBM/TRL in `docs/GLOSSARY_KNOWLEDGE_STATUS.md`; define independence criterion + κ for Replication→Replicated
- [x] Decision recorded: journal = inside the field (provenance objects are SPO triplets); act-as-performance = outside by design — `docs/NOEPEDIA_HARVEST_2026-10-12.md` §F
