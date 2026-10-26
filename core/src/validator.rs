//! Validator — checks deltas before they enter the event store.
//!
//! The validator enforces:
//! - Schema conformance
//! - Referential integrity (target_id must exist for UPDATE/DEPRECATE)
//! - Status transitions (e.g., PROPOSED→SUPPORTED is valid, SUPPORTED→PROPOSED is not)
//! - Permission boundaries (agent must be authorized)

use crate::delta::Delta;
use crate::event_store::EventStore;
use crate::aletheia::KnowledgeStatus;
use crate::types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Result of delta validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

/// The delta validator.
pub struct Validator {
    /// Currently known state (from event store replay).
    state: serde_json::Value,
}

impl Validator {
    /// Create a new validator from the event store's current state.
    pub fn new(store: &EventStore) -> Self {
        Self {
            state: store.replay(),
        }
    }

    /// Validate a delta against all rules.
    pub fn validate(&self, delta: &Delta) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Structural validation
        if let Err(struct_errors) = delta.validate_structure() {
            for e in struct_errors {
                errors.push(ValidationError {
                    code: "STRUCTURAL".into(),
                    message: e.to_string(),
                    field: None,
                });
            }
        }

        // 2. Referential integrity
        if matches!(
            delta.operation,
            DeltaOperation::Update | DeltaOperation::Deprecate
        ) {
            if let Some(target_id) = delta.target_id {
                if !self.object_exists(target_id) {
                    errors.push(ValidationError {
                        code: "REFERENTIAL_INTEGRITY".into(),
                        message: format!(
                            "Target object {} does not exist — cannot {:?} non-existent object",
                            target_id, delta.operation
                        ),
                        field: Some("target_id".into()),
                    });
                }
            }
        }

        // 3. Status transitions
        if let Some(target_id) = delta.target_id {
            if let Some(current_status) = self.get_object_status(target_id) {
                if let Some(new_status) = delta.payload.get("status").and_then(|v| v.as_str()) {
                    if let (Ok(from), Ok(to)) = (
                        KnowledgeStatus::try_from(current_status.as_str()),
                        KnowledgeStatus::try_from(new_status),
                    ) {
                        if !Self::is_valid_transition(&from, &to) {
                            errors.push(ValidationError {
                                code: "INVALID_STATUS_TRANSITION".into(),
                                message: format!(
                                    "Cannot transition from '{:?}' to '{:?}'",
                                    from, to
                                ),
                                field: Some("status".into()),
                            });
                        }
                    }
                }
            }
        }

        // 3a. Transaction membrane (Demon Core Harvest, DCT08–DCT09):
        // a delta cannot use rules introduced or changed by itself to
        // authorize that same delta. The effective rule-set for checking
        // delta N comes from committed state N-1 (this Validator's snapshot).
        errors.extend(self.membrane_check(delta));

        // 3b. Dependency policy (Demon Core Harvest, DCT05–DCT07): an upstream
        // status transition with known dependents requires an applicable
        // DependencyPolicy in the network; without one the downstream
        // consequence is undefined → lawful BLOCK (NEED / MISSING_RULE).
        errors.extend(self.dependency_policy_check(delta));

        // 3b. Two-witness rule (Aletheia harvest): REPLICATED requires ≥2
        // independent sources/replications, not two claims from one pool.
        if let Some(new_status) = delta.payload.get("status").and_then(|v| v.as_str()) {
            if new_status == "REPLICATED" {
                let n_sources = delta
                    .payload
                    .get("sources")
                    .and_then(|v| v.as_array())
                    .map_or(0, |a| a.len());
                if n_sources < 2 {
                    warnings.push(format!(
                        "Transition to REPLICATED with only {} source(s) — two independent witnesses required",
                        n_sources
                    ));
                }
            }
        }

        // 4. Content checks (warnings only for v1)
        if delta.payload.get("subject").and_then(|v| v.as_str()) == Some("") {
            warnings.push("Claim subject is empty — consider adding a meaningful subject".into());
        }

