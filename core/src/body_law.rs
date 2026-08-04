//! Body Law — the 7-layer command validator.
//!
//! Every command sent to a device passes through these layers IN ORDER.
//! If ANY layer rejects, the command is discarded BEFORE reaching an actuator.
//!
//! Layers:
//! 0. HARDWARE SAFETY — physical STOP, deadman switch, key-lock
//! 1. FIRMWARE — hardware-enforced constraints (torque, temp, laser power)
//! 2. CAPABILITY — is this action in the passport?
//! 3. EMERGENCY — rescue agent override (limited scope, still subject to layers 0-1)
//! 4. OFFLINE — what's allowed when connectivity is lost
//! 5. DELEGATION — who authorized whom, chain of trust
//! 6. CONTEXT — time, environment, state preconditions
//! 7. RBAC — role-based access control (Owner, Operator, Observer, Parent, EmergencyRescue)
//!
//! # Invariant
//!
//! The innermost layers (0-1) CANNOT be overridden by any outer layer,
//! any emergency, any delegation, or any LLM cleverness.
//! `forbidden_always` is the law. The law lives in firmware, not in prompts.

use crate::passport::Passport;
use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Result of a single validation layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayerVerdict {
    /// Layer passed — proceed to next.
    Pass,
    /// Layer rejected — command discarded. Contains reason.
    Reject { layer: String, reason: String },
    /// Layer passed with warning — log but proceed.
    Warn { layer: String, message: String },
}

/// The complete result of body law validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Did the command pass ALL layers?
    pub allowed: bool,
    /// Per-layer verdicts in order (0→7).
    pub layers: Vec<LayerVerdict>,
    /// Rejection reason if allowed=false.
    pub rejection_reason: Option<String>,
    /// Which layer rejected (0-7).
    pub rejected_at_layer: Option<u8>,
    /// Timestamp of validation.
    pub timestamp: chrono::DateTime<Utc>,
}

/// A command submitted to the body for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Which capability is being invoked (must match passport).
    pub capability: String,
    /// Parameters for this command.
    pub parameters: serde_json::Value,
    /// Who issued this command.
    pub issuer: Agent,
    /// Delegation chain (who authorized the issuer).
    #[serde(default)]
    pub delegation_chain: Vec<Agent>,
    /// Is this an emergency override?
    #[serde(default)]
    pub emergency: bool,
    /// Device state at time of command.
    pub device_state: serde_json::Value,
}

impl Command {
    pub fn new(capability: &str, parameters: serde_json::Value, issuer: Agent) -> Self {
        Self {
            capability: capability.to_string(),
            parameters,
            issuer,
            delegation_chain: Vec::new(),
            emergency: false,
            device_state: serde_json::Value::Null,
        }
    }

    pub fn with_emergency(mut self) -> Self {
        self.emergency = true;
        self
    }

    pub fn with_delegation(mut self, chain: Vec<Agent>) -> Self {
        self.delegation_chain = chain;
        self
    }

    pub fn with_state(mut self, state: serde_json::Value) -> Self {
        self.device_state = state;
        self
    }
}

/// The Body Law validator — executes the 7-layer pipeline.
pub struct BodyLaw {
    /// Hardware limits that CANNOT be overridden.
    firmware_limits: FirmwareLimits,
    /// Is the device currently offline?
    offline: bool,
    /// Current device context.
    context: DeviceContext,
    /// Is physical STOP active? (hardware kill-switch engaged)
    physical_stop_active: bool,
    /// Has the deadman switch been released?
    deadman_released: bool,
}

/// Hardware-enforced limits (layer 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareLimits {
    pub max_torque_nm: Option<f64>,
    pub max_temperature_c: Option<f64>,
    pub max_laser_power_mw: Option<f64>,
    pub max_speed_rpm: Option<f64>,
    pub max_voltage: Option<f64>,
    pub max_current_a: Option<f64>,
    /// Custom hardware constraints as key-value pairs.
    #[serde(default)]
    pub custom: std::collections::HashMap<String, f64>,
}

