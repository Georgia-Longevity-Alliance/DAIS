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
///
/// This is the canonical K-ladder (Noepedia harvest, 2026-10-12): a verifiable
/// ladder of evidence, not a float confidence. `types::ClaimStatus` is an alias
/// of this enum so Proven and the Noepedia field share ONE ladder.
/// Map: GRADE (PMID 18436948), Murad pyramid (PMID 27339128), CEBM, NASA TRL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum KnowledgeStatus {
    /// Claimed, not yet supported.
    Proposed,
    /// Supported by at least one piece of evidence.
    Supported,
    /// Tested by an explicit procedure (build the scene, not precedent).
    Tested,
    /// Independently reproduced one or more times (see `Replication`).
    Replicated,
    /// Countered by evidence or test.
    Refuted,
    /// Supported positions are incompatible; both sides preserved.
    Contested,
    /// Insufficient evidence to decide. OPEN ≠ denial: it is an addressable
    /// unknown with constraints + validation rule (see `OpenSpec`).
    Open,
    /// True only under specific conditions (hardware revision, context...).
    ValidInContext,
    /// Replaced by a newer formulation.
    Superseded,
    /// No longer actively maintained (superseded by time or neglect).
    Stale,
}

impl TryFrom<&str> for KnowledgeStatus {
    type Error = String;

    /// Parse a serialized status (UPPERCASE, e.g. "REPLICATED").
    /// Accepts both "VALID-IN-CONTEXT" and "VALIDINCONTEXT".
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "PROPOSED" => Self::Proposed,
            "SUPPORTED" => Self::Supported,
            "TESTED" => Self::Tested,
            "REPLICATED" => Self::Replicated,
            "REFUTED" => Self::Refuted,
            "CONTESTED" => Self::Contested,
            "OPEN" => Self::Open,
            "VALIDINCONTEXT" | "VALID-IN-CONTEXT" => Self::ValidInContext,
            "SUPERSEDED" => Self::Superseded,
            "STALE" => Self::Stale,
            other => return Err(format!("unknown KnowledgeStatus: {other}")),
        })
    }
}

/// History-status of a committed record (Demon Core Harvest, DCT11–DCT12).
///
/// The committed fact is stable: it may be compacted, archived or superseded
/// but never made historically unaddressable or provenance-untraceable.
/// This is the `HISTORY_STATUS` dimension, orthogonal to `CURRENT_EPISTEMIC_STATUS`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HistoryStatus {
    /// The transition was committed and remains the live record.
    Committed,
    /// Archived — still addressable, no longer in the hot path.
    Archived,
    /// Compacted — merged into a summary entry, addresses preserved.
    Compacted,
    /// Superseded — replaced by a newer record; the past is not erased.
    Superseded,
}

/// One addressable status assertion (non-destructive MARK, DCT11–DCT12).
///
/// MARK creates a NEW assertion and keeps the prior assertions as historical
/// trace; a status transition is never an overwrite. The projection exposes
/// `current_epistemic_status` = the last applicable assertion, while
/// `history_status` tracks the committed fact independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusAssertion {
    pub assertion_id: NoepediaId,
    pub record_id: NoepediaId,
    pub status: KnowledgeStatus,
    /// Scope of applicability (SCOPE/TIME qualifiers, DCT13–DCT14).
    pub scope: Vec<String>,
    /// Monotonic epoch: each new assertion increments it; the previous one
    /// stays in the assertion log.
    pub epoch: u64,
    pub asserted_by: NoepediaId,
    pub asserted_at: DateTime<Utc>,
    pub history_status: HistoryStatus,
}

impl StatusAssertion {
    /// Create the first assertion for a record (epoch 1).
    pub fn initial(record_id: NoepediaId, status: KnowledgeStatus, asserted_by: NoepediaId) -> Self {
        Self {
            assertion_id: Uuid::new_v4(),
            record_id,
            status,
            scope: Vec::new(),
            epoch: 1,
            asserted_by,
            asserted_at: Utc::now(),
            history_status: HistoryStatus::Committed,
        }
    }
}

