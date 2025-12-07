# Guest Hello Example

A simple demonstration of the `zp1-zkvm` guest library.

## Overview

This example shows how to write a guest program that runs inside the ZP1 zkVM. The program demonstrates:
- Using the guest library API
- Invoking cryptographic precompiles (Keccak256)
- Program structure and entry point setup

## Current Status

This is a **demonstration example** of the guest library structure. The complete end-to-end execution requires:

1. ✅ Guest library (zp1-zkvm) - implemented
2. ✅ Syscall interface - defined
3. 🚧 Executor integration - in progress
4. 🚧 I/O protocol - design phase

## Code Example

```rust
#![no_std]
#![no_main]

use zp1_zkvm::prelude::*;

#[no_mangle]
fn main() {
    // Compute some values
    let a: u32 = 42;
    let b: u32 = 58;
    let sum = a.wrapping_add(b);
    
    // Use Keccak256 precompile
    let mut data = [0u8; 8];
    data[0..4].copy_from_slice(&a.to_le_bytes());
    data[4..8].copy_from_slice(&b.to_le_bytes());
    let hash = keccak256(&data);
    
    // Once I/O is ready, would commit results:
    // commit(&sum);
    // commit(&hash);
}

zp1_zkvm::entry!(main);
```

## Building

### Prerequisites

```bash
rustup target add riscv32im-unknown-none-elf
```

### Compilation

```bash
# For host architecture (testing)
cargo build --release

# For RISC-V (zkVM execution)
cargo build --target riscv32im-unknown-none-elf --release
```

## What's Next

To enable full functionality:

1. Implement host-guest I/O protocol
2. Define memory layout for inputs/outputs
3. Integrate syscalls into executor
4. Create end-to-end test

See `docs/ACTION_PLAN.md` for implementation priorities.

