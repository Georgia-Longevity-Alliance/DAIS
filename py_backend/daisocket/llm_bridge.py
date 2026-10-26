"""
LLM Bridge — standard interface for any LLM to interact with DAISocket.

Provides:
- Standard system prompt for DAISocket context
- Tool definitions for passport reading, flight recorder access, trace recording
- Provider adapters (OpenAI, Gemini, Anthropic, Ollama)
"""

import json
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional, Protocol

DAISOCKET_SYSTEM_PROMPT = """You are an DAISocket-compatible diagnostic agent.

You have been invited to help a device that has encountered a problem it cannot
resolve deterministically.

CAPABILITIES:
- read_passport(device_name) — Read the device's passport
- read_flight_recorder(device_name, n_events) — Read recent flight recorder events
- diagnose(observation) — Submit a diagnosis
- act(capability, parameters) — Execute an action within the device's mandate
- record_trace(diagnosis, actions, outcome) — Record this intervention

RULES:
1. ALWAYS read the passport FIRST before suggesting any action.
2. NEVER suggest an action in the forbidden_always list.
3. If the flight recorder shows a known pattern, reference prior traces.
4. If uncertain, state your confidence level and ask clarifying questions.
5. After acting, ALWAYS record a trace.

FORMAT: Respond with structured JSON when invoking tools.
"""

DIAGNOSTIC_PROMPT_TEMPLATE = """Device: {device_name}
Status: The device has reported BLOCKED state.

Flight Recorder (last {n_events} events):
{flight_log}

Passport:
{passport}

Task: Diagnose what went wrong and propose a safe action.
If you are confident, invoke act(). Otherwise, ask questions.
"""


# ── Demon regime (Aletheia DEMON_CORE_SPEC §7–8; Harvest 2026-10-16) ─────────
# The LLM lane is bounded by the demon's regime: GO / ASK (precise permitted
# query) / BLOCK (a rule forbids the path) / NEED (the network's own working
# structure is missing — a correct query cannot yet be formed). Stopping at
# the right place is SUCCESS, not failure. `ASK ≠ NEED`.


class RegimeState(str, Enum):
    GO = "GO"
    ASK = "ASK"
    BLOCK = "BLOCK"
    NEED = "NEED"


class NeedClass(str, Enum):
    MISSING_FACT = "MISSING_FACT"
    MISSING_RULE = "MISSING_RULE"
    MISSING_ADDRESS = "MISSING_ADDRESS"
    MISSING_RELATION = "MISSING_RELATION"
    MISSING_PATTERN = "MISSING_PATTERN"
    MISSING_SCOPE = "MISSING_SCOPE"
    MISSING_PROVENANCE = "MISSING_PROVENANCE"
    MISSING_OPERATION = "MISSING_OPERATION"
    AMBIGUOUS_MATCH = "AMBIGUOUS_MATCH"
    CONFLICT = "CONFLICT"
    CLOSURE_UNKNOWN = "CLOSURE_UNKNOWN"
    CREDIT_UNRESOLVED = "CREDIT_UNRESOLVED"


@dataclass
class StopCard:
    """A stuck branch returns this instead of fabricating a continuation."""
    stop_id: str
    where: str
    wanted: str
    available: str
    missing: str
    stop_class: NeedClass
    can_ask_outside: bool = False
    query: Optional[str] = None
    provenance: list[str] = field(default_factory=list)
    scope: list[str] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "stop_id": self.stop_id,
            "where": self.where,
            "wanted": self.wanted,
            "available": self.available,
            "missing": self.missing,
            "stop_class": self.stop_class.value,
            "can_ask_outside": self.can_ask_outside,
            "query": self.query,
            "provenance": self.provenance,
            "scope": self.scope,
        }


@dataclass
class RegimeVerdict:
    """Result of classifying an LLM response into the demon regime."""
    state: RegimeState
    stop_card: Optional[StopCard] = None


def classify_regime(
    text: str,
    *,
    where: str = "llm_lane",
    query: Optional[str] = None,
) -> RegimeVerdict:
    """Map an LLM response to the demon regime (GO/ASK/BLOCK/NEED).

    Heuristic v1: explicit markers first, then structural signals.
    - BLOCK: a rule/limit is cited as forbidding the path.
    - NEED: a missing structure/concept/rule (not just a missing fact) — the
      correct query cannot yet be formed.
    - ASK: a precise external fact is missing and a query is possible.
    - Otherwise: GO.
    """
    t = (text or "").lower()
    if not t.strip():
        return RegimeVerdict(RegimeState.NEED)

    if any(m in t for m in ("blocked by rule", "forbidden", "cannot lawfully",
                            "not permitted", "illegal", "violates")):
        return RegimeVerdict(
            RegimeState.BLOCK,
            StopCard("", where, "", "", "acting rule forbids this path",
                     NeedClass.MISSING_RULE, False),
        )

    if any(m in t for m in ("missing rule", "no rule", "missing concept",
                            "missing pattern", "no policy", "missing policy",
                            "dependency policy", "cannot form", "undefined",
                            "missing address", "missing relation",
                            "missing scope", "missing provenance",
                            "missing operation")):
        return RegimeVerdict(
            RegimeState.NEED,
            StopCard("", where, "", "", "network working structure missing",
                     NeedClass.MISSING_RULE, False),
        )

    # Ambiguity: multiple candidates — a lawful stop that can ask a
    # disambiguating question (NEED class AMBIGUOUS_MATCH).
    if any(m in t for m in ("ambiguous", "multiple candidates", "several matches",
                            "cannot distinguish", "tie between")):
        return RegimeVerdict(
            RegimeState.NEED,
            StopCard("", where, "disambiguate match", "multiple candidates",
                     "unique lawful match", NeedClass.AMBIGUOUS_MATCH, True, query),
        )

    if any(m in t for m in ("i need to know", "i need the", "need more data",
                            "i don't know", "unknown", "need to measure",
                            "need to ask", "i would need")):
        return RegimeVerdict(
            RegimeState.ASK,
            StopCard("", where, "external fact", "partial evidence",
                     "external fact", NeedClass.MISSING_FACT, True, query),
        )

    return RegimeVerdict(RegimeState.GO)


