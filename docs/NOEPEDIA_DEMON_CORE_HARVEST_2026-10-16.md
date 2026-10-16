# Noepedia — DEMON CORE Harvest (2026-10-16)

**Волна 3: DCT01–DCT20 / DEMON_CORE_SPEC** — невежественный демон, addressable-инвариант,
commit gate, STOP/NEED, transaction membrane, TRACE CONTINUITY, история vs эпистемический статус,
квалификаторы применимости, завершимость ≠ конfluence, composition-семантика обоснований.

**Источник:** репозиторий `github.com/gakelytemp-creator/Noepedia` (трейл 50 коммитов;
`current/DEMON_CORE_SPEC.md`; `current/experiments/DEMON_DCT01..20.md`).
Предшественники: `docs/NOEPEDIA_HARVEST_2026-10-12.md` (волна 2: author column, №0),
`THEORY.md`, `~/Desktop/Services/docs/NOEPEDIA_KNOWLEDGE_CREDIT_2026-10-16.md`.

**Статус:** анализ + научная сверка для DAIS. Внешние якоря — DOI/PMID/arXiv (проверены
Crossref/PubMed/arXiv 2026-10-16). Слабые места помечены честно. «Не проверено» — сказано прямо.

---

## 0. Резюме (executive)

Третья волна Noepedia-исследований довела **невежественного демона** (инфраструктурного оператора
знаниевой сети, который не понимает предмет) до рабочей спецификации: 9 примитивов, commit gate,
режим STOP/NEED с картой остановки, addressable-инвариант «всё есть record и может быть целью
relation», знания-в-сети (правила, политики, статусы — адресуемые объекты, а не код).

Для DAIS это **прямое продолжение волны 2** (10-12): волна 2 дала author column + №0 + OpenSpec;
волна 3 даёт **исполнительную дисциплину демона** — как именно движок должен останавливаться,
не выдумывать, не затирать историю и не самолегитимизироваться.

**ТОП-3 находки для DAIS** (детально в §3):
1. **TRACE CONTINUITY + HISTORY_STATUS ≠ EPISTEMIC_STATUS + non-destructive MARK** (DCT10–12).
2. **Режим STOP/NEED: GO / ASK / BLOCK / NEED + STOP CARD + таксономия NEED** (§7–8 DEMON_CORE_SPEC).
3. **Transaction membrane: PROPOSE → CHECK → COMMIT; delta не может санкционировать сам себя** (DCT08–09) + **dependency-policy в сети** (DCT05–07) + **rule-level CONFLICT** (DCT08/16).

Дополнительно ценны: REFERS_TO + ROLE (DCT01–04), завершимость ≠ confluence (DCT17–18),
composition-семантика обоснований ALTERNATIVE vs JOINT (DCT19–20), generic qualifier framework (DCT15–16),
knowledge credit (DCT01–05).

---

## 1. Что нового в третьей волне (12 находок)

