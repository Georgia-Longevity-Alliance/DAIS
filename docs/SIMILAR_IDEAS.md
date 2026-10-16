# Идеи, подобные DAIS — Обзор ландшафта

## 1. ПРЯМЫЕ АНАЛОГИ

### Robot Passport / Machine Identity
| Проект | Суть | Отличие от DAIS |
|--------|------|----------------|
| **IETF ACE** (Authentication for Constrained Environments) | OAuth для IoT устройств | Нет Body Law, нет safety layers |
| **W3C DID** (Decentralized Identifiers) | Самосуверенная идентичность устройств | Нет аппаратных ограничений (STOP, deadman) |
| **SPIFFE** | Унифицированная идентичность в распределённых системах | Cloud-only, не для embedded |
| **AWS IoT Device Shadow** | Цифровой двойник устройства | Проприетарно, нет safety |

### Robot Safety
| Проект | Суть | Отличие от DAIS |
|--------|------|----------------|
| **ISO 13482** | Безопасность персональных роботов | Стандарт, не софт |
| **IEC 61508** | Функциональная безопасность | Промышленный стандарт |
| **ROS2 Safety Working Group** | Safety layer для Robot OS | Фокус на автономные авто |
| **Open Robotics IEC 61508** | Сертифицируемый ROS2 | Сложный, промышленный |

### Knowledge Verification / Anti-Hallucination
| Проект | Суть | Отличие от DAIS |
|--------|------|----------------|
| **Proven** (Gakely) | Provenance-aware knowledge | Часть AIS, интегрирована |
| **KERI** (Key Event Receipt Infrastructure) | Verifiable data chains | Нет S-P-O knowledge model |
| **Content Authenticity Initiative** (Adobe) | C2PA provenance standard | Медиа, не научное знание |
| **OpenAlex** | Открытый граф научных знаний | Нет verification layer |
| **Wikidata** | Структурированные знания | Нет provenance tracking |

## 2. АРХИТЕКТУРНЫЕ ПАТТЕРНЫ

### "Passport Pattern" — Device Identity + Capability Declaration
```
Device → Passport (capabilities, forbidden_always, emergency_contacts)
       → Body Law (7-layer validator)
       → Command Execution
```
**Аналоги:**
- Kubernetes PodSecurityPolicy
- Android App Permissions
- OAuth2 Scopes
- MacOS Hardened Runtime

### "Flight Recorder Pattern" — Black Box для устройств
```
All Events → Ring Buffer → On Anomaly: LLM Diagnosis → Trace Network
```
**Аналоги:**
- Aircraft FDR/CVR (чёрный ящик)
- Tesla Autopilot snapshot
- AWS CloudTrail
- PostgreSQL WAL (Write-Ahead Log)

### "Body Migration Pattern" — Интеллект отделён от тела
```
Brain → Body₁ → Body₂ → Body₃ (разные морфологии, один контроллер)
```
**Аналоги:**
- Docker containers (один образ → много инстансов)
- USB device driver model
- WebAssembly (write once, run anywhere)

## 3. СМЕЖНЫЕ ОБЛАСТИ ДЛЯ ИНТЕГРАЦИИ

### Роевая робототехника (Swarm Robotics)
- **Kilobots** (Harvard) — 1000+ простых роботов
- **ARGoS** — симулятор роев
- **Buzz** — язык программирования роев
- **DAIS fit:** Fleet Manager + роевая координация

### Цифровые двойники (Digital Twins)
- **Eclipse Ditto** — фреймворк digital twins
- **Azure Digital Twins** — Microsoft
- **DAIS fit:** Passport = дескриптор twin; Flight Recorder = история twin

### Объяснимый AI (XAI)
- **SHAP** — feature importance
- **LIME** — локальная интерпретируемость
- **DAIS fit:** Proven claims + confidence scores

### Формальная верификация
- **TLA+** — спецификация распределённых систем
- **Coq/Rocq** — доказательство теорем
- **DAIS fit:** Body Law верификация — формальное доказательство safety

### Edge AI / TinyML
- **TensorFlow Lite Micro** — ML на микроконтроллерах
- **ONNX Runtime** — портативный инференс
- **DAIS fit:** AnomalyDetector на ESP32

## 4. ПОТЕНЦИАЛЬНЫЕ ПАРТНЁРСТВА

| Организация | Проект | Синерия |
|-------------|--------|---------|
| **Open Robotics** | ROS2 + Safety | DAIS как safety layer для ROS2 |
| **Eclipse Foundation** | Ditto + Hono | IoT device management |
| **W3C** | DID + Verifiable Credentials | Стандартизация Passport |
| **NIST** | AI Safety Framework | DAIS как reference implementation |
| **MIT CSAIL** | Embodied Intelligence | Research partnership |

## 5. НЕОЧЕВИДНЫЕ ИДЕИ ДЛЯ AIS

### "Proof of Safety" — криптографическое доказательство
Каждая команда, прошедшая Body Law, получает zk-SNARK доказательство:
"Я проверил все 7 слоёв, вот proof, команда безопасна."

