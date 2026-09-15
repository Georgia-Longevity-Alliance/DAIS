//! Zone of Proximal Development (ZPD) — Vygotsky's scaffolding applied to bounded autonomy.
//!
//! Vygotsky (1896–1934): higher functions form from the outside, through interaction.
//! Learning happens in the **zone of proximal development** — the distance between
//! what a learner can do independently and what it can do with a more competent other.
//! Every ability passes three stages:
//!   1. **interpsychic** — the other tells/helps;
//!   2. **extragsychic** — the learner guides itself aloud;
//!   3. **intrapsychic** — the skill is internalized and automatic.
//!
//! Applied to DAIS/AISocket/Noepedia this formalizes a principle AISocket already states
//! ("known action should not repeatedly consume general intelligence"):
//! - The **more competent other** is the LLM; it scaffolds *just at the boundary*,
//!   never doing what the body already knows.
//! - Help is **faded** across repetitions (geometric decay): interpsychic → intrapsychic.
//! - When the scaffolded result is consolidated into Noepedia as a REPLICATED claim,
//!   the task moves from **proximal** to **actual development** and becomes Green
//!   (deterministic, ~pJ). This is the energy-honest asymptote `Cost(anomaly) → O(1)`.
//!
//! Three zones (after Vygotsky):
//! - **Actual development** — can do independently (green, cached, deterministic).
//! - **Proximal development** — can do with scaffolding (LLM at the boundary).
//! - **Inaccessible** — cannot do even with help (human / OPEN).
//!
//! See also: `noepedia.rs` (consolidation into reusable knowledge), `trace.rs`
//! (traces as evidence), `consolidator.rs` (OPEN/CONFLICT are legal states).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::noepedia::OutcomeKind;

/// The three zones, after Vygotsky.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ZpdZone {
    /// Can be done independently — deterministic, green, cached.
    ActualDevelopment,
    /// Can be done only with scaffolding from a more competent other (LLM).
    ProximalDevelopment,
    /// Cannot be done even with help — escalate to a human or leave OPEN.
    Inaccessible,
}

/// How much scaffolding was required (ordinal: more = more help).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldingLevel {
    /// Just observe — the body already knows.
    ObserveOnly,
    /// A hint / the relevant rule card.
    Hint,
    /// Guided diagnostic steps.
    GuidedSteps,
    /// Full help: LLM diagnoses and acts.
    FullHelp,
    /// Human required.
    HumanRequired,
}

/// Energy class from the DAIS taxonomy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnergyClass {
    /// <1 µJ — deterministic (SQL, Rust, graph traversal).
    #[default]
    Green,
    /// <1 mJ — small local model (classifier, anomaly detector).
    Yellow,
    /// ~1 J/token — LLM inference.
    Red,
}

/// Input describing a task, used to classify its zone.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskProfile {
    /// Is there a cached resolved trace (green)?
    pub has_cached_trace: bool,
    /// Is there a consolidated claim (Replicated)?
    pub has_consolidated_claim: bool,
    /// How many independent replications back it?
    pub replication_count: u32,
    /// Is the situation novel (no trace, no claim)?
    pub novel: bool,
    /// Does it hit a declared CONFLICT?
    pub in_conflict: bool,
    /// Is the relevant knowledge coverage sufficient?
    pub coverage_sufficient: bool,
    /// Is the action safety-critical (always human/firmware path)?
    pub safety_critical: bool,
    /// Energy class already known from the DAIS taxonomy.
    pub energy_class: EnergyClass,
}

/// What the system should do, given the zone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldAction {
    /// Green: actual development — run deterministically.
    ExecuteLocally,
    /// Green: cached trace lookup, no LLM.
    LookupTrace,
    /// Yellow: small model pre-filter.
    InvokeSmallModel,
    /// Red: LLM scaffolds at the ZPD boundary.
    InvokeLLM { with_context: bool },
    /// Inaccessible / safety-critical — human in the loop.
    EscalateToHuman,
    /// No help sufficient — register OPEN / request missing test.
    MarkOpen,
}