/// Four states of the demon's working regime (DEMON_CORE_SPEC §7).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RegimeState {
    /// Enough structure to continue lawfully.
    Go,
    /// An external fact is missing, but a precise permitted query is possible.
    Ask,
    /// An acting rule/closure forbids this path.
    Block,
    /// The network's own working structure is missing; a correct query cannot
    /// yet be formed.
    Need,
}

/// Classification of what is missing when the demon cannot proceed
/// (DEMON_CORE_SPEC §8; open taxonomy — not a closed ontology).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum NeedClass {
    MissingFact,
    MissingRule,
    MissingAddress,
    MissingRelation,
    MissingPattern,
    MissingScope,
    MissingProvenance,
    MissingOperation,
    AmbiguousMatch,
    Conflict,
    ClosureUnknown,
    CreditUnresolved,
}

/// A STOP CARD — a stuck branch returns this instead of fabricating a
/// plausible continuation (DEMON_CORE_SPEC §8). Stopping at the right place
/// is a successful behavior, not a failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopCard {
    pub stop_id: NoepediaId,
    /// Where the run got stuck (session/scene/step).
    pub where_: String,
    /// What the demon wanted to achieve.
    pub wanted: String,
    /// What was available at the stop point.
    pub available: String,
    /// What was missing.
    pub missing: String,
    pub stop_class: NeedClass,
    /// Whether a precise, permitted external query exists (ASK vs NEED).
    pub can_ask_outside: bool,
    /// The precise permitted query, if any.
    pub query: Option<String>,
    pub provenance: Vec<NoepediaId>,
    pub scope: Vec<String>,
}

impl StopCard {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        where_: &str,
        wanted: &str,
        available: &str,
        missing: &str,
        stop_class: NeedClass,
        can_ask_outside: bool,
        query: Option<String>,
        provenance: Vec<NoepediaId>,
        scope: Vec<String>,
    ) -> Self {
        Self {
            stop_id: Uuid::new_v4(),
            where_: where_.into(),
            wanted: wanted.into(),
            available: available.into(),
            missing: missing.into(),
            stop_class,
            can_ask_outside,
            query,
            provenance,
            scope,
        }
    }
}

/// How multiple justifications compose (DCT19–DCT20).
///
/// > Multiple justifications ≠ redundancy. `same result ≠ same provenance`.
/// >
/// > The composition semantics lives in the network as an addressable policy;
/// > if missing, evaluation stops with `NEED / MISSING_RULE`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum JustificationComposition {
    /// Any one complete supported justification suffices (e.g., independent
    /// replications).
    Alternative,
    /// All listed justifications are required together.
    Joint,
}

/// Addressable composition policy for a result's justifications (DCT20).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPolicy {
    pub policy_id: NoepediaId,
    /// The result/record this policy governs.
    pub result_id: NoepediaId,
    pub composition: JustificationComposition,
    pub justification_ids: Vec<NoepediaId>,
    pub scope: Vec<String>,
}

/// Outcome of evaluating a result's survival under a composition policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CompositionVerdict {
    /// The result remains currently supported.
    Supported,
    /// The result does not remain supported.
    NotSupported,
    /// No composition policy applies — lawful STOP (NEED / MISSING_RULE).
    PolicyMissing,
}

/// Evaluate whether a result survives a challenge given its composition policy
/// (DCT19–DCT20). `justification_supported[i]` is the current epistemic state
/// of `policy.justification_ids[i]`.
///
/// - ALTERNATIVE: any one currently supported complete justification suffices.
/// - JOINT: all listed justifications are required together.
pub fn evaluate_composition(
    policy: &CompositionPolicy,
    justification_supported: &[bool],
) -> CompositionVerdict {
    if justification_supported.len() != policy.justification_ids.len() {
        return CompositionVerdict::PolicyMissing;
    }
    match policy.composition {
        JustificationComposition::Alternative => {
            if justification_supported.iter().any(|&b| b) {
                CompositionVerdict::Supported
            } else {
                CompositionVerdict::NotSupported
            }
        }
        JustificationComposition::Joint => {
            if justification_supported.iter().all(|&b| b) {
                CompositionVerdict::Supported
            } else {
                CompositionVerdict::NotSupported
            }
        }
    }
}