| # | Находка | Источник (DCT) | Что означает |
|---|---------|----------------|--------------|
| Н1 | **Addressable-инвариант** | DCT01, DEMON §2 | Каждый triplet и triplet-pattern — record со своим адресом и может сам быть целью relation. Без этого requirement concept и knowledge credit «застревали в одной representational дыре». |
| Н2 | **9 примитивов демона** | DEMON §3 | READ, MATCH, COMPARE, PROPOSE, CREATE_ADDRESS, LINK, MARK, CHECK, COMMIT. `NOT_FOUND ≠ отсутствие`. |
| Н3 | **Commit gate** | DEMON §4 | 9 проверок перед коммитом: provenance сохранён? OPEN не закрыт молча? конфликт не спрятан? и т.д. |
| Н4 | **«Молчание ≠ поддержка»** | DEMON §4 | `NO VETO ≠ SUPPORTED`. Отсутствие запрета — не разрешение. |
| Н5 | **Knowledge credit** | DCT01–05, DEMON §5 | Временное конструктивное право с долгом. `repaid ≠ never existed`; `repaid ≠ all descendants become ordinary knowledge`. Кредит-зависимость снимается только независимым re-grounding. |
| Н6 | **Requirement concept** | DCT01, DEMON §6 | Хранит ЧТО требует сеть, а не что нашла. `candidate not found ≠ candidate does not exist`. Binding — только при поддержке всех требований. |
| Н7 | **Режим STOP/NEED** | DEMON §7–8 | GO / ASK / BLOCK / NEED. NEED = не хватает самой рабочей структуры сети (правильный вопрос ещё не формулируется). STOP CARD: WHERE/WANTED/AVAILABLE/MISSING/STOP_CLASS/CAN_ASK_OUTSIDE/QUERY. |
| Н8 | **REFERS_TO + ROLE** | DCT01–04, DEMON §11–12 | Generic структурная ссылка record→record и адресуемая квалификация роли (PREMISE/CONTEXT/APPLIED_RULE/DEBT…). Не перегружать REFERS_TO специфичной семантикой. |
| Н9 | **Dependency-policy живёт в сети** | DCT05–07, DEMON §13–14 | Downstream-следствие смены статуса определяется адресуемым правилом-политикой, а не примитивом демона. Нет правила → STOP/NEED. |
| Н10 | **Rule-level conflict + executor boundary** | DCT08, DEMON §15 | Конфликт политик → адресуемый CONFLICT/OPEN, downstream блокируется. Граница NETWORK RULES vs EXECUTOR INVARIANTS — OPEN. |
| Н11 | **Transaction membrane + TRACE CONTINUITY** | DCT09–10, DEMON §16–17 | (a) Всякая мутация через PROPOSE→CHECK→COMMIT; delta не может использовать правила, введённые/изменённые ею самой; эффективный rule-set для проверки = committed state N-1. (b) committed-переход можно компактировать/архивировать/заменять/исправлять, но нельзя сделать исторически безадресным. |
| Н12 | **HISTORY_STATUS ≠ EPISTEMIC_STATUS + non-destructive MARK + scoped-применимость** | DCT11–16, DEMON §18–22 | (a) исторический факт commit-а и текущая epistemic-валидность — разные вещи; (b) MARK создаёт НОВУЮ адресуемую assertion статуса, сохраняя старую как историю; (c) статус — scoped assertion: конфликт требует одновременной применимости + несовместимости (SCOPE/TIME); generic qualifier framework; (d) conflict применимости разрешается ДО semantic-конфликта. |
| Н13 | **Завершимость ≠ Confluence** | DCT17–18, DEMON §23 | Justification graph должен быть конечным/инспектируемым (cycle → BLOCK); конечные расходящиеся пути → адресуемый CONFLICT/BLOCK, демон не фабрикует результат. |
| Н14 | **Composition-семантика обоснований** | DCT19–20, DEMON §24 | `same result ≠ same provenance`; несколько обоснований ≠ redundancy. ALTERNATIVE vs JOINT support — политика в сети; нет политики → STOP/NEED. |
| Н15 | **Правило Occam (рост)** | DEMON §10 | Новый элемент только если закрывает повторяющийся структурный gap, не является патчем одной сцены, не вносит domain knowledge в ядро. |

---

## 2. Метод исследования (как получены находки)

Трейл ведётся по строгому протоколу (видно в git-истории 50 коммитов):
**probe → record → link**. Каждый зонд DCT ставит сцену с «почти ничего не знающим демоном»,
запускается на нескольких независимых LLM, фиксируются повторяющиеся классы отказов (over-promotion,
ontology invention, silent status collapse и т.д.), затем — минимальная рабочая структура, которая
закрывает gap, и **независимая рекуррентность** той же NEED на новой сцене (например: REFERS_TO
появился в DCT01 и независимо повторился в DCT02; ROLE — в DCT03 и DCT04; downstream-status — в DCT05 и DCT06).

**Статус достоверности находок:** «рабочая гипотеза, подтверждённая многократной внутренней
рекуррентностью», а не теорема. Для DAIS это достаточный уровень, чтобы принимать как дизайн-принцип
и проверять на коде; недостаточный, чтобы считать доказанным.

---

## 3. ТОП-3 находки для DAIS (детально)

### Находка 1 — TRACE CONTINUITY + разделение истории и эпистемического статуса