        if delta.payload.get("sources").and_then(|v| v.as_array()).map_or(false, |a| a.is_empty())
            && delta.operation == DeltaOperation::Create
        {
            warnings.push("Claim created without sources — PROPOSED status requires sources for elevation".into());
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Check if an object exists in the current state.
    fn object_exists(&self, id: Uuid) -> bool {
        self.state["objects"]
            .get(id.to_string())
            .is_some()
    }

    /// Get the current epistemic status of an object.
    ///
    /// (DCT11–DCT12 separation): transitions are validated against the
    /// CURRENT view (`current_epistemic_status`), not the committed history.
    fn get_object_status(&self, id: Uuid) -> Option<String> {
        self.state["objects"]
            .get(id.to_string())
            .and_then(|obj| {
                obj.get("current_epistemic_status")
                    .or_else(|| obj.get("status"))
            })
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
    }

    /// Transaction membrane (DCT08–DCT09): a delta may not use rules it
    /// introduces or changes to authorize that same delta. The snapshot
    /// (committed state N-1) is the only rule-source the delta may lean on.
    fn membrane_check(&self, delta: &Delta) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let introduced = self.introduced_rule_ids(delta);
        if introduced.is_empty() {
            return errors;
        }
        // Every rule-reference the delta uses as an authorizer/validator.
        for field in ["validated_by_rule", "authorized_by_rule", "rule_id"] {
            if let Some(auth) = delta.payload.get(field).and_then(|v| v.as_str()) {
                if introduced.iter().any(|id| id == auth) {
                    errors.push(ValidationError {
                        code: "SELF_REFERENCE_BAN".into(),
                        message: format!(
                            "transaction membrane: delta introduces rule {auth} and uses it to \
                             authorize itself (rule-set for delta N must come from committed N-1)"
                        ),
                        field: Some(field.into()),
                    });
                }
            }
        }
        errors
    }

    /// Rule ids introduced by this delta (self-authored rules).
    fn introduced_rule_ids(&self, delta: &Delta) -> Vec<String> {
        let mut ids = Vec::new();
        // A created object that IS a rule card (type == RULE_CARD).
        if delta.payload.get("type").and_then(|v| v.as_str()) == Some("RULE_CARD") {
            if let Some(id) = delta
                .payload
                .get("id")
                .or_else(|| delta.payload.get("rule_id"))
                .and_then(|v| v.as_str())
            {
                ids.push(id.to_string());
            }
        }
        ids
    }

