# MEMORY — DAIS

**Created:** 2026-07-31

## 2026-08-01 — AIS Global Server: Dedicated server 10yr ($18,000)

**Decision:** Dedicated server (~$150/мес, 10 лет) для глобальной платформы AIS.

**Rationale:** AIS обслуживает всех longevity-роботов (ARGUS-OS1/OS2/OS3 и будущие). Нужен выделенный сервер для Dashboard, Registry, Event Store, Noepedia API, Trace Network.

**Budget:** $18,000. Записано в: AIS/CONCEPT.md, AIS/PARAMETERS.md, ARGUS-OS1/CONCEPT.md, GLA/CONCEPT.md.

## 2026-07-31 — Project Creation

**Decision:** Created DAIS as umbrella project under Marketing/ — merging DAISocket + Proven into single implementation.

**Rationale:** 
- Gakely's DAISocket and Proven are conceptually strong but lack code
- ARGUS-OS1 is the perfect first integration target
- Rust for core protocol (safety + performance), Python for AI/ML, Phoenix for web

**Key insights from code review:**
- DAISocket: Python "Ready" but no Python code in repo — need to build it
- Proven: pure concept paper, no implementation — need MVP with 3-4 object types
- Both share architectural DNA: addressable, append-only, deterministic safety boundaries

**Next:**
1. Initialize Rust core with passport + body law
2. Autofix to 100
3. Python LLM bridge
4. Phoenix dashboard
5. ARGUS-OS1 integration demo
