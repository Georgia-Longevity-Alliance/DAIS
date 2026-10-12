//! Author column & №0 axiom — who actually controls an autonomous body.
//!
//! (Noepedia harvest, 2026-10-12; author-column rule v0.1)
//!
//! - Every decision/mandate record carries an author column: signature
//!   (provenance object), door (modality lane), time (F1 order).
//! - The column records the WRITING event (who placed the knot / issued the
//!   mandate), not the waking event.
//! - Subjectivity is TWO-axis (rule v0.1, after Gakely's animal counterexample,
//!   2026-10-12): FREEDOM = 1 − foreign-authored wakings/all (non-domination),
//!   AUTHORSHIP = own-authored wakings/all (whether the actor writes at all),
//!   plus a third orthogonal axis, quality of control (bench). A subject is
//!   free AND an author. An animal is free (nobody signs the jungle) but
//!   authorship 0 → not a subject.
//! - №0 is the root record (the only record outside a session): an axiom, named
//!   axiom. Content = label (self-consistent).
//! - Corollary: a read-only observer never appears as a Writing signature →
//!   it is never a subject. Observer and subject are non-overlapping classes.

use crate::types::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Whether an author-column event is the WRITING of a knot/mandate or the
/// WAKING on it. Only writing events grant authorship.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorEventKind {
    /// The actor placed the knot / issued the mandate.
    Writing,
    /// The actor woke on an existing knot (reaction).
    Waking,
}

/// One row in the author column of a decision record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorRecord {
    pub record_id: Id,
    /// The knot/mandate this row belongs to.
    pub knot_id: Id,
    /// Signature: for `Writing` — the author (placer); for `Waking` — the waker.
    pub signature: Id,
    /// Modality lane (which door the event entered through).
    pub door: String,
    pub kind: AuthorEventKind,
    pub time: DateTime<Utc>,
}

/// The author column of a body — append-only log of who wrote what, and who
/// woke on what.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorColumn {
    pub records: Vec<AuthorRecord>,
}

/// Window over which subjectivity is measured.
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Window {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }
}

impl AuthorColumn {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Append a WRITING event: who placed the knot / issued the mandate.
    pub fn record_writing(&mut self, knot_id: Id, signature: Id, door: &str) -> Id {
        self.push(AuthorEventKind::Writing, knot_id, signature, door)
    }

    /// Append a WAKING event: who woke on an existing knot.
    pub fn record_waking(&mut self, knot_id: Id, waker: Id, door: &str) -> Id {
        self.push(AuthorEventKind::Waking, knot_id, waker, door)
    }

    fn push(&mut self, kind: AuthorEventKind, knot_id: Id, signature: Id, door: &str) -> Id {
        let record = AuthorRecord {
            record_id: Uuid::new_v4(),
            knot_id,
            signature,
            door: door.to_string(),
            kind,
            time: Utc::now(),
        };
        let id = record.record_id;
        self.records.push(record);
        id
    }

    /// Author of a knot = the latest Writing signature for that knot.
    pub fn knot_author(&self, knot_id: Id) -> Option<Id> {
        self.records
            .iter()
            .rev()
            .find(|r| r.kind == AuthorEventKind::Writing && r.knot_id == knot_id)
            .map(|r| r.signature)
    }

    /// Has this actor ever authored (written) a knot/mandate?
    pub fn has_authored(&self, actor: Id) -> bool {
        self.records
            .iter()
            .any(|r| r.kind == AuthorEventKind::Writing && r.signature == actor)
    }

    /// Corollary 2 (read-only observer): an actor that never appears as a
    /// Writing signature is an observer and — by the subjectivity criterion —
    /// can never be a subject, regardless of how much it observes.
    pub fn is_observer(&self, actor: Id) -> bool {
        !self.has_authored(actor)
    }

    /// Split of wakings over `window` → (own-authored, foreign-authored, total).
    /// A waking whose knot has NO author (e.g. an animal's environmental
    /// alarms — nobody signed the jungle) is counted in `total` but neither as
    /// own nor as foreign. This is what keeps a low-signed universe from
    /// inflating authorship.
    fn waking_split(&self, actor: Id, window: Window) -> (f64, f64, f64) {
        let wakes: Vec<&AuthorRecord> = self
            .records
            .iter()
            .filter(|r| r.kind == AuthorEventKind::Waking && r.time >= window.start && r.time <= window.end)
            .collect();
        if wakes.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let own = wakes
            .iter()
            .filter(|r| self.knot_author(r.knot_id) == Some(actor))
            .count();
        let foreign = wakes
            .iter()
            .filter(|r| self.knot_author(r.knot_id).map_or(false, |a| a != actor))
            .count();
        (own as f64, foreign as f64, wakes.len() as f64)
    }

    /// FREEDOM axis (rule v0.1, two-axis): 1 − (foreign-authored wakings / all
    /// wakings). Measures non-domination — not being subject to other subjects.
    /// Animal counterexample (Gakely, 2026-10-12): the jungle signs nothing →
    /// freedom 1.0, yet the animal is our paradigm non-subject — because its
    /// AUTHORSHIP is 0 (it writes nothing). Hence freedom alone ≠ subject.
    pub fn freedom(&self, actor: Id, window: Window) -> f64 {
        let (_, foreign, total) = self.waking_split(actor, window);
        if total == 0.0 {
            0.0
        } else {
            1.0 - foreign / total
        }
    }

    /// AUTHORSHIP axis: share of wakings whose knots the actor itself authored
    /// (own / all wakings). Measures whether the actor writes at all.
    /// The anxious planner is an author (own is high) but a bad one — that is
    /// the third, orthogonal axis (quality of control).
    pub fn authorship(&self, actor: Id, window: Window) -> f64 {
        let (own, _, total) = self.waking_split(actor, window);
        if total == 0.0 {
            0.0
        } else {
            own / total
        }
    }

