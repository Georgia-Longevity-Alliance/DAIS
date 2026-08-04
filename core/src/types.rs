//! Shared types for DAIS protocol core.
//!
//! Every type in the DAIS ecosystem derives from these primitives.
//! Serialization via serde ensures JSON compatibility with
//! Python backend, Phoenix web, and on-wire protocols.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for every addressable entity in DAIS.
pub type Id = Uuid;

/// Semantic version following semver.org.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemVer {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Device platform identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Esp32,
    Arduino,
    RaspberryPi,
    Jetson,
    Android,
    Ios,
    Linux,
    Windows,
    Ros2,
    Browser,
    #[serde(untagged)]
    Custom(String),
}

/// Risk classification for a device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskClass {
    /// No physical effect possible (display, speaker).
    Informational,
    /// Minor physical effects, reversible (LED, motor <1W).
    Low,
    /// Significant physical effects (motor >1W, heater).
    Medium,
    /// Potential harm to humans or environment (laser, heavy machinery).
    High,
    /// Life-critical systems (ventilator, brake, weapon — forbidden).
    Critical,
}

/// A single capability declared in a device passport.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub risk: RiskClass,
}

/// A parameter for a capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParamType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub constraints: Option<Constraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Integer,
    Float,
    Boolean,
    String,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub regex: Option<String>,
}

/// An action that is ALWAYS forbidden — enforced in firmware.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForbiddenAction {
    pub name: String,
    pub reason: String,
    /// If true, not even emergency rescue agents can override.
    pub constitutional: bool,
}

// ── Safety Hardware (LEGO proposal: physical STOP) ──

/// Hardware-level safety configuration.
/// These are NOT software checks — they correspond to physical pins,
/// deadman switches, and hardware kill-switches on the device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetyHardware {
    /// GPIO pin for physical STOP button (None = no hardware stop).
    pub physical_stop_pin: Option<u8>,
    /// Deadman switch: if released for > this ms, emergency stop.
    pub deadman_switch_ms: Option<u32>,
    /// Hardware current limit (mA). 0 = no hardware limit.
    pub max_current_ma: Option<u32>,
    /// Hardware voltage limit (mV). 0 = no hardware limit.
    pub max_voltage_mv: Option<u32>,
    /// If true, device has a physical key-lock for parent/teacher.
    pub physical_key_lock: bool,
}

impl Default for SafetyHardware {
    fn default() -> Self {
        Self {
            physical_stop_pin: None,
            deadman_switch_ms: None,
            max_current_ma: None,
            max_voltage_mv: None,
            physical_key_lock: false,
        }
    }
}

// ── RBAC Roles (LEGO proposal: parent/teacher control) ──

/// Role-based access control for DAIS agents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AisRole {
    /// Full control: can modify passport, delegate, transfer ownership.
    Owner,
    /// Can operate device within mandate, cannot modify passport.
    Operator,
    /// Read-only: can view Flight Recorder, Traces, status.
    Observer,
    /// Can restrict capabilities, set time limits, approve commands.
    Parent,
    /// Limited access: read flight recorder, diagnose, safe restart only.
    EmergencyRescue,
}

// ── Multi-Body / Device Swarm (LEGO proposal: one intelligence, many bodies) ──

/// Record of a body change — when intelligence migrates to a new physical body.
/// The flight recorder and trace memory persist across body changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyChange {
    pub change_id: Id,
    pub previous_body_id: Id,
    pub new_body_id: Id,
    pub timestamp: DateTime<Utc>,
    /// Why the body changed (child rebuilt, module moved, upgrade).
    pub reason: String,
    /// Capabilities gained in the new body.
    pub gained_capabilities: Vec<String>,
    /// Capabilities lost from the old body.
    pub lost_capabilities: Vec<String>,
}

// ── Challenge Framework (LEGO proposal: One Brain, Many Bodies challenges) ──