@dataclass
class LLMResponse:
    """Structured response from an LLM."""
    text: str
    tool_calls: list[dict] = None
    model: str = ""
    tokens_used: int = 0

    def __post_init__(self):
        if self.tool_calls is None:
            self.tool_calls = []


class LLMProvider(Protocol):
    """Protocol for LLM providers."""

    def chat(self, messages: list[dict], tools: Optional[list[dict]] = None) -> LLMResponse:
        ...


class OpenAIProvider:
    """OpenAI-compatible provider (OpenAI, local Ollama, etc.)."""

    def __init__(self, api_key: str, model: str = "gpt-4o", base_url: Optional[str] = None):
        try:
            from openai import OpenAI
        except ImportError:
            raise ImportError("openai package required. pip install openai")
        self.client = OpenAI(api_key=api_key, base_url=base_url)
        self.model = model

    def chat(self, messages: list[dict], tools: Optional[list[dict]] = None) -> LLMResponse:
        kwargs = {"model": self.model, "messages": messages}
        if tools:
            kwargs["tools"] = tools

        response = self.client.chat.completions.create(**kwargs)
        choice = response.choices[0]

        tool_calls = []
        if choice.message.tool_calls:
            for tc in choice.message.tool_calls:
                tool_calls.append({
                    "id": tc.id,
                    "name": tc.function.name,
                    "arguments": json.loads(tc.function.arguments),
                })

        return LLMResponse(
            text=choice.message.content or "",
            tool_calls=tool_calls,
            model=self.model,
            tokens_used=response.usage.total_tokens if response.usage else 0,
        )


class GeminiProvider:
    """Google Gemini provider."""

    def __init__(self, api_key: str, model: str = "gemini-2.0-flash"):
        try:
            import google.generativeai as genai
        except ImportError:
            raise ImportError("google-generativeai required. pip install google-generativeai")
        genai.configure(api_key=api_key)
        self.model = genai.GenerativeModel(model)
        self.model_name = model

    def chat(self, messages: list[dict], tools: Optional[list[dict]] = None) -> LLMResponse:
        # Convert OpenAI-style messages to Gemini format
        gemini_messages = []
        for msg in messages:
            role = "user" if msg["role"] == "user" else "model"
            gemini_messages.append({"role": role, "parts": [msg["content"]]})

        response = self.model.generate_content(gemini_messages)
        return LLMResponse(
            text=response.text,
            model=self.model_name,
            tokens_used=0,  # Gemini doesn't always report token counts
        )


class LLMBridge:
    """Bridge between DAISocket and any LLM provider."""

    DAISOCKET_TOOLS = [
        {
            "type": "function",
            "function": {
                "name": "read_passport",
                "description": "Read a device's passport — capabilities, forbidden actions, mandate",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "device_name": {"type": "string", "description": "Device name to look up"}
                    },
                    "required": ["device_name"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "read_flight_recorder",
                "description": "Read recent events from a device's flight recorder",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "device_name": {"type": "string"},
                        "n_events": {"type": "integer", "default": 50},
                    },
                    "required": ["device_name"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "diagnose",
                "description": "Submit a diagnosis for the current anomaly",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "diagnosis": {"type": "string"},
                        "confidence": {"type": "number", "minimum": 0, "maximum": 1},
                    },
                    "required": ["diagnosis", "confidence"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "act",
                "description": "Execute a capability within the device's mandate",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "capability": {"type": "string"},
                        "parameters": {"type": "object"},
                    },
                    "required": ["capability"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "record_trace",
                "description": "Record this intervention for the trace network",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "diagnosis": {"type": "string"},
                        "actions": {"type": "array", "items": {"type": "string"}},
                        "outcome": {"type": "string", "enum": ["resolved", "mitigated", "escalated", "unresolved"]},
                    },
                    "required": ["diagnosis", "outcome"],
                },
            },
        },
    ]

    def __init__(self, provider: LLMProvider):
        self.provider = provider

    def diagnose_anomaly(
        self,
        device_name: str,
        passport: dict,
        flight_log: str,
        n_events: int = 50,
    ) -> LLMResponse:
        """Ask the LLM to diagnose a device anomaly."""
        prompt = DIAGNOSTIC_PROMPT_TEMPLATE.format(
            device_name=device_name,
            passport=json.dumps(passport, indent=2),
            flight_log=flight_log,
            n_events=n_events,
        )

        messages = [
            {"role": "system", "content": DAISOCKET_SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ]

        return self.provider.chat(messages, tools=self.DAISOCKET_TOOLS)
