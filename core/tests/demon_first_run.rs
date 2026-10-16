//! FIRST RUN — demon core end-to-end walk over an ARGUS-OS1 flavoured scenario.
//!
//! No hardware is required: this exercises the whole Demon Core pipeline
//! (Demon Core Harvest 2026-10-16, DCT01–DCT20) on a simulated centriole-
//! tracking anomaly, mirroring the real seed field
//! (`py_backend/daisocket/seed_centriole_field.json`):
//!
//!   1. trace → claim (PROPOSED, epoch 1)
//!   2. evidence/replication → SUPPORTED (epoch 2, non-destructive history)
//!   3. transaction membrane: self-serving rule delta is rejected
//!   4. dependency policy: challenged premise with a dependent and NO policy
//!      → lawful BLOCK + STOP CARD (NEED / MISSING_RULE)
//!   5. dependency policy present → transition passes
//!   6. Ed25519 journal signing + tamper detection + compaction w/ trace continuity
//!
//! Run: `cargo test --test demon_first_run`

use dais_core::event_store::{EventStore, JournalGuard};
use dais_core::noepedia::{
    CompositionVerdict, JustificationComposition, NeedClass,
};
use dais_core::validator::Validator;
use uuid::Uuid;

fn delta_create(store: &mut EventStore, target: Option<Uuid>, payload: serde_json::Value) -> Uuid {
    let pub_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let mut d = dais_core::delta::Delta::create(pub_id, author, Uuid::new_v4(), payload);
    d.target_id = target;
    store.append(&d)
}