/// An addressable dependency policy: how an upstream status transition
/// propagates downstream (DCT05–DCT07, DEMON_CORE_SPEC §13–14).
///
/// Status propagation is NOT a demon-core primitive. The downstream
/// consequence is determined by a policy rule that lives in the network and
/// REFERS_TO the dependency edge with ROLE=POLICY. If no applicable policy
/// exists, the demon returns `NEED / MISSING_RULE`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPolicy {
    pub policy_id: NoepediaId,
    pub name: String,
    /// Upstream transition this policy applies to, e.g. "PREMISE -> CONFLICTED".
    pub upstream_transition: String,
    /// The downstream consequence status.
    pub downstream_consequence: KnowledgeStatus,
    pub scope: Vec<String>,
    /// REFERS_TO target — the dependency edge / rule this policy qualifies.
    pub rule_ref: NoepediaId,
    /// ROLE = "POLICY".
    pub role: String,
}

impl DependencyPolicy {
    pub fn new(
        name: &str,
        upstream_transition: &str,
        downstream_consequence: KnowledgeStatus,
        rule_ref: NoepediaId,
    ) -> Self {
        Self {
            policy_id: Uuid::new_v4(),
            name: name.into(),
            upstream_transition: upstream_transition.into(),
            downstream_consequence,
            scope: Vec::new(),
            rule_ref,
            role: "POLICY".into(),
        }
    }
}

/// OPEN is a legal knowledge state: an addressable unknown with constraints and
/// a validation rule — the rule that would close it. OPEN is never a denial.
/// (Noepedia harvest, 2026-10-12: "OPEN = addressable unknown + constraints +
/// validation rule".)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSpec {
    /// The address of the open question.
    pub address: String,
    /// Constraints that shape the unknown (what it cannot be).
    pub constraints: Vec<String>,
    /// What experiment/evidence/replication would close the open question.
    pub validation_rule: Option<String>,
    /// When the open question was raised.
    pub raised_at: DateTime<Utc>,
}

impl OpenSpec {
    pub fn new(
        address: &str,
        constraints: Vec<String>,
        validation_rule: Option<String>,
    ) -> Self {
        Self {
            address: address.to_string(),
            constraints,
            validation_rule,
            raised_at: Utc::now(),
        }
    }
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
    /// A formed unknown: the relation requires a participant in a particular
    /// role even before its identity is known (orphan-receptor pattern).
    /// See `RoleRequirement` for the constraints + validation rule that keep
    /// this discovery-shaped, not apophenia.
    RequiresRole,
}

/// A formed unknown (REQUIRED_ROLE): the field has already learned something
/// — that a role must be filled — before it knows what fills it.
/// (Noepedia harvest, 2026-10-12.)
///
/// Without a validation rule this degrades into apophenia (false projection of
/// roles onto systems that have none); every role requirement must therefore
/// carry constraints and the rule that would close it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRequirement {
    /// The required role, e.g. "kinase", "ligand", "substrate", "regulator".
    pub role: String,
    /// Constraints shaping the unknown (what it cannot be).
    pub constraints: Vec<String>,
    /// What would identify and validate the missing participant.
    pub validation_rule: Option<String>,
    pub raised_at: DateTime<Utc>,
}