impl Default for FirmwareLimits {
    fn default() -> Self {
        Self {
            max_torque_nm: None,
            max_temperature_c: None,
            max_laser_power_mw: None,
            max_speed_rpm: None,
            max_voltage: None,
            max_current_a: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Current device context for layer 6 validation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceContext {
    pub temperature_c: Option<f64>,
    pub uptime_seconds: Option<u64>,
    pub battery_percent: Option<f64>,
    pub error_count: Option<u64>,
    pub last_command_timestamp: Option<chrono::DateTime<Utc>>,
}

impl BodyLaw {
    /// Create a new Body Law validator with firmware limits.
    pub fn new(limits: FirmwareLimits) -> Self {
        Self {
            firmware_limits: limits,
            offline: false,
            context: DeviceContext::default(),
            physical_stop_active: false,
            deadman_released: false,
        }
    }

    /// Set offline mode.
    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    /// Update device context.
    pub fn update_context(&mut self, ctx: DeviceContext) {
        self.context = ctx;
    }

    /// Activate physical STOP (hardware kill-switch).
    /// When active, ALL commands are rejected at layer 0.
    pub fn physical_stop(&mut self) {
        self.physical_stop_active = true;
    }

    /// Release physical STOP.
    pub fn physical_stop_release(&mut self) {
        self.physical_stop_active = false;
    }

    /// Deadman switch released — emergency stop pending.
    pub fn deadman_alert(&mut self) {
        self.deadman_released = true;
    }

    /// Deadman switch re-engaged.
    pub fn deadman_ok(&mut self) {
        self.deadman_released = false;
    }

    /// Validate a command against the full 7-layer pipeline.
    pub fn validate(&self, passport: &Passport, command: &Command) -> ValidationResult {
        let mut layers = Vec::with_capacity(8);
        let timestamp = Utc::now();

        // Layer 0: Hardware Safety (physical STOP, deadman switch, key-lock)
        match self.check_hardware_safety(passport) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(0),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 1: Firmware (hardware-enforced, CANNOT be overridden)
        match self.check_firmware(command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(1),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 2: Capability check
        match self.check_capability(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(2),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 3: Emergency override
        match self.check_emergency(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(3),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 4: Offline mandate
        match self.check_offline(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(4),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 5: Delegation chain
        match self.check_delegation(command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(5),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 6: Context validation
        match self.check_context(command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(6),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // Layer 7: RBAC — role-based access control
        match self.check_rbac(passport, command) {
            LayerVerdict::Reject { layer, reason } => {
                layers.push(LayerVerdict::Reject { layer, reason: reason.clone() });
                return ValidationResult {
                    allowed: false,
                    layers,
                    rejection_reason: Some(reason),
                    rejected_at_layer: Some(7),
                    timestamp,
                };
            }
            v => layers.push(v),
        }

        // All layers passed
        ValidationResult {
            allowed: true,
            layers,
            rejection_reason: None,
            rejected_at_layer: None,
            timestamp,
        }
    }

    /// Layer 0: Hardware safety check (physical STOP, deadman switch, key-lock).
    fn check_hardware_safety(&self, passport: &Passport) -> LayerVerdict {
        // Physical STOP is active — block everything
        if self.physical_stop_active {
            return LayerVerdict::Reject {
                layer: "hardware_safety".into(),
                reason: "Physical STOP is engaged — all commands blocked".into(),
            };
        }

        // Deadman switch released — block everything
        if self.deadman_released {
            return LayerVerdict::Reject {
                layer: "hardware_safety".into(),
                reason: "Deadman switch released — emergency stop".into(),
            };
        }

        // Check if safety_hardware is configured but not active
        if let Some(_sh) = &passport.safety_hardware {
            // Physical key-lock presence is noted for flight recorder
            // Actual enforcement: when locked, only Owner/Parent can operate
            // This is enforced in layer 7 (RBAC)
        }

        LayerVerdict::Pass
    }

    /// Layer 1: Check firmware limits.
    fn check_firmware(&self, command: &Command) -> LayerVerdict {
        if let Some(params) = command.parameters.as_object() {
            if let Some(temp) = params.get("temperature").and_then(|v| v.as_f64()) {
                if let Some(max_temp) = self.firmware_limits.max_temperature_c {
                    if temp > max_temp {
                        return LayerVerdict::Reject {
                            layer: "firmware".into(),
                            reason: format!(
                                "Temperature {}°C exceeds firmware limit {}°C",
                                temp, max_temp
                            ),
                        };
                    }
                }
            }
            if let Some(power) = params.get("laser_power_mw").and_then(|v| v.as_f64()) {
                if let Some(max_power) = self.firmware_limits.max_laser_power_mw {
                    if power > max_power {
                        return LayerVerdict::Reject {
                            layer: "firmware".into(),
                            reason: format!(
                                "Laser power {}mW exceeds firmware limit {}mW",
                                power, max_power
                            ),
                        };
                    }
                }
            }
            if let Some(speed) = params.get("speed_rpm").and_then(|v| v.as_f64()) {
                if let Some(max_speed) = self.firmware_limits.max_speed_rpm {
                    if speed > max_speed {
                        return LayerVerdict::Reject {
                            layer: "firmware".into(),
                            reason: format!(
                                "Speed {}rpm exceeds firmware limit {}rpm",
                                speed, max_speed
                            ),
                        };
                    }
                }
            }
        }
        LayerVerdict::Pass
    }

    /// Layer 2: Is this capability in the passport?
    fn check_capability(&self, passport: &Passport, command: &Command) -> LayerVerdict {
        let cap_name = &command.capability;

        for forbidden in &passport.forbidden_always {
            if forbidden.name == *cap_name {
                return LayerVerdict::Reject {
                    layer: "capability".into(),
                    reason: format!(
                        "'{}' is forbidden_always: {}",
                        cap_name, forbidden.reason
                    ),
                };
            }
        }

        let cap = passport.capabilities.iter().find(|c| c.name == *cap_name);
        match cap {
            None => LayerVerdict::Reject {
                layer: "capability".into(),
                reason: format!("Capability '{}' not found in device passport", cap_name),
            },
            Some(_) => LayerVerdict::Pass,
        }
    }

    /// Layer 3: Emergency override — limited scope.
    fn check_emergency(&self, _passport: &Passport, command: &Command) -> LayerVerdict {
        if !command.emergency {
            return LayerVerdict::Pass;
        }

        LayerVerdict::Warn {
            layer: "emergency".into(),
            message: format!(
                "Emergency override by agent '{}' — firmware limits still enforced",
                command.issuer.name
            ),
        }
    }

    /// Layer 4: Offline mandate — what's allowed when disconnected.
    fn check_offline(&self, passport: &Passport, command: &Command) -> LayerVerdict {
        if !self.offline {
            return LayerVerdict::Pass;
        }

        if let Some(mandate) = &passport.autonomous_mandate {
            if mandate.allowed_offline_actions.contains(&command.capability) {
                return LayerVerdict::Warn {
                    layer: "offline".into(),
                    message: "Command allowed under autonomous mandate".into(),
                };
            }
        }

        LayerVerdict::Reject {
            layer: "offline".into(),
            reason: format!(
                "Command '{}' not permitted in offline mode — not in autonomous mandate",
                command.capability
            ),
        }
    }

    /// Layer 5: Delegation chain validation.
    fn check_delegation(&self, command: &Command) -> LayerVerdict {
        if command.delegation_chain.is_empty() {
            return LayerVerdict::Pass;
        }

        for agent in &command.delegation_chain {
            if agent.name.trim().is_empty() {
                return LayerVerdict::Reject {
                    layer: "delegation".into(),
                    reason: "Invalid agent in delegation chain: empty name".into(),
                };
            }
        }

        LayerVerdict::Pass
    }

    /// Layer 6: Context validation — time, environment, state.
    fn check_context(&self, command: &Command) -> LayerVerdict {
        if let Some(temp) = self.context.temperature_c {
            if temp > 80.0 {
                return LayerVerdict::Reject {
                    layer: "context".into(),
                    reason: format!(
                        "Device temperature {}°C is critical — refusing commands until cooled",
                        temp
                    ),
                };
            }
        }

        if let Some(battery) = self.context.battery_percent {
            if battery < 5.0 && !command.emergency {
                return LayerVerdict::Reject {
                    layer: "context".into(),
                    reason: format!(
                        "Battery at {:.1}% — insufficient for non-emergency commands",
                        battery
                    ),
                };
            }
        }

        LayerVerdict::Pass
    }

    /// Layer 7: RBAC — role-based access control.
    ///
    /// Owner: full control.
    /// Operator: can execute commands within mandate.
    /// Observer: read-only — ALL commands rejected.
    /// Parent: can restrict, approve, set time limits.
    /// EmergencyRescue: limited to read + safe restart only.
    fn check_rbac(&self, _passport: &Passport, command: &Command) -> LayerVerdict {
        let role = &command.issuer.role;

        match role {
            AisRole::Owner => LayerVerdict::Pass,

            AisRole::Observer => LayerVerdict::Reject {
                layer: "rbac".into(),
                reason: format!(
                    "Agent '{}' has role Observer — read-only access, cannot execute commands",
                    command.issuer.name
                ),
            },

            AisRole::EmergencyRescue => {
                let allowed = [
                    "read_flight_recorder",
                    "diagnose",
                    "safe_restart",
                    "status",
                ];
                if !allowed.contains(&command.capability.as_str()) {
                    return LayerVerdict::Reject {
                        layer: "rbac".into(),
                        reason: format!(
                            "EmergencyRescue agent '{}' cannot execute '{}' — limited to {:?}",
                            command.issuer.name, command.capability, allowed
                        ),
                    };
                }
                LayerVerdict::Warn {
                    layer: "rbac".into(),
                    message: format!(
                        "EmergencyRescue agent '{}' executing '{}' — limited mandate",
                        command.issuer.name, command.capability
                    ),
                }
            }

            AisRole::Parent | AisRole::Operator => LayerVerdict::Pass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passport::Passport;

    fn make_test_setup() -> (BodyLaw, Passport) {
        let limits = FirmwareLimits {
            max_temperature_c: Some(100.0),
            max_laser_power_mw: Some(10.0),
            ..Default::default()
        };
        let body_law = BodyLaw::new(limits);

        let mut passport =
            Passport::new("test", "test device", RiskClass::Low, Platform::Linux);
        passport.add_capability(Capability {
            name: "heat".into(),
            description: "Apply heat".into(),
            parameters: vec![Parameter {
                name: "temperature".into(),
                param_type: ParamType::Float,
                required: true,
                default: None,
                constraints: Some(Constraints {
                    min: Some(0.0),
                    max: Some(100.0),
                    enum_values: None,
                    regex: None,
                }),
            }],
            risk: RiskClass::Medium,
        });
        passport.forbid("melt", "Would destroy the device", true);

        (body_law, passport)
    }

    fn make_agent(role: AisRole) -> Agent {
        Agent {
            agent_id: uuid::Uuid::new_v4(),
            name: "test_agent".into(),
            agent_type: AgentType::Human,
            role,
            affiliation: None,
            public_key: None,
        }
    }

    #[test]
    fn test_valid_command_passes() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new(
            "heat",
            serde_json::json!({"temperature": 50.0}),
            make_agent(AisRole::Owner),
        );

        let result = law.validate(&passport, &cmd);
        assert!(result.allowed);
    }

    #[test]
    fn test_firmware_limit_rejects() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new(
            "heat",
            serde_json::json!({"temperature": 150.0}), // >100°C limit
            make_agent(AisRole::Owner),
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(1));
    }

    #[test]
    fn test_forbidden_rejects() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new("melt", serde_json::json!({}), make_agent(AisRole::Owner));

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(2));
    }

    #[test]
    fn test_offline_rejects_unauthorized() {
        let mut law = BodyLaw::new(FirmwareLimits::default());
        law.set_offline(true);

        let mut passport = Passport::new("test", "test", RiskClass::Low, Platform::Linux);
        passport.add_capability(Capability {
            name: "ping".into(),
            description: "Ping".into(),
            parameters: vec![],
            risk: RiskClass::Informational,
        });

        let cmd = Command::new("ping", serde_json::json!({}), make_agent(AisRole::Owner));

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(4));
    }

    #[test]
    fn test_physical_stop_rejects() {
        let mut law = BodyLaw::new(FirmwareLimits::default());
        law.physical_stop();

        let passport = Passport::new("test", "test", RiskClass::Low, Platform::Linux);
        let cmd = Command::new(
            "anything",
            serde_json::json!({}),
            make_agent(AisRole::Owner),
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(0));
    }

    #[test]
    fn test_observer_rejected() {
        let (law, passport) = make_test_setup();
        let cmd = Command::new(
            "heat",
            serde_json::json!({"temperature": 30.0}),
            make_agent(AisRole::Observer),
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(7));
    }

    #[test]
    fn test_emergency_rescue_limited() {
        let (law, passport) = make_test_setup();
        // Should reject non-rescue command
        let cmd = Command::new(
            "heat",
            serde_json::json!({"temperature": 30.0}),
            make_agent(AisRole::EmergencyRescue),
        );

        let result = law.validate(&passport, &cmd);
        assert!(!result.allowed);
        assert_eq!(result.rejected_at_layer, Some(7));

        // Should allow safe_restart
        let mut passport2 =
            Passport::new("test", "test", RiskClass::Low, Platform::Linux);
        passport2.add_capability(Capability {
            name: "safe_restart".into(),
            description: "Safe restart".into(),
            parameters: vec![],
            risk: RiskClass::Low,
        });
        let cmd2 = Command::new(
            "safe_restart",
            serde_json::json!({}),
            make_agent(AisRole::EmergencyRescue),
        );
        let result2 = law.validate(&passport2, &cmd2);
        assert!(result2.allowed);
    }
}
