//! Trace Network — structured intervention records.
//!
//! Every LLM intervention is logged as a trace: what was read,
//! what was diagnosed, what was changed, under which mandate.
//! Solved-once problems are never paid for twice:
//! knowledge that cost joules to create is stored where it can
//! be reused for microjoules.

use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The Trace Network — a collection of intervention traces.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceNetwork {
    traces: Vec<InterventionTrace>,
}

impl TraceNetwork {
    pub fn new() -> Self {
        Self { traces: Vec::new() }
    }

    /// Record a new intervention trace.
    pub fn record(
        &mut self,
        device_id: Uuid,
        agent_id: Uuid,
        trigger_event_id: Uuid,
        diagnosis: &str,
        confidence: f64,
        actions_taken: Vec<TraceAction>,
        outcome: TraceOutcome,
        references: Vec<Uuid>,
        remix_of: Option<Uuid>,
    ) -> Uuid {
        let trace = InterventionTrace {
            trace_id: Uuid::new_v4(),
            device_id,
            agent_id,
            trigger_event_id,
            diagnosis: diagnosis.to_string(),
            confidence,
            actions_taken,
            outcome,
            timestamp: Utc::now(),
            references,
            remix_of,
            published: false,
            community_tags: Vec::new(),
            skill_level: None,
        };
        let id = trace.trace_id;
        self.traces.push(trace);
        id
    }

    /// Find traces for a specific device.
    pub fn find_by_device(&self, device_id: Uuid) -> Vec<&InterventionTrace> {
        self.traces
            .iter()
            .filter(|t| t.device_id == device_id)
            .collect()
    }

    /// Find traces that resolved a specific outcome.
    pub fn find_resolved(&self) -> Vec<&InterventionTrace> {
        self.traces
            .iter()
            .filter(|t| t.outcome == TraceOutcome::Resolved)
            .collect()
    }

    /// Find traces referencing a specific prior trace.
    pub fn find_referencing(&self, trace_id: Uuid) -> Vec<&InterventionTrace> {
        self.traces
            .iter()
            .filter(|t| t.references.contains(&trace_id))
            .collect()
    }

    /// Search traces by diagnosis text (case-insensitive substring).
    pub fn search_diagnosis(&self, query: &str) -> Vec<&InterventionTrace> {
        let q = query.to_lowercase();
        self.traces
            .iter()
            .filter(|t| t.diagnosis.to_lowercase().contains(&q))
            .collect()
    }

    /// Get trace by ID.
    pub fn get(&self, trace_id: Uuid) -> Option<&InterventionTrace> {
        self.traces.iter().find(|t| t.trace_id == trace_id)
    }

    /// Total traces stored.
    pub fn len(&self) -> usize {
        self.traces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }

    /// Publish a trace to the community library.
    /// Published traces are discoverable by other devices.
    pub fn publish_trace(
        &mut self,
        trace_id: Uuid,
        tags: Vec<String>,
        skill_level: Option<String>,
    ) -> Result<(), String> {
        if let Some(trace) = self.traces.iter_mut().find(|t| t.trace_id == trace_id) {
            trace.published = true;
            trace.community_tags = tags;
            trace.skill_level = skill_level;
            Ok(())
        } else {
            Err(format!("Trace {} not found", trace_id))
        }
    }

    /// Get all published traces (community library).
    pub fn community_library(&self) -> Vec<&InterventionTrace> {
        self.traces.iter().filter(|t| t.published).collect()
    }

    /// Find traces by community tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&InterventionTrace> {
        self.traces
            .iter()
            .filter(|t| t.community_tags.iter().any(|t2| t2 == tag))
            .collect()
    }

    /// Find remixes (improvements) of a given trace.
    pub fn find_remixes_of(&self, original_trace_id: Uuid) -> Vec<&InterventionTrace> {
        self.traces
            .iter()
            .filter(|t| t.remix_of == Some(original_trace_id))
            .collect()
    }

    /// Count remixes — measures community engagement.
    pub fn remix_count(&self, original_trace_id: Uuid) -> usize {
        self.find_remixes_of(original_trace_id).len()
    }
    pub fn to_llm_context(&self, max_traces: usize) -> String {
        let count = std::cmp::min(max_traces, self.traces.len());
        let recent: Vec<_> = self.traces.iter().rev().take(count).collect();

        let mut ctx = format!(
            "=== Trace Network ({} total, showing {}) ===\n\n",
            self.traces.len(),
            count
        );

        for trace in recent {
            ctx.push_str(&format!(
                "Trace {}:\n  Device: {}\n  Agent: {}\n  Diagnosis: {}\n  Confidence: {:.0}%\n  Outcome: {:?}\n  References: {}\n\n",
                trace.trace_id,
                trace.device_id,
                trace.agent_id,
                trace.diagnosis,
                trace.confidence * 100.0,
                trace.outcome,
                trace.references.len()
            ));
        }
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_find() {
        let mut tn = TraceNetwork::new();
        let device_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let trace_id = tn.record(
            device_id,
            agent_id,
            Uuid::new_v4(),
            "Laser power fluctuation due to thermal expansion",
            0.85,
            vec![],
            TraceOutcome::Resolved,
            vec![],
            None,
        );

        assert_eq!(tn.len(), 1);
        let found = tn.get(trace_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().device_id, device_id);
    }

    #[test]
    fn test_search_diagnosis() {
        let mut tn = TraceNetwork::new();
        tn.record(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Photobleaching due to excessive 488nm exposure",
            0.9,
            vec![],
            TraceOutcome::Resolved,
            vec![],
            None,
        );
        tn.record(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Temperature drift in microfluidic chamber",
            0.7,
            vec![],
            TraceOutcome::Mitigated,
            vec![],
            None,
        );

        let results = tn.search_diagnosis("photobleaching");
        assert_eq!(results.len(), 1);
    }
}
