# TODO — DAIS

**Updated:** 2026-08-01

## Phase 0: Foundation ✅ COMPLETE
- [x] Create core Rust project (`core/Cargo.toml` + `src/lib.rs`)
- [x] Implement Passport struct + serialization (serde)
- [x] Implement Body Law — 6-layer validation pipeline
- [x] Implement Flight Recorder — ring buffer
- [x] Implement Trace Network — intervention trace struct
- [x] Implement Delta Protocol — legal operations
- [x] Implement Event Store — append-only log (SQLite)
- [x] Implement Validator — schema + status transitions
- [x] Implement Consolidator + Renderer
- [x] Create Python backend (`py_backend/`)
- [x] Implement LLM Bridge — standard prompt + tool interface
- [x] Implement HTTP Client — googuly.online + local registry
- [x] Implement PassportInterview — Socratic interview agent
- [x] Create Phoenix web project (`web/`)
- [x] Implement Dashboard LiveView
- [x] Implement Passport Interview LiveView (chat interface)
- [x] Implement local passport registry (SQLite API)
- [x] Google OAuth integration
- [x] Integration: ARGUS-OS1 Passport example (650+ lines JSON)
- [x] All core files (11/11)
- [x] 33 tests passing, 0 warnings

## Phase 1: ARGUS-OS1 Integration
- [ ] Deploy Flight Recorder on Jetson Orin NX
- [ ] Connect local registry to ARGUS-OS1
- [ ] LLM anomaly diagnosis test (3 simulated failures)
- [ ] Trace Network: record + retrieve across sessions
- [ ] Proven: publish ARGUS-OS1 findings as knowledge field

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
