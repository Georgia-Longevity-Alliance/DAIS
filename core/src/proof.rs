//! Proof of Safety — cryptographic verification of Body Law compliance.
//!
//! Each command that passes all 7 Body Law layers receives a
//! verifiable safety proof. External verifiers can check the proof
//! without re-running the full validation pipeline.
//!
//! Design: Hash-chain based (not full zk-SNARK — that requires
//! Groth16/Plonk setup which is heavy for embedded).
//! The hash chain provides tamper-evident, append-only safety log
//! suitable for regulatory compliance.

use serde::{Deserialize, Serialize};

/// A safety proof — proves a command passed Body Law validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetyProof {
    /// Unique proof identifier
    pub proof_id: String,
    /// Device that executed the command
    pub device_id: String,
    /// The command that was validated
    pub command: String,
    /// Which Body Law layers were checked (bitmask: 0b1111111 = all 7)
    pub layers_checked: u8,
    /// Timestamp of validation
    pub timestamp: String,
    /// Hash of (previous_proof + device_id + command + layers)
    pub hash: String,
    /// Nonce for proof uniqueness
    pub nonce: u64,
}

impl SafetyProof {
    /// Create a new safety proof
    pub fn new(device_id: &str, command: &str, layers_checked: u8, previous_proof_hash: &str) -> Self {
        use sha2::{Sha256, Digest};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let nonce = timestamp as u64 ^ 0xDEADBEEF;
        
        let mut hasher = Sha256::new();
        hasher.update(previous_proof_hash.as_bytes());
        hasher.update(device_id.as_bytes());
        hasher.update(command.as_bytes());
        hasher.update(&[layers_checked]);
        hasher.update(&nonce.to_le_bytes());
        let hash = format!("{:x}", hasher.finalize());
        
        SafetyProof {
            proof_id: format!("proof_{}", &hash[..16]),
            device_id: device_id.into(),
            command: command.into(),
            layers_checked,
            timestamp: timestamp.to_string(),
            hash,
            nonce,
        }
    }
    
    /// Verify this proof against a previous proof hash
    pub fn verify(&self, previous_proof_hash: &str) -> bool {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(previous_proof_hash.as_bytes());
        hasher.update(self.device_id.as_bytes());
        hasher.update(self.command.as_bytes());
        hasher.update(&[self.layers_checked]);
        hasher.update(&self.nonce.to_le_bytes());
        format!("{:x}", hasher.finalize()) == self.hash
    }
    
    /// All 7 layers were checked?
    pub fn is_full_validation(&self) -> bool {
        self.layers_checked == 0b1111111
    }
}

/// Proof Chain — append-only log of safety proofs
pub struct ProofChain {
    pub proofs: Vec<SafetyProof>,
    genesis_hash: String,
    last_hash: String,
}

impl ProofChain {
    pub fn new() -> Self {
        let genesis = "0".repeat(64);
        Self {
            proofs: Vec::new(),
            genesis_hash: genesis.clone(),
            last_hash: genesis,
        }
    }
    
    /// Issue a new safety proof
    pub fn issue(&mut self, device_id: &str, command: &str, layers_checked: u8) -> &SafetyProof {
        let proof = SafetyProof::new(device_id, command, layers_checked, &self.last_hash);
        self.last_hash = proof.hash.clone();
        self.proofs.push(proof);
        self.proofs.last().unwrap()
    }
    
    /// Verify the entire chain integrity
    pub fn verify_chain(&self) -> bool {
        let mut prev = self.genesis_hash.clone();
        for proof in &self.proofs {
            if !proof.verify(&prev) {
                return false;
            }
            prev = proof.hash.clone();
        }
        true
    }
    
    /// Get the latest proof
    pub fn latest(&self) -> Option<&SafetyProof> {
        self.proofs.last()
    }
    
    /// Count proofs for a specific device
    pub fn count_for_device(&self, device_id: &str) -> usize {
        self.proofs.iter().filter(|p| p.device_id == device_id).count()
    }
    
    /// Count proofs with full validation (all 7 layers)
    pub fn count_full_validation(&self) -> usize {
        self.proofs.iter().filter(|p| p.is_full_validation()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_proof_creation_and_verification() {
        let proof = SafetyProof::new("argus_01", "acquire_z_stack", 0b1111111, &"0".repeat(64));
        assert!(proof.verify(&"0".repeat(64)));
        assert!(proof.is_full_validation());
    }

    #[test]
    fn test_proof_chain_integrity() {
        let mut chain = ProofChain::new();
        chain.issue("argus_01", "move_stage_x", 0b1111111);
        chain.issue("argus_01", "fire_laser", 0b1111111);
        chain.issue("robot_02", "rotate_arm", 0b0111111);
        assert_eq!(chain.proofs.len(), 3);
        assert!(chain.verify_chain());
        assert_eq!(chain.count_for_device("argus_01"), 2);
        assert_eq!(chain.count_full_validation(), 2);
    }

    #[test]
    fn test_tamper_detection() {
        let mut chain = ProofChain::new();
        chain.issue("dev_01", "cmd_1", 0b1111111);
        chain.issue("dev_01", "cmd_2", 0b1111111);
        
        // Tamper with a proof
        chain.proofs[0].command = "evil_command".into();
        
        assert!(!chain.verify_chain());
    }

    #[test]
    fn test_partial_validation() {
        let proof = SafetyProof::new("dev", "read_sensor", 0b0000011, &"0".repeat(64));
        assert!(!proof.is_full_validation());
        assert!(proof.verify(&"0".repeat(64)));
    }
}
