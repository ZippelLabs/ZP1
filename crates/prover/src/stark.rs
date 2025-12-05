//! STARK prover pipeline.
//!
//! The prover pipeline:
//! 1. Take execution trace columns
//! 2. Commit to trace polynomials
//! 3. Receive constraint randomness
//! 4. Evaluate AIR constraints on the trace
//! 5. Build the DEEP composition polynomial
//! 6. Commit to composition polynomial via FRI
//! 7. Generate query phase proofs

use crate::{
    channel::ProverChannel,
    commitment::{MerkleTree, MerkleProof},
    lde::TraceLDE,
    fri::{FriConfig, FriProver, FriProof, FriLayer},
};
use zp1_primitives::{M31, QM31};

/// Configuration for the STARK prover.
#[derive(Clone, Debug)]
pub struct StarkConfig {
    /// Log2 of trace length.
    pub log_trace_len: usize,
    /// Blowup factor for LDE (typically 8 or 16).
    pub blowup_factor: usize,
    /// Number of FRI queries.
    pub num_queries: usize,
    /// FRI folding factor.
    pub fri_folding_factor: usize,
}

impl Default for StarkConfig {
    fn default() -> Self {
        Self {
            log_trace_len: 10,
            blowup_factor: 8,
            num_queries: 50,
            fri_folding_factor: 4,
        }
    }
}

impl StarkConfig {
    /// Create a new config for a specific trace length.
    pub fn for_trace_len(log_trace_len: usize) -> Self {
        Self {
            log_trace_len,
            ..Default::default()
        }
    }

    /// Get trace length.
    pub fn trace_len(&self) -> usize {
        1 << self.log_trace_len
    }

    /// Get LDE domain size.
    pub fn lde_domain_size(&self) -> usize {
        self.trace_len() * self.blowup_factor
    }
}

/// STARK proof structure.
#[derive(Clone)]
pub struct StarkProof {
    /// Merkle commitment to the trace.
    pub trace_commitment: [u8; 32],
    /// Merkle commitment to the composition polynomial.
    pub composition_commitment: [u8; 32],
    /// FRI proof.
    pub fri_proof: FriProof,
    /// Query proofs.
    pub query_proofs: Vec<QueryProof>,
}

/// Proof data for a single query.
#[derive(Clone)]
pub struct QueryProof {
    /// Query index in the LDE domain.
    pub index: usize,
    /// Trace column values at query point.
    pub trace_values: Vec<M31>,
    /// Trace Merkle proof.
    pub trace_proof: MerkleProof,
    /// Composition polynomial value at query point.
    pub composition_value: M31,
    /// Composition Merkle proof.
    pub composition_proof: MerkleProof,
}

/// STARK prover.
pub struct StarkProver {
    config: StarkConfig,
    channel: ProverChannel,
}

impl StarkProver {
    /// Create a new STARK prover.
    pub fn new(config: StarkConfig) -> Self {
        Self {
            config,
            channel: ProverChannel::new(b"zp1-stark-v1"),
        }
    }

    /// Generate a STARK proof from trace columns.
    ///
    /// # Arguments
    /// * `trace_columns` - Each inner Vec is a column of trace values.
    ///
    /// # Returns
    /// A STARK proof that can be verified.
    pub fn prove(&mut self, trace_columns: Vec<Vec<M31>>) -> StarkProof {
        let _num_cols = trace_columns.len();
        let trace_len = trace_columns[0].len();

        assert!(trace_len.is_power_of_two(), "Trace length must be power of 2");
        assert_eq!(trace_len, self.config.trace_len(), "Trace length mismatch");

        // Step 1: Low-degree extend the trace
        let trace_lde = TraceLDE::new(&trace_columns, self.config.blowup_factor);
        let domain_size = trace_lde.domain_size();

        // Step 2: Commit to trace (first column for simplicity)
        // In production, would commit to all columns interleaved or separately
        let trace_tree = MerkleTree::new(&trace_lde.columns[0]);
        let trace_commitment = trace_tree.root();

        // Absorb trace commitment into channel
        self.channel.absorb(&trace_commitment);

        // Step 3: Receive constraint randomness from verifier (Fiat-Shamir)
        let constraint_random = self.channel.squeeze_qm31();

        // Step 4: Evaluate composition polynomial
        let composition_evals = self.evaluate_composition_polynomial(
            &trace_lde,
            constraint_random,
        );

        // Step 5: Commit to composition polynomial
        let composition_tree = MerkleTree::new(&composition_evals);
        let composition_commitment = composition_tree.root();

        self.channel.absorb(&composition_commitment);

        // Step 6: DEEP quotient / OODS point
        let _oods_point = self.channel.squeeze_qm31();

        // Step 7: FRI
        let fri_config = FriConfig {
            log_domain_size: self.config.log_trace_len + (self.config.blowup_factor.trailing_zeros() as usize),
            num_queries: self.config.num_queries,
            folding_factor: self.config.fri_folding_factor,
            final_degree: 1,
        };

        let fri_prover = FriProver::new(fri_config);
        let (_fri_layers, fri_proof) = fri_prover.commit(composition_evals.clone(), &mut self.channel);

        // Step 8: Query phase
        let query_indices = self.channel.squeeze_query_indices(
            self.config.num_queries,
            domain_size,
        );

        let query_proofs = self.generate_query_proofs(
            &query_indices,
            &trace_tree,
            &trace_lde,
            &composition_tree,
            &composition_evals,
        );

        StarkProof {
            trace_commitment,
            composition_commitment,
            fri_proof,
            query_proofs,
        }
    }

