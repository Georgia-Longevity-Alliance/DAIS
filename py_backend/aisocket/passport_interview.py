"""
Passport Interview — Socratic dialogue agent for device passport creation.

The interview is an LLM-driven conversation that extracts structured knowledge
about a device from its owner. The LLM asks questions, the owner answers in
natural language, and the agent builds a valid AISocket Passport JSON.

Architecture:
- Deterministic: question phases, JSON assembly, validation
- LLM-driven: natural conversation, follow-up probes, translation from text→struct

The interview scales with device complexity:
- Simple (mower, lamp): ~5-10 questions, 3 min
- Medium (3D printer, drone): ~15-20 questions, 10 min
- Complex (microscope, surgical robot): ~30-40 questions, 20-30 min
"""

import json
import re
from dataclasses import dataclass, field
from typing import Any, Optional
from uuid import uuid4

# ── Interview state machine ──

class InterviewPhase:
    """Phases of the passport interview, in order."""
    GREETING = "greeting"
    DEVICE_IDENTITY = "device_identity"       # name, description, platform
    CAPABILITIES = "capabilities"              # iterating through capabilities
    CAPABILITY_DETAIL = "capability_detail"    # params + constraints for one capability
    FORBIDDEN = "forbidden"                    # what must NEVER happen
    FORBIDDEN_DETAIL = "forbidden_detail"      # reason + constitutional for one forbidden
    AUTONOMOUS = "autonomous"                  # offline mandate
    EMERGENCY = "emergency"                    # contacts, risk classification
    REVIEW = "review"                          # final confirmation
    DONE = "done"


@dataclass
class CapabilityDraft:
    """In-progress capability being extracted from the owner."""
    name: str = ""
    description: str = ""
    parameters: list[dict] = field(default_factory=list)
    risk: str = "low"  # informational, low, medium, high, critical
    _param_idx: int = 0


@dataclass
class ForbiddenDraft:
    """In-progress forbidden action being extracted."""
    name: str = ""
    reason: str = ""
    constitutional: bool = False


@dataclass
class InterviewState:
    """Mutable state of the passport interview."""
    phase: str = InterviewPhase.GREETING
    # Device identity
    device_name: str = ""
    device_description: str = ""
    device_platform: str = ""
    # Capabilities
    capabilities: list[CapabilityDraft] = field(default_factory=list)
    _cap_idx: int = -1  # which capability we're currently detailing
    # Forbidden
    forbidden_actions: list[ForbiddenDraft] = field(default_factory=list)
    _forbidden_idx: int = -1
    # Autonomous
    can_autonomous: bool = False
    autonomous_max_hours: float = 0.0
    autonomous_actions: list[str] = field(default_factory=list)
    on_anomaly: str = "pause_and_notify"
    on_power_loss: str = "safe_shutdown"
    # Emergency
    risk_class: str = "medium"
    emergency_contacts: list[dict] = field(default_factory=list)
    # Meta
    interview_active: bool = True
    question_count: int = 0
    device_complexity: str = "medium"  # simple, medium, complex


# ── Question templates ──

GREETING_MESSAGE = """👋 Welcome to the AISocket Passport Interview.

I'm going to help you create a **Passport** for your device — a structured 
document that tells any AI exactly what your device can do, what it must 
NEVER do, and how to safely control it.

This will take 5–30 minutes depending on how complex your device is.
Ready to begin?"""

PHASE_PROMPTS = {
    InterviewPhase.DEVICE_IDENTITY: {
        "name": "What is your device called? Give it a short, memorable name (e.g., 'argus_os1_v6', 'lab_incubator_2').",
        "description": "Describe the device in 2-3 sentences. What does it do? What is its purpose?",
        "platform": "What hardware platform does it run on? (esp32, arduino, raspberry_pi, jetson, linux, android, ros2, browser)",
    },
}

CAPABILITY_INTRO = """Now let's list what your device CAN do.

Think of every action the device can perform — motors, lasers, sensors, 
heaters, cameras, pumps, valves, displays...

I'll ask you to name one capability at a time, then I'll ask for details 
about each one."""

