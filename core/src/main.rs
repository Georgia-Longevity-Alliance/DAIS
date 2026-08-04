use dais_core::anomaly::{AnomalyDetector, AnomalyThresholds, FailureSimulator};
use dais_core::argus::{ArgusRegistry, ExperimentType, bootstrap_knowledge_claims};
use dais_core::proof::ProofChain;
use dais_core::ota::EnergyScheduler;
use dais_core::types::*;
use dais_core::flight_recorder::FlightRecorder;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  DAIS × ARGUS-OS1 — LIVE DEMO                 ║");
    println!("║  24hr C. elegans centriole tracking experiment ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    let mut detector = AnomalyDetector::new(AnomalyThresholds::default());
    let mut registry = ArgusRegistry::new();
    let mut proof_chain = ProofChain::new();
    let mut flight_rec = FlightRecorder::new(256);

    let session_id = registry.start_session(ExperimentType::CellDivisionTracking);
    println!("[00:00] 🟢 Session {} — 64 C. elegans embryos\n", &session_id);

    // Normal operation hours 0-5
    for hour in 1..=5 {
        flight_rec.record(EventType::SensorReading, Severity::Info,
            serde_json::json!({"laser_mw": 5.0, "temp_c": 25.0, "hour": hour}));
        proof_chain.issue("argus_os1", "acquire_z_stack", 0b1111111);
        thread::sleep(Duration::from_millis(150));
        if hour % 2 == 0 { println!("[{:02}:00] ✅ {} Z-stacks — nominal", hour, hour * 12); }
    }

    // LASER SPIKE
    println!("\n[05:30] ⚠️  LASER POWER SPIKE");
    let anomalies = detector.scan(&FailureSimulator::simulate_laser_spike());
    for a in &anomalies {
        println!("  🔴 {}: {} → {}", a.id, a.description, a.suggested_action);
        flight_rec.record_anomaly(&a.description, Severity::Critical);
    }
    proof_chain.issue("argus_os1", "laser_emergency_reduce", 0b1111111);
    if let Some(s) = registry.sessions.iter_mut().find(|s| s.session_id == session_id) {
        s.anomaly_count += anomalies.len() as u64;
    }
    println!("  ✅ Laser normalized — acquisition resumed");

    // FOCUS DRIFT
    println!("\n[14:20] ⚠️  FOCUS DRIFT");
    let anomalies = detector.scan(&FailureSimulator::simulate_focus_drift());
    for a in &anomalies {
        println!("  🟡 {}: {} → {}", a.id, a.description, a.suggested_action);
    }
    flight_rec.record(EventType::SensorReading, Severity::Warning,
        serde_json::json!({"autofix": "recalibrated", "quality_after": 0.87}));
    proof_chain.issue("argus_os1", "autofocus_recalibrate", 0b1111111);

    // STAGE SLIP
    println!("\n[20:45] ⚠️  STAGE SLIP");
    let anomalies = detector.scan(&FailureSimulator::simulate_stage_slip());
    for a in &anomalies {
        println!("  🔴 {}: {} → {}", a.id, a.description, a.suggested_action);
        flight_rec.record_anomaly(&a.description, Severity::Critical);
    }
    proof_chain.issue("argus_os1", "stage_emergency_stop", 0b1111111);
    proof_chain.issue("argus_os1", "stage_rehome", 0b1111111);

    // END
    println!("\n[24:00] 🟢 Experiment complete");
    if let Some(s) = registry.sessions.iter_mut().find(|s| s.session_id == session_id) {
        s.divisions_detected = 8;
        s.end();
        let claims = s.publish_findings();
        println!("  📊 Divisions: 8 | ⚠️ Anomalies: {} | 📝 Claims: {}", s.anomaly_count, claims.len());
    }

    // PROVEN
    println!("\n─── PROVEN KNOWLEDGE ───");
    for c in &bootstrap_knowledge_claims() {
        println!("  📚 {} {} {}", c.subject, c.predicate, c.object);
    }

    // SAFETY
    println!("\n─── SAFETY REPORT ───");
    println!("  🔐 Proofs: {} | 🔗 Chain: {} | ✅ Full validation: {}",
        proof_chain.proofs.len(),
        if proof_chain.verify_chain() { "VALID" } else { "TAMPERED" },
        proof_chain.count_full_validation());
    println!("  📋 Flight Recorder: {} events", flight_rec.len());

    // ENERGY
    let sched = EnergyScheduler::new(dais_core::ota::PowerBudget {
        device_id: "argus_os1".into(), battery_wh: 100.0, current_charge_wh: 15.0,
        charge_percent: 0.15, consumption_w: 8.0, estimated_runtime_hours: 1.875,
    });
    let task = dais_core::ota::EnergyTask {
        task_id: "next_z".into(), priority: 50, estimated_energy_wh: 5.0,
        deadline_hours: Some(8.0), can_delegate: false,
    };
    print!("  🔋 Battery 15% → next Z-stack: ");
    match sched.decide(&task, &[]) {
        dais_core::ota::TaskDecision::ExecuteNow => println!("Execute now"),
        dais_core::ota::TaskDecision::ExecuteLater { wait_hours } => println!("Delayed {}h", wait_hours),
        _ => println!("Delegated/Rejected"),
    }

    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║  ✅ DEMO COMPLETE — DAIS ready for ARGUS-OS1  ║");
    println!("╚══════════════════════════════════════════════════╝");
}
