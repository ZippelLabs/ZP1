//! I/O operations for guest programs
//!
//! This module provides functions for communicating with the host:
//! - Reading inputs from the host
//! - Committing outputs to the public journal
//! - Providing hints to the prover (not verified)

use serde::{Deserialize, Serialize};

/// Read typed input from the host
///
/// This function reads serialized data from the host and deserializes it.
/// The data must implement `serde::Deserialize`.
///
/// # Example
///
/// ```rust,ignore
/// let x: u32 = read();
/// let data: Vec<u8> = read();
/// ```
pub fn read<T: for<'de> Deserialize<'de>>() -> T {
    #[cfg(target_arch = "riscv32")]
    {
        // TODO: Implement actual I/O mechanism
        // For now, this is a placeholder that will be implemented
        // when the host-guest I/O protocol is defined
        panic!("read() not yet implemented");
    }
    
    #[cfg(not(target_arch = "riscv32"))]
    {
        panic!("read() only works in zkVM guest (riscv32 target)");
    }
}

/// Commit typed output to the public journal
///
/// This function serializes the value and commits it to the public outputs
/// that will be verified as part of the proof.
///
/// # Example
///
/// ```rust,ignore
/// commit(&42u32);
/// commit(&vec![1u8, 2, 3, 4]);
/// ```
pub fn commit<T: Serialize>(value: &T) {
    #[cfg(target_arch = "riscv32")]
    {
        // TODO: Implement actual I/O mechanism
        // This will use the COMMIT syscall once integrated
        panic!("commit() not yet implemented");
    }
    
    #[cfg(not(target_arch = "riscv32"))]
    {
        panic!("commit() only works in zkVM guest (riscv32 target)");
    }
}

/// Provide hint to prover (not cryptographically verified)
///
/// Hints allow guest programs to provide data to the prover that
/// is not part of the public inputs/outputs. This is useful for
/// optimization but should never be trusted for correctness.
///
/// # Security
///
/// **WARNING**: Hints are NOT verified. Never use hints for security-critical
/// data. Always verify hint data within the guest program.
///
/// # Example
///
/// ```rust,ignore
/// // Provide a hint about expected intermediate value
/// hint(&intermediate_computation);
/// ```
pub fn hint<T: Serialize>(_value: &T) {
    #[cfg(target_arch = "riscv32")]
    {
        // Hints are optional, so we just no-op for now
        // When implemented, this will use the HINT syscall
    }
}

/// Write to stdout (for debugging)
///
/// This is primarily for testing and debugging guest programs.
/// Output is not part of the verified computation.
pub fn print(msg: &str) {
    #[cfg(target_arch = "riscv32")]
    {
        let bytes = msg.as_bytes();
        unsafe {
            core::arch::asm!(
                "li a7, 0x01",  // WRITE syscall
                "li a0, 1",      // fd = stdout
                "mv a1, {ptr}",
                "mv a2, {len}",
                "ecall",
                ptr = in(reg) bytes.as_ptr(),
                len = in(reg) bytes.len(),
                out("a0") _,  // clobber return value
            );
        }
    }
    
    #[cfg(not(target_arch = "riscv32"))]
    {
        // For testing outside zkVM
        #[cfg(feature = "std")]
        println!("{}", msg);
    }
}
