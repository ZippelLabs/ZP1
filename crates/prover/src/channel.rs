//! Fiat-Shamir transcript channel for the prover using Plonky3.

use p3_challenger::{CanObserve, CanSample, DuplexChallenger};
use p3_poseidon2::Poseidon2;
use zp1_primitives::{M31, QM31, to_p3, from_p3};
use p3_mersenne_31::{Poseidon2ExternalLayerMersenne31, Poseidon2InternalLayerMersenne31};
use rand::SeedableRng;
use rand::rngs::StdRng;

// Poseidon2 configuration: Width 16, M31 field
type Permutation = Poseidon2<zp1_primitives::P3M31, Poseidon2ExternalLayerMersenne31<16>, Poseidon2InternalLayerMersenne31, 16, 5>;
type Challenger = DuplexChallenger<zp1_primitives::P3M31, Permutation, 16, 8>;

/// Prover channel for Fiat-Shamir transcript.
#[derive(Clone)]
pub struct ProverChannel {
    challenger: Challenger,
}

impl ProverChannel {
    /// Create a new prover channel.
    pub fn new(_domain_separator: &[u8]) -> Self {
        // Initialize Poseidon2 permutation
        let mut rng = StdRng::seed_from_u64(42);
        let permutation = Poseidon2::new_from_rng_128(&mut rng);
        let mut challenger = DuplexChallenger::new(permutation);
        
        // TODO: Absorb domain separator properly (convert to field elements)
        // For now we just start fresh
        Self { challenger }
    }

    /// Absorb bytes into the transcript.
    pub fn absorb(&mut self, data: &[u8]) {
        // Naive byte absorption - pack into field elements
        // In a real implementation, we'd use a proper byte-to-field packing
        for chunk in data.chunks(4) {
            let mut bytes = [0u8; 4];
            bytes[0..chunk.len()].copy_from_slice(chunk);
            let val = u32::from_le_bytes(bytes) % 2147483647; // M31 modulus
            self.challenger.observe(to_p3(M31::new(val)));
        }
    }

    /// Absorb a 32-byte commitment.
    pub fn absorb_commitment(&mut self, commitment: &[u8; 32]) {
        self.absorb(commitment);
    }

    /// Absorb an M31 field element.
    pub fn absorb_felt(&mut self, felt: M31) {
        self.challenger.observe(to_p3(felt));
    }

    /// Squeeze a challenge in M31.
    pub fn squeeze_challenge(&mut self) -> M31 {
        from_p3(self.challenger.sample())
    }

    /// Squeeze a challenge in QM31 (extension field).
    pub fn squeeze_extension_challenge(&mut self) -> QM31 {
        let c0 = self.squeeze_challenge();
        let c1 = self.squeeze_challenge();
        let c2 = self.squeeze_challenge();
        let c3 = self.squeeze_challenge();
        QM31::new(c0, c1, c2, c3)
    }

    /// Alias for squeeze_extension_challenge.
    pub fn squeeze_qm31(&mut self) -> QM31 {
        self.squeeze_extension_challenge()
    }

    /// Squeeze n query indices in range [0, domain_size).
    pub fn squeeze_query_indices(&mut self, n: usize, domain_size: usize) -> Vec<usize> {
        let mut indices = Vec::with_capacity(n);
        for _ in 0..n {
            let val = self.squeeze_challenge().value() as usize;
            indices.push(val % domain_size);
        }
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
