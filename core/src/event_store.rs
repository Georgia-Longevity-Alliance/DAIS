//! Event Store — append-only log of all knowledge operations.
//!
//! Every delta, every publication creation, every consolidation
//! is recorded in the event store. The current state is a PROJECTION
//! of this log — the log itself is the source of truth.
//!
//! In production: PostgreSQL with WAL. For embedded/MVP: SQLite.

use crate::delta::Delta;
use crate::types::*;
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Ed25519 journal guard (P2, TRACE CONTINUITY).
///
/// An append-only log is only a *discipline* until it is *tamper-evident*:
/// the guard signs the canonical digest of the whole entry log. A compacted,
/// archived or corrected log can be re-signed; a silently rewritten log
/// cannot pass verification.
#[derive(Debug, Clone)]
pub struct JournalGuard {
    signing_key: Option<SigningKey>,
    verifying_key: Option<VerifyingKey>,
}

impl Default for JournalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl JournalGuard {
    /// A guard that can verify but not sign (read-only audit).
    pub fn new() -> Self {
        Self {
            signing_key: None,
            verifying_key: None,
        }
    }

    /// Deterministic key pair from a 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key: Some(signing_key),
            verifying_key: Some(verifying_key),
        }
    }

    /// The verifying key (for distribution/audit), if present.
    pub fn verifying_key_bytes(&self) -> Option<[u8; 32]> {
        self.verifying_key.map(|vk| vk.to_bytes())
    }

    /// Sign the canonical digest of the entry log. `None` if read-only.
    pub fn sign(&self, entries: &[EventLogEntry]) -> Option<Vec<u8>> {
        let signing_key = self.signing_key.as_ref()?;
        let digest = log_digest(entries);
        Some(signing_key.sign(&digest).to_bytes().to_vec())
    }

    /// Verify a signature over the entry log. `None`/`false` if no key.
    pub fn verify(&self, entries: &[EventLogEntry], signature: &[u8]) -> bool {
        let Some(verifying_key) = self.verifying_key else {
            return false;
        };
        let Ok(sig_bytes) = <[u8; 64]>::try_from(signature) else {
            return false;
        };
        let digest = log_digest(entries);
        verifying_key
            .verify(&digest, &Signature::from_bytes(&sig_bytes))
            .is_ok()
    }
}

/// Canonical digest over the full entry log. Every serialized entry (including
/// `archived` / `compacted` markers) is hashed, so compaction and archiving
/// are themselves covered by the signature.
fn log_digest(entries: &[EventLogEntry]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for entry in entries {
        let canonical = serde_json::to_vec(entry).unwrap_or_default();
        hasher.update((canonical.len() as u64).to_le_bytes());
        hasher.update(&canonical);
    }
    let d = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&d);
    out
}

/// Ensure a materialized object carries the status projection fields.
///
/// `status` is kept for compatibility; `current_epistemic_status` is the
/// canonical current view; `history_status` is the committed-fact dimension;
/// `status_assertions` is the non-destructive assertion log (DCT11–DCT12).
fn ensure_status_fields(obj: &mut serde_json::Value, author: Option<Uuid>) {
    if obj.get("status").is_none() {
        obj["status"] = serde_json::json!("PROPOSED");
    }
    let status = obj["status"]
        .as_str()
        .unwrap_or("PROPOSED")
        .to_string();
    obj["current_epistemic_status"] = serde_json::json!(status);
    obj["history_status"] = serde_json::json!("COMMITTED");
    if obj.get("status_assertions").is_none() {
        let scope = obj
            .get("scope")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        obj["status_assertions"] = serde_json::json!([{
            "epoch": 1,
            "status": status,
            "scope": scope,
            "asserted_by": author.map(|a| a.to_string()).unwrap_or_default(),
        }]);
    }
}