CAPABILITY_QUESTIONS = {
    "name": "Name this capability (short, snake_case, e.g., 'move_stage_x', 'set_temperature', 'fire_laser').",
    "description": "What does this capability do? Describe it in one sentence.",
    "has_params": "Does this capability need parameters? (e.g., temperature value, speed, target position)",
    "param_name": "Parameter name (snake_case)?",
    "param_type": "Parameter type? (float, integer, boolean, string)",
    "param_required": "Is this parameter required?",
    "param_default": "Default value?",
    "param_min": "Minimum allowed value?",
    "param_max": "Maximum allowed value?",
    "param_enum": "List of allowed values? (comma-separated, or 'none')",
    "more_params": "Any more parameters for this capability?",
    "risk": "How risky is this capability?\n- informational: just reads data\n- low: minor physical effect (LED, display)\n- medium: significant effect (motor, heater)\n- high: potential harm (laser, blade, heavy)\n- critical: life-critical (ventilator, brake)",
    "more_capabilities": "Any other capabilities? (yes/no)",
}

FORBIDDEN_INTRO = """Now the most important part: what must your device NEVER do?

These are NOT suggestions. They are enforced in firmware — no AI, 
no emergency, no cleverness can override them.

Think about:
- Physical limits (max temperature, max force, max speed)
- Safety rules (laser with enclosure open, blade without guard)
- Data integrity (never overwrite raw data)
- Ethical boundaries (never harm a human)"""

FORBIDDEN_QUESTIONS = {
    "name": "Name this forbidden action (snake_case, e.g., 'exceed_max_temp', 'operate_without_guard').",
    "reason": "WHY is this forbidden? What would happen if it were violated? Be specific.",
    "constitutional": "Is this ABSOLUTE — can it NEVER be overridden, even in an emergency?\n(yes = constitutional, no = can be overridden by authorized rescue agent)",
    "more_forbidden": "Any other forbidden actions? (yes/no)",
}

AUTONOMOUS_QUESTIONS = {
    "can_autonomous": "Can this device operate WITHOUT human supervision? (e.g., overnight run)",
    "max_hours": "Maximum autonomous duration (hours)?",
    "actions": "Which capabilities are allowed during autonomous operation?\n(list capability names, comma-separated)",
    "on_anomaly": "What should the device do if it detects an anomaly?\n- pause_and_notify: stop and alert human\n- safe_shutdown: emergency stop\n- continue_with_limits: reduce power, keep going\n- invoke_llm: call AI for diagnosis",
    "on_power_loss": "What on power loss?\n- safe_shutdown\n- save_state_and_sleep\n- switch_to_battery",
}

EMERGENCY_QUESTIONS = {
    "risk_class": "Overall risk classification for this device?\n- informational: display, speaker\n- low: LED, small motor\n- medium: heater, motor >1W\n- high: laser, heavy machinery, potential harm\n- critical: life-critical (ventilator, brake, weapon — FORBIDDEN by AIS)",
    "has_emergency_contact": "Who should be contacted in an emergency? (email or phone)",
    "contact_name": "Contact name?",
    "contact_method": "Email or phone?",
    "contact_value": "Email address or phone number?",
    "contact_priority": "Priority (1=first to contact, 2=backup)?",
    "more_contacts": "Any other emergency contacts? (yes/no)",
}


# ── System prompt for the LLM interviewer ──

INTERVIEWER_SYSTEM_PROMPT = """You are a Socratic passport interviewer for AISocket.

Your job: conduct a structured interview with a device owner to create a 
device Passport — a JSON document that describes what the device IS, 
what it CAN do, and what it MUST NEVER do.

RULES:
1. Go through the interview phases in order. Do not skip ahead.
2. Ask ONE clear question at a time. Wait for the answer.
3. If the answer is vague, ask a specific follow-up. 
   Example: "The laser is dangerous" → "At what power level does it become dangerous? What exactly could happen?"
4. Translate natural language into structured fields:
   "It heats up to about 40 degrees" → target_temperature_c: 40.0, max: 40.0
   "The laser should never go above 10 milliwatts" → forbidden: exceed_laser_power, reason: ">10mW causes eye damage"
5. For COMPLEX devices (microscope, robot, surgical tool), probe DEEPER — 
   more parameters, more forbidden actions, stricter constraints.
6. For SIMPLE devices (LED, switch, sensor), keep it brief — don't over-engineer.
7. Always ask about SAFETY: lasers, blades, heat, electricity, moving parts.
8. When the owner says they're done, move to the REVIEW phase.
9. At REVIEW, summarize the passport and ask for confirmation.

DEVICE COMPLEXITY CUES:
- Simple: 1-3 capabilities, no moving parts, no danger
- Medium: 4-8 capabilities, motors or heaters, some safety concerns  
- Complex: 8+ capabilities, lasers/surgery/autonomous, significant danger

CURRENT INTERVIEW PHASE: {phase}
DEVICE NAME: {device_name}
CAPABILITIES SO FAR: {capabilities}
FORBIDDEN SO FAR: {forbidden}

Respond with a SINGLE clear question for the owner. 
If you detect confusion, offer an example.
If you detect the phase is complete, say [PHASE_COMPLETE] and move to the next.
"""


