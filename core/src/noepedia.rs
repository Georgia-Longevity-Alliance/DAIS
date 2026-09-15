//! Noepedia grammar — explicit, addressable, inspectable knowledge field.
//!
//! Noepedia stores knowledge OUTSIDE any model's weights: objects participate
//! in many independent rule-governed evaluative networks. Every network carries
//! a RULE_CARD; every placement is inspectable; coverage is recorded separately
//! from confidence; replication is an operation, not decoration; OPEN and
//! CONFLICT are legal knowledge states.
//!
//! This module implements the Noepedia minimal working objects so that DAIS can
//! consolidate flight-recorder traces and experiment outcomes into reusable
//! knowledge with full provenance (see `delta.rs`, `event_store.rs`, `consolidator.rs`).
//!
//! Mapping to the Noepedia spec:
//! ```text
//! ENTITY/CLAIM/RELATION/NETWORK  -> Network, Relation, objects
//! RULE_CARD                      -> RuleCard (+ RuleRevision history)
//! PLACEMENT / PLACEMENT_ARGUMENT -> Placement, PlacementArgument
//! COVERAGE                       -> Coverage
//! EVIDENCE / COUNTEREVIDENCE     -> types::Evidence (+ RelationKind::Contradicts)
//! OBSERVATION / TEST / RESULT    -> Observation, Test, ResultStatus
//! REPLICATION                    -> Replication
//! CONTEXT (VALID-IN-CONTEXT)     -> KnowledgeContext
//! OPEN / CONFLICT                -> KnowledgeStatus::Open, consolidator::ConflictRecord
//! REVISION                       -> RuleRevision, types::DeltaOperation::Update
//! SESSION / OUTCOME              -> Session, Outcome
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for every addressable Noepedia object.
pub type NoepediaId = Uuid;

/// Knowledge state of an object. OPEN and CONFLICT are first-class.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum KnowledgeStatus {
    /// Claimed, not yet supported.
    Proposed,
    /// Supported by at least one piece of evidence.
    Supported,
    /// Independently reproduced one or more times (see `Replication`).
    Replicated,
    /// Countered by evidence or test.
    Refuted,
    /// Supported positions are incompatible; both sides preserved.
    Contested,
    /// Insufficient evidence to decide.
    Open,
    /// True only under specific conditions (hardware revision, context...).
    ValidInContext,
    /// Replaced by a newer formulation.
    Superseded,
}

/// Conditions under which knowledge applies (VALID-IN-CONTEXT).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeContext {
    pub context_id: NoepediaId,
    /// e.g. "hardware revision B", "C. elegans at 25C", "protocol v2".
    pub label: String,
    pub conditions: Vec<String>,
    /// Free-form bounds: revision, tissue, temperature, protocol version.
    pub validity_bounds: serde_json::Value,
}

/// Result of an observation, test, or replication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResultStatus {
    Supports,
    Contradicts,
    Inconclusive,
}

/// A recorded observation — distinct from inference and from knowledge.
/// Observations are the raw material that later becomes claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub observation_id: NoepediaId,
    pub agent_id: NoepediaId,
    pub session_id: NoepediaId,
    pub description: String,
    pub method: String,
    pub conditions: KnowledgeContext,
    pub recorded_at: DateTime<Utc>,
}

/// A verification procedure and its result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    pub test_id: NoepediaId,
    pub claim_id: NoepediaId,
    pub procedure: String,
    pub conditions: KnowledgeContext,
    pub result: ResultStatus,
    pub executed_at: DateTime<Utc>,
}

/// An independent repetition of a result. Replication is an operation,
/// not decoration: thresholds depend on domain and consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replication {
    pub replication_id: NoepediaId,
    pub claim_id: NoepediaId,
    /// who/what repeated it (human, another lab, another ARGUS device...).
    pub agent_id: NoepediaId,
    pub method: String,
    pub conditions: KnowledgeContext,
    pub outcome: ResultStatus,
    pub replicated_at: DateTime<Utc>,
}

/// A revision of a network rule. Changing a rule is allowed;
/// changing it silently is not — every change is an inspectable revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRevision {
    pub old_rule: String,
    pub problem: String,
    pub argument: String,
    pub new_rule: String,
    pub revised_at: DateTime<Utc>,
}

/// A network's rule: what it evaluates and how objects are placed/ranked.
/// The navigator reads the rule card; it does not hold the rule in weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCard {
    pub rule_id: NoepediaId,
    pub network_id: NoepediaId,
    pub title: String,
    /// What the network evaluates.
    pub evaluates: String,
    /// How objects are ranked within it.
    pub ranking_criterion: String,
    pub allowed_relations: Vec<String>,
    pub revision_history: Vec<RuleRevision>,
    pub version: u32,
}

/// A typed relation between two objects. The governing rule is explicit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub relation_id: NoepediaId,
    pub source: NoepediaId,
    pub target: NoepediaId,
    pub kind: RelationKind,
    pub rule_id: NoepediaId,
}

/// Relation kinds from the Noepedia semiotic networks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    IsPartOf,
    IsInstanceOf,
    DiffersFrom,
    Supports,
    Contradicts,
    FunctionsAs,
    ValidInContext,
    BelongsToPrototype,
    RankedByReliability,
    SimilarTo,
}