/// A standardised test scenario for AIS-compatible devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub challenge_id: Id,
    pub name: String,
    pub description: String,
    /// What this challenge tests.
    pub category: ChallengeCategory,
    /// Steps to execute.
    pub scenario: Vec<ChallengeStep>,
    /// Pass criteria.
    pub pass_criteria: Vec<PassCriterion>,
    /// Estimated energy cost to run (joules).
    pub energy_cost_j: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeCategory {
    Safety,
    Autonomy,
    Diagnosis,
    TraceReuse,
    EnergyEfficiency,
    Collaboration,
    Adaptation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeStep {
    pub order: u32,
    pub action: String,
    pub expected: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassCriterion {
    pub name: String,
    pub check: String,
    pub weight: f64, // 0.0–1.0, sum of weights = score denominator
}

/// Result of running a challenge against a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub result_id: Id,
    pub challenge_id: Id,
    pub device_id: Id,
    pub timestamp: DateTime<Utc>,
    pub passed: bool,
    pub score: f64, // 0.0–100.0
    pub step_results: Vec<StepResult>,
    pub traces_generated: Vec<Id>,
    pub total_energy_j: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_order: u32,
    pub passed: bool,
    pub actual: String,
    pub duration_ms: u64,
    pub notes: Option<String>,
}

// ── Unlockables / Prizes (LEGO proposal: prizes that create the next invention) ──

/// Something unlocked by completing challenges or resolving anomalies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unlockable {
    pub unlock_id: Id,
    pub name: String,
    pub description: String,
    /// What must be achieved to unlock.
    pub requirements: Vec<UnlockRequirement>,
    /// What becomes available.
    pub reward: UnlockReward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockRequirement {
    ChallengesCompleted { count: usize },
    ChallengesPassed { count: usize },
    AnomaliesResolved { count: usize },
    ScoreThreshold { score: f64 },
    SpecificChallenge { challenge_id: Id },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnlockReward {
    AdvancedDiagnosticMode,
    ExtendedAutonomousMandate { max_hours: u64 },
    NewCapability { capability: String },
    AccessToDevice { device_id: Id },
    KnowledgePack { publication_id: Id },
}

// ── Improvement Loop (LEGO: challenge→construct→test→result→new_parts) ──

/// Tracks a full improvement cycle: challenge → solution → learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementLoop {
    pub loop_id: Id,
    pub device_id: Id,
    pub started: DateTime<Utc>,
    pub completed: Option<DateTime<Utc>>,
    pub trigger: String, // what started this loop (anomaly, challenge, curiosity)
    pub diagnosis: Option<String>,
    pub solution_applied: Option<String>,
    pub result: Option<LoopResult>,
    pub traces_linked: Vec<Id>,
    pub unlocked: Vec<Id>, // Unlockable IDs earned
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoopResult {
    Improved { metric: String, before: f64, after: f64 },
    Solved { description: String },
    Learned { insight: String },
    Failed { reason: String },
}

/// Autonomous mandate: what the device may do when offline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousMandate {
    pub max_duration_seconds: u64,
    pub allowed_offline_actions: Vec<String>,
    pub on_anomaly: AnomalyPolicy,
    pub on_power_loss: PowerLossPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyPolicy {
    PauseAndNotify,
    SafeShutdown,
    ContinueWithLimits,
    InvokeLLM,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerLossPolicy {
    SafeShutdown,
    SaveStateAndSleep,
    SwitchToBattery,
}

/// Emergency contact for rescue agents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmergencyContact {
    pub name: String,
    pub method: ContactMethod,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContactMethod {
    Email(String),
    Phone(String),
    Webhook(String),
    Mqtt(String),
}

// ── Flight Recorder types ──

/// A single event in the flight recorder ring buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightEvent {
    pub event_id: Id,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub severity: Severity,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Boot,
    Heartbeat,
    CommandReceived,
    CommandExecuted,
    CommandRejected,
    Anomaly,
    StateChange,
    SensorReading,
    LLMInvoked,
    LLMResponse,
    Shutdown,
    Error,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

// ── Trace Network types ──

/// An intervention trace — what an LLM did and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTrace {
    pub trace_id: Id,
    pub device_id: Id,
    pub agent_id: Id,
    pub trigger_event_id: Id,
    pub diagnosis: String,
    pub confidence: f64,
    pub actions_taken: Vec<TraceAction>,
    pub outcome: TraceOutcome,
    pub timestamp: DateTime<Utc>,
    pub references: Vec<Id>,
    /// If this trace is a remix/improvement of a prior trace (LEGO: rebuild, change one idea).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remix_of: Option<Id>,
    /// Whether this trace is published to the community library.
    #[serde(default)]
    pub published: bool,
    /// Tags for community discovery.
    #[serde(default)]
    pub community_tags: Vec<String>,
    /// Skill level: beginner, intermediate, advanced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAction {
    pub capability: String,
    pub parameters: serde_json::Value,
    pub result: ActionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    Success,
    PartialSuccess { reason: String },
    Failed { reason: String },
    Blocked { layer: String, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TraceOutcome {
    Resolved,
    Mitigated,
    Escalated,
    Unresolved,
}

// ── Proven types ──

/// Status of a knowledge claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ClaimStatus {
    Proposed,
    Supported,
    Tested,
    Replicated,
    Contested,
    Refuted,
    Superseded,
    Stale,
    Open,
}

/// Delta operation for knowledge revision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DeltaOperation {
    Create,
    Update,
    Relate,
    Deprecate,
    Consolidate,
}

/// A proposed change to the knowledge field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub delta_id: Id,
    pub publication_id: Id,
    pub author_id: Id,
    pub session_id: Id,
    pub operation: DeltaOperation,
    pub target_id: Option<Id>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

/// A knowledge claim — the fundamental unit of Proven.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub claim_id: Id,
    pub publication_id: Id,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub status: ClaimStatus,
    pub sources: Vec<Id>,
    pub evidence: Vec<Id>,
    pub authors: Vec<Id>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

/// A source for a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub source_id: Id,
    pub title: String,
    pub source_type: SourceType,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    JournalArticle,
    Preprint,
    Dataset,
    Experiment,
    Observation,
    PersonalCommunication,
    Software,
    Other(String),
}

/// Evidence supporting or refuting a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_id: Id,
    pub claim_id: Id,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub strength: f64, // 0.0–1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceType {
    Supporting,
    Contradicting,
    Inconclusive,
}

/// Agent identity — human, LLM, or program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub agent_id: Id,
    pub name: String,
    pub agent_type: AgentType,
    /// RBAC role for access control.
    #[serde(default = "default_role")]
    pub role: AisRole,
    pub affiliation: Option<String>,
    pub public_key: Option<String>,
}

fn default_role() -> AisRole {
    AisRole::Operator
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Human,
    LLM { model: String },
    Program { version: String },
    Team,
    Institution,
}

// ── Event Store types ──

/// An entry in the append-only event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub event_id: Id,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Id,
    pub operation: DeltaOperation,
    pub payload: serde_json::Value,
    pub signature: Option<String>,
}