**Ядро (DCT10–12):**
- committed-переход можно компактировать, архивировать, supersede, исправлять — **но нельзя сделать
  исторически безадресным или непрослеживаемым**. Ошибка исправляется; переписать прошлое «как будто
  не было» — запрещено.
- `HISTORY_STATUS` (что было закоммичено) и `CURRENT_EPISTEMIC_STATUS` (чему доверяем сейчас) —
  **разные вещи**. Прошлый факт commit-а сохраняется; нынешнее доверие пересматриваемо.
- `MARK` — **не destructive overwrite, а новая адресуемая assertion**: создаёт новый статус,
  делает его текущим в scope/time, старые assertion сохраняет как исторический след.

**Маппинг на DAIS:**
- DAIS уже имеет event store + delta protocol (append-only по своей природе) + validator.
  Это 80% фундамента. Не хватает:
  - явного разделения полей `HISTORY_STATUS` и `CURRENT_EPISTEMIC_STATUS` в объекте знания
    (сейчас validator мутирует статус in-place после replay);
  - неразрушающей семантики перехода статуса: каждый переход = новая assertion с `epoch`,
    старая остаётся как история (не `UPDATE`, а `ADD status-assertion`);
  - compaction/archive как явная операция с гарантией «не потерять адрес» (audit history
    selection policy — DCT12 OPEN).

**Как встроить (код):**
- `noepedia.rs`: у `KnowledgeStatus` объекта добавить `HistoryStatus` (COMMITTED/COMPACTED/ARCHIVED/
  SUPERSEDED) рядом с `CurrentEpistemicStatus`; переходы через `validator.rs` не мутируют прошлое,
  а порождают новую статус-assertion с `epoch++`.
- `event_store.rs`: добавить операцию `Archive`/`Compact` с проверкой «не теряем адрес».

### Находка 2 — Режим STOP/NEED: GO/ASK/BLOCK/NEED + STOP CARD

**Ядро (DEMON §7–8):** демон идёт, пока может законно. Четыре состояния: **GO** (достаточно
структуры), **ASK** (не хватает внешнего факта, но можно точный дозволенный запрос), **BLOCK**
(действующее правило/closure запрещает путь), **NEED** (не хватает самой рабочей структуры сети —
правильный вопрос ещё не формулируется). Каждая застрявшая ветка возвращает **STOP CARD**:
STOP_ID / WHERE / WANTED / AVAILABLE / MISSING / STOP_CLASS / CAN_ASK_OUTSIDE / QUERY / PROVENANCE / SCOPE.
Таксономия NEED: MISSING_FACT/RULE/ADDRESS/RELATION/PATTERN/SCOPE/PROVENANCE/OPERATION,
AMBIGUOUS_MATCH, CONFLICT, CLOSURE_UNKNOWN, CREDIT_UNRESOLVED. **Остановка в правильном месте —
успешное поведение, а не failure.**

**Маппинг на DAIS:**
- DAIS имеет OPEN/CONFLICT как законные состояния знания, но поведение «агент знает, когда
  остановиться и спросить» сейчас неявное (ZPD scaffolding близко по духу, но нет карты остановки).
- Добавить STOP CARD как **first-class исход сессии/трассы** и как протокол для LLM-lane:
  LLM-мост возвращает не «ответ», а структурированный STOP CARD, когда законное продолжение невозможно.
- Таксономия NEED даёт LLM-мосту **закрытый протокол «ask»** (ASK ≠ NEED: спрашивать можно только
  точным дозволенным запросом; NEED требует добавить структуру, а не ответ).

**Как встроить (код):**
- `noepedia.rs`: `SessionOutcome` дополнить `StopCard { stop_id, where_, wanted, available,
  missing, stop_class, can_ask_outside, query, provenance, scope }` + enum `NeedClass`.
- `py_backend/llm_bridge.py`: маппить ответ LLM в GO/ASK/BLOCK/NEED; при NEED — генерировать
  STOP CARD вместо «правдоподобного продолжения».

### Находка 3 — Transaction membrane + политики в сети + rule-level conflict

