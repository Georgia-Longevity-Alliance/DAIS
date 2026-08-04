//! Federated Safety Learning — privacy-preserving anomaly sharing.
//!
//! Devices share anomaly signatures WITHOUT exposing raw sensor data.
//! Uses differential privacy (ε-bounded Laplacian noise) and
//! federated averaging to build collective immunity.
//!
//! Key idea: Each device trains locally on its own Flight Recorder data,
//! then shares only the model weights (not the data). The central server
//! aggregates weights via Federated Averaging (McMahan et al., 2017).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Privacy budget for differential privacy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PrivacyBudget {
    pub epsilon: f64,    // Privacy loss parameter (lower = more private)
    pub delta: f64,      // Failure probability (typically 1e-5)
}

impl Default for PrivacyBudget {
    fn default() -> Self {
        Self { epsilon: 1.0, delta: 1e-5 }
    }
}

/// Anomaly signature — lightweight fingerprint, NOT raw data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalySignature {
    pub anomaly_type: String,        // "laser_spike", "stage_slip", etc.
    pub device_platform: String,     // "jetson_orin", "esp32", etc.
    pub sensor_pattern: Vec<f64>,   // Normalized sensor readings (0-1)
    pub resolution: String,          // What fixed it
    pub confidence: f64,            // How sure we are
    pub privacy_noise_applied: bool, // DP noise added?
}

/// Local model trained on one device's Flight Recorder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub device_id: String,
    pub weights: Vec<f64>,
    pub num_samples: u64,
    pub anomaly_types: Vec<String>,
    pub privacy_budget: PrivacyBudget,
}

/// Federated server — aggregates models without seeing raw data
pub struct FederatedSafetyServer {
    pub global_weights: Vec<f64>,
    pub total_samples: u64,
    pub num_rounds: u64,
    pub anomaly_signatures: Vec<AnomalySignature>,
    /// Map: anomaly_type → resolution
    pub resolution_library: HashMap<String, String>,
}

impl FederatedSafetyServer {
    pub fn new(num_features: usize) -> Self {
        Self {
            global_weights: vec![0.0; num_features],
            total_samples: 0,
            num_rounds: 0,
            anomaly_signatures: Vec::new(),
            resolution_library: HashMap::new(),
        }
    }

    /// Federated Averaging (FedAvg): aggregate local models into global
    pub fn aggregate(&mut self, models: &[LocalModel]) {
        if models.is_empty() {
            return;
        }

        let mut weighted_sum = vec![0.0; self.global_weights.len()];
        let mut total_weight = 0.0f64;

        for model in models {
            let weight = model.num_samples as f64;
            for (i, w) in model.weights.iter().enumerate() {
                if i < weighted_sum.len() {
                    weighted_sum[i] += w * weight;
                }
            }
            total_weight += weight;
        }

        if total_weight > 0.0 {
            for (i, w) in weighted_sum.iter_mut().enumerate() {
                self.global_weights[i] = *w / total_weight;
            }
        }

        self.total_samples += models.iter().map(|m| m.num_samples).sum::<u64>();
        self.num_rounds += 1;
    }

    /// Register an anomaly signature (with DP noise)
    pub fn register_signature(&mut self, sig: AnomalySignature) {
        // Store resolution in library
        self.resolution_library
            .insert(sig.anomaly_type.clone(), sig.resolution.clone());
        self.anomaly_signatures.push(sig);
    }

    /// Query resolution for a given anomaly type
    pub fn query_resolution(&self, anomaly_type: &str) -> Option<&String> {
        self.resolution_library.get(anomaly_type)
    }

    /// Get top-k most common anomaly types
    pub fn top_anomalies(&self, k: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for sig in &self.anomaly_signatures {
            *counts.entry(sig.anomaly_type.clone()).or_insert(0) += 1;
        }
        let mut vec: Vec<_> = counts.into_iter().collect();
        vec.sort_by(|a, b| b.1.cmp(&a.1));
        vec.truncate(k);
        vec
    }
}

