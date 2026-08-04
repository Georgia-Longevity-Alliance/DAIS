//! Neural Safety — DAIS for brain-computer interfaces and neural devices.
//!
//! Extends Body Law with neural-specific safety layers:
//! - Stimulation limits (current, frequency, duration)
//! - Seizure detection & emergency shutdown
//! - Neural data privacy (HIPAA/GDPR for brain data)
//! - EEG/ECoG device passport templates
//!
//! "The brain is the most safety-critical organ. A neural device
//!  without a Body Law is like a pacemaker without a watchdog."

use serde::{Deserialize, Serialize};
use crate::fleet::SafetyIntegrityLevel;

/// Neural device type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NeuralDeviceType {
    EEGHeadset,
    ECoGImplant,
    DeepBrainStimulator,
    TranscranialMagneticStim,
    OptogeneticStimulator,
    NeuralProsthetic,
    BCIController,
    Other(String),
}

/// Neural-specific safety limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralSafetyLimits {
    /// Maximum stimulation current (mA)
    pub max_current_ma: f64,
    /// Maximum stimulation frequency (Hz)
    pub max_frequency_hz: f64,
    /// Maximum pulse duration (µs)
    pub max_pulse_duration_us: f64,
    /// Maximum charge per phase (µC)
    pub max_charge_per_phase_uc: f64,
    /// Maximum session duration (minutes)
    pub max_session_duration_min: f64,
    /// Temperature limit at electrode-tissue interface (°C)
    pub max_tissue_temp_c: f64,
    /// Shannon limit: log(charge_density) must be below this for safe stimulation
    pub shannon_limit_k: f64,
}

impl Default for NeuralSafetyLimits {
    fn default() -> Self {
        Self {
            max_current_ma: 10.0,
            max_frequency_hz: 200.0,
            max_pulse_duration_us: 500.0,
            max_charge_per_phase_uc: 4.0,
            max_session_duration_min: 120.0,
            max_tissue_temp_c: 39.0,
            shannon_limit_k: 1.85,
        }
    }
}

/// Neural data sensitivity levels (GDPR special category: Article 9)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeuralDataSensitivity {
    Aggregated,     // Population statistics
    Anonymized,     // Per-session, no identity
    Pseudonymized,  // Research ID, reversible
    Identified,     // Full identity linked
    RawBrainData,   // Direct neural recordings — MAX sensitivity
}

/// Neural device passport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralPassport {
    pub device_id: String,
    pub device_type: NeuralDeviceType,
    pub manufacturer: String,
    pub safety_limits: NeuralSafetyLimits,
    pub data_sensitivity: NeuralDataSensitivity,
    pub requires_ethics_approval: bool,
    pub ethics_protocol_id: Option<String>,
    pub emergency_shutdown_pin: Option<u8>,
    pub stimulation_channels: u8,
    pub recording_channels: u8,
    pub sample_rate_hz: u32,
}

impl NeuralPassport {
    pub fn validate(&self) -> Result<(), String> {
        // Shannon limit check
        let charge_density = self.safety_limits.max_charge_per_phase_uc / 0.01; // assume 0.01 cm² electrode
        let log_qd = charge_density.ln();
        if log_qd > self.safety_limits.shannon_limit_k {
            return Err(format!("Shannon limit exceeded: ln({:.1}) = {:.2} > {:.2}", 
                charge_density, log_qd, self.safety_limits.shannon_limit_k));
        }
        Ok(())
    }
}

/// Seizure detector — real-time neural anomaly detection
pub struct SeizureDetector {
    pub baseline_eeg: Vec<f64>,
    pub threshold: f64,
    pub window_size: usize,
    pub consecutive_alerts: usize,
    pub max_consecutive: usize,
    pub shutdown_triggered: bool,
}

impl SeizureDetector {
    pub fn new(threshold: f64, window_size: usize, max_consecutive: usize) -> Self {
        Self {
            baseline_eeg: Vec::new(), threshold, window_size,
            consecutive_alerts: 0, max_consecutive, shutdown_triggered: false,
        }
    }

