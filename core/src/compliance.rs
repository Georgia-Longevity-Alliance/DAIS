//! WASI Bridge — WebAssembly sandbox for DAIS safety rules.
//!
//! Body Law can be compiled to WASM and executed in a sandboxed
//! WebAssembly runtime on ANY device (ESP32 to Jetson to cloud).
//! This provides an additional hardware-independent safety boundary.
//!
//! Key advantage: safety rules written once, verified once, run everywhere.

use serde::{Deserialize, Serialize};

/// WASM module with embedded Body Law safety rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSafetyModule {
    pub module_id: String,
    pub wasm_bytes: Vec<u8>,
    pub entry_function: String,
    pub memory_limit_kb: u32,
    pub fuel_limit: u64,         // Max instructions before timeout
    pub hash: String,            // SHA-256 of wasm_bytes
    pub version: String,
}

/// Result from WASM safety check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSafetyResult {
    pub module_id: String,
    pub passed: bool,
    pub fuel_used: u64,
    pub memory_used_kb: u32,
    pub output_data: Option<Vec<u8>>,
    pub error_message: Option<String>,
}

/// WASI runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiConfig {
    pub max_memory_kb: u32,
    pub max_fuel: u64,
    pub allowed_imports: Vec<String>,  // e.g., ["sensor_read", "actuator_write"]
    pub forbidden_imports: Vec<String>, // e.g., ["network_access", "file_system"]
    pub timeout_ms: u64,
}

impl Default for WasiConfig {
    fn default() -> Self {
        Self {
            max_memory_kb: 256,
            max_fuel: 1_000_000,
            allowed_imports: vec!["sensor_read".into(), "actuator_write".into()],
            forbidden_imports: vec!["network_access".into(), "file_system".into(), "os_command".into()],
            timeout_ms: 1000,
        }
    }
}

/// WASI Runtime — executes safety modules in sandbox
pub struct WasiRuntime {
    pub config: WasiConfig,
    pub modules: Vec<WasmSafetyModule>,
    pub executions: u64,
    pub violations: u64,
}

impl WasiRuntime {
    pub fn new(config: WasiConfig) -> Self {
        Self { config, modules: Vec::new(), executions: 0, violations: 0 }
    }

    /// Register a WASM safety module
    pub fn register(&mut self, module: WasmSafetyModule) -> Result<(), String> {
        if module.memory_limit_kb > self.config.max_memory_kb {
            return Err(format!("Memory limit {}KB > max {}KB", module.memory_limit_kb, self.config.max_memory_kb));
        }
        if module.fuel_limit > self.config.max_fuel {
            return Err(format!("Fuel limit {} > max {}", module.fuel_limit, self.config.max_fuel));
        }
        self.modules.push(module);
        Ok(())
    }