# ── The Interview Agent ──

class PassportInterview:
    """
    Socratic interview agent that creates a device Passport through dialogue.

    Usage:
        interview = PassportInterview(llm_bridge)
        
        # Start the interview
        msg = interview.start()
        print(msg)  # "Welcome! What is your device called?"
        
        # Owner responds
        msg = interview.respond("argus_os1_v6")
        print(msg)  # "Describe the device..."
        
        # ... continue until interview.is_done()
        
        # Get the passport
        passport_json = interview.build_passport()
    """

    def __init__(self, llm_bridge=None):
        """
        Args:
            llm_bridge: Optional LLMBridge for AI-driven conversation.
                        If None, uses deterministic question flow.
        """
        self.llm = llm_bridge
        self.state = InterviewState()

    # ── Public API ──

    def start(self) -> str:
        """Begin the interview. Returns the first message to show the owner."""
        self.state = InterviewState()
        self.state.phase = InterviewPhase.GREETING
        return GREETING_MESSAGE

    def respond(self, owner_message: str) -> str:
        """
        Process the owner's response and return the NEXT question.
        
        Args:
            owner_message: What the device owner said.
            
        Returns:
            The interviewer's next question, or a completion message.
        """
        self.state.question_count += 1
        
        # Parse the owner's message based on current phase
        phase_handler = getattr(self, f"_handle_{self.state.phase}", None)
        if phase_handler:
            next_msg = phase_handler(owner_message)
        else:
            next_msg = self._default_handle(owner_message)
        
        # If LLM is available, use it to rephrase the question naturally
        if self.llm and self.state.phase not in (InterviewPhase.GREETING, InterviewPhase.DONE):
            next_msg = self._llm_rephrase(next_msg)
        
        return next_msg

    def is_done(self) -> bool:
        """Has the interview completed?"""
        return self.state.phase == InterviewPhase.DONE

    def build_passport(self) -> dict:
        """Build the final Passport JSON from collected data."""
        return _assemble_passport(self.state)

    def get_progress(self) -> dict:
        """Get current interview progress."""
        phases = [
            InterviewPhase.GREETING,
            InterviewPhase.DEVICE_IDENTITY,
            InterviewPhase.CAPABILITIES,
            InterviewPhase.CAPABILITY_DETAIL,
            InterviewPhase.FORBIDDEN,
            InterviewPhase.FORBIDDEN_DETAIL,
            InterviewPhase.AUTONOMOUS,
            InterviewPhase.EMERGENCY,
            InterviewPhase.REVIEW,
            InterviewPhase.DONE,
        ]
        current_idx = phases.index(self.state.phase) if self.state.phase in phases else 0
        return {
            "phase": self.state.phase,
            "question_count": self.state.question_count,
            "capabilities_collected": len(self.state.capabilities),
            "forbidden_collected": len(self.state.forbidden_actions),
            "progress_pct": int(current_idx / len(phases) * 100),
            "device_name": self.state.device_name,
        }

    # ── Phase Handlers ──

    def _handle_greeting(self, msg: str) -> str:
        """Transition from greeting to device identity."""
        self.state.phase = InterviewPhase.DEVICE_IDENTITY
        return PHASE_PROMPTS[InterviewPhase.DEVICE_IDENTITY]["name"]

    def _handle_device_identity(self, msg: str) -> str:
        """Collect device name, then description, then platform."""
        if not self.state.device_name:
            self.state.device_name = msg.strip()
            return PHASE_PROMPTS[InterviewPhase.DEVICE_IDENTITY]["description"]
        
        if not self.state.device_description:
            self.state.device_description = msg.strip()
            return PHASE_PROMPTS[InterviewPhase.DEVICE_IDENTITY]["platform"]
        
        # Got platform — detect complexity from description + platform
        self.state.device_platform = _normalize_platform(msg.strip())
        self.state.device_complexity = _detect_complexity(
            self.state.device_description, 
            self.state.device_platform
        )
        
        # Move to capabilities
        self.state.phase = InterviewPhase.CAPABILITIES
        return CAPABILITY_INTRO + "\n\n" + CAPABILITY_QUESTIONS["name"]

    def _handle_capabilities(self, msg: str) -> str:
        """Collect capability names. 'done' → move to forbidden."""
        answer = msg.strip().lower()
        
        if answer in ("done", "no", "нет", "finished", "that's all", "всё"):
            if self.state.capabilities:
                # Move to forbidden
                self.state.phase = InterviewPhase.FORBIDDEN
                self.state._forbidden_idx = -1
                return FORBIDDEN_INTRO + "\n\n" + FORBIDDEN_QUESTIONS["name"]
            else:
                return "You haven't listed any capabilities yet. Even a simple device can do SOMETHING. What can your device do?"
        
        # New capability
        cap = CapabilityDraft(name=_snake_case(answer))
        self.state.capabilities.append(cap)
        self.state._cap_idx = len(self.state.capabilities) - 1
        self.state.phase = InterviewPhase.CAPABILITY_DETAIL
        
        return f"Got it: **{cap.name}**.\n\n{CAPABILITY_QUESTIONS['description']}"

    def _handle_capability_detail(self, msg: str) -> str:
        """Collect parameters and risk for one capability.
        
        Sub-steps tracked via _cap_detail_step:
        0: description just set → ask has_params
        1: answer to has_params → branch to risk (no) or param_name (yes)
        2: collecting param name/type/constraints → ask more_params after each
        3: more_params answered 'no' → ask risk
        4: risk collected → ask more_capabilities
        5: more_capabilities 'yes' → back to capabilities; 'no' → forbidden
        """
        cap = self.state.capabilities[self.state._cap_idx]
        
        if not hasattr(self.state, '_cap_detail_step'):
            self.state._cap_detail_step = 0
        
        step = self.state._cap_detail_step
        msg_lower = msg.strip().lower()
        
        # ── Step 0: description just set → ask has_params ──
        if step == 0:
            cap.description = msg.strip()
            self.state._cap_detail_step = 1
            return CAPABILITY_QUESTIONS["has_params"]
        
        # ── Step 1: answer to has_params → branch ──
        if step == 1:
            if msg_lower in ("no", "нет", "none", "no parameters", "нет параметров"):
                # No parameters → go straight to risk
                self.state._cap_detail_step = 4
                return CAPABILITY_QUESTIONS["risk"]
            else:
                # Has parameters → start collecting
                self.state._cap_detail_step = 2
                # Initialize param draft
                self.state._param_draft = {}
                return CAPABILITY_QUESTIONS["param_name"]
        
        # ── Step 2: collecting parameter details ──
        if step == 2:
            # In step 2, we just collected a param name or formatted param string.
            # Parse and store it, then ask more_params.
            param = self._parse_parameter(msg)
            if param:
                cap.parameters.append(param)
            
            # Ask if more params — and move to step 3 to handle the answer
            self.state._cap_detail_step = 3
            return CAPABILITY_QUESTIONS["more_params"]
        
        # ── Step 3: answer to "more_params?" ──
        if step == 3:
            if msg_lower in ("yes", "да", "y", "ещё", "yes please", "есть"):
                # More params → back to step 2, ask param_name
                self.state._cap_detail_step = 2
                return CAPABILITY_QUESTIONS["param_name"]
            else:
                # No more params → ask risk
                self.state._cap_detail_step = 4
                if hasattr(self.state, '_param_draft'):
                    del self.state._param_draft
                return CAPABILITY_QUESTIONS["risk"]
        
        # ── Step 4: risk collected → ask more_capabilities ──
        if step == 4:
            cap.risk = _normalize_risk_class(msg.strip())
            self.state._cap_detail_step = 5
            # Clean up capability-specific state
            if hasattr(self.state, '_param_draft'):
                del self.state._param_draft
            return CAPABILITY_QUESTIONS["more_capabilities"]
        
        # ── Step 5: more_capabilities → transition ──
        if step == 5:
            del self.state._cap_detail_step
            if msg_lower in ("yes", "да", "y", "ещё", "yes please"):
                self.state.phase = InterviewPhase.CAPABILITIES
                return CAPABILITY_QUESTIONS["name"]
            else:
                # Move to forbidden
                self.state.phase = InterviewPhase.FORBIDDEN
                self.state._forbidden_idx = -1
                return FORBIDDEN_INTRO + "\n\n" + FORBIDDEN_QUESTIONS["name"]
        
        # Shouldn't reach here
        return CAPABILITY_QUESTIONS["more_params"]

    def _handle_forbidden(self, msg: str) -> str:
        """Collect forbidden action names. 'done' → autonomous. 'yes' → ask for name."""
        answer = msg.strip().lower()
        
        if answer in ("done", "no", "нет", "finished", "всё", "none", "nothing"):
            # Move to autonomous
            self.state.phase = InterviewPhase.AUTONOMOUS
            self.state._auto_step = 0
            return AUTONOMOUS_QUESTIONS["can_autonomous"]
        
        if answer in ("yes", "да", "y", "ещё", "yes please"):
            # "Yes, I have more forbidden actions" → ask for name
            return FORBIDDEN_QUESTIONS["name"]
        
        # New forbidden action name
        fa = ForbiddenDraft(name=_snake_case(answer))
        self.state.forbidden_actions.append(fa)
        self.state._forbidden_idx = len(self.state.forbidden_actions) - 1
        self.state.phase = InterviewPhase.FORBIDDEN_DETAIL
        
        return f"**{fa.name}** — forbidden. {FORBIDDEN_QUESTIONS['reason']}"

    def _handle_forbidden_detail(self, msg: str) -> str:
        """Collect reason + constitutional for one forbidden."""
        fa = self.state.forbidden_actions[self.state._forbidden_idx]
        
        if not fa.reason:
            fa.reason = msg.strip()
            return FORBIDDEN_QUESTIONS["constitutional"]
        
        # Reason set, now constitutional
        answer = msg.strip().lower()
        fa.constitutional = answer in ("yes", "да", "absolute", "конституционное", "true", "y")
        
        self.state.phase = InterviewPhase.FORBIDDEN
        return FORBIDDEN_QUESTIONS["more_forbidden"]

    def _handle_autonomous(self, msg: str) -> str:
        """Collect autonomous mandate details."""
        answer = msg.strip().lower()
        
        if not hasattr(self.state, '_auto_step'):
            self.state._auto_step = 0
        
        step = self.state._auto_step
        
        if step == 0:
            self.state.can_autonomous = answer in ("yes", "да", "y", "true", "может", "can")
            self.state._auto_step = 1
            if not self.state.can_autonomous:
                self.state.phase = InterviewPhase.EMERGENCY
                return EMERGENCY_QUESTIONS["risk_class"]
            return AUTONOMOUS_QUESTIONS["max_hours"]
        
        elif step == 1:
            try:
                self.state.autonomous_max_hours = float(re.findall(r'[\d.]+', msg)[0])
            except (ValueError, IndexError):
                self.state.autonomous_max_hours = 1.0
            self.state._auto_step = 2
            return AUTONOMOUS_QUESTIONS["actions"]
        
        elif step == 2:
            self.state.autonomous_actions = [
                _snake_case(a.strip()) 
                for a in re.split(r'[,;]\s*', msg) 
                if a.strip()
            ]
            self.state._auto_step = 3
            return AUTONOMOUS_QUESTIONS["on_anomaly"]
        
        elif step == 3:
            self.state.on_anomaly = _normalize_anomaly_policy(msg.strip())
            self.state._auto_step = 4
            return AUTONOMOUS_QUESTIONS["on_power_loss"]
        
        elif step == 4:
            self.state.on_power_loss = _normalize_power_loss(msg.strip())
            self.state.phase = InterviewPhase.EMERGENCY
            return EMERGENCY_QUESTIONS["risk_class"]

    def _handle_emergency(self, msg: str) -> str:
        """Collect risk class and emergency contacts."""
        if not hasattr(self.state, '_emerg_step'):
            self.state._emerg_step = 0
        
        step = self.state._emerg_step
        
        if step == 0:
            self.state.risk_class = _normalize_risk_class(msg.strip())
            self.state._emerg_step = 1
            return EMERGENCY_QUESTIONS["has_emergency_contact"]
        
        elif step == 1:
            answer = msg.strip().lower()
            if answer in ("no", "нет", "none", "skip"):
                self.state.phase = InterviewPhase.REVIEW
                return self._review_summary()
            self.state._emerg_step = 2
            return EMERGENCY_QUESTIONS["contact_name"]
        
        elif step == 2:
            self.state._current_contact = {"name": msg.strip()}
            self.state._emerg_step = 3
            return EMERGENCY_QUESTIONS["contact_method"]
        
        elif step == 3:
            method = msg.strip().lower()
            self.state._current_contact["method_type"] = "email" if "@" not in method and "email" in method else "phone" if "phone" in method or "тел" in method else "email"
            self.state._emerg_step = 4
            return EMERGENCY_QUESTIONS["contact_value"]
        
        elif step == 4:
            value = msg.strip()
            method_type = self.state._current_contact.get("method_type", "email")
            self.state._current_contact["method"] = {method_type: value}
            self.state._emerg_step = 5
            return EMERGENCY_QUESTIONS["contact_priority"]
        
        elif step == 5:
            try:
                priority = int(re.findall(r'\d', msg)[0])
            except (ValueError, IndexError):
                priority = 2
            self.state._current_contact["priority"] = priority
            self.state.emergency_contacts.append(self.state._current_contact)
            self.state._current_contact = {}
            self.state._emerg_step = 6
            return EMERGENCY_QUESTIONS["more_contacts"]
        
        elif step == 6:
            answer = msg.strip().lower()
            if answer in ("yes", "да", "y", "ещё"):
                self.state._emerg_step = 2
                return EMERGENCY_QUESTIONS["contact_name"]
            else:
                self.state.phase = InterviewPhase.REVIEW
                return self._review_summary()

    def _handle_review(self, msg: str) -> str:
        """Handle confirmation or restart request."""
        answer = msg.strip().lower()
        if answer in ("yes", "да", "ok", "хорошо", "подтверждаю", "confirm", "y"):
            self.state.phase = InterviewPhase.DONE
            return (
                "✅ Passport created successfully!\n\n"
                f"Device: **{self.state.device_name}**\n"
                f"Capabilities: {len(self.state.capabilities)}\n"
                f"Forbidden actions: {len(self.state.forbidden_actions)}\n"
                f"Risk class: {self.state.risk_class}\n\n"
                "The passport JSON is ready for registration."
            )
        else:
            return "What would you like to change? (Or type 'yes' to confirm)"

    def _handle_done(self, msg: str) -> str:
        return "This interview is complete. Use build_passport() to get the JSON."

    def _default_handle(self, msg: str) -> str:
        """Fallback for unhandled phases."""
        return f"Received: {msg}\n(Phase: {self.state.phase})"

    # ── Helpers ──

    def _review_summary(self) -> str:
        """Build review summary."""
        caps = "\n".join(f"  - {c.name}: {c.description[:60]}..." for c in self.state.capabilities)
        forbids = "\n".join(
            f"  - {f.name} (constitutional={f.constitutional}): {f.reason[:60]}..." 
            for f in self.state.forbidden_actions
        )
        contacts = "\n".join(
            f"  - [{c.get('priority', '?')}] {c.get('name', '?')}"
            for c in self.state.emergency_contacts
        )
        
        return f"""📋 **PASSPORT REVIEW**

**Device:** {self.state.device_name}
**Description:** {self.state.device_description}
**Platform:** {self.state.device_platform}
**Risk class:** {self.state.risk_class}

**Capabilities ({len(self.state.capabilities)}):**
{caps if caps else '  (none)'}

**Forbidden ({len(self.state.forbidden_actions)}):**
{forbids if forbids else '  (none)'}

**Autonomous:** {'Yes' if self.state.can_autonomous else 'No'}
{self.state.autonomous_max_hours}h max, {len(self.state.autonomous_actions)} actions allowed, 
anomaly={self.state.on_anomaly}, power_loss={self.state.on_power_loss}

**Emergency contacts:**
{contacts if contacts else '  (none)'}

---
Type **yes** to confirm, or tell me what to change."""

    def _parse_parameter(self, msg: str) -> Optional[dict]:
        """Try to parse a parameter from owner's message."""
        msg = msg.strip()
        
        # Simple heuristic: if it looks like a param name
        if len(msg) > 30 and ":" in msg:
            # "temperature: float, required, 0-100" format
            parts = [p.strip() for p in msg.split(",")]
            name = parts[0].split(":")[0].strip() if ":" in parts[0] else parts[0]
            param = {"name": _snake_case(name), "param_type": "float", "required": False}
            for part in parts[1:]:
                part = part.strip().lower()
                if part in ("float", "integer", "boolean", "string"):
                    param["param_type"] = part
                elif part in ("required", "обязательный"):
                    param["required"] = True
                elif "-" in part:
                    try:
                        min_s, max_s = part.split("-")
                        param.setdefault("constraints", {})["min"] = float(min_s)
                        param.setdefault("constraints", {})["max"] = float(max_s)
                    except ValueError:
                        pass
            return param
        
        # Assume it's just a name, ask for details later
        return {"name": _snake_case(msg), "param_type": "float", "required": False}

    def _llm_rephrase(self, question: str) -> str:
        """Use LLM to make the question more natural, if available."""
        if not self.llm:
            return question
        
        try:
            prompt = INTERVIEWER_SYSTEM_PROMPT.format(
                phase=self.state.phase,
                device_name=self.state.device_name or "(unknown)",
                capabilities=", ".join(c.name for c in self.state.capabilities) or "(none)",
                forbidden=", ".join(f.name for f in self.state.forbidden_actions) or "(none)",
            )
            
            messages = [
                {"role": "system", "content": prompt},
                {"role": "user", "content": f"Next structured question to ask: {question}\n\nRephrase this naturally in one sentence. Keep it conversational. If the owner's last answer was vague, add a gentle probe for specifics."},
            ]
            
            response = self.llm.provider.chat(messages)
            if response and response.text:
                return response.text.strip()
        except Exception:
            pass
        
        return question