    /// Dependency policy (DCT05–DCT07): an upstream status transition with
    /// known dependents requires an applicable `DependencyPolicy` in the
    /// network. Without one, the downstream consequence is undefined — the
    /// demon does not invent it, it BLOCKs (NEED / MISSING_RULE).
    fn dependency_policy_check(&self, delta: &Delta) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let Some(new_status) = delta.payload.get("status").and_then(|v| v.as_str()) else {
            return errors;
        };
        let Some(target_id) = delta.target_id else {
            return errors;
        };
        let dependents = self.dependents_of(target_id);
        if dependents.is_empty() {
            return errors;
        }
        let transition = format!("PREMISE -> {new_status}");
        if self.find_applicable_policy(&transition).is_none() {
            errors.push(ValidationError {
                code: "MISSING_DEPENDENCY_POLICY".into(),
                message: format!(
                    "upstream transition '{transition}' has {} dependent(s) but no applicable \
                     DependencyPolicy — downstream consequence undefined (NEED / MISSING_RULE)",
                    dependents.len()
                ),
                field: Some("status".into()),
            });
        }
        errors
    }

    /// Dependents: objects that hold a supporting relation onto `id`
    /// (the premise they depend on).
    fn dependents_of(&self, id: Uuid) -> Vec<Uuid> {
        let mut out = Vec::new();
        if let Some(relations) = self.state["relations"].as_array() {
            for r in relations {
                let target = r
                    .get("target")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());
                let rel = r.get("relation").and_then(|v| v.as_str());
                if target == Some(id)
                    && matches!(rel, Some("supports") | Some("depends_on"))
                {
                    if let Some(src) = r
                        .get("source")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                    {
                        out.push(src);
                    }
                }
            }
        }
        out
    }

    /// Find an applicable dependency policy for an upstream transition.
    /// Policies live in the network as addressable objects with `role == "POLICY"`.
    pub fn find_applicable_policy(
        &self,
        transition: &str,
    ) -> Option<crate::aletheia::DependencyPolicy> {
        use crate::aletheia::DependencyPolicy;
        if let Some(objects) = self.state["objects"].as_object() {
            for (_, obj) in objects {
                if obj.get("role").and_then(|v| v.as_str()) != Some("POLICY") {
                    continue;
                }
                let up = obj
                    .get("upstream_transition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if up == transition {
                    return Some(DependencyPolicy {
                        policy_id: obj
                            .get("policy_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .unwrap_or_default(),
                        name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                        upstream_transition: up.into(),
                        downstream_consequence: obj
                            .get("downstream_consequence")
                            .and_then(|v| v.as_str())
                            .and_then(|s| KnowledgeStatus::try_from(s).ok())
                            .unwrap_or(KnowledgeStatus::Open),
                        scope: obj
                            .get("scope")
                            .and_then(|v| v.as_array())
                            .map(|a| {
                                a.iter()
                                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default(),
                        rule_ref: obj
                            .get("rule_ref")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .unwrap_or_default(),
                        role: "POLICY".into(),
                    });
                }
            }
        }
        None
    }

    /// Convenience: return a NEED STOP CARD when a dependency policy is missing
    /// (lawful BLOCK with a classified stop).
    pub fn missing_policy_stop_card(&self, transition: &str) -> crate::aletheia::StopCard {
        crate::aletheia::StopCard::new(
            "validator / dependency-policy check",
            "propagate upstream status transition downstream",
            &format!("transition: {transition}; dependents exist"),
            "applicable DependencyPolicy rule",
            crate::aletheia::NeedClass::MissingRule,
            false,
            None,
            vec![],
            vec![],
        )
    }

    /// Valid status transitions on the canonical K-ladder.
    ///
    /// PROPOSED → SUPPORTED → TESTED → REPLICATED
    ///          → CONTESTED → REFUTED → STALE
    /// ANY → SUPERSEDED
    /// ANY → OPEN (reset) / VALID-IN-CONTEXT (contextual re-qualification)
    fn is_valid_transition(from: &KnowledgeStatus, to: &KnowledgeStatus) -> bool {
        use KnowledgeStatus::*;
        matches!(
            (from, to),
            (Proposed, Supported)
                | (Proposed, Contested)
                | (Proposed, Refuted)
                | (Proposed, Stale)
                | (Supported, Tested)
                | (Supported, Contested)
                | (Supported, Refuted)
                | (Supported, Stale)
                | (Tested, Replicated)
                | (Tested, Contested)
                | (Tested, Refuted)
                | (Replicated, Contested)
                | (Replicated, Refuted)
                | (Replicated, Stale)
                | (_, Superseded)
                | (_, Open)
                | (_, ValidInContext)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store_with_object(obj_id: Uuid) -> EventStore {
        let mut store = EventStore::new();
        let mut delta = Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
        );
        delta.target_id = Some(obj_id);
        store.append(&delta);
        store
    }

    #[test]
    fn test_update_existing_object() {
        let obj_id = Uuid::new_v4();
        let store = make_store_with_object(obj_id);
        let validator = Validator::new(&store);

        let delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "SUPPORTED"}),
        );

        let report = validator.validate(&delta);
        assert!(report.valid, "Expected valid update: {:?}", report.errors);
    }

    #[test]
    fn test_update_nonexistent_object() {
        let store = EventStore::new();
        let validator = Validator::new(&store);

        let delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(), // nonexistent
            serde_json::json!({"status": "SUPPORTED"}),
        );

        let report = validator.validate(&delta);
        assert!(!report.valid);
    }

    #[test]
    fn test_invalid_status_transition() {
        let obj_id = Uuid::new_v4();
        let mut store = make_store_with_object(obj_id);

        // Set to SUPPORTED first
        let delta = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "SUPPORTED"}),
        );
        store.append(&delta);

        // Re-create validator with updated state
        let validator = Validator::new(&store);
        let delta2 = Delta::update(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "PROPOSED"}), // invalid back-transition
        );

        let report = validator.validate(&delta2);
        assert!(!report.valid);
    }

    #[test]
    fn test_transaction_membrane_blocks_self_authorization() {
        // P1 / DCT08–DCT09: a delta cannot introduce a rule and use that same
        // rule to authorize itself.
        let store = EventStore::new();
        let validator = Validator::new(&store);

        let self_serving = Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({
                "type": "RULE_CARD",
                "id": "R-NEW",
                "rule_id": "R-NEW",
                "validated_by_rule": "R-NEW",  // same rule authorizes itself
            }),
        );
        let report = validator.validate(&self_serving);
        assert!(
            report.errors.iter().any(|e| e.code == "SELF_REFERENCE_BAN"),
            "expected SELF_REFERENCE_BAN, got: {:?}",
            report.errors
        );
    }

    #[test]
    fn test_transaction_membrane_allows_external_rule_authorization() {
        // A delta authorized by a PRE-EXISTING rule (committed N-1) is fine.
        let mut store = EventStore::new();
        let author = Uuid::new_v4();
        let mut rule_delta = Delta::create(
            Uuid::new_v4(),
            author,
            Uuid::new_v4(),
            serde_json::json!({"type": "RULE_CARD", "id": "R-EXISTING"}),
        );
        rule_delta.target_id = Some(Uuid::new_v4());
        store.append(&rule_delta);
        let validator = Validator::new(&store);

        let delta = Delta::create(
            Uuid::new_v4(),
            author,
            Uuid::new_v4(),
            serde_json::json!({
                "type": "CLAIM",
                "validated_by_rule": "R-EXISTING",  // not introduced by this delta
            }),
        );
        let report = validator.validate(&delta);
        assert!(
            !report.errors.iter().any(|e| e.code == "SELF_REFERENCE_BAN"),
            "external rule authorization must pass the membrane: {:?}",
            report.errors
        );
    }

    #[test]
    fn test_missing_dependency_policy_blocks_transition() {
        // P1 / DCT05–DCT07: upstream transition with dependents but no
        // applicable policy → lawful BLOCK (NEED / MISSING_RULE).
        let premise_id = Uuid::new_v4();
        let dependent_id = Uuid::new_v4();
        let mut store = EventStore::new();
        let author = Uuid::new_v4();
        let pub_id = Uuid::new_v4();

        let mut create = Delta::create(
            pub_id,
            author,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "SUPPORTED"}),
        );
        create.target_id = Some(premise_id);
        store.append(&create);
        // dependent supports the premise (depends on it)
        let mut dep = Delta::create(
            pub_id,
            author,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "SUPPORTED"}),
        );
        dep.target_id = Some(dependent_id);
        store.append(&dep);
        store.append(&Delta::relate(
            pub_id,
            author,
            Uuid::new_v4(),
            dependent_id,
            premise_id,
            "supports",
        ));

        let validator = Validator::new(&store);
        let upgrade = Delta::update(
            pub_id,
            author,
            Uuid::new_v4(),
            premise_id,
            serde_json::json!({"status": "CONTESTED"}),
        );
        let report = validator.validate(&upgrade);
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.code == "MISSING_DEPENDENCY_POLICY"),
            "expected MISSING_DEPENDENCY_POLICY, got: {:?}",
            report.errors
        );
        assert!(!report.valid);

        // With a matching policy present, the same transition passes the
        // dependency check (status transition itself must still be valid).
        let mut policy = Delta::create(
            pub_id,
            author,
            Uuid::new_v4(),
            serde_json::json!({
                "type": "DEPENDENCY_POLICY",
                "policy_id": "P-1",
                "name": "premise conflict propagates",
                "upstream_transition": "PREMISE -> CONTESTED",
                "downstream_consequence": "CONTESTED",
                "role": "POLICY",
                "rule_ref": "E-1",
            }),
        );
        policy.target_id = Some(Uuid::new_v4());
        store.append(&policy);
        let validator2 = Validator::new(&store);
        let report2 = validator2.validate(&upgrade);
        assert!(
            !report2.errors.iter().any(|e| e.code == "MISSING_DEPENDENCY_POLICY"),
            "applicable policy must satisfy the dependency check: {:?}",
            report2.errors
        );
    }
}