    /// Execute safety check (simulated — real impl uses wasmtime/wasmer)
    pub fn execute(&mut self, module_id: &str, input: &[u8]) -> Option<WasmSafetyResult> {
        self.executions += 1;
        let module = self.modules.iter().find(|m| m.module_id == module_id)?;
        
        // Simulated WASM execution
        let fuel_used = (input.len() as u64 * 10).min(module.fuel_limit);
        let passed = input.len() < 4096; // Simple size check
        
        if !passed {
            self.violations += 1;
        }

        Some(WasmSafetyResult {
            module_id: module_id.into(),
            passed,
            fuel_used,
            memory_used_kb: (input.len() / 1024) as u32,
            output_data: if passed { Some(b"SAFE".to_vec()) } else { None },
            error_message: if passed { None } else { Some("Input exceeds safety limit".into()) },
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// NIST AI RMF + EU AI Act Compliance
// ═══════════════════════════════════════════════════════════════

/// NIST AI Risk Management Framework mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistRmfMapping {
    pub dais_component: String,
    pub nist_function: NistFunction,
    pub nist_category: String,
    pub implementation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NistFunction {
    Govern,
    Map,
    Measure,
    Manage,
}

/// EU AI Act risk category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EuAiRiskLevel {
    Minimal,
    Limited,
    High,
    Unacceptable,
}

/// EU AI Act compliance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuAiCompliance {
    pub device_id: String,
    pub risk_level: EuAiRiskLevel,
    pub requires_conformity_assessment: bool,
    pub requires_human_oversight: bool,
    pub requires_technical_documentation: bool,
    pub requires_incident_reporting: bool,
    pub dais_safety_layers: Vec<String>,
}

impl EuAiCompliance {
    pub fn assess(device_type: &str, safety_level: crate::fleet::SafetyIntegrityLevel) -> Self {
        let (risk, assessment, oversight) = match safety_level {
            crate::fleet::SafetyIntegrityLevel::SIL0 => (EuAiRiskLevel::Minimal, false, false),
            crate::fleet::SafetyIntegrityLevel::SIL1 => (EuAiRiskLevel::Limited, false, false),
            crate::fleet::SafetyIntegrityLevel::SIL2 => (EuAiRiskLevel::Limited, true, true),
            crate::fleet::SafetyIntegrityLevel::SIL3 => (EuAiRiskLevel::High, true, true),
            crate::fleet::SafetyIntegrityLevel::SIL4 => (EuAiRiskLevel::Unacceptable, true, true),
        };

        Self {
            device_id: device_type.into(),
            risk_level: risk,
            requires_conformity_assessment: assessment,
            requires_human_oversight: oversight,
            requires_technical_documentation: risk >= EuAiRiskLevel::Limited,
            requires_incident_reporting: risk >= EuAiRiskLevel::High,
            dais_safety_layers: vec![
                "Body Law (7 layers)".into(),
                "Flight Recorder".into(),
                "Safety Proof Chain".into(),
                "Emergency Stop".into(),
            ],
        }
    }
}

/// NIST AI RMF → DAIS mapping
pub fn nist_rmf_mappings() -> Vec<NistRmfMapping> {
    vec![
        NistRmfMapping {
            dais_component: "Body Law (7 layers)".into(),
            nist_function: NistFunction::Govern,
            nist_category: "Policies and accountability".into(),
            implementation: "Hardware-enforced safety layers with proof chain".into(),
        },
        NistRmfMapping {
            dais_component: "Flight Recorder".into(),
            nist_function: NistFunction::Measure,
            nist_category: "Monitoring and testing".into(),
            implementation: "Ring buffer with anomaly detection and trace network".into(),
        },
        NistRmfMapping {
            dais_component: "Proven Knowledge".into(),
            nist_function: NistFunction::Map,
            nist_category: "Context and risk mapping".into(),
            implementation: "S-P-O claims with confidence scores and evidence".into(),
        },
        NistRmfMapping {
            dais_component: "Fleet Manager".into(),
            nist_function: NistFunction::Manage,
            nist_category: "Risk treatment and response".into(),
            implementation: "Fleet-wide emergency stop and health monitoring".into(),
        },
    ]
}

/// MQTT Safety Bridge — industrial IoT protocol safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttSafetyBridge {
    pub broker_url: String,
    pub client_id: String,
    pub allowed_topics: Vec<String>,
    pub max_message_size: u32,
    pub max_rate_per_second: u32,
    pub require_tls: bool,
    pub qos_level: u8,
}

impl Default for MqttSafetyBridge {
    fn default() -> Self {
        Self {
            broker_url: "mqtt://localhost:1883".into(),
            client_id: "dais_bridge".into(),
            allowed_topics: vec!["/dais/#".into(), "/sensors/#".into()],
            max_message_size: 4096,
            max_rate_per_second: 100,
            require_tls: true,
            qos_level: 2,  // Exactly once
        }
    }
}

impl MqttSafetyBridge {
    pub fn validate_topic(&self, topic: &str) -> bool {
        for allowed in &self.allowed_topics {
            if topic.starts_with(allowed.trim_end_matches('#')) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_register_and_execute() {
        let mut rt = WasiRuntime::new(WasiConfig::default());
        let module = WasmSafetyModule {
            module_id: "body_law_v1".into(),
            wasm_bytes: vec![0, 1, 2, 3],
            entry_function: "validate".into(),
            memory_limit_kb: 128,
            fuel_limit: 100_000,
            hash: "abc123".into(),
            version: "1.0".into(),
        };
        assert!(rt.register(module).is_ok());
        let result = rt.execute("body_law_v1", b"test command").unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_eu_ai_act_compliance() {
        let compliance = EuAiCompliance::assess("surgical_robot", crate::fleet::SafetyIntegrityLevel::SIL3);
        assert_eq!(compliance.risk_level, EuAiRiskLevel::High);
        assert!(compliance.requires_conformity_assessment);
        assert!(compliance.requires_incident_reporting);
    }

    #[test]
    fn test_nist_rmf_mappings() {
        let mappings = nist_rmf_mappings();
        assert_eq!(mappings.len(), 4);
        assert!(mappings.iter().any(|m| m.dais_component.contains("Body Law")));
    }

    #[test]
    fn test_mqtt_topic_validation() {
        let bridge = MqttSafetyBridge::default();
        assert!(bridge.validate_topic("/dais/command"));
        assert!(bridge.validate_topic("/sensors/temperature"));
        assert!(!bridge.validate_topic("/admin/shutdown"));
        assert!(!bridge.validate_topic("/evil/topic"));
    }
}
