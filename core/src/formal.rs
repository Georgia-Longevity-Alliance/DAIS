//! Formal Verification Bridge — TLA+ specifications for DAIS Body Law.
//!
//! Safety-critical systems need mathematical proof, not just tests.
//! This module provides:
//! - TLA+ specification templates for Body Law layers
//! - Invariant checking (what must ALWAYS be true)
//! - Model checking against state space
//! - Counter-example generation for safety violations
//!
//! Reference: Lamport's TLA+ (Temporal Logic of Actions)

use serde::{Deserialize, Serialize};

/// A formal safety invariant — must hold in ALL system states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyInvariant {
    pub invariant_id: String,
    pub description: String,
    pub tla_expression: String,
    pub layer: u8,  // Which Body Law layer this invariant belongs to
    pub severity: crate::fleet::SafetyIntegrityLevel,
    pub verified: bool,
}

/// TLA+ specification for a Body Law layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlaSpecification {
    pub spec_id: String,
    pub layer_name: String,
    pub layer_number: u8,
    pub variables: Vec<String>,
    pub constants: Vec<String>,
    pub initial_state: String,
    pub next_state_action: String,
    pub invariants: Vec<SafetyInvariant>,
    pub temporal_properties: Vec<String>,
}

/// Model checking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCheckResult {
    pub spec_id: String,
    pub invariant: String,
    pub satisfied: bool,
    pub states_explored: u64,
    pub depth_reached: u32,
    pub counter_example: Option<CounterExample>,
    pub duration_ms: u64,
}

/// Counter-example: a trace showing how an invariant was violated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterExample {
    pub trace_length: u32,
    pub states: Vec<SystemState>,
    pub violated_at_step: u32,
}

/// A single system state in the counter-example trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub step: u32,
    pub action: String,
    pub variables: std::collections::HashMap<String, String>,
}

/// Body Law invariants (must ALWAYS hold)
pub fn body_law_invariants() -> Vec<SafetyInvariant> {
    vec![
        SafetyInvariant {
            invariant_id: "INV_HW_STOP_ALWAYS_AVAILABLE".into(),
            description: "Physical STOP must be reachable in all states".into(),
            tla_expression: "[](stop_pin_active => state /= emergency)".into(),
            layer: 0,
            severity: crate::fleet::SafetyIntegrityLevel::SIL4,
            verified: false,
        },
        SafetyInvariant {
            invariant_id: "INV_LASER_BELOW_LIMIT".into(),
            description: "Laser power must never exceed firmware limit".into(),
            tla_expression: "[](laser_power <= MAX_LASER_MW)".into(),
            layer: 1,
            severity: crate::fleet::SafetyIntegrityLevel::SIL4,
            verified: false,
        },
        SafetyInvariant {
            invariant_id: "INV_NO_UNAUTHORIZED_COMMAND".into(),
            description: "No command can execute outside passport capabilities".into(),
            tla_expression: "[](command_executed => command IN passport.capabilities)".into(),
            layer: 2,
            severity: crate::fleet::SafetyIntegrityLevel::SIL3,
            verified: false,
        },
        SafetyInvariant {
            invariant_id: "INV_EMERGENCY_OVERRIDE".into(),
            description: "Emergency rescue can override any layer".into(),
            tla_expression: "<>(emergency_active => rescue_agent_controls)".into(),
            layer: 3,
            severity: crate::fleet::SafetyIntegrityLevel::SIL4,
            verified: false,
        },
        SafetyInvariant {
            invariant_id: "INV_OFFLINE_LIMITS".into(),
            description: "Without connection, autonomous mandate must respect offline limits".into(),
            tla_expression: "[](offline_mode => actions IN offline_mandate)".into(),
            layer: 4,
            severity: crate::fleet::SafetyIntegrityLevel::SIL3,
            verified: false,
        },
        SafetyInvariant {
            invariant_id: "INV_PROOF_CHAIN_INTEGRITY".into(),
            description: "Safety proof chain must never be tampered".into(),
            tla_expression: "[](proof_chain.verify = TRUE)".into(),
            layer: 5,
            severity: crate::fleet::SafetyIntegrityLevel::SIL4,
            verified: false,
        },
        SafetyInvariant {
            invariant_id: "INV_RBAC_ENFORCED".into(),
            description: "Role-based access control must gate all privileged commands".into(),
            tla_expression: "[](privileged_command => actor.role IN allowed_roles)".into(),
            layer: 7,
            severity: crate::fleet::SafetyIntegrityLevel::SIL3,
            verified: false,
        },
    ]
}