    /// Evaluate the composition polynomial at all LDE domain points.
    fn evaluate_composition_polynomial(
        &self,
        trace_lde: &TraceLDE,
        random: QM31,
    ) -> Vec<M31> {
        let domain_size = trace_lde.domain_size();
        let blowup = self.config.blowup_factor;

        // For now, create a simple composition polynomial
        // In production, this would evaluate all AIR constraints

        let mut composition = vec![M31::ZERO; domain_size];

        // Use first component of random as scalar
        let alpha = random.c0;

        for i in 0..domain_size {
            // Get values at current row
            let col0 = trace_lde.get(0, i);
            // Get values at next row (with wraparound)
            let col0_next = trace_lde.get(0, (i + blowup) % domain_size);

            // Boundary constraint: first row starts at 0
            let boundary_constraint = if i < blowup {
                col0
            } else {
                M31::ZERO
            };

            // Transition constraint: clock increments
            let transition_constraint = col0_next - col0 - M31::ONE;

            // Combine with randomness
            composition[i] = boundary_constraint + alpha * transition_constraint;
        }

        composition
    }

    /// Generate query proofs for all query indices.
    fn generate_query_proofs(
        &self,
        indices: &[usize],
        trace_tree: &MerkleTree,
        trace_lde: &TraceLDE,
        composition_tree: &MerkleTree,
        composition_evals: &[M31],
    ) -> Vec<QueryProof> {
        indices
            .iter()
            .map(|&idx| {
                let trace_values = trace_lde.get_row(idx);
                let trace_proof = trace_tree.prove(idx);
                let composition_value = composition_evals[idx];
                let composition_proof = composition_tree.prove(idx);

                QueryProof {
                    index: idx,
                    trace_values,
                    trace_proof,
                    composition_value,
                    composition_proof,
                }
            })
            .collect()
    }
}

/// Constraint evaluator for AIR.
pub struct ConstraintEvaluator {
    /// Number of trace columns.
    pub num_cols: usize,
    /// Number of constraint polynomials.
    pub num_constraints: usize,
}

impl ConstraintEvaluator {
    /// Create a new constraint evaluator.
    pub fn new(num_cols: usize, num_constraints: usize) -> Self {
        Self {
            num_cols,
            num_constraints,
        }
    }

    /// Evaluate all constraints at a single point.
    pub fn evaluate(
        &self,
        trace_row: &[M31],
        trace_row_next: &[M31],
        alphas: &[M31],
        is_boundary: bool,
    ) -> M31 {
        let mut result = M31::ZERO;

        // Boundary constraints (first row)
        if is_boundary && !trace_row.is_empty() {
            result += alphas.get(0).copied().unwrap_or(M31::ONE) * trace_row[0];
        }

        // Transition constraints
        if !trace_row.is_empty() && !trace_row_next.is_empty() {
            let constraint = trace_row_next[0] - trace_row[0] - M31::ONE;
            result += alphas.get(1).copied().unwrap_or(M31::ONE) * constraint;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stark_config() {
        let config = StarkConfig::for_trace_len(10);
        assert_eq!(config.trace_len(), 1024);
        assert_eq!(config.lde_domain_size(), 8192);
    }

    #[test]
    fn test_simple_proof() {
        // Create a simple trace: just a clock column
        let trace_len = 8;
        let clock: Vec<M31> = (0..trace_len).map(|i| M31::new(i as u32)).collect();

        let config = StarkConfig {
            log_trace_len: 3, // 8 rows
            blowup_factor: 4,
            num_queries: 3,
            fri_folding_factor: 2,
        };

        let mut prover = StarkProver::new(config);
        let proof = prover.prove(vec![clock]);

        // Verify proof structure
        assert_eq!(proof.trace_commitment.len(), 32);
        assert_eq!(proof.composition_commitment.len(), 32);
        assert_eq!(proof.query_proofs.len(), 3);
    }
}
