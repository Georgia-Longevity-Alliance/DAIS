//! Anomaly Detection & Diagnosis — ARGUS-OS1 Integration
//!
//! Monitors Flight Recorder events for anomalies and generates
//! LLM-friendly diagnosis reports for automated resolution.
//!
//! # Simulated Failure Scenarios
//! 1. Laser Power Spike — 488nm laser exceeds firmware limit
//! 2. Stage Slip — XY stage loses position during timelapse
//! 3. Focus Drift — Z-focus degrades over long acquisition

use crate::types::{EventType, FlightEvent, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyCategory {
    LaserPowerSpike,
    StageSlip,
    FocusDrift,
    TemperatureExcursion,
    StorageFull,
    NetworkLoss,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: String,
    pub severity: crate::types::Severity,
    pub category: AnomalyCategory,
    pub description: String,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyThresholds {
    pub max_laser_power_mw: f64,
    pub max_temperature_c: f64,
    pub max_focus_drift_um_per_hour: f64,
    pub max_free_space_percent: f64,
}

impl Default for AnomalyThresholds {
    fn default() -> Self {
        Self {
            max_laser_power_mw: 10.0,
            max_temperature_c: 37.0,
            max_focus_drift_um_per_hour: 2.0,
            max_free_space_percent: 5.0,
        }
    }
}

pub struct AnomalyDetector {
    thresholds: AnomalyThresholds,
    counter: u64,
}

impl AnomalyDetector {
    pub fn new(thresholds: AnomalyThresholds) -> Self {
        Self { thresholds, counter: 0 }
    }

    pub fn scan(&mut self, events: &[FlightEvent]) -> Vec<Anomaly> {
        let mut batch = Vec::new();
        for event in events {
            if let Some(a) = self.check(event) {
                batch.push(a);
            }
        }
        batch
    }

    fn check(&mut self, event: &FlightEvent) -> Option<Anomaly> {
        let p = &event.payload;

        if event.event_type == EventType::SensorReading {
            if let Some(power) = p.get("laser_power_mw").and_then(|v| v.as_f64()) {
                if power > self.thresholds.max_laser_power_mw {
                    return Some(self.make(AnomalyCategory::LaserPowerSpike,
                        format!("Laser {:.1}mW > {:.1}mW limit", power, self.thresholds.max_laser_power_mw),
                        "Reduce laser power. Check PWM signal."));
                }
            }
            if let Some(temp) = p.get("temperature_c").and_then(|v| v.as_f64()) {
                if temp > self.thresholds.max_temperature_c {
                    return Some(self.make(AnomalyCategory::TemperatureExcursion,
                        format!("Temp {:.1}°C > {:.1}°C limit", temp, self.thresholds.max_temperature_c),
                        "Check cooling fan. Reduce laser duty cycle."));
                }
            }
            if let Some(drift) = p.get("focus_drift_um_per_h").and_then(|v| v.as_f64()) {
                if drift > self.thresholds.max_focus_drift_um_per_hour {
                    return Some(self.make(AnomalyCategory::FocusDrift,
                        format!("Focus drift {:.1}µm/h > {:.1}µm/h", drift, self.thresholds.max_focus_drift_um_per_hour),
                        "Re-run autofocus. Check environment temp stability."));
                }
            }
        }

        if event.event_type == EventType::Anomaly {
            if let Some(desc) = p.get("description").and_then(|v| v.as_str()) {
                if desc.contains("stage") || desc.contains("slip") {
                    return Some(self.make(AnomalyCategory::StageSlip, desc.into(),
                        "Stop stage. Check obstruction. Re-home."));
                }
            }
        }

        if event.event_type == EventType::SensorReading {
            if let Some(pct) = p.get("free_space_percent").and_then(|v| v.as_f64()) {
                if pct < self.thresholds.max_free_space_percent {
                    return Some(self.make(AnomalyCategory::StorageFull,
                        format!("Storage {:.1}% free < {:.1}% threshold", pct, self.thresholds.max_free_space_percent),
                        "Offload data to server."));
                }
            }
        }

        None
    }

    fn make(&mut self, category: AnomalyCategory, description: String, action: &str) -> Anomaly {
        self.counter += 1;
        Anomaly {
            id: format!("ANOM-{:04}", self.counter),
            severity: match category {
                AnomalyCategory::LaserPowerSpike | AnomalyCategory::TemperatureExcursion => Severity::Critical,
                _ => Severity::Warning,
            },
            category,
            description,
            suggested_action: action.into(),
        }
    }
}

/// Simulated failure scenarios for testing
pub struct FailureSimulator;

impl FailureSimulator {
    pub fn simulate_laser_spike() -> Vec<FlightEvent> {
        use chrono::Utc;
        use uuid::Uuid;
        let t = Utc::now();
        vec![
            FlightEvent {
                event_id: Uuid::new_v4(), timestamp: t,
                event_type: EventType::SensorReading, severity: Severity::Info,
                payload: serde_json::json!({"laser_power_mw": 23.5, "channel": "488nm", "expected": 10.0}),
            },
            FlightEvent {
                event_id: Uuid::new_v4(), timestamp: t,
                event_type: EventType::SensorReading, severity: Severity::Warning,
                payload: serde_json::json!({"temperature_c": 35.2, "location": "laser_driver", "trend": "rising"}),
            },
        ]
    }

    pub fn simulate_stage_slip() -> Vec<FlightEvent> {
        vec![
            FlightEvent {
                event_id: uuid::Uuid::new_v4(), timestamp: chrono::Utc::now(),
                event_type: EventType::Anomaly, severity: Severity::Warning,
                payload: serde_json::json!({"description": "stage slip detected on X axis, displacement 2500µm", "axis": "X", "motor": "TMC2209_1"}),
            },
        ]
    }

    pub fn simulate_focus_drift() -> Vec<FlightEvent> {
        vec![
            FlightEvent {
                event_id: uuid::Uuid::new_v4(), timestamp: chrono::Utc::now(),
                event_type: EventType::SensorReading, severity: Severity::Warning,
                payload: serde_json::json!({"focus_drift_um_per_h": 3.8, "duration_h": 8.0, "total_drift_um": 30.4}),
            },
        ]
    }

    pub fn all_scenarios() -> Vec<(String, Vec<FlightEvent>)> {
        vec![
            ("Laser Spike".into(), Self::simulate_laser_spike()),
            ("Stage Slip".into(), Self::simulate_stage_slip()),
            ("Focus Drift".into(), Self::simulate_focus_drift()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_laser_spike() {
        let mut d = AnomalyDetector::new(AnomalyThresholds::default());
        let a = d.scan(&FailureSimulator::simulate_laser_spike());
        assert!(a.iter().any(|x| x.category == AnomalyCategory::LaserPowerSpike));
        assert!(a.iter().any(|x| x.severity == Severity::Critical));
    }

    #[test]
    fn test_detect_stage_slip() {
        let mut d = AnomalyDetector::new(AnomalyThresholds::default());
        let a = d.scan(&FailureSimulator::simulate_stage_slip());
        assert!(a.iter().any(|x| x.category == AnomalyCategory::StageSlip));
    }

    #[test]
    fn test_detect_focus_drift() {
        let mut d = AnomalyDetector::new(AnomalyThresholds::default());
        let a = d.scan(&FailureSimulator::simulate_focus_drift());
        assert!(a.iter().any(|x| x.category == AnomalyCategory::FocusDrift));
    }

    #[test]
    fn test_no_false_positives() {
        let mut d = AnomalyDetector::new(AnomalyThresholds::default());
        let normal = vec![FlightEvent {
            event_id: uuid::Uuid::new_v4(), timestamp: chrono::Utc::now(),
            event_type: EventType::SensorReading, severity: Severity::Info,
            payload: serde_json::json!({"laser_power_mw": 5.0, "temperature_c": 25.0}),
        }];
        assert!(d.scan(&normal).is_empty());
    }

    #[test]
    fn test_all_scenarios() {
        let scenarios = FailureSimulator::all_scenarios();
        assert_eq!(scenarios.len(), 3);
        let mut d = AnomalyDetector::new(AnomalyThresholds::default());
        let mut total = 0;
        for (_, events) in &scenarios {
            total += d.scan(events).len();
        }
        assert!(total >= 3);
    }
}