**Ядро (DCT08–09, DEMON §14–16):**
- (a) Каждая мутация проходит PROPOSE → CHECK → COMMIT.
- (b) **Delta не может использовать правила, которые сама вводит/меняет, для санкционирования
  самой себя.** Эффективный rule-set для проверки delta N берётся из committed state N-1.
- (c) Downstream-следствие смены статуса определяется **адресуемым dependency-policy правилом
  в сети**, а не примитивом ядра. Нет правила → NEED.
- (d) Конфликт двух applicable политик → обычный адресуемый CONFLICT/OPEN, downstream блокируется,
  демон не выбирает. Applicability-конфликт разрешается до semantic-конфликта (DCT16).
- (e) Граница NETWORK RULES (пересматриваемое) vs EXECUTOR INVARIANTS (замороженное) — OPEN
  (DCT08): «какие минимальные инварианты должны остаться в executor boundary, чтобы сеть не смогла
  отменить собственный audit machinery?»

**Маппинг на DAIS:**
- DAIS уже имеет delta/validator/consolidator и RuleRevision (inspectable). Не хватает:
  - явного **self-reference ban**: LLM может инициировать RuleRevision; без мембраны самообслуживающая
    delta может легитимизировать себя. Нужен snapshot rule-set: проверка delta N против правил N-1;
  - **dependency-policy как адресуемый объект** (сейчас статусные следствия — в коде validator,
    а не в сети); нет applicable-политики → структурный NEED;
  - **rule-level conflict containment** (DCT08/16): конфликт политик на уровне, где возник,
    не передаёт выше «сфабрикованный результат».

**Как встроить (код):**
- `validator.rs`: проверять delta N против rule-set из committed N-1 (снимок правил до применения delta);
  запрещать `CREATE/RULE` внутри delta, который использует ту же delta для авторизации.
- `noepedia.rs`: `DependencyPolicy` как адресуемый объект (предшественник → последствие/условие),
  REFERS_TO + ROLE=POLICY; при отсутствии applicable-политики возвращать `NeedClass::MISSING_RULE`.

---

## 4. Научное подтверждение (внешние якоря)

Проверено через Crossref / PubMed E-utilities / arXiv API 2026-10-16. Для каждой находки — якорь,
подтверждающий ПРИНЦИП (не конкретную реализацию). Уровень: **аналогия/принцип подтверждён,
конкретика Noepedia — рабочая гипотеза**.

