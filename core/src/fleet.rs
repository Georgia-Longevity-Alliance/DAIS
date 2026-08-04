//! Fleet Manager — multi-device coordination for AIS
//!
//! Manages multiple DAISocket devices: registration, health monitoring,
//! resource allocation, and cross-device trace correlation.

use crate::types::{Id, Severity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A device in the DAIS fleet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetDevice {
    pub device_id: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub current_task: Option<String>,
    pub health: DeviceHealth,
    pub last_heartbeat: String,
    pub capabilities: Vec<String>,
    pub location: Option<DeviceLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceType {
    Microscope,
    RobotArm,
    MobileRobot,
    SensorArray,
    Actuator,
    Camera,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Maintenance,
    EmergencyStop,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHealth {
    pub cpu_temp_c: Option<f64>,
    pub memory_free_mb: Option<u64>,
    pub storage_free_mb: Option<u64>,
    pub battery_percent: Option<f64>,
    pub uptime_hours: f64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLocation {
    pub room: String,
    pub building: String,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
}

/// Fleet manager — the central authority for all devices
pub struct FleetManager {
    pub devices: HashMap<String, FleetDevice>,
    pub fleet_name: String,
    pub max_devices: usize,
}

impl FleetManager {
    pub fn new(fleet_name: &str, max_devices: usize) -> Self {
        Self {
            devices: HashMap::new(),
            fleet_name: fleet_name.into(),
            max_devices,
        }
    }

    /// Register a new device
    pub fn register(&mut self, device: FleetDevice) -> Result<(), String> {
        if self.devices.len() >= self.max_devices {
            return Err(format!("Fleet full: {}/{} devices", self.devices.len(), self.max_devices));
        }
        let id = device.device_id.clone();
        self.devices.insert(id, device);
        Ok(())
    }

    /// Get device by ID
    pub fn get(&self, device_id: &str) -> Option<&FleetDevice> {
        self.devices.get(device_id)
    }

    /// Update device health
    pub fn update_health(&mut self, device_id: &str, health: DeviceHealth) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.health = health;
            device.last_heartbeat = chrono::Utc::now().to_rfc3339();
            if device.status == DeviceStatus::Offline {
                device.status = DeviceStatus::Online;
            }
        }
    }

    /// Emergency stop — all devices OR specific device
    pub fn emergency_stop(&mut self, device_id: Option<&str>) -> Vec<String> {
        let mut stopped = Vec::new();
        if let Some(id) = device_id {
            if let Some(device) = self.devices.get_mut(id) {
                device.status = DeviceStatus::EmergencyStop;
                stopped.push(id.to_string());
            }
        } else {
            for (id, device) in &mut self.devices {
                device.status = DeviceStatus::EmergencyStop;
                stopped.push(id.clone());
            }
        }
        stopped
    }

    /// Resume after emergency stop
    pub fn resume(&mut self, device_id: &str) -> bool {
        if let Some(device) = self.devices.get_mut(device_id) {
            if device.status == DeviceStatus::EmergencyStop {
                device.status = DeviceStatus::Online;
                return true;
            }
        }
        false
    }

    /// Find available device by capability
    pub fn find_by_capability(&self, capability: &str) -> Vec<&FleetDevice> {
        self.devices.values()
            .filter(|d| d.status == DeviceStatus::Online && d.capabilities.iter().any(|c| c.contains(capability)))
            .collect()
    }

    /// Fleet health report
    pub fn health_report(&self) -> FleetHealthReport {
        let total = self.devices.len();
        let online = self.devices.values().filter(|d| d.status == DeviceStatus::Online).count();
        let emergency = self.devices.values().filter(|d| d.status == DeviceStatus::EmergencyStop).count();
        let degraded = self.devices.values().filter(|d| d.status == DeviceStatus::Degraded).count();
        
        FleetHealthReport {
            fleet_name: self.fleet_name.clone(),
            total_devices: total,
            online,
            offline: total - online,
            emergency_stop: emergency,
            degraded,
            avg_cpu_temp: self.devices.values()
                .filter_map(|d| d.health.cpu_temp_c)
                .sum::<f64>() / online.max(1) as f64,
            total_errors: self.devices.values().map(|d| d.health.error_count).sum(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetHealthReport {
    pub fleet_name: String,
    pub total_devices: usize,
    pub online: usize,
    pub offline: usize,
    pub emergency_stop: usize,
    pub degraded: usize,
    pub avg_cpu_temp: f64,
    pub total_errors: u64,
}

/// Safety Level — aligned with ISO 13482 / IEC 61508
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SafetyIntegrityLevel {
    SIL0,  // No safety requirement
    SIL1,  // Minor injury possible
    SIL2,  // Serious injury possible
    SIL3,  // Life-threatening
    SIL4,  // Catastrophic
}

/// Audit Trail — immutable compliance log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entry_id: String,
    pub timestamp: String,
    pub device_id: String,
    pub event_type: AuditEventType,
    pub description: String,
    pub operator_id: Option<String>,
    pub severity: Severity,
    pub hash: String,  // SHA-256 chain for immutability
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    DeviceRegistered,
    DeviceDeregistered,
    EmergencyStop,
    SafetyLimitExceeded,
    MaintenancePerformed,
    FirmwareUpdate,
    OperatorOverride,
    AnomalyDetected,
    AnomalyResolved,
}

/// Audit trail with hash chain integrity
pub struct AuditTrail {
    pub entries: Vec<AuditEntry>,
    last_hash: String,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self { entries: Vec::new(), last_hash: "0".repeat(64) }
    }

    pub fn record(&mut self, device_id: &str, event_type: AuditEventType, description: &str, operator: Option<&str>, severity: Severity) -> &AuditEntry {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&self.last_hash);
        hasher.update(device_id.as_bytes());
        hasher.update(description.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        
        let entry = AuditEntry {
            entry_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            device_id: device_id.into(),
            event_type,
            description: description.into(),
            operator_id: operator.map(|s| s.into()),
            severity,
            hash: hash.clone(),
        };
        
        self.last_hash = hash;
        self.entries.push(entry.clone());
        self.entries.last().unwrap()
    }

    /// Verify hash chain integrity
    pub fn verify(&self) -> bool {
        use sha2::{Sha256, Digest};
        let mut prev = "0".repeat(64);
        for entry in &self.entries {
            let mut hasher = Sha256::new();
            hasher.update(prev.as_bytes());
            hasher.update(entry.device_id.as_bytes());
            hasher.update(entry.description.as_bytes());
            if format!("{:x}", hasher.finalize()) != entry.hash {
                return false;
            }
            prev = entry.hash.clone();
        }
        true
    }
}

/// Hardware Abstraction Layer — unified device interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub platform: HardwarePlatform,
    pub safety_level: SafetyIntegrityLevel,
    pub has_physical_stop: bool,
    pub has_deadman_switch: bool,
    pub max_current_ma: f64,
    pub watchdog_timeout_ms: u32,
    pub comm_protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HardwarePlatform {
    Esp32,
    Arduino,
    RaspberryPi,
    JetsonOrin,
    X86_64,
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_registration() {
        let mut fm = FleetManager::new("test_lab", 10);
        let device = FleetDevice {
            device_id: "argus_01".into(),
            device_type: DeviceType::Microscope,
            status: DeviceStatus::Online,
            current_task: None,
            health: DeviceHealth {
                cpu_temp_c: Some(42.0), memory_free_mb: Some(2048),
                storage_free_mb: Some(50000), battery_percent: None,
                uptime_hours: 24.0, error_count: 0,
            },
            last_heartbeat: "now".into(),
            capabilities: vec!["z_stack_acquisition".into(), "autofocus".into()],
            location: Some(DeviceLocation { room: "Lab1".into(), building: "Main".into(), gps_lat: None, gps_lon: None }),
        };
        assert!(fm.register(device).is_ok());
        assert_eq!(fm.devices.len(), 1);
    }

    #[test]
    fn test_emergency_stop_all() {
        let mut fm = FleetManager::new("test", 10);
        for i in 0..3 {
            fm.register(FleetDevice {
                device_id: format!("dev_{}", i), device_type: DeviceType::RobotArm,
                status: DeviceStatus::Online, current_task: None,
                health: DeviceHealth { cpu_temp_c: None, memory_free_mb: None, storage_free_mb: None, battery_percent: None, uptime_hours: 1.0, error_count: 0 },
                last_heartbeat: "now".into(), capabilities: vec![],
                location: None,
            }).unwrap();
        }
        let stopped = fm.emergency_stop(None);
        assert_eq!(stopped.len(), 3);
        for id in &stopped {
            assert_eq!(fm.get(id).unwrap().status, DeviceStatus::EmergencyStop);
        }
    }

    #[test]
    fn test_find_by_capability() {
        let mut fm = FleetManager::new("test", 10);
        fm.register(FleetDevice {
            device_id: "cam_01".into(), device_type: DeviceType::Camera,
            status: DeviceStatus::Online, current_task: None,
            health: DeviceHealth { cpu_temp_c: None, memory_free_mb: None, storage_free_mb: None, battery_percent: None, uptime_hours: 1.0, error_count: 0 },
            last_heartbeat: "now".into(), capabilities: vec!["image_capture".into()],
            location: None,
        }).unwrap();
        let found = fm.find_by_capability("image");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_audit_trail_integrity() {
        let mut trail = AuditTrail::new();
        trail.record("argus_01", AuditEventType::EmergencyStop, "Laser overcurrent detected", Some("operator_1"), Severity::Critical);
        trail.record("argus_01", AuditEventType::AnomalyResolved, "Laser driver replaced", Some("operator_1"), Severity::Info);
        assert_eq!(trail.entries.len(), 2);
        assert!(trail.verify());
    }

    #[test]
    fn test_health_report() {
        let mut fm = FleetManager::new("lab", 10);
        fm.register(FleetDevice {
            device_id: "dev_1".into(), device_type: DeviceType::SensorArray,
            status: DeviceStatus::Online, current_task: None,
            health: DeviceHealth { cpu_temp_c: Some(40.0), memory_free_mb: Some(1000), storage_free_mb: Some(10000), battery_percent: None, uptime_hours: 10.0, error_count: 2 },
            last_heartbeat: "now".into(), capabilities: vec![],
            location: None,
        }).unwrap();
        let report = fm.health_report();
        assert_eq!(report.online, 1);
        assert_eq!(report.total_errors, 2);
    }
}
