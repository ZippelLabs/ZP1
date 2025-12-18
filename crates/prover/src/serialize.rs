//! Proof serialization using serde.
//!
//! Provides serialization/deserialization for proofs and verification keys.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zp1_primitives::M31;

/// Serialize M31 as u32.
pub fn serialize_m31<S: Serializer>(val: &M31, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u32(val.as_u32())
}

/// Deserialize M31 from u32.
pub fn deserialize_m31<'de, D: Deserializer<'de>>(deserializer: D) -> Result<M31, D::Error> {
    let val = u32::deserialize(deserializer)?;
    Ok(M31::new(val))
}

/// Serialize Vec<M31> as Vec<u32>.
pub fn serialize_m31_vec<S: Serializer>(vals: &[M31], serializer: S) -> Result<S::Ok, S::Error> {
    let u32_vals: Vec<u32> = vals.iter().map(|v| v.as_u32()).collect();
    u32_vals.serialize(serializer)
}

/// Deserialize Vec<M31> from Vec<u32>.
pub fn deserialize_m31_vec<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<M31>, D::Error> {
    let u32_vals: Vec<u32> = Vec::deserialize(deserializer)?;
    Ok(u32_vals.into_iter().map(M31::new).collect())
}

/// Serializable STARK proof.
#[derive(Clone, Serialize, Deserialize)]
pub struct SerializableProof {
    /// Trace commitment (Merkle root).
    #[serde(with = "hex_array")]
    pub trace_commitment: [u8; 32],

    /// Composition polynomial commitment.
    #[serde(with = "hex_array")]
    pub composition_commitment: [u8; 32],

    /// FRI layer commitments.
    #[serde(with = "hex_vec")]
    pub fri_commitments: Vec<[u8; 32]>,

    /// FRI final polynomial coefficients.
    #[serde(
        serialize_with = "serialize_m31_vec",
        deserialize_with = "deserialize_m31_vec"
    )]
    pub fri_final_poly: Vec<M31>,

    /// Query proofs.
    pub query_proofs: Vec<SerializableQueryProof>,

    /// Configuration used for this proof.
    pub config: ProofConfig,
}

/// Serializable query proof.
#[derive(Clone, Serialize, Deserialize)]
pub struct SerializableQueryProof {
    /// Query index in the domain.
    pub index: usize,

    /// Trace values at query point.
    #[serde(
        serialize_with = "serialize_m31_vec",
        deserialize_with = "deserialize_m31_vec"
    )]
    pub trace_values: Vec<M31>,

    /// Composition value at query point.
    #[serde(serialize_with = "serialize_m31", deserialize_with = "deserialize_m31")]
    pub composition_value: M31,

    /// Merkle authentication paths.
    pub merkle_paths: Vec<MerklePath>,

    /// FRI layer values.
    #[serde(
        serialize_with = "serialize_m31_vec",
        deserialize_with = "deserialize_m31_vec"
    )]
    pub fri_values: Vec<M31>,
}

/// Merkle authentication path.
#[derive(Clone, Serialize, Deserialize)]
pub struct MerklePath {
    /// Sibling hashes from leaf to root.
    #[serde(with = "hex_vec")]
    pub siblings: Vec<[u8; 32]>,
}

/// Proof configuration.
#[derive(Clone, Serialize, Deserialize)]
pub struct ProofConfig {
    /// Log2 of trace length.
    pub log_trace_len: usize,
    /// Blowup factor for LDE.
    pub blowup_factor: usize,
    /// Number of FRI queries.
    pub num_queries: usize,
    /// FRI folding factor.
    pub fri_folding_factor: usize,
    /// Security level (bits).
    pub security_bits: usize,
    /// Entry point PC value.
    pub entry_point: u32,
}

/// Serializable verification key.
#[derive(Clone, Serialize, Deserialize)]
pub struct VerificationKey {
    /// Configuration.
    pub config: ProofConfig,
    /// AIR constraints hash.
    #[serde(with = "hex_array")]
    pub constraints_hash: [u8; 32],
    /// Public inputs commitment.
    #[serde(with = "hex_array")]
    pub public_inputs_hash: [u8; 32],
}

/// Hex serialization for fixed-size arrays.
mod hex_array {
    use super::hex;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Expected 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

/// Hex serialization for vectors of fixed-size arrays.
mod hex_vec {
    use super::hex;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[[u8; 32]], serializer: S) -> Result<S::Ok, S::Error> {
        let strs: Vec<String> = bytes.iter().map(|b| hex::encode(b)).collect();
        strs.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<[u8; 32]>, D::Error> {
        let strs: Vec<String> = Vec::deserialize(deserializer)?;
        strs.into_iter()
            .map(|s| {
                let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
                if bytes.len() != 32 {
                    return Err(serde::de::Error::custom("Expected 32 bytes"));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(arr)
            })
            .collect()
    }
}

/// Hex encoding/decoding helper.
pub mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("Odd length hex string".into());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

impl SerializableProof {
    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to binary (bincode).
    pub fn to_bytes(&self) -> Vec<u8> {
        // Simple binary format: JSON for now
        // In production, use proper binary encoding
        self.to_json().unwrap_or_default().into_bytes()
    }

    /// Deserialize from binary.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let json = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        Self::from_json(json).map_err(|e| e.to_string())
    }

    /// Get proof size in bytes.
    pub fn size(&self) -> usize {
        self.to_bytes().len()
    }
}

impl VerificationKey {
    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_config_serde() {
        let config = ProofConfig {
            log_trace_len: 10,
            blowup_factor: 8,
            num_queries: 50,
            fri_folding_factor: 4,
            security_bits: 100,
            entry_point: 0x0,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: ProofConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.log_trace_len, 10);
        assert_eq!(parsed.security_bits, 100);
    }

    #[test]
    fn test_hex_roundtrip() {
        let bytes = [
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45,
            0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
            0x23, 0x45, 0x67, 0x89,
        ];

        let encoded = hex::encode(&bytes);
        let decoded = hex::decode(&encoded).unwrap();

        assert_eq!(decoded, bytes.to_vec());
    }

    #[test]
    fn test_verification_key_serde() {
        let vk = VerificationKey {
            config: ProofConfig {
                log_trace_len: 12,
                blowup_factor: 8,
                num_queries: 30,
                fri_folding_factor: 2,
                security_bits: 128,
                entry_point: 0x0,
            },
            constraints_hash: [1u8; 32],
            public_inputs_hash: [2u8; 32],
        };

        let json = vk.to_json().unwrap();
        let parsed = VerificationKey::from_json(&json).unwrap();

        assert_eq!(parsed.config.log_trace_len, 12);
        assert_eq!(parsed.constraints_hash, [1u8; 32]);
    }
}