### "Federated Safety Learning"
Устройства делятся аномалиями без раскрытия сырых данных:
- Federated Learning на эмбеддедах Flight Recorder
- Приватность (никакие сырые данные не покидают устройство)
- Коллективный иммунитет

### "Digital Genome" для роботов
Каждое устройство наследует safety-профиль от родительского класса:
```
BaseRobot → WheeledRobot → DeliveryRobot → MyCustomBot
          ↳ FlyingRobot → Quadcopter → DJI_Mavic
```
С каждым уровнем добавляются capabilities и forbidden_always.

## 6. НОВЫЕ ПОДОБНЫЕ ИДЕИ (обзор 2026-10-16, волна 3 Noepedia)

> Мастер: `docs/NOEPEDIA_DEMON_CORE_HARVEST_2026-10-16.md`. Это аналоги/параллели, найденные
> в научных базах при проверке демон-ядра. Класс: прямое подтверждение / аналогия / смежное.

### Knowledge / Provenance (прямые аналоги)
| Идея | Где | Класс | Связь с DAIS |
|------|-----|-------|--------------|
| **PROV-AGENT** — unified provenance для агентных AI-систем | eScience 2025, 10.1109/escience65000.2025.00093 | прямое | Proven chain — аналог PROV-AGENT для устройств; можно взять их схему взаимодействий |
| **Responsible Agentic AI Requires Explicit Provenance** | SSRN 10.2139/ssrn.6518458 | прямое | Подтверждает «provenance — предпосылка ответственности» |
| **RDF-star + Named Graphs** — триплеты с метаданными | 10.1007/978-3-031-21422-6_18 | прямое | Модель «метаданные о триплетах» для status assertions поверх SPO |
| **Event-sourced agent systems** — «The Log is the Agent», ESAA-Security | arXiv 2025/26 | прямое | Event store DAIS — ровно этот паттерн; берём их auditable-схемы |

### Abstention / stopping (прямые аналоги для STOP/NEED)
| Идея | Где | Класс | Связь с DAIS |
|------|-----|-------|--------------|
| **Learning to Say "I Don't Know"** — abstention для LLM | TechRxiv 10.36227/techrxiv.175682660.02761872/v1 | прямое | Обоснование STOP CARD как результата, а не ошибки |
| **Selective "Selective Prediction"** — калибровка abstention | ACL 2024, 10.18653/v1/2024.findings-acl.767 | прямое | Трейдофф over/under-abstention — слабое место нашей карты остановки |
| **Hallucination Is Inevitable for LLMs with the OWA** | 10.20944/preprints202510.0689.v1 | принцип | NOT_FOUND ≠ absent — якорь для ASK vs NEED |
| **Self-Trust and Epistemic Humility** | 10.1093/oso/9780190864873.003.0014 | аналогия | Эпистемическое смирение агента = NEED-режим |

### Logs / immutability (для TRACE CONTINUITY)
| Идея | Где | Класс | Связь с DAIS |
|------|-----|-------|--------------|
| **SoK: Authenticated Prefix Relations** — append-only логи | arXiv | прямое | Основа для «нельзя сделать историю безадресной» |
| **Certificate Transparency** | 10.1145/2668152.2668154 | прямое | Дисциплина неизменяемого лога в продакшене |
| **2P-BFT-Log** — append-only в adversarial среде | arXiv | смежное | Для федеративных реестров (Phase 2) |

### Reasoning / rules (для мембраны и политик)
| Идея | Где | Класс | Связь с DAIS |
|------|-----|-------|--------------|
| **Dung abstract argumentation** — конфликты и семантики | arXiv | прямое | Rule-level conflict → addressable CONFLICT |
| **Datalog stratified negation** — правила по слоям | IJCAI 2018, 10.24963/ijcai.2018/259 | аналогия | Политики-в-сети, fixpoint по слоям |
| **Multi-context systems** | 10.1016/j.datak.2022.102044 | аналогия | Scoped-статусы VALID-IN-CONTEXT |
| **Term rewriting: termination ≠ confluence** | 10.1016/S0747-7171(87)80010-X; arXiv 1203.5121 | прямое | Завершимость ≠ Confluence — та же пара понятий |
| **AGM belief revision** — прошлое убеждение ≠ текущее | arXiv | прямое | HISTORY_STATUS ≠ EPISTEMIC_STATUS |
| **Justification logic / minimal proofs** | arXiv; 10.2139/ssrn.7366621 | аналогия | ALTERNATIVE vs JOINT composition |

### Authorization (для мембраны)
| Идея | Где | Класс | Связь с DAIS |
|------|-----|-------|--------------|
| **Reflective authorization / policy layers** | 10.1007/0-387-23483-7_20; 10.1109/csac.2003.1254336 | принцип | «Delta не санкционирует саму себя» — TCB-дисциплина |

### "Safety Marketplace"
Как Proven marketplace, но для safety-правил:
- "Laser safety protocol for microscopy" — 50 credits
- "Mobile robot indoor navigation limits" — 30 credits
- Купил правило → применил → верифицировал → published proof
