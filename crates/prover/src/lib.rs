//! zp1-prover: STARK prover with DEEP FRI.
//!
//! CPU and GPU backends for commitment, LDE, constraint evaluation, and FRI.

pub mod bitwise_tables;
pub mod channel;
pub mod commitment;
pub mod delegation;
pub mod fri;
pub mod gpu;
pub mod lde;
pub mod logup;
pub mod memory;
pub mod parallel;
pub mod ram;
pub mod recursion;
pub mod serialize;
pub mod snark;
pub mod stark;

pub use channel::ProverChannel;
pub use commitment::MerkleTree;
pub use delegation::{
    DelegationArgumentProver, DelegationCall, DelegationColumns, DelegationResult,
    DelegationSubtree, DelegationType,
};
pub use gpu::{detect_devices, DeviceType, GpuBackend, GpuDevice, GpuError};
pub use lde::{LdeDomain, TraceLDE};
pub use logup::{LogUpProver, LookupTable, PermutationArgument, RangeCheck};
pub use memory::{MemoryAccess, MemoryColumns, MemoryConsistencyProver, MemoryOp};
pub use parallel::{parallel_fri_fold, parallel_lde, parallel_merkle_tree, ParallelConfig};
pub use ram::{ChunkMemorySubtree, RamAccess, RamArgumentProver, RamColumns, RamOp};
pub use recursion::{RecursionConfig, RecursiveProof, RecursiveProver, SegmentedProver};
pub use serialize::{ProofConfig, SerializableProof, VerificationKey};
pub use snark::{
    groth16_wrapper, halo2_wrapper, plonk_wrapper, SnarkConfig, SnarkError, SnarkProof,
    SnarkSystem, SnarkVerifier, SnarkWrapper,
};
pub use stark::{QueryProof, StarkConfig, StarkProof, StarkProver};