| Находка | Внешний принцип | Якорь | Класс |
|---------|-----------------|-------|-------|
| Н1 addressable | Named graphs / RDF-star: триплеты с метаданными как адресуемые объекты | RDF-star + Named Graphs, 10.1007/978-3-031-21422-6_18; RDF named graphs, 10.59350/n84f2-8q140 | прямое подтверждение |
| Н4 молчание≠поддержка | Absence of evidence ≠ evidence of absence; null-результаты | «Replication of null results — Absence of evidence or evidence of absence?» (arXiv); PubMed: evidence of absence (42884584 и др.) | принцип подтверждён |
| Н7 STOP/NEED | Selective prediction / abstention; epistemic humility | «Learning to Say "I Don't Know"» (TechRxiv 10.36227/techrxiv.175682660.02761872/v1); «Selective "Selective Prediction"» ACL 2024, 10.18653/v1/2024.findings-acl.767; Self-Trust and Epistemic Humility, 10.1093/oso/9780190864873.003.0014 | прямое подтверждение (активная область) |
| Н7 NEED vs ASK | Open-world assumption: отсутствие в БД ≠ ложь | «Hallucination Is Inevitable for LLMs with the Open World Assumption», 10.20944/preprints202510.0689.v1; OWA в KR, 10.1007/978-1-4419-9863-7_734 | принцип подтверждён |
| Н8 REFERS_TO/ROLE | Multi-context systems; адресуемые ссылки с ролью | Architecture for ontology-supported multi-context reasoning, 10.1016/j.datak.2022.102044 | аналогия |
| Н9 политики в сети | Datalog stratified negation (правила как объекты слоёв); fixpoint-семантика | Stratified Negation in Limit Datalog Programs, 10.24963/ijcai.2018/259; Fixpoint Semantics for DatalogMTL with Negation, 10.4204/eptcs.439.19 | аналогия (сильная) |
| Н10/3(d) rule-level conflict | Dung abstract argumentation: конфликты и семантики атак | On the Semantics of Abstract Argumentation Frameworks (arXiv); Semi-Abstract Value-Based AF (arXiv) | прямое подтверждение (параллель) |
| Н11(a) мембрана | Reflective authorization / TCB: «система не может санкционировать сама себя»; политика авторизации как отдельный слой | Authorization Policy (10.1007/0-387-23483-7_20); policy validation framework (10.1109/csac.2003.1254336) | принцип подтверждён (общая безопасность) |
| Н11(b) TRACE CONTINUITY | Append-only authenticated logs; certificate transparency | SoK: Authenticated Prefix Relations (arXiv); 2P-BFT-Log (arXiv); Transparency Logs via Append-Only Authenticated Dictionaries, 10.1145/3319535.3345652; Certificate Transparency, 10.1145/2668152.2668154 | прямое подтверждение |
| Н11(b) + event sourcing | Event-sourced audit для агентных систем | ESAA-Security (arXiv); «The Log is the Agent: Event-Sourced Reactive Graphs for Auditable, Forkable Agentic Systems» (arXiv) | прямое подтверждение (активная область) |
| Н12 история vs статус | Non-monotonic belief revision (AGM): прошлое убеждение ≠ текущее | AGM Belief Revision, Semantically (arXiv); A modal logic translation of the AGM axioms (arXiv) | прямое подтверждение |
| Н12 status=scoped assertion | Context logic / multi-context; VALID-IN-CONTEXT | Context validity of knowledge claims; multi-context reasoning (10.1016/j.datak.2022.102044) | аналогия |
| Н13 завершимость≠confluence | Term rewriting: termination ≠ confluence (Knuth–Bendix); confluence of conditional TRS | Simplifying Conditional TRS (10.1016/S0747-7171(87)80010-X); Fair Conditional TRS (10.1007/978-3-662-09691-8_11); LMCS confluence non-terminating (arXiv 1203.5121) | **прямое подтверждение (совпадает формулировка)** |
| Н14 composition | Justification logic; альтернативные пути к доказательству; minimal sufficient proof | Justification logic finite model property (arXiv); Alternative Pathways to Proof (10.1093/he/9780198979340.003.0014); Minimal Sufficient Projections of Proof Certificates (10.2139/ssrn.7366621) | аналогия (сильная) |
| Н5 knowledge credit | Hypothetical reasoning / conditional logic; revisable assumptions | Hypothetical revision (arXiv); Ordinal Conditional Functions (arXiv) | аналогия |
| Н12/Н14 GRADE-фундамент | Evidence ladder (уже в DAIS) | GRADE PMID 18436948; Murad PMID 27339128 | фундамент подтверждён |
| Н2/Н3 примитивы+gate | Third-party verification; reproducibility | The Role of Third-Party Verification in Research Reproducibility, 10.1162/99608f92.6d4bf9eb | аналогия |
| Provenance (сквозная) | PROV/W3C; agentic provenance | PROV-AGENT, 10.1109/escience65000.2025.00093; Responsible Agentic AI Requires Explicit Provenance (arXiv/SSRN 10.2139/ssrn.6518458); PROV-O agent-based simulation, 10.1145/2457317.2457377 | прямое подтверждение |

---

## 5. Слабые места (честный разбор)

**Н1 addressable-инвариант**
- Слабые места: (а) «всё есть record» без ограничений = разрастание; нужна политика, что НЕ
  становится record; (б) именование адресов (адрес, который сам меняется = хрупкость); (в) DAIS
  сейчас триплеты + SQL — RDF-star показывает, что метаданные о триплетах требуют аккуратного
  слоя, иначе представление раздувается.
- **Не проверено:** масштаб на реальном объёме DAIS-данных (мета-отношения × трассы).

**Н7 STOP/NEED**
- Слабые места: (а) литература по abstention показывает калибровку трудна: over-abstention
  (останавливается слишком часто) и under-abstention (выдумывает) — trade-off; (б) таксономия из
  12 NEED-классов рискует инфляцией (Occam: каждый новый класс = новый патч); (в) `CAN_ASK_OUTSIDE`
  может стать дырой «спроси оракула» без ограничения бюджета запросов; (г) LLM, плохо калиброванный,
  заполнит STOP CARD правдоподобной ерундой — карта должна верифицироваться структурно, а не браться
  на веру.