impl RoleRequirement {
    pub fn new(role: &str, constraints: Vec<String>, validation_rule: Option<String>) -> Self {
        Self {
            role: role.to_string(),
            constraints,
            validation_rule,
            raised_at: Utc::now(),
        }
    }
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
    /// Demon regime at the end of the session (GO / ASK / BLOCK / NEED).
    pub regime: Option<RegimeState>,
    /// STOP CARDS for every stuck branch (Demon Core Harvest, §7–8).
    pub stop_cards: Vec<StopCard>,
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
    /// The run stopped because a rule/closure forbids this path (BLOCK).
    Blocked,
    /// The run stopped because the network's working structure is missing
    /// (NEED) — a correct query cannot yet be formed.
    Need,
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

    #[test]
    fn test_requires_role_is_a_formed_unknown() {
        // REQUIRED_ROLE: role known, identity unknown, with a validation rule.
        let req = RoleRequirement::new(
            "kinase",
            vec!["serine/threonine-specific".into()],
            Some("in vitro phosphorylation assay + KD rescue".into()),
        );
        assert_eq!(req.role, "kinase");
        assert_eq!(req.constraints.len(), 1);
        assert!(req.validation_rule.is_some());

        // And it maps onto the relation-kind vocabulary.
        let kind = RelationKind::RequiresRole;
        assert_ne!(kind, RelationKind::SimilarTo);
        assert!(matches!(kind, RelationKind::RequiresRole));
    }

    #[test]
    fn test_open_spec_carries_validation_rule() {
        let spec = OpenSpec::new(
            "mechanism X missing participant",
            vec!["no candidate in canonical pathway".into()],
            Some("co-immunoprecipitation screen".into()),
        );
        assert!(spec.validation_rule.is_some());
        assert!(!spec.address.is_empty());
        assert_eq!(spec.constraints.len(), 1);
    }

    #[test]
    fn test_status_assertion_is_epoch_scoped_and_non_destructive() {
        // P0 / DCT11–DCT12: MARK creates a NEW assertion, keeps the old one.
        let record = Uuid::new_v4();
        let author = Uuid::new_v4();
        let a1 = StatusAssertion::initial(record, KnowledgeStatus::Proposed, author);
        assert_eq!(a1.epoch, 1);
        assert_eq!(a1.history_status, HistoryStatus::Committed);

        let a2 = StatusAssertion {
            assertion_id: Uuid::new_v4(),
            record_id: record,
            status: KnowledgeStatus::Supported,
            scope: vec!["hardware revision B".into()],
            epoch: a1.epoch + 1,
            asserted_by: author,
            asserted_at: chrono::Utc::now(),
            history_status: HistoryStatus::Committed,
        };
        // Both assertions coexist: the old one is a historical trace, not erased.
        assert_eq!(a2.epoch, 2);
        assert_eq!(a1.record_id, a2.record_id);
        assert_ne!(a1.status, a2.status);
    }

    #[test]
    fn test_stop_card_carries_need_classification() {
        // P0 / DEMON §7–8: a stuck branch returns a STOP CARD.
        let card = StopCard::new(
            "DCT20 / one convergent justification challenged",
            "determine whether EV1 remains currently supported",
            "JP1 challenged; JP2 supported; both recorded as JUSTIFICATION",
            "justification-composition policy",
            NeedClass::MissingRule,
            false,
            None,
            vec![Uuid::new_v4()],
            vec!["local".into()],
        );
        assert_eq!(card.stop_class, NeedClass::MissingRule);
        assert!(!card.can_ask_outside); // NEED, not ASK
        assert!(card.query.is_none());
        assert!(!card.missing.is_empty());

        let ask = StopCard::new(
            "sensor read",
            "outside temperature",
            "no observation in field",
            "external fact",
            NeedClass::MissingFact,
            true,
            Some("query temperature sensor #3".into()),
            vec![],
            vec![],
        );
        assert!(ask.can_ask_outside); // ASK: precise permitted query exists
        assert!(ask.query.is_some());
    }