    /// Feed EEG sample, return true if seizure detected
    pub fn process_sample(&mut self, sample: f64) -> SeizureStatus {
        self.baseline_eeg.push(sample);
        if self.baseline_eeg.len() > self.window_size {
            self.baseline_eeg.remove(0);
        }

        if self.baseline_eeg.len() < self.window_size {
            return SeizureStatus::Calibrating;
        }

        let mean = self.baseline_eeg.iter().sum::<f64>() / self.baseline_eeg.len() as f64;
        let variance = self.baseline_eeg.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / self.baseline_eeg.len() as f64;
        let std_dev = variance.sqrt();

        // Simple threshold: if sample exceeds mean + threshold*std, flag
        if (sample - mean).abs() > self.threshold * std_dev {
            self.consecutive_alerts += 1;
        } else {
            self.consecutive_alerts = self.consecutive_alerts.saturating_sub(1);
        }

        if self.consecutive_alerts >= self.max_consecutive {
            self.shutdown_triggered = true;
            SeizureStatus::SeizureDetected {
                confidence: (self.consecutive_alerts as f64 / self.max_consecutive as f64).min(1.0),
            }
        } else {
            SeizureStatus::Normal
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeizureStatus {
    Calibrating,
    Normal,
    SeizureDetected { confidence: f64 },
}

/// Neural Data Privacy — differential privacy for brain recordings
pub struct NeuralPrivacyGuard {
    pub sensitivity: NeuralDataSensitivity,
    pub epsilon: f64,
    pub access_log: Vec<String>,
}

impl NeuralPrivacyGuard {
    pub fn new(sensitivity: NeuralDataSensitivity) -> Self {
        let epsilon = match sensitivity {
            NeuralDataSensitivity::Aggregated => 10.0,
            NeuralDataSensitivity::Anonymized => 5.0,
            NeuralDataSensitivity::Pseudonymized => 1.0,
            NeuralDataSensitivity::Identified => 0.1,
            NeuralDataSensitivity::RawBrainData => 0.01,
        };
        Self { sensitivity, epsilon, access_log: Vec::new() }
    }

    /// Sanitize neural data before sharing
    pub fn sanitize(&self, data: &[f64], noise_scale: f64) -> Vec<f64> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        data.iter()
            .map(|&x| x + rng.gen_range(-noise_scale..noise_scale) / self.epsilon)
            .collect()
    }

    /// Log data access (GDPR audit requirement)
    pub fn log_access(&mut self, user: &str, purpose: &str) {
        self.access_log.push(format!("{}: {} accessed for {}", 
            chrono::Utc::now().to_rfc3339(), user, purpose));
    }
}

/// Pre-built neural device templates
pub fn neural_device_templates() -> Vec<NeuralPassport> {
    vec![
        NeuralPassport {
            device_id: "eeg_headset_v1".into(),
            device_type: NeuralDeviceType::EEGHeadset,
            manufacturer: "OpenBCI".into(),
            safety_limits: NeuralSafetyLimits {
                max_current_ma: 0.0, // No stimulation, recording only
                max_frequency_hz: 0.0,
                max_pulse_duration_us: 0.0,
                max_charge_per_phase_uc: 0.0,
                max_session_duration_min: 480.0,
                max_tissue_temp_c: 37.0,
                shannon_limit_k: 1.85,
            },
            data_sensitivity: NeuralDataSensitivity::RawBrainData,
            requires_ethics_approval: true,
            ethics_protocol_id: None,
            emergency_shutdown_pin: None,
            stimulation_channels: 0,
            recording_channels: 8,
            sample_rate_hz: 250,
        },
        NeuralPassport {
            device_id: "dbs_implant_v1".into(),
            device_type: NeuralDeviceType::DeepBrainStimulator,
            manufacturer: "Medtronic".into(),
            safety_limits: NeuralSafetyLimits::default(),
            data_sensitivity: NeuralDataSensitivity::Identified,
            requires_ethics_approval: true,
            ethics_protocol_id: Some("IRB-2026-001".into()),
            emergency_shutdown_pin: Some(7),
            stimulation_channels: 4,
            recording_channels: 2,
            sample_rate_hz: 1000,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_passport_validation() {
        let mut passport = neural_device_templates()[0].clone();
        assert!(passport.validate().is_ok());
        
        // Exceed Shannon limit
        passport.safety_limits.max_charge_per_phase_uc = 100.0;
        assert!(passport.validate().is_err());
    }



    #[test]
    fn test_privacy_guard() {
        let guard = NeuralPrivacyGuard::new(NeuralDataSensitivity::RawBrainData);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sanitized = guard.sanitize(&data, 1.0);
        assert_eq!(sanitized.len(), 5);
        // Sanitized data should differ from original
        assert!(sanitized.iter().zip(data.iter()).any(|(s, o)| (s - o).abs() > 0.01));
    }

    #[test]
    fn test_privacy_epsilon_by_sensitivity() {
        let raw = NeuralPrivacyGuard::new(NeuralDataSensitivity::RawBrainData);
        let agg = NeuralPrivacyGuard::new(NeuralDataSensitivity::Aggregated);
        // Raw brain data should have much stricter privacy (lower epsilon)
        assert!(raw.epsilon < agg.epsilon);
    }
}