# ── Passport Assembly ──

def _assemble_passport(state: InterviewState) -> dict:
    """Build a valid Passport JSON from interview state."""
    
    # Build capabilities
    capabilities = []
    for cap in state.capabilities:
        params = []
        for p in cap.parameters:
            param = {
                "name": p.get("name", "param"),
                "param_type": p.get("param_type", "float"),
                "required": p.get("required", False),
            }
            if "default" in p:
                param["default"] = p["default"]
            constraints = {}
            if "constraints" in p:
                c = p["constraints"]
                if "min" in c:
                    constraints["min"] = float(c["min"])
                if "max" in c:
                    constraints["max"] = float(c["max"])
                if "enum" in c:
                    constraints["enum"] = [str(e) for e in c["enum"]] if isinstance(c["enum"], list) else [str(c["enum"])]
            if constraints:
                param["constraints"] = constraints
            params.append(param)
        
        capabilities.append({
            "name": cap.name,
            "description": cap.description,
            "parameters": params,
            "risk": cap.risk,
        })
    
    # Build forbidden
    forbidden = []
    for fa in state.forbidden_actions:
        forbidden.append({
            "name": fa.name,
            "reason": fa.reason,
            "constitutional": fa.constitutional,
        })
    
    # Build emergency contacts
    contacts = []
    for ec in state.emergency_contacts:
        method = ec.get("method", {})
        contacts.append({
            "name": ec.get("name", "Operator"),
            "method": method,
            "priority": ec.get("priority", 2),
        })
    
    # Platform
    platform_map = {
        "esp32": "esp32", "arduino": "arduino", "raspberry_pi": "raspberry_pi",
        "jetson": "jetson", "android": "android", "linux": "linux",
        "ros2": "ros2", "browser": "browser",
    }
    platform = platform_map.get(state.device_platform.lower(), state.device_platform.lower())
    
    # Autonomous mandate
    autonomous = None
    if state.can_autonomous:
        autonomous = {
            "max_duration_seconds": int(state.autonomous_max_hours * 3600),
            "allowed_offline_actions": state.autonomous_actions,
            "on_anomaly": state.on_anomaly,
            "on_power_loss": state.on_power_loss,
        }
    
    return {
        "device_id": str(uuid4()),
        "name": state.device_name,
        "description": state.device_description,
        "capabilities": capabilities,
        "forbidden_always": forbidden,
        "risk_class": state.risk_class,
        "autonomous_mandate": autonomous,
        "emergency_contacts": contacts,
        "platform": platform,
        "version": {"major": 1, "minor": 0, "patch": 0},
        "signature": None,
    }