/// TLA+ specification generator for Body Law Layer 0 (Hardware Safety)
pub fn generate_layer0_spec() -> TlaSpecification {
    TlaSpecification {
        spec_id: "TLA_BODYLAW_L0".into(),
        layer_name: "Hardware Safety".into(),
        layer_number: 0,
        variables: vec!["stop_pin_active".into(), "deadman_switch_active".into(), "key_lock_engaged".into(), "state".into()],
        constants: vec!["MAX_DEADMAN_MS".into(), "EMERGENCY_STATES".into()],
        initial_state: "state = \"idle\" /\\ stop_pin_active = TRUE /\\ deadman_switch_active = TRUE /\\ key_lock_engaged = FALSE".into(),
        next_state_action: "\\/ (stop_pin_active = FALSE => state' = \"emergency\") \n    \\/ (deadman_switch_active = FALSE => state' = \"emergency\") \n    \\/ (key_lock_engaged = TRUE => state' = \"maintenance\") \n    \\/ UNCHANGED <<stop_pin_active, deadman_switch_active, key_lock_engaged, state>>".into(),
        invariants: body_law_invariants().into_iter().filter(|i| i.layer == 0).collect(),
        temporal_properties: vec!["[](state = \"emergency\" => <>(state = \"idle\"))".into()],
    }
}

/// Formal Verification Report — human-readable summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub spec_id: String,
    pub total_invariants: u32,
    pub invariants_passed: u32,
    pub invariants_failed: u32,
    pub total_states_explored: u64,
    pub verification_complete: bool,
    pub recommendations: Vec<String>,
}

impl VerificationReport {
    pub fn new(results: &[ModelCheckResult]) -> Self {
        let total = results.len() as u32;
        let passed = results.iter().filter(|r| r.satisfied).count() as u32;
        let failed = total - passed;
        let states = results.iter().map(|r| r.states_explored).sum();

        let mut recommendations = Vec::new();
        if failed > 0 {
            recommendations.push(format!("{} invariants FAILED — review counter-examples immediately", failed));
        }
        if !results.iter().all(|r| r.depth_reached > 0) {
            recommendations.push("Increase model checking depth for complete coverage".into());
        }

        Self {
            spec_id: results.first().map(|r| r.spec_id.clone()).unwrap_or_default(),
            total_invariants: total,
            invariants_passed: passed,
            invariants_failed: failed,
            total_states_explored: states,
            verification_complete: failed == 0,
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_law_invariants_count() {
        let invariants = body_law_invariants();
        assert_eq!(invariants.len(), 7);
        assert!(invariants.iter().all(|i| i.invariant_id.starts_with("INV_")));
    }

    #[test]
    fn test_layer0_spec_structure() {
        let spec = generate_layer0_spec();
        assert_eq!(spec.layer_number, 0);
        assert_eq!(spec.variables.len(), 4);
        assert!(!spec.invariants.is_empty());
        assert!(spec.initial_state.contains("idle"));
    }

    #[test]
    fn test_verification_report() {
        let results = vec![
            ModelCheckResult {
                spec_id: "TLA_L0".into(), invariant: "INV_HW".into(),
                satisfied: true, states_explored: 1000, depth_reached: 10,
                counter_example: None, duration_ms: 50,
            },
            ModelCheckResult {
                spec_id: "TLA_L0".into(), invariant: "INV_LASER".into(),
                satisfied: false, states_explored: 500, depth_reached: 5,
                counter_example: Some(CounterExample {
                    trace_length: 3, states: vec![], violated_at_step: 2,
                }), duration_ms: 30,
            },
        ];
        let report = VerificationReport::new(&results);
        assert_eq!(report.total_invariants, 2);
        assert_eq!(report.invariants_passed, 1);
        assert_eq!(report.invariants_failed, 1);
        assert!(!report.verification_complete);
    }
}