    /// Third axis placeholder: quality of control (outcome-weighted success of
    /// the actor's own mandates). Not yet measured — goes to bench.
    pub fn control_quality(&self, _actor: Id, _window: Window) -> Option<f64> {
        None // bench: outcome-weighted success of own mandates
    }

    /// Subject test (v0, two axes): a subject is BOTH free AND an author.
    /// Animal: freedom 1 but authorship 0 → not a subject.
    /// Agenda-slave: both low → not a subject.
    pub fn is_subject(&self, actor: Id, window: Window, threshold: f64) -> bool {
        self.freedom(actor, window) > threshold && self.authorship(actor, window) > threshold
    }
}

/// Theorem (corollary 2): observation = reading without writing; power = writing.
/// An observer (has_authored == false) is never a subject.
pub fn observer_never_subject(has_authored: bool) -> bool {
    !has_authored
}

/// №0 — the axiom, named axiom (option B).
///
/// The only record written outside a session; the root of the field. Its
/// content equals its label — the first self-consistent object, not a
/// hypothesis. Amended only by sessions with a quorum; the hard core
/// (Body Law layers 1–2) is not overridable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteNo0 {
    pub version: u32,
    /// Canonical text of the axiom.
    pub axiom: String,
    /// Revision procedure.
    pub revision_rule: String,
    pub declared_at: DateTime<Utc>,
}

impl Default for StatuteNo0 {
    fn default() -> Self {
        Self {
            version: 1,
            axiom: "Nothing derived becomes quiet truth. The root is declared and revised by sessions, with a quorum. The content of №0 = the label of №0.".into(),
            revision_rule: "Amended only by sessions with a quorum; content must equal its label (self-consistency check).".into(),
            declared_at: Utc::now(),
        }
    }
}

impl StatuteNo0 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Self-consistency: content = label (non-empty and grounded in the quorum rule).
    pub fn is_self_consistent(&self) -> bool {
        !self.axiom.trim().is_empty() && self.revision_rule.contains("quorum")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn wide_window() -> Window {
        Window::new(Utc::now() - Duration::days(1), Utc::now() + Duration::days(1))
    }

    #[test]
    fn test_subjectivity_counts_foreign_authored_wakings() {
        let vano = Uuid::new_v4();
        let niko = Uuid::new_v4();
        let mut col = AuthorColumn::new();

        // Three knots Vano wakes on: two authored by Vano, one by Niko.
        let ka = Uuid::new_v4();
        let kb = Uuid::new_v4();
        let kc = Uuid::new_v4();
        col.record_writing(ka, vano, "sensor");
        col.record_writing(kb, vano, "sensor");
        col.record_writing(kc, niko, "sensor");

        col.record_waking(ka, vano, "sensor");
        col.record_waking(kb, vano, "sensor");
        col.record_waking(kc, vano, "sensor");

        let w = wide_window();
        // 2 of 3 knots are Vano's → freedom = 1 − 1/3 = 2/3; authorship = 2/3.
        assert!((col.freedom(vano, w) - 2.0 / 3.0).abs() < 1e-9);
        assert!((col.authorship(vano, w) - 2.0 / 3.0).abs() < 1e-9);
        assert!(col.is_subject(vano, w, 0.5));
        // Niko never woke in the window → neither free-witnessed nor author.
        assert!(!col.is_subject(niko, w, 0.5));
    }

    #[test]
    fn test_animal_is_free_but_not_author() {
        // Gakely's counterexample (2026-10-12): the jungle does not sign.
        // An animal wakes on alarms whose knots have NO author → freedom is
        // maximal (nobody dominates it) but authorship is 0 (it writes nothing).
        let animal = Uuid::new_v4();
        let mut col = AuthorColumn::new();
        let alarm1 = Uuid::new_v4();
        let alarm2 = Uuid::new_v4();
        // No record_writing for these knots — nobody signed the jungle.
        col.record_waking(alarm1, animal, "sensor");
        col.record_waking(alarm2, animal, "sensor");

        let w = wide_window();
        assert!((col.freedom(animal, w) - 1.0).abs() < 1e-9); // free
        assert_eq!(col.authorship(animal, w), 0.0); // but not an author
        assert!(!col.is_subject(animal, w, 0.5)); // not a subject
    }

    #[test]
    fn test_writing_does_not_count_as_waking() {
        let actor = Uuid::new_v4();
        let mut col = AuthorColumn::new();
        let knot = Uuid::new_v4();
        // Many writings but zero wakings → no evidence of subjecthood.
        col.record_writing(knot, actor, "llm");
        assert_eq!(col.freedom(actor, wide_window()), 0.0);
        assert_eq!(col.authorship(actor, wide_window()), 0.0);
    }

    #[test]
    fn test_observer_never_subject() {
        let observer = Uuid::new_v4();
        let owner = Uuid::new_v4();
        let mut col = AuthorColumn::new();
        let knot = Uuid::new_v4();
        col.record_writing(knot, owner, "sensor");
        col.record_waking(knot, observer, "sensor");

        assert!(!col.has_authored(observer));
        assert!(col.has_authored(owner));
        assert!(col.is_observer(observer));
        // Observer reads (wakes) but never writes → not a subject by construction.
        assert!(observer_never_subject(col.has_authored(observer)));
    }

    #[test]
    fn test_statute_no0_self_consistent() {
        let no0 = StatuteNo0::new();
        assert!(no0.is_self_consistent());
        assert_eq!(no0.version, 1);
        assert!(no0.axiom.contains("quorum"));
    }
}