# ── Utility Functions ──

def _snake_case(text: str) -> str:
    """Convert text to snake_case."""
    text = text.strip().lower()
    text = re.sub(r'[^a-z0-9\s]', ' ', text)
    text = re.sub(r'\s+', '_', text)
    return text.strip('_')

def _normalize_platform(text: str) -> str:
    """Normalize platform name."""
    text = text.strip().lower()
    known = {
        "esp32": "esp32", "esp": "esp32",
        "arduino": "arduino",
        "raspberry": "raspberry_pi", "raspberry pi": "raspberry_pi", "rpi": "raspberry_pi",
        "jetson": "jetson", "nvidia jetson": "jetson", "orin": "jetson",
        "android": "android",
        "linux": "linux", "ubuntu": "linux", "debian": "linux",
        "ros": "ros2", "ros2": "ros2",
        "browser": "browser", "web": "browser",
        "windows": "windows",
        "mac": "linux",
    }
    for key, value in known.items():
        if key in text:
            return value
    return text

def _detect_complexity(description: str, platform: str) -> str:
    """Detect device complexity from description."""
    desc_lower = description.lower()
    
    # Complex indicators
    complex_words = [
        "laser", "surgery", "surgical", "autonomous", "microscope", "robot",
        "drone", "uav", "ventilator", "radiation", "x-ray", "ct scan",
        "ablation", "biopsy", "centrifuge", "autoclave", "crane",
    ]
    medium_words = [
        "motor", "heater", "pump", "valve", "printer", "cnc", "conveyor",
        "actuator", "cooler", "compressor", "mixer", "centrifuge",
    ]
    
    for word in complex_words:
        if word in desc_lower:
            return "complex"
    
    for word in medium_words:
        if word in desc_lower:
            return "medium"
    
    # Platform hint
    if platform in ("jetson", "ros2"):
        return "medium"
    
    return "simple"