/// Append a NEW status assertion to an existing object (non-destructive MARK).
/// The prior assertions remain in `status_assertions`; `current_epistemic_status`
/// becomes the new value.
fn apply_status_assertion(obj: &mut serde_json::Value, new_status: &str, author: Option<Uuid>) {
    let mut assertions = obj
        .get("status_assertions")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let epoch = assertions.len() as u64 + 1;
    let scope = obj.get("scope").cloned().unwrap_or(serde_json::json!([]));
    assertions.push(serde_json::json!({
        "epoch": epoch,
        "status": new_status,
        "scope": scope,
        "asserted_by": author.map(|a| a.to_string()).unwrap_or_default(),
    }));
    obj["status"] = serde_json::json!(new_status);
    obj["current_epistemic_status"] = serde_json::json!(new_status);
    obj["status_assertions"] = serde_json::json!(assertions);
}

/// The append-only event store.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventStore {
    entries: Vec<EventLogEntry>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Append a delta to the event log.
    ///
    /// Returns the event_id of the recorded entry.
    pub fn append(&mut self, delta: &Delta) -> Uuid {
        let entry = EventLogEntry {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: delta.author_id,
            operation: delta.operation.clone(),
            payload: serde_json::to_value(delta).unwrap_or_default(),
            signature: delta.signature.clone(),
            archived: false,
            compacted: false,
            compacted_into: None,
        };
        let id = entry.event_id;
        self.entries.push(entry);
        id
    }

    /// Append a raw event log entry.
    pub fn append_raw(&mut self, entry: EventLogEntry) -> Uuid {
        let id = entry.event_id;
        self.entries.push(entry);
        id
    }

    /// Get all entries for a publication, ordered by timestamp.
    pub fn get_publication_history(&self, publication_id: Uuid) -> Vec<&EventLogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Ok(delta) = serde_json::from_value::<Delta>(e.payload.clone()) {
                    delta.publication_id == publication_id
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get entries by actor.
    pub fn get_by_actor(&self, actor_id: Uuid) -> Vec<&EventLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.actor_id == actor_id)
            .collect()
    }

    /// Get entries since a timestamp.
    pub fn get_since(&self, since: chrono::DateTime<Utc>) -> Vec<&EventLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read-only view of the entry log (audit/render).
    pub fn entries(&self) -> &[EventLogEntry] {
        &self.entries
    }

    /// Replay all entries and build the current projection.
    /// This is the core of CQRS: event log → current state.
    ///
    /// Non-destructive status (Demon Core Harvest, DCT11–DCT12): a status
    /// transition never overwrites the object's past — it appends a new
    /// `status_assertions` row (with `epoch`), keeps the prior rows, and
    /// exposes `current_epistemic_status` (last applicable assertion) and
    /// `history_status` (committed fact). `status` is kept for compatibility.
    pub fn replay(&self) -> serde_json::Value {
        let mut state = serde_json::json!({
            "objects": {},
            "relations": [],
            "publications": {}
        });

        for entry in &self.entries {
            if let Ok(delta) = serde_json::from_value::<Delta>(entry.payload.clone()) {
                match delta.operation {
                    DeltaOperation::Create => {
                        if let Some(obj) = state["objects"].as_object_mut() {
                            let id = delta.target_id.unwrap_or(delta.delta_id);
                            let mut payload = delta.payload.clone();
                            ensure_status_fields(&mut payload, Some(delta.author_id));
                            obj.insert(id.to_string(), payload);
                        }
                    }
                    DeltaOperation::Update => {
                        if let Some(obj) = state["objects"].as_object_mut() {
                            if let Some(target_id) = delta.target_id {
                                if let Some(existing) =
                                    obj.get_mut(&target_id.to_string())
                                {
                                    // Merge non-status fields; the status is
                                    // appended as a NEW assertion, never overwritten.
                                    if let Some(payload_obj) = delta.payload.as_object() {
                                        for (k, v) in payload_obj {
                                            if k != "status"
                                                && k != "current_epistemic_status"
                                                && k != "status_assertions"
                                            {
                                                existing[k] = v.clone();
                                            }
                                        }
                                    }
                                    if let Some(new_status) = delta
                                        .payload
                                        .get("status")
                                        .and_then(|v| v.as_str())
                                    {
                                        apply_status_assertion(
                                            existing,
                                            new_status,
                                            Some(delta.author_id),
                                        );
                                    }
                                } else {
                                    // Target not present yet → materialize from
                                    // this delta's payload.
                                    let mut payload = delta.payload.clone();
                                    ensure_status_fields(
                                        &mut payload,
                                        Some(delta.author_id),
                                    );
                                    obj.insert(target_id.to_string(), payload);
                                }
                            }
                        }
                    }
                    DeltaOperation::Deprecate => {
                        if let Some(obj) = state["objects"].as_object_mut() {
                            if let Some(target_id) = delta.target_id {
                                if let Some(existing) =
                                    obj.get_mut(&target_id.to_string())
                                {
                                    apply_status_assertion(
                                        existing,
                                        "SUPERSEDED",
                                        Some(delta.author_id),
                                    );
                                    existing["history_status"] =
                                        serde_json::json!("SUPERSEDED");
                                }
                            }
                        }
                    }
                    DeltaOperation::Relate => {
                        // Record relations so downstream dependency checks can
                        // find dependents (Demon Core Harvest, DCT05–DCT07).
                        if let Some(relations) = state["relations"].as_array_mut() {
                            relations.push(serde_json::json!({
                                "source": delta
                                    .target_id
                                    .map(|x| x.to_string()),
                                "target": delta
                                    .payload
                                    .get("target")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                "relation": delta
                                    .payload
                                    .get("relation")
                                    .and_then(|v| v.as_str()),
                                "delta_id": delta.delta_id.to_string(),
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }

        state
    }

    /// Archive event entries. Archived entries remain addressable and
    /// replayable (TRACE CONTINUITY) — they are only marked `Archived`.
    pub fn archive(&mut self, event_ids: &[Uuid]) -> Result<(), String> {
        let mut missing = Vec::new();
        for id in event_ids {
            if let Some(entry) =
                self.entries.iter_mut().find(|e| e.event_id == *id)
            {
                entry.archived = true;
            } else {
                missing.push(id.to_string());
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "archive: unknown event ids: {}",
                missing.join(", ")
            ))
        }
    }

    /// Compact event entries into a single summary entry. The merged event ids
    /// are preserved in `compacted_into`, so the addresses remain resolvable
    /// (address-preservation guarantee, TRACE CONTINUITY).
    pub fn compact(
        &mut self,
        event_ids: &[Uuid],
        summary: &str,
    ) -> Result<Uuid, String> {
        let mut missing = Vec::new();
        for id in event_ids {
            if !self.entries.iter().any(|e| e.event_id == *id) {
                missing.push(id.to_string());
            }
        }
        if !missing.is_empty() {
            return Err(format!(
                "compact: unknown event ids: {}",
                missing.join(", ")
            ));
        }

        let ids = event_ids.to_vec();
        let payload = serde_json::json!({
            "compacted_ids": ids.iter().map(|i| i.to_string()).collect::<Vec<_>>(),
            "summary": summary,
        });
        let mut summary_entry = EventLogEntry {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: Uuid::nil(),
            operation: DeltaOperation::Consolidate,
            payload,
            signature: None,
            archived: false,
            compacted: true,
            compacted_into: None,
        };
        let summary_id = summary_entry.event_id;

        for entry in &mut self.entries {
            if ids.contains(&entry.event_id) {
                entry.compacted = true;
                entry.compacted_into = Some(summary_id);
            }
        }
        // The summary entry itself references the addresses it absorbed.
        summary_entry.payload["compacted_ids"] =
            serde_json::json!(ids.iter().map(|i| i.to_string()).collect::<Vec<_>>());
        self.entries.push(summary_entry);
        Ok(summary_id)
    }

    /// Verify that every compacted/archived entry still resolves to its
    /// summary or remains addressable (TRACE CONTINUITY invariant check).
    pub fn check_trace_continuity(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for entry in &self.entries {
            // A summary entry is the absorber, not the absorbed.
            let is_summary = entry.operation == DeltaOperation::Consolidate
                && entry.payload.get("compacted_ids").is_some();
            if entry.compacted
                && entry.compacted_into.is_none()
                && !is_summary
            {
                violations.push(format!(
                    "event {} compacted without summary pointer",
                    entry.event_id
                ));
            }
        }
        // Summary entries must list at least one absorbed address.
        for entry in &self.entries {
            if entry.operation == DeltaOperation::Consolidate
                && entry.compacted
            {
                let ids = entry.payload
                    .get("compacted_ids")
                    .and_then(|v| v.as_array())
                    .map_or(0, |a| a.len());
                if ids == 0 {
                    violations.push(format!(
                        "summary event {} has no absorbed addresses",
                        entry.event_id
                    ));
                }
            }
        }
        violations
    }

    /// Sign the current log with the journal guard (P2 / TRACE CONTINUITY).
    pub fn sign_log(&self, guard: &JournalGuard) -> Option<Vec<u8>> {
        guard.sign(&self.entries)
    }

    /// Verify a signature over the current log.
    pub fn verify_log(&self, guard: &JournalGuard, signature: &[u8]) -> bool {
        guard.verify(&self.entries, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_retrieve() {
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let delta = Delta::create(
            pub_id,
            author_id,
            Uuid::new_v4(),
            serde_json::json!({"claim": "test"}),
        );

        let event_id = store.append(&delta);
        assert_eq!(store.len(), 1);

        let history = store.get_publication_history(pub_id);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].event_id, event_id);
    }

    #[test]
    fn test_replay_builds_state() {
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let obj_id = Uuid::new_v4();

        let mut delta = Delta::create(
            pub_id,
            author_id,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
        );
        delta.target_id = Some(obj_id);
        store.append(&delta);

        let state = store.replay();
        assert!(state["objects"][obj_id.to_string()].is_object());
    }

    #[test]
    fn test_status_transition_is_non_destructive() {
        // P0 / DCT11–DCT12: an UPDATE appends a new status assertion; the
        // prior assertion remains in `status_assertions` (never an overwrite).
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let obj_id = Uuid::new_v4();

        let mut create = Delta::create(
            pub_id,
            author_id,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
        );
        create.target_id = Some(obj_id);
        store.append(&create);

        let upd = Delta::update(
            pub_id,
            author_id,
            Uuid::new_v4(),
            obj_id,
            serde_json::json!({"status": "SUPPORTED"}),
        );
        store.append(&upd);

        let state = store.replay();
        let obj = &state["objects"][obj_id.to_string()];
        assert_eq!(obj["current_epistemic_status"], serde_json::json!("SUPPORTED"));
        assert_eq!(obj["status"], serde_json::json!("SUPPORTED"));
        let assertions = obj["status_assertions"].as_array().unwrap();
        assert_eq!(assertions.len(), 2, "prior assertion must be kept");
        assert_eq!(assertions[0]["status"], serde_json::json!("PROPOSED"));
        assert_eq!(assertions[1]["status"], serde_json::json!("SUPPORTED"));
        assert_eq!(assertions[0]["epoch"], serde_json::json!(1));
        assert_eq!(assertions[1]["epoch"], serde_json::json!(2));
        assert_eq!(obj["history_status"], serde_json::json!("COMMITTED"));
    }

    #[test]
    fn test_deprecate_preserves_history_and_sets_superseded() {
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let obj_id = Uuid::new_v4();

        let mut create = Delta::create(
            pub_id,
            author_id,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "SUPPORTED"}),
        );
        create.target_id = Some(obj_id);
        store.append(&create);
        store.append(&Delta::deprecate(
            pub_id,
            author_id,
            Uuid::new_v4(),
            obj_id,
            "superseded by newer formulation",
            None,
        ));

        let state = store.replay();
        let obj = &state["objects"][obj_id.to_string()];
        assert_eq!(obj["status"], serde_json::json!("SUPERSEDED"));
        assert_eq!(obj["history_status"], serde_json::json!("SUPERSEDED"));
        let assertions = obj["status_assertions"].as_array().unwrap();
        // initial SUPPORTED assertion survives + SUPERSEDED appended
        assert_eq!(assertions.len(), 2);
        assert_eq!(assertions[0]["status"], serde_json::json!("SUPPORTED"));
        assert_eq!(assertions[1]["status"], serde_json::json!("SUPERSEDED"));
    }

    #[test]
    fn test_archive_keeps_entries_addressable() {
        // P2 / TRACE CONTINUITY: archiving marks, never removes.
        let mut store = EventStore::new();
        let e1 = store.append(&Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({"claim": "a"}),
        ));
        store.archive(&[e1]).unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.entries[0].archived);
        // Replay still yields the object (address preserved).
        let state = store.replay();
        assert!(state["objects"].as_object().map_or(false, |o| !o.is_empty()));
        assert!(store.check_trace_continuity().is_empty());
    }

    #[test]
    fn test_compact_preserves_addresses() {
        // P2 / TRACE CONTINUITY: compaction absorbs addresses into a summary
        // entry; every compacted entry points to its summary.
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author = Uuid::new_v4();
        let mut e1 = Delta::create(
            pub_id,
            author,
            Uuid::new_v4(),
            serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
        );
        e1.target_id = Some(Uuid::new_v4());
        let id1 = store.append(&e1);
        let id2 = store.append(&Delta::update(
            pub_id,
            author,
            Uuid::new_v4(),
            e1.target_id.unwrap(),
            serde_json::json!({"status": "SUPPORTED"}),
        ));

        let summary_id = store.compact(&[id1, id2], "coalesced proposal history").unwrap();
        assert_eq!(store.check_trace_continuity().len(), 0);
        // The summary entry lists the absorbed addresses.
        let summary = store
            .entries
            .iter()
            .find(|e| e.event_id == summary_id)
            .unwrap();
        let ids = summary.payload["compacted_ids"].as_array().unwrap();
        assert_eq!(ids.len(), 2);
        // Original entries now point into the summary.
        let e1_entry = store.entries.iter().find(|e| e.event_id == id1).unwrap();
        assert!(e1_entry.compacted);
        assert_eq!(e1_entry.compacted_into, Some(summary_id));
    }

    #[test]
    fn test_compact_missing_event_is_error() {
        let mut store = EventStore::new();
        let res = store.compact(&[Uuid::new_v4()], "x");
        assert!(res.is_err());
    }

    #[test]
    fn test_journal_signature_roundtrip() {
        // P2 / TRACE CONTINUITY: sign & verify the log.
        let mut store = EventStore::new();
        store.append(&Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({"claim": "a"}),
        ));
        let guard = JournalGuard::from_seed([7u8; 32]);
        let sig = store.sign_log(&guard).expect("signing key present");
        assert!(store.verify_log(&guard, &sig));
        // Read-only guard cannot sign but rejects forged verification.
        let readonly = JournalGuard::new();
        assert!(readonly.sign(&store.entries).is_none());
        assert!(!readonly.verify(&store.entries, &sig));
    }

    #[test]
    fn test_journal_signature_detects_tamper() {
        // A silently rewritten log fails verification (append-only is not
        // enough — tamper-evidence requires the signature).
        let mut store = EventStore::new();
        store.append(&Delta::create(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            serde_json::json!({"claim": "a"}),
        ));
        let guard = JournalGuard::from_seed([9u8; 32]);
        let sig = store.sign_log(&guard).unwrap();

        // Tamper: rewrite the stored payload.
        store.entries[0].payload =
            serde_json::json!({"claim": "silently rewritten"});
        assert!(!store.verify_log(&guard, &sig));
    }

    #[test]
    fn test_journal_signature_covers_compaction() {
        // Compaction/archiving are legitimate re-signing events; the invariant
        // is that a signature over the NEW committed log verifies.
        let mut store = EventStore::new();
        let pub_id = Uuid::new_v4();
        let author = Uuid::new_v4();
        let id1 = store.append(&Delta::create(
            pub_id,
            author,
            Uuid::new_v4(),
            serde_json::json!({"claim": "a"}),
        ));
        let id2 = store.append(&Delta::create(
            pub_id,
            author,
            Uuid::new_v4(),
            serde_json::json!({"claim": "b"}),
        ));
        store.compact(&[id1, id2], "coalesced").unwrap();
        let guard = JournalGuard::from_seed([3u8; 32]);
        let sig = store.sign_log(&guard).unwrap();
        assert!(store.verify_log(&guard, &sig));
        assert!(store.check_trace_continuity().is_empty());
    }
}
