# MAP — DAIS

```
DAIS/                           # Root: core files only
├── _pi.md                     # Rules for pi
├── CONCEPT.md                 # Concept + architecture
├── TODO.md                    # Task list
├── PARAMETERS.md              # Protocol + safety parameters
├── MAP.md                     # This file
├── STATE.md                   # Current status
├── MEMORY.md                  # Decision history
├── README.md                  # Public-facing overview
│
├── core/                      # Rust — protocol implementation
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/
│       ├── lib.rs             # Public API + prelude
│       ├── types.rs           # Shared types + serde
│       ├── passport.rs        # Passport struct
│       ├── core/src/body_law.rs        # 6-layer validator
│       ├── flight_recorder.rs # Ring buffer
│       ├── trace.rs           # Intervention traces
│       ├── delta.rs           # Proven delta protocol
│       ├── event_store.rs     # Append-only log
│       ├── validator.rs       # Delta validator
│       ├── consolidator.rs    # Knowledge consolidation
│       └── renderer.rs        # Article renderer
│
├── py_backend/                # Python — AI/ML + HTTP
│   ├── requirements.txt
│   └── aisocket/
│       ├── core/src/main.rs
│       ├── core/src/main.rs          # Registry HTTP client
│       ├── llm_bridge.py      # LLM ↔ DAISocket
│       └── noepedia_api.py    # Proven client
│
├── web/                        # Elixir/Phoenix — Dashboard
│   ├── mix.exs
│   ├── mix.lock
│   ├── config/
│   ├── lib/
│   │   ├── web.ex
│   │   ├── web_web.ex
│   │   └── web_web/
│   │       ├── router.ex
│   │       ├── endpoint.ex
│   │       ├── live/
│   │       │   ├── dashboard_live.ex
│   │       │   └── device_live.ex
│   │       └── controllers/
│   │           └── mix.exs
│   └── test/
│
├── integration/                # End-to-end tests
├── docs/                       # Documentation
├── scripts/                    # Utility scripts
└── _archive/                   # Reference: DAISocket_ref, Proven_ref
```