#[test]
fn first_run_argus_os1_demon_pipeline() {
    let mut store = EventStore::new();

    // ── 1. ARGUS-OS1 anomaly trace → claim (PROPOSED) ─────────────────
    let p2 = Uuid::new_v4(); // "P2 phototoxicity ceiling" claim
    let author = Uuid::new_v4();
    let pub_id = Uuid::new_v4();
    let mut create = dais_core::delta::Delta::create(
        pub_id,
        author,
        Uuid::new_v4(),
        serde_json::json!({
            "type": "CLAIM",
            "subject": "P2 phototoxicity ceiling",
            "predicate": "requires",
            "object": "division_rate > 90% of dark control",
            "status": "PROPOSED",
            "valid_in": "C. elegans embryo, 25C, 488/561/405nm",
        }),
    );
    create.target_id = Some(p2);
    store.append(&create);

    // ── 2. Evidence → SUPPORTED (epoch 2; history kept) ────────────────
    let update = dais_core::delta::Delta::update(
        pub_id,
        author,
        Uuid::new_v4(),
        p2,
        serde_json::json!({
            "status": "SUPPORTED",
            "sources": ["Go/No-Go: rate <90% -> light-sheet (V8)", "sister-pair replication"],
        }),
    );
    store.append(&update);

    let state = store.replay();
    let obj = &state["objects"][p2.to_string()];
    assert_eq!(obj["current_epistemic_status"], serde_json::json!("SUPPORTED"));
    let assertions = obj["status_assertions"].as_array().unwrap();
    assert_eq!(assertions.len(), 2, "PROPOSED + SUPPORTED, history kept");
    assert_eq!(assertions[0]["status"], serde_json::json!("PROPOSED"));
    assert_eq!(assertions[0]["epoch"], serde_json::json!(1));
    assert_eq!(assertions[1]["status"], serde_json::json!("SUPPORTED"));
    assert_eq!(assertions[1]["epoch"], serde_json::json!(2));

    // ── 3. Transaction membrane: self-serving rule delta rejected ──────
    let validator = Validator::new(&store);
    let self_serving = dais_core::delta::Delta::create(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        serde_json::json!({
            "type": "RULE_CARD",
            "id": "R-SELF",
            "rule_id": "R-SELF",
            "validated_by_rule": "R-SELF",
        }),
    );
    let report = validator.validate(&self_serving);
    assert!(
        report.errors.iter().any(|e| e.code == "SELF_REFERENCE_BAN"),
        "membrane must reject self-authorization"
    );

    // ── 4. Dependency policy: challenged premise, dependent, NO policy ──
    let p6 = Uuid::new_v4(); // "P6 SAS-1 -> SAS-4 latency" claim (dependent)
    let mut dep = dais_core::delta::Delta::create(
        pub_id,
        author,
        Uuid::new_v4(),
        serde_json::json!({"type": "CLAIM", "subject": "P6 SAS-1 -> SAS-4 latency",
                           "status": "SUPPORTED"}),
    );
    dep.target_id = Some(p6);
    store.append(&dep);
    store.append(&dais_core::delta::Delta::relate(
        pub_id, author, Uuid::new_v4(), p6, p2, "supports",
    ));

    let validator = Validator::new(&store);
    let challenge = dais_core::delta::Delta::update(
        pub_id,
        author,
        Uuid::new_v4(),
        p2,
        serde_json::json!({"status": "CONTESTED"}),
    );
    let report = validator.validate(&challenge);
    assert!(
        report.errors.iter().any(|e| e.code == "MISSING_DEPENDENCY_POLICY"),
        "challenged premise with dependents and no policy must BLOCK"
    );
    assert!(!report.valid);

    // The demon does not invent the downstream consequence — it produces a
    // classified STOP CARD (NEED / MISSING_RULE).
    let card = validator.missing_policy_stop_card("PREMISE -> CONTESTED");
    assert_eq!(card.stop_class, NeedClass::MissingRule);
    assert!(!card.can_ask_outside, "NEED, not ASK");

    // ── 5. Dependency policy present → same transition passes ───────────
    let policy_id = delta_create(
        &mut store,
        None,
        serde_json::json!({
            "type": "DEPENDENCY_POLICY",
            "policy_id": "P-POLICY",
            "name": "premise conflict propagates",
            "upstream_transition": "PREMISE -> CONTESTED",
            "downstream_consequence": "CONTESTED",
            "role": "POLICY",
            "rule_ref": "E-P2P6",
        }),
    );
    let _ = store.archive(&[policy_id]); // archiving a policy is legal; stays addressable

    let validator = Validator::new(&store);
    let applicable = validator.find_applicable_policy("PREMISE -> CONTESTED");
    assert!(applicable.is_some(), "policy must be found after replay");
    assert_eq!(applicable.unwrap().role, "POLICY");

    let report = validator.validate(&challenge);
    assert!(
        !report.errors.iter().any(|e| e.code == "MISSING_DEPENDENCY_POLICY"),
        "applicable policy satisfies the dependency check"
    );

    // ── 6. Composition semantics (DCT19–DCT20): one path challenged ─────
    let verdict = dais_core::noepedia::evaluate_composition(
        &dais_core::noepedia::CompositionPolicy {
            policy_id: Uuid::new_v4(),
            result_id: Uuid::new_v4(),
            composition: JustificationComposition::Alternative,
            justification_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            scope: vec![],
        },
        &[false, true],
    );
    assert_eq!(verdict, CompositionVerdict::Supported);

    // ── 7. Ed25519 journal signing + tamper + compaction ────────────────
    let guard = JournalGuard::from_seed([42u8; 32]);
    let sig = store.sign_log(&guard).expect("signing key present");
    assert!(store.verify_log(&guard, &sig), "honest log verifies");

    // A signature is bound to the exact log: any change to the committed log
    // (a silent rewrite would be one such change) fails the old signature.
    // Here we simulate the same by verifying the signature over a log that
    // gained one extra committed entry — verification must fail.
    let mut grown = store.clone();
    delta_create(
        &mut grown,
        None,
        serde_json::json!({"type": "CLAIM", "status": "PROPOSED"}),
    );
    assert!(
        !grown.verify_log(&guard, &sig),
        "old signature must not cover a changed log (tamper-evident)"
    );

    // Compaction preserves addresses (TRACE CONTINUITY).
    let ids: Vec<Uuid> = store.entries().iter().map(|e| e.event_id).take(2).collect();
    store.compact(&ids, "coalesced early history").unwrap();
    assert!(store.check_trace_continuity().is_empty());

    // ── Final projection sanity ─────────────────────────────────────────
    let state = store.replay();
    assert_eq!(state["objects"][p2.to_string()]["status"], serde_json::json!("SUPPORTED"));
}