/// A rule-governed evaluative projection:
/// `Network = objects + relations + rule card + history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub network_id: NoepediaId,
    pub name: String,
    pub description: String,
    pub rule_card: RuleCard,
    pub objects: Vec<NoepediaId>,
    pub relations: Vec<Relation>,
    pub history: Vec<NoepediaId>,
    pub created: DateTime<Utc>,
}

/// An object's position inside a network, with inspectable support structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub placement_id: NoepediaId,
    pub network_id: NoepediaId,
    pub object_id: NoepediaId,
    pub prototype: Option<NoepediaId>,
    pub supporting_features: Vec<String>,
    pub important_differences: Vec<String>,
    pub alternatives_considered: Vec<NoepediaId>,
    pub missing_tests: Vec<NoepediaId>,
    pub arguments: Vec<PlacementArgument>,
}

/// Why a placement is proposed or accepted — the meaning-edit protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementArgument {
    pub argument_id: NoepediaId,
    pub placement_id: NoepediaId,
    pub network_id: NoepediaId,
    /// Evidence supporting the placement.
    pub supporting: Vec<NoepediaId>,
    /// Evidence contradicting it.
    pub counterevidence: Vec<NoepediaId>,
    pub alternatives_considered: Vec<String>,
    pub accepted: bool,
    pub argued_by: NoepediaId,
    pub argued_at: DateTime<Utc>,
}

/// Coverage — distinguishes "evaluated networks agree" from
/// "the relevant networks have been sufficiently surveyed".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub coverage_id: NoepediaId,
    pub conclusion_id: NoepediaId,
    pub relevant_networks: u32,
    pub evaluated: u32,
    pub not_yet_evaluated: u32,
    pub supporting: u32,
    pub neutral_unrelated: u32,
    pub conflicting: u32,
    pub status: KnowledgeStatus,
}

/// A bounded episode of work (agent + objective + traces + outcome).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: NoepediaId,
    pub agent_id: NoepediaId,
    pub started: DateTime<Utc>,
    pub ended: Option<DateTime<Utc>>,
    pub objective: String,
    pub traces: Vec<NoepediaId>,
    pub outcome: Option<Outcome>,
}

/// Real-world feedback from a session or experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub outcome_id: NoepediaId,
    pub session_id: NoepediaId,
    pub kind: OutcomeKind,
    pub description: String,
    pub evidence_id: Option<NoepediaId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeKind {
    Resolved,
    Refuted,
    NewOpen,
    ReusableKnowledge,
    EscalatedToHuman,
    EscalatedToLLM,
}

/// Convenience constructors for the most common Noepedia objects.
impl Network {
    /// Create a new network with a fresh rule card (v1).
    pub fn new(name: &str, description: &str, evaluates: &str, ranking: &str) -> Self {
        let network_id = Uuid::new_v4();
        let rule_card = RuleCard {
            rule_id: Uuid::new_v4(),
            network_id,
            title: format!("{}: rule", name),
            evaluates: evaluates.to_string(),
            ranking_criterion: ranking.to_string(),
            allowed_relations: Vec::new(),
            revision_history: Vec::new(),
            version: 1,
        };
        Self {
            network_id,
            name: name.to_string(),
            description: description.to_string(),
            rule_card,
            objects: Vec::new(),
            relations: Vec::new(),
            history: Vec::new(),
            created: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_with_rule_card() {
        let net = Network::new(
            "functional_similarity",
            "rank objects by similarity of performed function, ignoring manufacturer",
            "performed function",
            "functional similarity ignoring manufacturer and origin",
        );
        assert_eq!(net.rule_card.network_id, net.network_id);
        assert_eq!(net.rule_card.version, 1);
        assert!(net.objects.is_empty());
    }

    #[test]
    fn test_rule_revision_is_inspectable() {
        let mut net = Network::new("n", "d", "e", "r");
        net.rule_card.revision_history.push(RuleRevision {
            old_rule: "rank by similarity".into(),
            problem: "manufacturer bias".into(),
            argument: "independence of origin is required".into(),
            new_rule: "rank by function only".into(),
            revised_at: Utc::now(),
        });
        assert_eq!(net.rule_card.revision_history.len(), 1);
        assert_eq!(net.rule_card.version, 1);
    }

    #[test]
    fn test_coverage_math() {
        let c = Coverage {
            coverage_id: Uuid::new_v4(),
            conclusion_id: Uuid::new_v4(),
            relevant_networks: 25,
            evaluated: 18,
            not_yet_evaluated: 7,
            supporting: 11,
            neutral_unrelated: 5,
            conflicting: 2,
            status: KnowledgeStatus::Open,
        };
        assert_eq!(c.relevant_networks, c.evaluated + c.not_yet_evaluated);
    }

    #[test]
    fn test_replication_tracks_independence() {
        let r = Replication {
            replication_id: Uuid::new_v4(),
            claim_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            method: "SAS-4::GFP tracking on ARGUS-OS2".into(),
            conditions: KnowledgeContext {
                context_id: Uuid::new_v4(),
                label: "hardware revision B".into(),
                conditions: vec!["60x/1.2 WI".into()],
                validity_bounds: serde_json::json!({}),
            },
            outcome: ResultStatus::Supports,
            replicated_at: Utc::now(),
        };
        assert_eq!(r.outcome, ResultStatus::Supports);
        assert_eq!(r.conditions.label, "hardware revision B");
    }
}
