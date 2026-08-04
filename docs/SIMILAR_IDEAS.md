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

### "Safety Marketplace"
Как Proven marketplace, но для safety-правил:
- "Laser safety protocol for microscopy" — 50 credits
- "Mobile robot indoor navigation limits" — 30 credits
- Купил правило → применил → верифицировал → published proof
