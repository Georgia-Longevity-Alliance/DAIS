//! ARGUS-OS1 Integration Bridge — connects DAIS to the microscope platform.
//!
//! Provides: (1) ARGUS-specific event types, (2) knowledge field publishing,
//! (3) session management, (4) registry connection helpers.

use crate::types::{EventType, FlightEvent, Id, Severity};
use serde::{Deserialize, Serialize};

/// ARGUS-OS1 specific event context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgusEvent {
    pub session_id: String,
    pub experiment_type: ExperimentType,
    pub well_position: Option<WellPosition>,
    pub channel: Option<AcquisitionChannel>,
    pub embryo_count: Option<u32>,
    pub division_detected: Option<bool>,
}

/// Types of ARGUS-OS1 experiments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentType {
    TimelapseAcquisition,
    ZStackAcquisition,
    AblationProtocol,
    CellDivisionTracking,
    CentrioleImaging,
    AutofocusCalibration,
    Unknown,
}

/// Well plate position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellPosition {
    pub plate_id: String,
    pub well: String,  // e.g. "A1", "H12"
    pub x_um: f64,
    pub y_um: f64,
    pub z_um: f64,
}

/// Acquisition channel (laser/filter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionChannel {
    pub name: String,
    pub wavelength_nm: u16,
    pub power_mw: f64,
    pub exposure_ms: u32,
}

/// ARGUS-OS1 Knowledge Claim — published to Proven
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgusKnowledgeClaim {
    pub claim_id: Id,
    pub subject: String,      // e.g., "C. elegans embryo at 4-cell stage"
    pub predicate: String,    // e.g., "exhibits_centriole_asymmetry"
    pub object: String,       // e.g., "mother_centriole_in_P1_lineage"
    pub status: ClaimStatus,
    pub confidence: f64,
    pub source_session: String,
    pub evidence_type: EvidenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaimStatus {
    Observed,
    Inferred,
    Hypothesized,
    Falsified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    TimelapseImage,
    ZStackProjection,
    DivisionEvent,
    AblationResult,
    ManualAnnotation,
}

/// Session record — persistent trace across ARGUS-OS1 runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgusSession {
    pub session_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub experiment_type: ExperimentType,
    pub total_events: u64,
    pub anomaly_count: u64,
    pub divisions_detected: u64,
    pub knowledge_claims: Vec<ArgusKnowledgeClaim>,
    pub events_snapshot: Vec<FlightEvent>,
}

impl ArgusSession {
    pub fn new(session_id: &str, experiment_type: ExperimentType) -> Self {
        Self {
            session_id: session_id.into(),
            started_at: chrono::Utc::now().to_rfc3339(),
            ended_at: None,
            experiment_type,
            total_events: 0,
            anomaly_count: 0,
            divisions_detected: 0,
            knowledge_claims: Vec::new(),
            events_snapshot: Vec::new(),
        }
    }

    pub fn end(&mut self) {
        self.ended_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Publish key findings as Proven knowledge claims
    pub fn publish_findings(&mut self) -> Vec<ArgusKnowledgeClaim> {
        let mut claims = Vec::new();
        
        // Example: if divisions were detected
        if self.divisions_detected > 0 {
            claims.push(ArgusKnowledgeClaim {
                claim_id: uuid::Uuid::new_v4(),
                subject: "C. elegans embryo population".into(),
                predicate: "underwent_cell_division".into(),
                object: format!("{} divisions detected", self.divisions_detected),
                status: ClaimStatus::Observed,
                confidence: 0.98,
                source_session: self.session_id.clone(),
                evidence_type: EvidenceType::DivisionEvent,
            });
        }

        // Example: anomaly report
        if self.anomaly_count > 0 {
            claims.push(ArgusKnowledgeClaim {
                claim_id: uuid::Uuid::new_v4(),
                subject: "ARGUS-OS1 acquisition".into(),
                predicate: "experienced_anomalies".into(),
                object: format!("{} anomalies during session", self.anomaly_count),
                status: ClaimStatus::Observed,
                confidence: 0.95,
                source_session: self.session_id.clone(),
                evidence_type: EvidenceType::TimelapseImage,
            });
        }

        // Record claim IDs
        for claim in &claims {
            self.knowledge_claims.push(claim.clone());
        }
        
        claims
    }
}

/// Registry bridge — connects DAIS local registry to ARGUS-OS1
pub struct ArgusRegistry {
    pub sessions: Vec<ArgusSession>,
    pub current_session: Option<String>,
}

impl ArgusRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            current_session: None,
        }
    }

    pub fn start_session(&mut self, experiment_type: ExperimentType) -> String {
        let sid = format!("argus_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0000"));
        let session = ArgusSession::new(&sid, experiment_type);
        self.current_session = Some(sid.clone());
        self.sessions.push(session);
        sid
    }

    pub fn end_current_session(&mut self) {
        if let Some(ref sid) = self.current_session.clone() {
            if let Some(session) = self.sessions.iter_mut().find(|s| s.session_id == *sid) {
                session.end();
            }
        }
        self.current_session = None;
    }

    pub fn get_session(&self, session_id: &str) -> Option<&ArgusSession> {
        self.sessions.iter().find(|s| s.session_id == session_id)
    }

    /// Cross-session query: find all sessions where X happened
    pub fn query_sessions(&self, predicate: &str) -> Vec<&ArgusSession> {
        self.sessions.iter().filter(|s| {
            s.knowledge_claims.iter().any(|c| {
                c.predicate.contains(predicate) || c.object.contains(predicate)
            })
        }).collect()
    }

    pub fn total_divisions(&self) -> u64 {
        self.sessions.iter().map(|s| s.divisions_detected).sum()
    }

    pub fn total_anomalies(&self) -> u64 {
        self.sessions.iter().map(|s| s.anomaly_count).sum()
    }
}

