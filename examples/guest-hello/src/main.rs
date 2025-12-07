//! Simple guest program demonstrating zp1-zkvm usage
//!
//! This program reads two numbers, adds them, computes their hash,
//! and commits the result to the public journal.

#![no_std]
#![no_main

]

use zp1_zkvm::prelude::*;

#[no_mangle]
fn main() {
    // For now, just demonstrate the API structure
    // Actual I/O will be implemented once the protocol is ready
    
    // Example: Would read inputs like this
    // let a: u32 = read();
    // let b: u32 = read();
    
    // Hardcode for demonstration
    let a: u32 = 42;
    let b: u32 = 58;
    
    // Compute sum
    let sum = a.wrapping_add(b);
    
    // Compute hash of inputs using Keccak256 precompile
    let mut data = [0u8; 8];
    data[0..4].copy_from_slice(&a.to_le_bytes());
    data[4..8].copy_from_slice(&b.to_le_bytes());
    let hash = keccak256(&data);
    
    // In a real implementation, would commit like this:
    // commit(&sum);
    // commit(&hash);
    
    // For now, just demonstrate the functions compile
    let _result = (sum, hash);
    
    // Print for debugging (when running in host mode)
    #[cfg(not(target_arch = "riscv32"))]
    {
        print("Guest program completed successfully!");
    }
}

// Set up entry point and panic handler
zp1_zkvm::entry!(main);