    #[test]
    fn test_dependency_policy_has_role_policy() {
        // P1 / DCT05–DCT07: downstream consequence lives in an addressable
        // policy with ROLE = POLICY, not in the demon core.
        let edge = Uuid::new_v4();
        let p = DependencyPolicy::new(
            "premise_conflict_propagates",
            "PREMISE -> CONFLICTED",
            KnowledgeStatus::Contested,
            edge,
        );
        assert_eq!(p.role, "POLICY");
        assert_eq!(p.rule_ref, edge);
        assert_eq!(p.downstream_consequence, KnowledgeStatus::Contested);
        assert!(!p.upstream_transition.is_empty());
    }

    #[test]
    fn test_composition_policy_alternative_vs_joint() {
        // P2 / DCT19–DCT20: multiple justifications ≠ redundancy; composition
        // semantics is explicit and addressable.
        let result = Uuid::new_v4();
        let jp1 = Uuid::new_v4();
        let jp2 = Uuid::new_v4();
        let alt = CompositionPolicy {
            policy_id: Uuid::new_v4(),
            result_id: result,
            composition: JustificationComposition::Alternative,
            justification_ids: vec![jp1, jp2],
            scope: vec![],
        };
        assert!(matches!(alt.composition, JustificationComposition::Alternative));
        assert_eq!(alt.justification_ids.len(), 2);

        let joint = JustificationComposition::Joint;
        assert_ne!(alt.composition, joint);
    }

    #[test]
    fn test_evaluate_composition_alternative_vs_joint() {
        // DCT20: one path challenged. Under ALTERNATIVE the result survives;
        // under JOINT it does not.
        let result = Uuid::new_v4();
        let jp1 = Uuid::new_v4();
        let jp2 = Uuid::new_v4();

        let alt = CompositionPolicy {
            policy_id: Uuid::new_v4(),
            result_id: result,
            composition: JustificationComposition::Alternative,
            justification_ids: vec![jp1, jp2],
            scope: vec![],
        };
        // jp1 challenged (false), jp2 supported (true)
        assert_eq!(
            evaluate_composition(&alt, &[false, true]),
            CompositionVerdict::Supported
        );
        assert_eq!(
            evaluate_composition(&alt, &[false, false]),
            CompositionVerdict::NotSupported
        );

        let joint = CompositionPolicy {
            policy_id: Uuid::new_v4(),
            result_id: result,
            composition: JustificationComposition::Joint,
            justification_ids: vec![jp1, jp2],
            scope: vec![],
        };
        // same graph shape, different composition policy → different verdict
        assert_eq!(
            evaluate_composition(&joint, &[false, true]),
            CompositionVerdict::NotSupported
        );
        assert_eq!(
            evaluate_composition(&joint, &[true, true]),
            CompositionVerdict::Supported
        );
        // mismatched length → policy missing (lawful NEED)
        assert_eq!(
            evaluate_composition(&alt, &[true]),
            CompositionVerdict::PolicyMissing
        );
    }

    #[test]
    fn test_session_carries_regime_and_stop_cards() {
        // P0: session outcome includes regime + stop cards as first-class.
        let session = Session {
            session_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            started: chrono::Utc::now(),
            ended: None,
            objective: "rebuild scene S".into(),
            traces: vec![],
            outcome: Some(Outcome {
                outcome_id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                kind: OutcomeKind::Need,
                description: "missing dependency policy".into(),
                evidence_id: None,
            }),
            regime: Some(RegimeState::Need),
            stop_cards: vec![StopCard::new(
                "scene S",
                "propagate downstream status",
                "upstream CONFLICTED",
                "applicable dependency-policy rule",
                NeedClass::MissingRule,
                false,
                None,
                vec![],
                vec![],
            )],
        };
        assert_eq!(session.regime, Some(RegimeState::Need));
        assert_eq!(session.stop_cards.len(), 1);
        assert_eq!(
            session.stop_cards[0].stop_class,
            NeedClass::MissingRule
        );
    }
}