/// Pre-built ARGUS-OS1 knowledge claims for Proven bootstrap
pub fn bootstrap_knowledge_claims() -> Vec<ArgusKnowledgeClaim> {
    vec![
        ArgusKnowledgeClaim {
            claim_id: uuid::Uuid::new_v4(),
            subject: "C. elegans".into(),
            predicate: "has_no_somatic_centrioles".into(),
            object: "true".into(),
            status: ClaimStatus::Observed,
            confidence: 0.99,
            source_session: "literature".into(),
            evidence_type: EvidenceType::ManualAnnotation,
        },
        ArgusKnowledgeClaim {
            claim_id: uuid::Uuid::new_v4(),
            subject: "centriole".into(),
            predicate: "inherited_asymmetrically".into(),
            object: "in_stem_cell_divisions".into(),
            status: ClaimStatus::Observed,
            confidence: 0.95,
            source_session: "literature".into(),
            evidence_type: EvidenceType::ManualAnnotation,
        },
        ArgusKnowledgeClaim {
            claim_id: uuid::Uuid::new_v4(),
            subject: "ARGUS-OS1".into(),
            predicate: "can_track".into(),
            object: "centriole_inheritance_in_real_time".into(),
            status: ClaimStatus::Observed,
            confidence: 0.90,
            source_session: "bootstrap".into(),
            evidence_type: EvidenceType::TimelapseImage,
        },
        ArgusKnowledgeClaim {
            claim_id: uuid::Uuid::new_v4(),
            subject: "mother_centriole".into(),
            predicate: "accumulates_polyglutamylation".into(),
            object: "over_organismal_lifespan".into(),
            status: ClaimStatus::Hypothesized,
            confidence: 0.70,
            source_session: "CEDAR_hypothesis".into(),
            evidence_type: EvidenceType::ManualAnnotation,
        },
        ArgusKnowledgeClaim {
            claim_id: uuid::Uuid::new_v4(),
            subject: "ARGUS-OS1".into(),
            predicate: "detects_anomalies".into(),
            object: "laser_spike_stage_slip_focus_drift".into(),
            status: ClaimStatus::Observed,
            confidence: 0.88,
            source_session: "bootstrap".into(),
            evidence_type: EvidenceType::ManualAnnotation,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let mut reg = ArgusRegistry::new();
        let sid = reg.start_session(ExperimentType::CellDivisionTracking);
        assert!(reg.current_session.is_some());
        
        if let Some(s) = reg.get_session(&sid) {
            assert_eq!(s.experiment_type, ExperimentType::CellDivisionTracking);
            assert_eq!(s.total_events, 0);
        }
        
        reg.end_current_session();
        assert!(reg.current_session.is_none());
    }

    #[test]
    fn test_session_findings_publication() {
        let mut reg = ArgusRegistry::new();
        let sid = reg.start_session(ExperimentType::CentrioleImaging);
        
        if let Some(s) = reg.sessions.iter_mut().find(|s| s.session_id == sid) {
            s.divisions_detected = 12;
            s.anomaly_count = 2;
            let claims = s.publish_findings();
            assert_eq!(claims.len(), 2);
            assert!(claims[0].object.contains("12"));
            assert!(claims[1].object.contains("2"));
        }
    }

    #[test]
    fn test_cross_session_query() {
        let mut reg = ArgusRegistry::new();
        
        let s1 = reg.start_session(ExperimentType::CellDivisionTracking);
        if let Some(s) = reg.sessions.iter_mut().find(|s| s.session_id == s1) {
            s.divisions_detected = 5;
            s.publish_findings();
        }
        reg.end_current_session();
        
        let s2 = reg.start_session(ExperimentType::CellDivisionTracking);
        if let Some(s) = reg.sessions.iter_mut().find(|s| s.session_id == s2) {
            s.divisions_detected = 8;
            s.publish_findings();
        }
        reg.end_current_session();
        
        let results = reg.query_sessions("division");
        assert_eq!(results.len(), 2);
        assert_eq!(reg.total_divisions(), 13);
    }

    #[test]
    fn test_bootstrap_claims() {
        let claims = bootstrap_knowledge_claims();
        assert_eq!(claims.len(), 5);
        assert!(claims.iter().any(|c| c.subject == "C. elegans"));
        assert!(claims.iter().any(|c| c.predicate == "accumulates_polyglutamylation"));
    }

    #[test]
    fn test_argus_event_creation() {
        let event = ArgusEvent {
            session_id: "test_001".into(),
            experiment_type: ExperimentType::CentrioleImaging,
            well_position: Some(WellPosition {
                plate_id: "M24".into(), well: "B3".into(),
                x_um: 1500.0, y_um: 2200.0, z_um: 45.0,
            }),
            channel: Some(AcquisitionChannel {
                name: "GFP".into(), wavelength_nm: 488,
                power_mw: 5.0, exposure_ms: 200,
            }),
            embryo_count: Some(64),
            division_detected: Some(false),
        };
        assert_eq!(event.well_position.as_ref().unwrap().well, "B3");
        assert_eq!(event.channel.as_ref().unwrap().wavelength_nm, 488);
    }
}