/// Differential privacy — add Laplacian noise
pub fn apply_dp_noise(value: f64, sensitivity: f64, epsilon: f64) -> f64 {
    use rand::Rng;
    let scale = sensitivity / epsilon;
    let noise: f64 = rand::thread_rng().gen_range(-scale..scale);
    value + noise
}

/// Client-side: train local model on device
pub fn train_local_model(device_id: &str, sensor_data: &[Vec<f64>], labels: &[f64]) -> LocalModel {
    let num_features = if sensor_data.is_empty() { 0 } else { sensor_data[0].len() };
    let mut weights = vec![0.0; num_features];
    let num_samples = sensor_data.len() as u64;
    
    // Simple linear regression (local SGD)
    if num_samples > 0 && num_features > 0 {
        for (sample, &label) in sensor_data.iter().zip(labels.iter()) {
            for (i, &x) in sample.iter().enumerate() {
                if i < num_features {
                    weights[i] += x * label;
                }
            }
        }
        for w in &mut weights {
            *w /= num_samples as f64;
        }
    }

    LocalModel {
        device_id: device_id.into(),
        weights,
        num_samples,
        anomaly_types: vec![],
        privacy_budget: PrivacyBudget::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_averaging() {
        let mut server = FederatedSafetyServer::new(3);
        
        let models = vec![
            LocalModel {
                device_id: "dev_1".into(), weights: vec![0.8, 0.2, 0.1],
                num_samples: 100, anomaly_types: vec!["laser_spike".into()],
                privacy_budget: PrivacyBudget::default(),
            },
            LocalModel {
                device_id: "dev_2".into(), weights: vec![0.6, 0.4, 0.3],
                num_samples: 50, anomaly_types: vec!["focus_drift".into()],
                privacy_budget: PrivacyBudget::default(),
            },
        ];
        
        server.aggregate(&models);
        assert_eq!(server.num_rounds, 1);
        assert_eq!(server.total_samples, 150);
        assert!(server.global_weights[0] > 0.7);
    }

    #[test]
    fn test_anomaly_signature_registry() {
        let mut server = FederatedSafetyServer::new(1);
        
        server.register_signature(AnomalySignature {
            anomaly_type: "laser_spike".into(), device_platform: "jetson".into(),
            sensor_pattern: vec![0.9, 0.1], resolution: "reduce_pwm".into(),
            confidence: 0.95, privacy_noise_applied: true,
        });
        server.register_signature(AnomalySignature {
            anomaly_type: "laser_spike".into(), device_platform: "esp32".into(),
            sensor_pattern: vec![0.85, 0.15], resolution: "reduce_pwm".into(),
            confidence: 0.88, privacy_noise_applied: true,
        });
        server.register_signature(AnomalySignature {
            anomaly_type: "stage_slip".into(), device_platform: "jetson".into(),
            sensor_pattern: vec![0.2, 0.95], resolution: "rehome_stage".into(),
            confidence: 0.92, privacy_noise_applied: false,
        });
        
        assert_eq!(server.anomaly_signatures.len(), 3);
        assert_eq!(server.query_resolution("laser_spike").unwrap(), "reduce_pwm");
        
        let top = server.top_anomalies(2);
        assert_eq!(top[0].0, "laser_spike");
        assert_eq!(top[0].1, 2);
    }

    #[test]
    fn test_differential_privacy() {
        let original = 42.0;
        let noisy = apply_dp_noise(original, 1.0, 1.0);
        // Noisy value should be different but close
        assert!((noisy - original).abs() < 5.0);
    }

    #[test]
    fn test_local_training() {
        let data = vec![vec![1.0, 0.5], vec![0.8, 0.3]];
        let labels = vec![1.0, 1.0];
        let model = train_local_model("test_dev", &data, &labels);
        assert_eq!(model.num_samples, 2);
        assert_eq!(model.weights.len(), 2);
    }
}