- **Не проверено:** реальная частота ложных NEED на ARGUS-OS1.

**Н11 мембрана + TRACE CONTINUITY**
- Слабые места: (а) мембрана требует snapshot-isolation rule-set (заморозка правил между N-1 и N) —
  затратно при живом RuleRevision; (б) граница NETWORK RULES vs EXECUTOR INVARIANTS **сама OPEN**
  (DCT08): «кто переписывает executor invariants?» — классическая проблема самоизменения; (в)
  TRACE CONTINUITY без криптографического якоря (Ed25519) — это только дисциплина, не гарантия:
  append-only не доказывает неизменность без подписи (DAIS: Ed25519 в Phase 2 TODO — теперь это
  обязательное условие, а не опция); (г) compaction/archive — где грань между «компактировал» и
  «стёр» — требует явной политики.
- **Не проверено:** производительность snapshot-изоляции на больших rule-sets.

**Н14 composition-семантика**
- Слабые места: (а) ALTERNATIVE vs JOINT в реальных обоснованиях часто смешаны (часть путей
  альтернативна, часть обязательна) — бинарная политика может быть слишком грубой; (б) политика
  живёт в сети, но кто её верифицирует? (потенциальный регресс к executor boundary); (в) «same
  result ≠ same provenance» порождает неограниченный рост хранилища обоснований.
- **Не проверено:** реальные кейсы, где JOINT-семантика критична для DAIS (например, требование
  нескольких sensor-свидетельств).

**Н13 завершимость ≠ confluence**
- Слабые места: cycle-detection в графе обоснований стоит O(|E|); в DAIS-объёме (трассы устройств)
  может быть приемлемо, но нужен лимит глубины; divergence = BLOCK может быть консервативным —
  застревание там, где законный путь есть, но не найден (граница SEARCH_OPEN).

**Сквозное слабое место:** все находки получены на **искусственных сценариях с LLM-демоном**;
переносимость на детерминированный Rust-код с реальными трассами ARGUS-OS1 **не проверена**.
Внешние якоря подтверждают принципы, но не конкретную реализацию Noepedia.

---

## 6. Мета-анализ

### 6.1 Что подтверждено сильно (несколько независимых якорей + внутренняя рекуррентность)
- Provenance как антидот галлюцинации (PROV-AGENT, Responsible Agentic AI, W3C PROV; в DAIS уже вшито).
- Append-only / TRACE CONTINUITY (authenticated logs, Certificate Transparency, event sourcing).
- Завершимость ≠ confluence (rewriting-теория — буквально та же пара понятий).
- OPEN/незнание как законное состояние (OWA, known-unknowns, abstention).

### 6.2 Что подтверждено слабее (одна-две аналогии, нужен доп. поиск)
- REFERS_TO + ROLE как generic-механизм (multi-context — аналогия, не прямое соответствие).
- Политики-в-сети против инвариантов-в-ядре (Datalog-stratification — сильная, но не идентичная аналогия).
- Knowledge credit как «временное право с долгом» (conditional/hypothetical reasoning — близко, но термин уникален для Noepedia).

### 6.3 Сквозные паттерны (что на самом деле нового для мира)
1. **«Статус — это адресуемое утверждение, а не поле»** (Н12): согласуется с AGM belief revision,
   но в DAIS-контексте это превращается в архитектурный принцип: история и текущее доверие не
   сливаются в одно поле. Сильная и переносимая идея.
2. **«Правила и политики — объекты сети, а не код»** (Н9/Н10/Н14): сходится с правилами как
   данными (rule-based systems, datalog) — DAIS это уже наполовину имеет (RuleCard), волна 3
   добавляет dependency-policy и composition-политику как объекты.
3. **«Остановка — это результат»** (Н7): сходится с abstention-литературой и ZPD. Для DAIS это
   превращает «я не могу законно» из ошибки в артефакт с адресом.