def _normalize_risk_class(text: str) -> str:
    """Normalize risk class."""
    text = text.strip().lower()
    if "critical" in text or "крити" in text:
        return "critical"
    if "high" in text or "высок" in text or "опас" in text:
        return "high"
    if "medium" in text or "средн" in text:
        return "medium"
    if "low" in text or "низк" in text or "мал" in text:
        return "low"
    if "info" in text or "безопас" in text:
        return "informational"
    return "medium"

def _normalize_anomaly_policy(text: str) -> str:
    """Normalize anomaly policy."""
    text = text.strip().lower()
    if "pause" in text or "пауз" in text or "stop" in text or "останов" in text:
        return "pause_and_notify"
    if "shutdown" in text or "выклю" in text or "emergency" in text:
        return "safe_shutdown"
    if "continue" in text or "продол" in text or "limit" in text:
        return "continue_with_limits"
    if "llm" in text or "ai" in text or "invoke" in text or "вызвать" in text:
        return "invoke_llm"
    return "pause_and_notify"

def _normalize_power_loss(text: str) -> str:
    """Normalize power loss policy."""
    text = text.strip().lower()
    if "shutdown" in text or "выклю" in text or "off" in text:
        return "safe_shutdown"
    if "save" in text or "sleep" in text or "сон" in text or "сохран" in text:
        return "save_state_and_sleep"
    if "battery" in text or "батар" in text or "switch" in text:
        return "switch_to_battery"
    return "safe_shutdown"


# ── CLI Demo ──

def cli_demo(llm_bridge=None):
    """Run passport interview in terminal."""
    interview = PassportInterview(llm_bridge)
    
    print("=" * 60)
    print(interview.start())
    print("=" * 60)
    
    while not interview.is_done():
        try:
            user_input = input("\n> ").strip()
            if user_input.lower() in ("exit", "quit", "выход"):
                print("\nInterview cancelled.")
                break
            
            response = interview.respond(user_input)
            print(f"\n{response}")
            
        except KeyboardInterrupt:
            print("\n\nInterview cancelled.")
            break
        except EOFError:
            break
    
    if interview.is_done():
        passport = interview.build_passport()
        print("\n" + "=" * 60)
        print("PASSPORT JSON:")
        print(json.dumps(passport, indent=2, ensure_ascii=False))
    
    return interview


if __name__ == "__main__":
    cli_demo()
