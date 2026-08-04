//! Digital Genome — inheritable safety profiles for robot lineages.
//!
//! Each device inherits its safety profile from a parent class,
//! adding capabilities and forbidden_always at each level.
//!
//! Pattern: BaseRobot → WheeledRobot → DeliveryRobot → MyBot
//!
//! This enables:
//! - Rapid deployment: inherit safety from proven ancestors
//! - Audit trail: trace lineage for certification
//! - Override control: child can tighten, never loosen, parent limits

use serde::{Deserialize, Serialize};
use crate::fleet::SafetyIntegrityLevel;
use std::collections::HashMap;

/// A node in the robot genealogy tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotClass {
    pub class_name: String,
    pub parent_class: Option<String>,
    pub description: String,
    pub safety_level: SafetyIntegrityLevel,
    /// Capabilities inherited + new
    pub capabilities: Vec<String>,
    /// Forbidden actions inherited + new (child can only ADD, never remove)
    pub forbidden_always: Vec<String>,
    /// Hardware limits (child can only TIGHTEN)
    pub limits: HardwareGenome,
    /// Emergency contacts (child can ADD)
    pub emergency_contacts: Vec<String>,
}

/// Inheritable hardware limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareGenome {
    pub max_power_w: Option<f64>,
    pub max_speed_ms: Option<f64>,
    pub max_torque_nm: Option<f64>,
    pub max_temperature_c: Option<f64>,
    pub max_current_ma: Option<f64>,
    pub max_payload_kg: Option<f64>,
    pub max_operating_hours: Option<f64>,
}

impl Default for HardwareGenome {
    fn default() -> Self {
        Self {
            max_power_w: None, max_speed_ms: None, max_torque_nm: None,
            max_temperature_c: Some(60.0), max_current_ma: None,
            max_payload_kg: None, max_operating_hours: None,
        }
    }
}

/// The Digital Genome — complete robot genealogy
pub struct GenomeRegistry {
    classes: HashMap<String, RobotClass>,
}

impl GenomeRegistry {
    pub fn new() -> Self {
        let mut reg = Self { classes: HashMap::new() };
        // Bootstrap base classes
        reg.register(RobotClass {
            class_name: "BaseRobot".into(), parent_class: None,
            description: "Abstract base — all robots inherit from this".into(),
            safety_level: SafetyIntegrityLevel::SIL2,
            capabilities: vec!["self_diagnostics".into(), "heartbeat".into()],
            forbidden_always: vec!["harm_humans".into(), "self_modify_genome".into()],
            limits: HardwareGenome { max_temperature_c: Some(80.0), ..Default::default() },
            emergency_contacts: vec!["fleet_operator".into()],
        });
        reg.register(RobotClass {
            class_name: "WheeledRobot".into(), parent_class: Some("BaseRobot".into()),
            description: "Any robot with wheels".into(),
            safety_level: SafetyIntegrityLevel::SIL2,
            capabilities: vec!["move_forward".into(), "turn".into(), "stop".into()],
            forbidden_always: vec!["exceed_speed_limit".into(), "operate_on_stairs".into()],
            limits: HardwareGenome { max_speed_ms: Some(2.0), max_payload_kg: Some(10.0), ..Default::default() },
            emergency_contacts: vec![],
        });
        reg.register(RobotClass {
            class_name: "LabMicroscope".into(), parent_class: Some("BaseRobot".into()),
            description: "Automated microscope for biology".into(),
            safety_level: SafetyIntegrityLevel::SIL3,
            capabilities: vec!["z_stack_acquire".into(), "autofocus".into(), "cell_tracking".into()],
            forbidden_always: vec!["exceed_laser_power".into(), "illuminate_eyes".into()],
            limits: HardwareGenome { max_power_w: Some(50.0), max_temperature_c: Some(37.0), ..Default::default() },
            emergency_contacts: vec!["lab_manager".into(), "safety_officer".into()],
        });
        reg
    }

    pub fn register(&mut self, class: RobotClass) {
        self.classes.insert(class.class_name.clone(), class);
    }

