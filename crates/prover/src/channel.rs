//! Fiat-Shamir transcript channel for the prover using SHA-256.
//!
//! This uses the same SHA-256-based transcript construction as the verifier
//! channel, ensuring prover and verifier derive identical challenges from the
//! same sequence of transcript messages.
//!
//! # Domain Separation
//!
//! Every channel is initialized by hashing the domain separator, binding all
//! subsequent challenges to the specific protocol context.
//!
//! # Byte-to-Field Encoding
//!
//! `absorb` prefixes each call with the byte-length of the data before hashing,
//! making the encoding injective for sequences of variable-length inputs.

use sha2::{Digest, Sha256};
use zp1_primitives::{M31, QM31};

/// Prover channel for Fiat-Shamir transcript (mirrors `VerifierChannel`).
#[derive(Clone)]
pub struct ProverChannel {
    hasher: Sha256,
}

impl ProverChannel {
    /// Create a new prover channel bound to `domain_separator`.
    ///
    /// The domain separator is absorbed first, so two channels with different
    /// separators always produce different challenges.
    pub fn new(domain_separator: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        // Length-prefix the domain separator to avoid extension collisions
        hasher.update((domain_separator.len() as u64).to_le_bytes());
        hasher.update(domain_separator);
        Self { hasher }
    }

    /// Absorb arbitrary bytes into the transcript.
    ///
    /// The byte-length is written before the data so that, e.g.,
    /// `absorb(b"ab")` and `absorb(b"a"); absorb(b"b")` produce
    /// different transcript states.
    pub fn absorb(&mut self, data: &[u8]) {
        // Length prefix for injectivity across variable-length inputs
        self.hasher.update((data.len() as u64).to_le_bytes());
        self.hasher.update(data);
    }

    /// Absorb a 32-byte commitment into the transcript.
    pub fn absorb_commitment(&mut self, commitment: &[u8; 32]) {
        self.absorb(commitment);
    }

    /// Absorb an M31 field element into the transcript.
    pub fn absorb_felt(&mut self, felt: M31) {
        self.hasher.update(felt.as_u32().to_le_bytes());
    }

    /// Squeeze a challenge in M31.
    pub fn squeeze_challenge(&mut self) -> M31 {
        let hash = self.hasher.clone().finalize();
        self.hasher.update(&hash);
        let bytes: [u8; 4] = hash[0..4].try_into().unwrap();
        let val = u32::from_le_bytes(bytes);
        M31::new(val % M31::P)
    }

    /// Squeeze a challenge in QM31 (four independent M31 challenges).
    pub fn squeeze_extension_challenge(&mut self) -> QM31 {
        let c0 = self.squeeze_challenge();
        let c1 = self.squeeze_challenge();
        let c2 = self.squeeze_challenge();
        let c3 = self.squeeze_challenge();
        QM31::new(c0, c1, c2, c3)
    }

    /// Alias for `squeeze_extension_challenge`.
    pub fn squeeze_qm31(&mut self) -> QM31 {
        self.squeeze_extension_challenge()
    }

    /// Squeeze `n` query indices in `[0, domain_size)`.
    pub fn squeeze_query_indices(&mut self, n: usize, domain_size: usize) -> Vec<usize> {
        let mut indices = Vec::with_capacity(n);
        while indices.len() < n {
            let hash = self.hasher.clone().finalize();
            self.hasher.update(&hash);
            for chunk in hash.chunks(4) {
                if indices.len() >= n {
                    break;
                }
                let bytes: [u8; 4] = chunk.try_into().unwrap();
                let val = u32::from_le_bytes(bytes) as usize;
                indices.push(val % domain_size);
            }
        }
        indices.truncate(n);
        indices
    }
}

impl Default for ProverChannel {
    fn default() -> Self {
        Self::new(b"zp1-default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_deterministic() {
        let mut ch1 = ProverChannel::new(b"test");
        let mut ch2 = ProverChannel::new(b"test");

        ch1.absorb(b"test data");
        ch2.absorb(b"test data");

        let c1 = ch1.squeeze_challenge();
        let c2 = ch2.squeeze_challenge();

        assert_eq!(c1, c2);
    }

    #[test]
    fn test_domain_separator_matters() {
        let mut ch1 = ProverChannel::new(b"protocol-a");
        let mut ch2 = ProverChannel::new(b"protocol-b");

        ch1.absorb(b"same data");
        ch2.absorb(b"same data");

        // Different domain separators must yield different challenges
        assert_ne!(ch1.squeeze_challenge(), ch2.squeeze_challenge());
    }

    #[test]
    fn test_absorb_injective() {
        // absorb(b"ab") vs absorb(b"a") + absorb(b"b") must differ
        let mut ch1 = ProverChannel::new(b"test");
        ch1.absorb(b"ab");

        let mut ch2 = ProverChannel::new(b"test");
        ch2.absorb(b"a");
        ch2.absorb(b"b");

        assert_ne!(ch1.squeeze_challenge(), ch2.squeeze_challenge());
    }

    #[test]
    fn test_query_indices() {
        let mut ch = ProverChannel::new(b"test");
        ch.absorb(b"seed");

        let indices = ch.squeeze_query_indices(10, 1024);
        assert_eq!(indices.len(), 10);
        for &idx in &indices {
            assert!(idx < 1024);
        }
    }
}
