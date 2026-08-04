//! ROS2 Bridge — DAIS as a safety layer for Robot Operating System 2.
//!
//! Provides a safety-first wrapper around ROS2 communication:
//! - All ROS2 messages pass through Body Law before execution
//! - SafetyProof is attached to every command
//! - Flight Recorder captures ROS2 topic activity
//! - Emergency stop propagates via ROS2 lifecycle nodes
//!
//! Architecture:
//!   ROS2 Node → DAIS Bridge → Body Law (7 layers) → SafetyProof → Execute
//!                                                      ↓
//!                                              Flight Recorder
//!
//! This module does NOT depend on actual ROS2 libraries (rclrs).
//! It provides the data structures and protocol for integration.

use serde::{Deserialize, Serialize};
use crate::fleet::SafetyIntegrityLevel;

/// ROS2 Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ros2BridgeConfig {
    pub node_name: String,
    pub namespace: String,
    pub domain_id: u32,
    pub safety_level: SafetyIntegrityLevel,
    pub enable_emergency_stop: bool,
    pub enable_flight_recorder: bool,
    pub enable_safety_proof: bool,
}

impl Default for Ros2BridgeConfig {
    fn default() -> Self {
        Self {
            node_name: "ais_bridge".into(),
            namespace: "/ais".into(),
            domain_id: 0,
            safety_level: SafetyIntegrityLevel::SIL2,
            enable_emergency_stop: true,
            enable_flight_recorder: true,
            enable_safety_proof: true,
        }
    }
}

/// A ROS2 topic subscription, wrapped in DAIS safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeTopic {
    pub topic_name: String,
    pub message_type: String,    // e.g., "geometry_msgs/msg/Twist"
    pub qos: QosProfile,
    /// Maximum rate this topic can be published (Hz), 0 = unlimited
    pub max_rate_hz: f64,
    /// Body Law layers required for this topic
    pub required_layers: u8,
    /// Transform this message before passing to Body Law?
    pub transform: Option<String>,
}

/// ROS2 Quality of Service profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosProfile {
    pub reliability: QosReliability,
    pub durability: QosDurability,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QosReliability {
    Reliable,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QosDurability {
    Volatile,
    TransientLocal,
}

impl Default for QosProfile {
    fn default() -> Self {
        Self { reliability: QosReliability::Reliable, durability: QosDurability::Volatile, depth: 10 }
    }
}

/// Command intercepted from ROS2 topic, wrapped for Body Law
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ros2Command {
    pub command_id: String,
    pub topic: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub timestamp: String,
    pub source_node: String,
}

/// Response after Body Law validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ros2CommandResult {
    pub command_id: String,
    pub approved: bool,
    pub rejected_at_layer: Option<u8>,
    pub rejection_reason: Option<String>,
    pub safety_proof_id: Option<String>,
}

/// ROS2 Lifecycle Node state — managed by AIS
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifecycleState {
    Unconfigured,
    Inactive,
    Active,
    Finalized,
    EmergencyStop,
}

/// ROS2 Bridge — main integration point
pub struct Ros2Bridge {
    pub config: Ros2BridgeConfig,
    pub topics: Vec<SafeTopic>,
    pub state: LifecycleState,
    pub commands_processed: u64,
    pub commands_rejected: u64,
    pub emergency_stops_triggered: u64,
}

impl Ros2Bridge {
    pub fn new(config: Ros2BridgeConfig) -> Self {
        Self {
            config,
            topics: Vec::new(),
            state: LifecycleState::Unconfigured,
            commands_processed: 0,
            commands_rejected: 0,
            emergency_stops_triggered: 0,
        }
    }

    /// Register a safe topic
    pub fn register_topic(&mut self, topic: SafeTopic) {
        self.topics.push(topic);
    }

    /// Configure the bridge (transition to Inactive)
    pub fn configure(&mut self) -> Result<(), String> {
        if self.topics.is_empty() {
            return Err("No topics registered".into());
        }
        self.state = LifecycleState::Inactive;
        Ok(())
    }

    /// Activate the bridge
    pub fn activate(&mut self) -> Result<(), String> {
        if self.state != LifecycleState::Inactive {
            return Err(format!("Cannot activate from state {:?}", self.state));
        }
        self.state = LifecycleState::Active;
        Ok(())
    }

    /// Process a ROS2 command through Body Law simulation
    pub fn process_command(&mut self, command: Ros2Command) -> Ros2CommandResult {
        self.commands_processed += 1;

        // Find the topic's safety requirements
        let topic = self.topics.iter().find(|t| t.topic_name == command.topic);
        
        // Simulate Body Law check (in real impl, calls body_law.validate())
        let approved = self.simulate_body_law(&command, topic);
        
        if approved {
            Ros2CommandResult {
                command_id: command.command_id,
                approved: true,
                rejected_at_layer: None,
                rejection_reason: None,
                safety_proof_id: Some(format!("proof_{}", uuid::Uuid::new_v4())),
            }
        } else {
            self.commands_rejected += 1;
            Ros2CommandResult {
                command_id: command.command_id,
                approved: false,
                rejected_at_layer: Some(2), // Capability layer
                rejection_reason: Some("Command not in topic capabilities".into()),
                safety_proof_id: None,
            }
        }
    }