/// The result of classifying a task into a zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldPlan {
    pub task_id: Uuid,
    pub zone: ZpdZone,
    pub scaffolding: ScaffoldingLevel,
    pub action: ScaffoldAction,
    pub rationale: Vec<String>,
}

/// A bounded scaffolded episode: how much help was given and how it faded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldSession {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub device_id: Uuid,
    pub zone: ZpdZone,
    pub level: ScaffoldingLevel,
    pub attempts: u32,
    /// 1.0 = full help, 0.0 = internalized (Vygotsky: interpsychic → intrapsychic).
    pub fading: f32,
    pub started: DateTime<Utc>,
    pub completed: Option<DateTime<Utc>>,
    pub outcome: Option<OutcomeKind>,
}

impl ScaffoldSession {
    /// After a successful scaffolded resolution, fade the help geometrically:
    /// 1.0 → 0.5 → 0.25 → … When fading reaches ~0 the task is internalized and
    /// becomes part of *actual development* (green, no further LLM needed).
    pub fn fade(&mut self) -> f32 {
        self.attempts += 1;
        self.fading = (self.fading / 2.0).max(0.0);
        self.fading
    }

    /// True when the skill has been internalized (ZPD closed, moved to actual development).
    pub fn internalized(&self) -> bool {
        self.fading <= 0.05
    }

    /// Mark the session complete with an outcome.
    pub fn finish(&mut self, outcome: OutcomeKind) {
        self.completed = Some(Utc::now());
        self.outcome = Some(outcome);
    }
}

/// The ZPD engine — decides *when* to scaffold and *how much*.
pub struct Scaffolder;

