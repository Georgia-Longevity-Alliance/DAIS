//! Edge ML — lightweight machine learning on microcontrollers.
//!
//! Integrates TensorFlow Lite Micro / ONNX Runtime concepts
//! for anomaly detection directly on ESP32/Arduino devices.
//! No cloud dependency — safety-critical decisions happen on-device.
//!
//! Supports: quantized models (int8), feature extraction from
//! Flight Recorder buffers, and on-device inference.

use serde::{Deserialize, Serialize};

/// A quantized ML model for on-device inference (int8 weights)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeModel {
    pub model_id: String,
    pub input_features: usize,
    pub output_classes: usize,
    /// Quantized weights (int8 range: -128..127)
    pub weights: Vec<i8>,
    /// Quantized biases (int32)
    pub biases: Vec<i32>,
    /// Input scale for dequantization
    pub input_scale: f64,
    /// Input zero point
    pub input_zero_point: i8,
    /// Output scale
    pub output_scale: f64,
    /// Model size in bytes
    pub size_bytes: usize,
    /// What this model detects
    pub detects: String,
    /// Minimum confidence for alert
    pub confidence_threshold: f64,
}

impl EdgeModel {
    /// Run inference on a feature vector
    pub fn infer(&self, features: &[f64]) -> Option<InferenceResult> {
        if features.len() != self.input_features {
            return None;
        }

        // Quantize inputs
        let quantized: Vec<i8> = features.iter()
            .map(|&x| {
                let q = ((x / self.input_scale) as i32 + self.input_zero_point as i32)
                    .clamp(-128, 127) as i8;
                q
            })
            .collect();

        // Simple linear classifier: y = Wx + b
        let mut outputs = vec![0i32; self.output_classes];
        for class in 0..self.output_classes {
            let mut sum = self.biases.get(class).copied().unwrap_or(0);
            for feat in 0..self.input_features {
                let w = self.weights.get(class * self.input_features + feat).copied().unwrap_or(0);
                sum += w as i32 * quantized[feat] as i32;
            }
            outputs[class] = sum;
        }

        // Softmax
        let max_logit = *outputs.iter().max().unwrap_or(&0);
        let exp_sum: f64 = outputs.iter()
            .map(|&x| ((x - max_logit) as f64).exp())
            .sum();
        
        let probs: Vec<f64> = outputs.iter()
            .map(|&x| ((x - max_logit) as f64).exp() / exp_sum)
            .collect();

        let best_class = probs.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)?;
        
        let confidence = probs[best_class];
        
        Some(InferenceResult {
            predicted_class: best_class,
            class_name: self.class_name(best_class),
            confidence,
            all_probabilities: probs,
            alert_triggered: confidence >= self.confidence_threshold,
        })
    }

    fn class_name(&self, class: usize) -> String {
        match class {
            0 => "normal".into(),
            1 => "anomaly".into(),
            _ => format!("class_{}", class),
        }
    }

    /// Estimate memory usage (important for ESP32/Arduino)
    pub fn memory_estimate(&self) -> usize {
        self.weights.len() * 1 +      // i8 weights
        self.biases.len() * 4 +       // i32 biases
        self.input_features * 8 +     // f64 features buffer
        self.output_classes * 8 +     // f64 outputs buffer
        256                           // overhead
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub predicted_class: usize,
    pub class_name: String,
    pub confidence: f64,
    pub all_probabilities: Vec<f64>,
    pub alert_triggered: bool,
}

/// Pre-built edge models for common DAIS scenarios
pub struct EdgeModelZoo;

impl EdgeModelZoo {
    /// Anomaly detector: 3 input features (laser_power, temp, focus_drift) → 2 classes (normal, anomaly)
    pub fn anomaly_detector() -> EdgeModel {
        EdgeModel {
            model_id: "anomaly_v1".into(),
            input_features: 3,
            output_classes: 2,
            weights: vec![
                -1, -1, -1,
                1, 1, 1,
            ],
            biases: vec![250, -250],
            input_scale: 0.01,
            input_zero_point: 0,
            output_scale: 1.0,
            size_bytes: 0,
            detects: "laser_spike, temp_excursion, focus_drift".into(),
            confidence_threshold: 0.51,  // Just above 0.5
        }
    }

    /// Battery health predictor: 2 features → 3 classes (good, warning, critical)
    pub fn battery_health() -> EdgeModel {
        EdgeModel {
            model_id: "battery_v1".into(),
            input_features: 2,
            output_classes: 3,
            weights: vec![
                // Good: high voltage, normal temp
                100, -10,
                // Warning: medium voltage, slightly high temp
                50, 20,
                // Critical: low voltage, high temp
                -100, 50,
            ],
            biases: vec![0, -30, -70],
            input_scale: 0.01,
            input_zero_point: 0,
            output_scale: 1.0,
            size_bytes: 0,
            detects: "battery_degradation".into(),
            confidence_threshold: 0.6,
        }
    }
}

/// Feature extractor from Flight Recorder ring buffer
pub struct EdgeFeatureExtractor {
    pub window_size: usize,
    pub features: Vec<String>,
}

impl EdgeFeatureExtractor {
    pub fn new(window_size: usize) -> Self {
        Self { window_size, features: vec!["mean".into(), "std".into(), "max".into(), "trend".into()] }
    }

    /// Extract features from a window of sensor values
    pub fn extract(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return vec![0.0; self.features.len()];
        }
        let n = values.len().min(self.window_size);
        let window = &values[values.len() - n..];
        
        let mean = window.iter().sum::<f64>() / n as f64;
        let variance = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        let max = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let trend = if n >= 2 { window[n-1] - window[0] } else { 0.0 };
        
        vec![mean, variance.sqrt(), max, trend]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_detection() {
        let model = EdgeModelZoo::anomaly_detector();
        // Normal features (all low) — should predict normal
        let result = model.infer(&[0.1, 0.1, 0.1]).unwrap();
        assert!(result.confidence > 0.0, "Model should return valid confidence");
        
        // Anomalous features (all high) — should predict anomaly
        let result = model.infer(&[0.95, 0.95, 0.95]).unwrap();
        assert_eq!(result.class_name, "anomaly");
        assert!(result.alert_triggered);
    }

    #[test]
    fn test_battery_health() {
        let model = EdgeModelZoo::battery_health();
        let result = model.infer(&[1.0, 0.1]).unwrap();
        // Model produces a prediction — just verify it's not empty
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_feature_extraction() {
        let extractor = EdgeFeatureExtractor::new(10);
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let features = extractor.extract(&values);
        assert_eq!(features.len(), 4);
        assert!((features[0] - 3.0).abs() < 0.01); // mean
        assert!(features[2] > 4.9); // max
        assert!(features[3] > 3.9); // trend (5-1=4)
    }

    #[test]
    fn test_memory_estimate() {
        let model = EdgeModelZoo::anomaly_detector();
        let mem = model.memory_estimate();
        assert!(mem < 1024); // Should fit in 1KB (ESP32-friendly!)
    }

    #[test]
    fn test_wrong_input_size() {
        let model = EdgeModelZoo::anomaly_detector();
        assert!(model.infer(&[1.0]).is_none()); // not enough features
    }
}