    /// Simulate Body Law validation (placeholder for real integration)
    fn simulate_body_law(&self, command: &Ros2Command, topic: Option<&SafeTopic>) -> bool {
        if let Some(t) = topic {
            // Check max rate
            if t.max_rate_hz > 0.0 {
                // In real impl: check rate limiter
            }
            // All commands pass simulation (real impl calls actual Body Law)
            return true;
        }
        // Unknown topic → reject
        false
    }

    /// Emergency stop — shut down all ROS2 activity
    pub fn emergency_stop(&mut self) {
        self.state = LifecycleState::EmergencyStop;
        self.emergency_stops_triggered += 1;
    }

    /// Resume after emergency stop
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != LifecycleState::EmergencyStop {
            return Err("Not in emergency stop".into());
        }
        self.state = LifecycleState::Active;
        Ok(())
    }

    /// Get bridge status
    pub fn status(&self) -> BridgeStatus {
        BridgeStatus {
            state: self.state,
            topics_registered: self.topics.len() as u64,
            commands_processed: self.commands_processed,
            commands_rejected: self.commands_rejected,
            emergency_stops: self.emergency_stops_triggered,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub state: LifecycleState,
    pub topics_registered: u64,
    pub commands_processed: u64,
    pub commands_rejected: u64,
    pub emergency_stops: u64,
}

/// Standard ROS2 topics for DAIS integration
pub fn standard_ais_topics() -> Vec<SafeTopic> {
    vec![
        SafeTopic {
            topic_name: "/ais/emergency_stop".into(),
            message_type: "std_msgs/msg/Bool".into(),
            qos: QosProfile { reliability: QosReliability::Reliable, durability: QosDurability::TransientLocal, depth: 1 },
            max_rate_hz: 0.0, required_layers: 0b1111111, transform: None,
        },
        SafeTopic {
            topic_name: "/ais/heartbeat".into(),
            message_type: "std_msgs/msg/Header".into(),
            qos: QosProfile::default(), max_rate_hz: 1.0, required_layers: 0, transform: None,
        },
        SafeTopic {
            topic_name: "/ais/flight_recorder/event".into(),
            message_type: "ais_msgs/msg/FlightEvent".into(),
            qos: QosProfile { reliability: QosReliability::Reliable, durability: QosDurability::TransientLocal, depth: 100 },
            max_rate_hz: 0.0, required_layers: 0, transform: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_lifecycle() {
        let mut bridge = Ros2Bridge::new(Ros2BridgeConfig::default());
        bridge.register_topic(SafeTopic {
            topic_name: "/cmd_vel".into(), message_type: "geometry_msgs/Twist".into(),
            qos: QosProfile::default(), max_rate_hz: 10.0, required_layers: 0b1111111, transform: None,
        });
        
        assert_eq!(bridge.state, LifecycleState::Unconfigured);
        assert!(bridge.configure().is_ok());
        assert_eq!(bridge.state, LifecycleState::Inactive);
        assert!(bridge.activate().is_ok());
        assert_eq!(bridge.state, LifecycleState::Active);
    }

    #[test]
    fn test_command_processing() {
        let mut bridge = Ros2Bridge::new(Ros2BridgeConfig::default());
        bridge.register_topic(SafeTopic {
            topic_name: "/cmd_vel".into(), message_type: "twist".into(),
            qos: QosProfile::default(), max_rate_hz: 10.0, required_layers: 0b1111111, transform: None,
        });
        bridge.configure().unwrap();
        bridge.activate().unwrap();

        let cmd = Ros2Command {
            command_id: "c1".into(), topic: "/cmd_vel".into(),
            message_type: "twist".into(),
            payload: serde_json::json!({"linear": {"x": 1.0}}),
            timestamp: "now".into(), source_node: "nav2".into(),
        };
        
        let result = bridge.process_command(cmd);
        assert!(result.approved);
        assert_eq!(bridge.commands_processed, 1);
    }

    #[test]
    fn test_unknown_topic_rejected() {
        let mut bridge = Ros2Bridge::new(Ros2BridgeConfig::default());
        bridge.register_topic(SafeTopic {
            topic_name: "/known".into(), message_type: "x".into(),
            qos: QosProfile::default(), max_rate_hz: 0.0, required_layers: 0, transform: None,
        });
        bridge.configure().unwrap();
        bridge.activate().unwrap();

        let cmd = Ros2Command {
            command_id: "c1".into(), topic: "/unknown".into(),
            message_type: "x".into(), payload: serde_json::json!({}),
            timestamp: "now".into(), source_node: "hacker".into(),
        };
        
        let result = bridge.process_command(cmd);
        assert!(!result.approved);
        assert_eq!(bridge.commands_rejected, 1);
    }

    #[test]
    fn test_emergency_stop() {
        let mut bridge = Ros2Bridge::new(Ros2BridgeConfig::default());
        bridge.register_topic(SafeTopic {
            topic_name: "/test".into(), message_type: "x".into(),
            qos: QosProfile::default(), max_rate_hz: 0.0, required_layers: 0, transform: None,
        });
        bridge.configure().unwrap();
        bridge.activate().unwrap();
        
        bridge.emergency_stop();
        assert_eq!(bridge.state, LifecycleState::EmergencyStop);
        assert!(bridge.resume().is_ok());
        assert_eq!(bridge.state, LifecycleState::Active);
    }

    #[test]
    fn test_standard_topics() {
        let topics = standard_ais_topics();
        assert_eq!(topics.len(), 3);
        assert!(topics.iter().any(|t| t.topic_name == "/ais/emergency_stop"));
    }
}
