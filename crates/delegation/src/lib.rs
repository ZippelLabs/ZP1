//! zp1-delegation: Delegated gadgets for precompiles.
//!
//! BLAKE2s/BLAKE3 hash circuits, Keccak-256 for Ethereum, ECRECOVER for signatures, and U256 bigint operations.

pub mod blake;
pub mod bigint;
pub mod keccak;
pub mod ecrecover;