impl Scaffolder {
    /// Classify a task into its zone and pick the scaffolding level.
    ///
    /// Order of checks (from cheapest to most expensive):
    /// 1. Safety-critical → firmware/human path, never LLM.
    /// 2. Actual development → consolidated REPLICATED claim + coverage → run locally.
    /// 3. Cached trace → lookup (green), no LLM.
    /// 4. Conflict → preserve both sides, request a discriminating measurement.
    /// 5. Novel → LLM scaffolds at the ZPD boundary (red).
    /// 6. Insufficient coverage → register OPEN, request the missing network/test.
    pub fn classify(profile: &TaskProfile) -> ScaffoldPlan {
        let task_id = Uuid::new_v4();

        if profile.safety_critical {
            return ScaffoldPlan {
                task_id,
                zone: ZpdZone::Inaccessible,
                scaffolding: ScaffoldingLevel::HumanRequired,
                action: ScaffoldAction::EscalateToHuman,
                rationale: vec![
                    "safety-critical: firmware/human path, prompt is not a safety mechanism"
                        .into(),
                ],
            };
        }

        if profile.has_cached_trace
            && profile.has_consolidated_claim
            && profile.replication_count >= 1
            && profile.coverage_sufficient
        {
            return ScaffoldPlan {
                task_id,
                zone: ZpdZone::ActualDevelopment,
                scaffolding: ScaffoldingLevel::ObserveOnly,
                action: ScaffoldAction::ExecuteLocally,
                rationale: vec![
                    "consolidated claim REPLICATED + coverage sufficient → actual development"
                        .into(),
                    "energy: Green (<1 µJ)".into(),
                ],
            };
        }

        if profile.has_cached_trace && !profile.novel {
            return ScaffoldPlan {
                task_id,
                zone: ZpdZone::ActualDevelopment,
                scaffolding: ScaffoldingLevel::ObserveOnly,
                action: ScaffoldAction::LookupTrace,
                rationale: vec!["cached resolved trace; no LLM needed (Green)".into()],
            };
        }

        if profile.in_conflict {
            return ScaffoldPlan {
                task_id,
                zone: ZpdZone::ProximalDevelopment,
                scaffolding: ScaffoldingLevel::GuidedSteps,
                action: ScaffoldAction::InvokeSmallModel,
                rationale: vec!["declared CONFLICT: preserve both positions, pick a test".into()],
            };
        }

        if profile.novel {
            let level = if profile.energy_class == EnergyClass::Yellow {
                ScaffoldingLevel::Hint
            } else {
                ScaffoldingLevel::FullHelp
            };
            return ScaffoldPlan {
                task_id,
                zone: ZpdZone::ProximalDevelopment,
                scaffolding: level,
                action: ScaffoldAction::InvokeLLM { with_context: true },
                rationale: vec!["novel: LLM scaffolds just at the ZPD boundary".into()],
            };
        }

        ScaffoldPlan {
            task_id,
            zone: ZpdZone::ProximalDevelopment,
            scaffolding: ScaffoldingLevel::Hint,
            action: ScaffoldAction::MarkOpen,
            rationale: vec!["insufficient coverage: register OPEN, request missing network/test".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> TaskProfile {
        TaskProfile {
            has_cached_trace: false,
            has_consolidated_claim: false,
            replication_count: 0,
            novel: false,
            in_conflict: false,
            coverage_sufficient: false,
            safety_critical: false,
            energy_class: EnergyClass::Red,
        }
    }

    #[test]
    fn safety_critical_never_llm() {
        let mut p = profile();
        p.safety_critical = true;
        let plan = Scaffolder::classify(&p);
        assert_eq!(plan.zone, ZpdZone::Inaccessible);
        assert_eq!(plan.action, ScaffoldAction::EscalateToHuman);
    }

    #[test]
    fn consolidated_replicated_runs_locally() {
        let mut p = profile();
        p.has_cached_trace = true;
        p.has_consolidated_claim = true;
        p.replication_count = 2;
        p.coverage_sufficient = true;
        let plan = Scaffolder::classify(&p);
        assert_eq!(plan.zone, ZpdZone::ActualDevelopment);
        assert_eq!(plan.action, ScaffoldAction::ExecuteLocally);
    }

    #[test]
    fn cached_trace_is_green_lookup() {
        let mut p = profile();
        p.has_cached_trace = true;
        let plan = Scaffolder::classify(&p);
        assert_eq!(plan.zone, ZpdZone::ActualDevelopment);
        assert_eq!(plan.action, ScaffoldAction::LookupTrace);
    }

    #[test]
    fn novel_scaffolded_by_llm() {
        let mut p = profile();
        p.novel = true;
        p.energy_class = EnergyClass::Red;
        let plan = Scaffolder::classify(&p);
        assert_eq!(plan.zone, ZpdZone::ProximalDevelopment);
        assert_eq!(plan.action, ScaffoldAction::InvokeLLM { with_context: true });
    }

    #[test]
    fn conflict_preserves_both_sides() {
        let mut p = profile();
        p.in_conflict = true;
        let plan = Scaffolder::classify(&p);
        assert_eq!(plan.zone, ZpdZone::ProximalDevelopment);
        assert_eq!(plan.action, ScaffoldAction::InvokeSmallModel);
    }

    #[test]
    fn fading_closes_the_zpd() {
        let mut s = ScaffoldSession {
            session_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            zone: ZpdZone::ProximalDevelopment,
            level: ScaffoldingLevel::FullHelp,
            attempts: 0,
            fading: 1.0,
            started: Utc::now(),
            completed: None,
            outcome: None,
        };
        assert!(!s.internalized());
        s.fade(); // 0.5
        s.fade(); // 0.25
        s.fade(); // 0.125
        assert!(!s.internalized());
        s.fade(); // 0.0625
        s.fade(); // 0.03125
        assert!(s.internalized());
        assert_eq!(s.attempts, 5);
    }

    #[test]
    fn insufficient_coverage_is_open() {
        let p = profile(); // nothing known
        let plan = Scaffolder::classify(&p);
        assert_eq!(plan.action, ScaffoldAction::MarkOpen);
    }
}