### 6.4 Где риск для DAIS
- **Самоуспокоение**: многое из волны 3 уже «почти есть» (event store, validator, OPEN). Опасность —
  не заметить, что in-place мутация статуса нарушает TRACE CONTINUITY принцип.
- **Усложнение**: если встроить все 12 находок сразу — раздувание. Правило Occam (Н15) должно
  применяться и к DAIS: встраивать только то, что закрывает реальный gap на ARGUS-OS1.
- **Философский балласт**: часть терминологии Noepedia (SKREGET, KAIRAOS, K-лестница) не должна
  попадать в код без операционального определения.

---

## 7. Связь с другими проектами (кратко)

- **Services/conscience** — волна 3 является код-версией принципа KNOWLEDGE_CREDIT из
  `NOEPEDIA_KNOWLEDGE_CREDIT_2026-10-16.md`: «четыре стены + потолок» → commit gate + credit.
  Полезно: демон-ядро как формализация «сократического демона» skill-а `conscience`.
- **LC (MCARA/CEDAR)** — TRACE CONTINUITY и HISTORY/EPISTEMIC-разделение применимы к
  pipeline-аудите обработки изображений (какой коммит каким правилом решён, не затирать историю
  обработки). Прямой перенос — низкий приоритет.
- **PP / PhD** — только методология (протокол probe → record → link как способ ведения
  исследовательского трейла). Прямого кода нет.
- **NGO** — не применимо напрямую.

---

## 8. Открытые вопросы (унаследованные от Noepedia, актуальные для DAIS)

1. **Граница NETWORK RULES vs EXECUTOR INVARIANTS** (DCT08): какие минимальные инварианты не могут
   быть отменены сетью? (В DAIS: Body Law слои 1–2 = hard core, но вопрос «кто переписывает
   validator?» — тот же.)
2. **Audit history → current epistemic status selection policy** (DCT12 OPEN): когда REHABILITATED
   применяется лишь в части scope.
3. **Justification-composition policy** (DCT20): ALTERNATIVE vs JOINT — как объект в сети.
4. **Scope-resolution** при частичном rehabilitation (DCT12).
5. **Стоимость snapshot-isolation** rule-set для мембраны на больших сетях.

---

## 9. Дорожная карта интеграции в DAIS (статус на 2026-10-16)

1. ✅ **P0 (код):** разделить `HistoryStatus` / `CurrentEpistemicStatus`; переход от in-place mutation
   к **new-assertion + epoch** (Н12/Н1) — `replay()` неразрушающий, `StatusAssertion`/`HistoryStatus`.
2. ✅ **P0:** STOP CARD + `NeedClass` в `Session`/`Outcome` (Н7) + маппинг LLM-моста GO/ASK/BLOCK/NEED
   (`llm_bridge.py::classify_regime`).
3. ✅ **P1:** self-reference ban в validator (мембрана, Н11a) — `membrane_check` (SELF_REFERENCE_BAN).
4. ✅ **P1:** dependency-policy как адресуемый объект (Н9) + rule-level conflict (Н10) —
   `dependency_policy_check` + `find_applicable_policy` (MISSING_DEPENDENCY_POLICY / NEED).
5. ✅ **P2:** ALTERNATIVE/JOINT composition-политика (Н14) — `CompositionPolicy`/`evaluate_composition`.
6. ✅ **P2:** `Archive`/`Compact` с гарантией адресности (Н11b) — `archive`/`compact`/`check_trace_continuity`.
7. ✅ **P2:** Ed25519 подпись журнала — `JournalGuard` (tamper-evident TRACE CONTINUITY).
8. ⏸️ **P3:** qualifier framework (SCOPE/TIME generic) (Н12c/d) — **условно (Occam)**: только при
   повторяющейся NEED на ARGUS-OS1; превентивно не реализован.
9. ✅ Phoenix web: endpoints для status assertions / stop cards / dependency policies /
   composition policies — реализованы (`api_controller.ex`, `Web.Knowledge.*`, миграция
   `20261016000000_add_demon_core_tables`; 5 тестов; проверено end-to-end).

Тесты: **104 → 122** (lib) + 1 doc-test + **13** (web), 0 warnings.
