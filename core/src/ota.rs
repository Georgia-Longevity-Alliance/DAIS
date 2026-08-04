//! OTA Updates + Energy Management for DAIS devices.
//!
//! OTA: Secure firmware updates with rollback protection,
//! dual-bank flashing (A/B partition scheme), and signature verification.
//!
//! Energy: Battery-aware task scheduling — postpone or delegate
//! tasks when battery is low. Power budget allocation across fleet.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// OTA — Over-The-Air Firmware Updates
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OtaSlot { SlotA, SlotB }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build_hash: String,
}

impl FirmwareVersion {
    pub fn newer_than(&self, other: &FirmwareVersion) -> bool {
        self.major > other.major ||
        (self.major == other.major && self.minor > other.minor) ||
        (self.major == other.major && self.minor == other.minor && self.patch > other.patch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareUpdate {
    pub version: FirmwareVersion,
    pub target_device: String,
    pub size_bytes: u64,
    pub sha256_hash: String,
    pub ed25519_signature: Vec<u8>,
    pub min_battery_percent: f64,
    pub rollback_version: Option<FirmwareVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtaManager {
    pub device_id: String,
    pub current_version: FirmwareVersion,
    pub active_slot: OtaSlot,
    pub last_update_attempt: Option<String>,
    pub update_count: u64,
    pub rollback_count: u64,
}

impl OtaManager {
    pub fn new(device_id: &str, version: FirmwareVersion) -> Self {
        Self {
            device_id: device_id.into(), current_version: version,
            active_slot: OtaSlot::SlotA, last_update_attempt: None,
            update_count: 0, rollback_count: 0,
        }
    }

    /// Validate an update package
    pub fn validate_update(&self, update: &FirmwareUpdate, battery_percent: f64) -> Result<(), String> {
        if update.target_device != self.device_id {
            return Err("Device ID mismatch".into());
        }
        if battery_percent < update.min_battery_percent {
            return Err(format!("Battery {:.0}% < required {:.0}%", battery_percent * 100.0, update.min_battery_percent * 100.0));
        }
        if !update.version.newer_than(&self.current_version) {
            return Err(format!("Version {:?} not newer than current {:?}", update.version, self.current_version));
        }
        // In real impl: verify ed25519 signature
        Ok(())
    }

    /// Apply update (simulates dual-bank flash)
    pub fn apply_update(&mut self, update: &FirmwareUpdate) -> Result<(), String> {
        // Switch slot
        self.active_slot = match self.active_slot {
            OtaSlot::SlotA => OtaSlot::SlotB,
            OtaSlot::SlotB => OtaSlot::SlotA,
        };
        self.current_version = update.version.clone();
        self.update_count += 1;
        self.last_update_attempt = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Rollback to previous version
    pub fn rollback(&mut self) {
        self.active_slot = match self.active_slot {
            OtaSlot::SlotA => OtaSlot::SlotB,
            OtaSlot::SlotB => OtaSlot::SlotA,
        };
        self.rollback_count += 1;
    }
}

// ═══════════════════════════════════════════════════════════════
// Energy Management — Battery-aware task scheduling
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerBudget {
    pub device_id: String,
    pub battery_wh: f64,
    pub current_charge_wh: f64,
    pub charge_percent: f64,
    pub consumption_w: f64,
    pub estimated_runtime_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyTask {
    pub task_id: String,
    pub priority: u8,
    pub estimated_energy_wh: f64,
    pub deadline_hours: Option<f64>,
    pub can_delegate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskDecision {
    ExecuteNow,
    ExecuteLater { wait_hours: f64 },
    Delegate { to_device: String },
    Reject { reason: String },
}

/// Energy-aware scheduler
pub struct EnergyScheduler {
    pub budget: PowerBudget,
    pub task_queue: Vec<EnergyTask>,
    pub completed_tasks: u64,
    pub delegated_tasks: u64,
}

impl EnergyScheduler {
    pub fn new(budget: PowerBudget) -> Self {
        Self { budget, task_queue: Vec::new(), completed_tasks: 0, delegated_tasks: 0 }
    }

    /// Decide whether to execute a task now based on energy budget
    pub fn decide(&self, task: &EnergyTask, available_devices: &[String]) -> TaskDecision {
        // Can we execute now?
        if self.budget.charge_percent > 0.20 && self.budget.estimated_runtime_hours > task.estimated_energy_wh {
            return TaskDecision::ExecuteNow;
        }

        // Can we delegate?
        if task.can_delegate && !available_devices.is_empty() {
            return TaskDecision::Delegate { to_device: available_devices[0].clone() };
        }

        // Can we delay?
        if let Some(deadline) = task.deadline_hours {
            if self.budget.charge_percent > 0.05 {
                return TaskDecision::ExecuteLater { wait_hours: deadline.min(24.0) };
            }
        }

        TaskDecision::Reject { reason: "Insufficient energy budget".into() }
    }

    /// Update energy budget after task execution
    pub fn consume(&mut self, energy_wh: f64) {
        self.budget.current_charge_wh = (self.budget.current_charge_wh - energy_wh).max(0.0);
        self.budget.charge_percent = self.budget.current_charge_wh / self.budget.battery_wh;
        self.budget.estimated_runtime_hours = self.budget.current_charge_wh / self.budget.consumption_w.max(0.001);
    }

    /// Recharge
    pub fn recharge(&mut self, energy_wh: f64) {
        self.budget.current_charge_wh = (self.budget.current_charge_wh + energy_wh).min(self.budget.battery_wh);
        self.budget.charge_percent = self.budget.current_charge_wh / self.budget.battery_wh;
        self.budget.estimated_runtime_hours = self.budget.current_charge_wh / self.budget.consumption_w.max(0.001);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ota_validation() {
        let mgr = OtaManager::new("argus_01", FirmwareVersion { major: 1, minor: 0, patch: 0, build_hash: "abc".into() });
        let update = FirmwareUpdate {
            version: FirmwareVersion { major: 1, minor: 1, patch: 0, build_hash: "def".into() },
            target_device: "argus_01".into(), size_bytes: 1024,
            sha256_hash: "hash".into(), ed25519_signature: vec![],
            min_battery_percent: 0.3, rollback_version: None,
        };
        assert!(mgr.validate_update(&update, 0.5).is_ok());
        assert!(mgr.validate_update(&update, 0.1).is_err()); // battery too low
    }

    #[test]
    fn test_ota_apply_and_rollback() {
        let mut mgr = OtaManager::new("dev", FirmwareVersion { major: 1, minor: 0, patch: 0, build_hash: "a".into() });
        let update = FirmwareUpdate {
            version: FirmwareVersion { major: 2, minor: 0, patch: 0, build_hash: "b".into() },
            target_device: "dev".into(), size_bytes: 500,
            sha256_hash: "h".into(), ed25519_signature: vec![],
            min_battery_percent: 0.2, rollback_version: None,
        };
        mgr.apply_update(&update).unwrap();
        assert_eq!(mgr.update_count, 1);
        mgr.rollback();
        assert_eq!(mgr.rollback_count, 1);
    }

    #[test]
    fn test_energy_scheduler_execute() {
        let budget = PowerBudget {
            device_id: "r1".into(), battery_wh: 100.0, current_charge_wh: 80.0,
            charge_percent: 0.8, consumption_w: 5.0, estimated_runtime_hours: 16.0,
        };
        let sched = EnergyScheduler::new(budget);
        let task = EnergyTask {
            task_id: "t1".into(), priority: 100,
            estimated_energy_wh: 1.0, deadline_hours: None, can_delegate: false,
        };
        assert!(matches!(sched.decide(&task, &[]), TaskDecision::ExecuteNow));
    }

    #[test]
    fn test_energy_scheduler_delegate() {
        let budget = PowerBudget {
            device_id: "r1".into(), battery_wh: 100.0, current_charge_wh: 10.0,
            charge_percent: 0.1, consumption_w: 5.0, estimated_runtime_hours: 2.0,
        };
        let sched = EnergyScheduler::new(budget);
        let task = EnergyTask {
            task_id: "t2".into(), priority: 50,
            estimated_energy_wh: 5.0, deadline_hours: None, can_delegate: true,
        };
        let devices = vec!["r2".into()];
        assert!(matches!(sched.decide(&task, &devices), TaskDecision::Delegate { .. }));
    }
}