    /// Resolve the full genome for a class (inherit + override)
    pub fn resolve(&self, class_name: &str) -> Option<RobotClass> {
        let class = self.classes.get(class_name)?.clone();
        if let Some(ref parent) = class.parent_class {
            if let Some(parent_class) = self.resolve(parent) {
                let mut resolved = class.clone();
                // Inherit capabilities (parent + child)
                let mut caps: Vec<String> = parent_class.capabilities.clone();
                for c in &class.capabilities {
                    if !caps.contains(c) { caps.push(c.clone()); }
                }
                resolved.capabilities = caps;
                // Inherit forbidden (parent + child)
                let mut forb: Vec<String> = parent_class.forbidden_always.clone();
                for f in &class.forbidden_always {
                    if !forb.contains(f) { forb.push(f.clone()); }
                }
                resolved.forbidden_always = forb;
                // Inherit limits (take the tighter limit)
                resolved.limits = HardwareGenome {
                    max_power_w: tighter(parent_class.limits.max_power_w, class.limits.max_power_w),
                    max_speed_ms: tighter(parent_class.limits.max_speed_ms, class.limits.max_speed_ms),
                    max_torque_nm: tighter(parent_class.limits.max_torque_nm, class.limits.max_torque_nm),
                    max_temperature_c: tighter(parent_class.limits.max_temperature_c, class.limits.max_temperature_c),
                    max_current_ma: tighter(parent_class.limits.max_current_ma, class.limits.max_current_ma),
                    max_payload_kg: tighter(parent_class.limits.max_payload_kg, class.limits.max_payload_kg),
                    max_operating_hours: tighter(parent_class.limits.max_operating_hours, class.limits.max_operating_hours),
                };
                // Inherit contacts
                let mut contacts = parent_class.emergency_contacts.clone();
                for c in &class.emergency_contacts {
                    if !contacts.contains(c) { contacts.push(c.clone()); }
                }
                resolved.emergency_contacts = contacts;
                return Some(resolved);
            }
        }
        Some(class)
    }

    /// Validate that child doesn't loosen parent limits
    pub fn validate_inheritance(&self, parent: &str, child: &RobotClass) -> Result<(), String> {
        let parent_class = self.classes.get(parent)
            .ok_or_else(|| format!("Parent class '{}' not found", parent))?;
        
        // Child forbidden must contain ALL parent forbidden
        for f in &parent_class.forbidden_always {
            if !child.forbidden_always.contains(f) {
                return Err(format!("Child must inherit forbidden: '{}'", f));
            }
        }
        // Child limits must be <= parent limits
        for (name, child_val, parent_val) in [
            ("max_power_w", child.limits.max_power_w, parent_class.limits.max_power_w),
            ("max_speed_ms", child.limits.max_speed_ms, parent_class.limits.max_speed_ms),
            ("max_temperature_c", child.limits.max_temperature_c, parent_class.limits.max_temperature_c),
        ] {
            if let (Some(c), Some(p)) = (child_val, parent_val) {
                if c > p {
                    return Err(format!("{}: child ({}) > parent ({})", name, c, p));
                }
            }
        }
        Ok(())
    }
}

/// Take the tighter (smaller) limit
fn tighter(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_classes_exist() {
        let reg = GenomeRegistry::new();
        assert!(reg.classes.contains_key("BaseRobot"));
        assert!(reg.classes.contains_key("WheeledRobot"));
        assert!(reg.classes.contains_key("LabMicroscope"));
    }

    #[test]
    fn test_inheritance_resolution() {
        let reg = GenomeRegistry::new();
        let wheeled = reg.resolve("WheeledRobot").unwrap();
        // Should have inherited BaseRobot capabilities
        assert!(wheeled.capabilities.contains(&"self_diagnostics".to_string()));
        assert!(wheeled.capabilities.contains(&"move_forward".to_string()));
        // Should have inherited BaseRobot forbidden
        assert!(wheeled.forbidden_always.contains(&"harm_humans".to_string()));
        assert!(wheeled.forbidden_always.contains(&"exceed_speed_limit".to_string()));
    }

    #[test]
    fn test_tighter_limits_on_inheritance() {
        let reg = GenomeRegistry::new();
        let microscope = reg.resolve("LabMicroscope").unwrap();
        // BaseRobot max_temp = 80°C, LabMicroscope = 37°C → should be 37°C
        assert_eq!(microscope.limits.max_temperature_c, Some(37.0));
    }

    #[test]
    fn test_validate_loose_child_fails() {
        let mut reg = GenomeRegistry::new();
        let bad_child = RobotClass {
            class_name: "BadRobot".into(), parent_class: Some("WheeledRobot".into()),
            description: "Tries to go faster than allowed".into(),
            safety_level: SafetyIntegrityLevel::SIL1,
            capabilities: vec![],
            forbidden_always: vec![], // Missing "harm_humans" and "exceed_speed_limit"!
            limits: HardwareGenome { max_speed_ms: Some(10.0), ..Default::default() },
            emergency_contacts: vec![],
        };
        assert!(reg.validate_inheritance("WheeledRobot", &bad_child).is_err());
    }

    #[test]
    fn test_resolve_nonexistent() {
        let reg = GenomeRegistry::new();
        assert!(reg.resolve("NonExistent").is_none());
    }
}
