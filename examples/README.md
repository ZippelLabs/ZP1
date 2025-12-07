# ZP1 Examples

Collection of example programs demonstrating ZP1's zkVM capabilities.

## Examples Overview

### 1. **fibonacci** - Basic Computation
Simple Fibonacci number calculation showing basic RISC-V execution.
- **Complexity**: Beginner
- **Instructions**: ~100 cycles
- **Demonstrates**: Arithmetic, loops, memory writes

### 2. **keccak** - Keccak-256 Hashing
Demonstrates accelerated Keccak-256 precompile with delegation.
- **Complexity**: Intermediate
- **Speedup**: 100,000x vs pure RISC-V
- **Demonstrates**: Syscalls, delegation, cryptographic acceleration

### 3. **sha256** - SHA-256 Hashing
Shows SHA-256 precompile performance improvements.
- **Complexity**: Intermediate
- **Speedup**: 40,000-100,000x
- **Demonstrates**: SHA-256 delegation, multiple hashes

### 4. **ecrecover** - Ethereum Signature Recovery
Critical for Ethereum: recovers addresses from signatures.
- **Complexity**: Advanced
- **Speedup**: 50,000-100,000x
- **Demonstrates**: ECRECOVER, EIP-155 support, Ethereum integration

### 5. **memory-test** - Memory Operations
Comprehensive test of all load/store instructions.
- **Complexity**: Intermediate
- **Demonstrates**: Memory model, LogUp argument, consistency proofs

### 6. **guest-hello** - Guest Program Template
Starting template for writing your own guest programs.
- **Complexity**: Beginner
- **Demonstrates**: Project structure, no_std Rust

## Quick Start

### Prerequisites

```bash
# Install RISC-V target
rustup target add riscv32im-unknown-none-elf

# Install cargo-binutils for objcopy
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

### Building an Example

```bash
cd examples/fibonacci
cargo build --release --target riscv32im-unknown-none-elf
cargo objcopy --release -- -O binary fibonacci.bin
```

### Running with ZP1

```bash
cd /Users/zippellabs/Developer/zp1
cargo run --release -- prove fibonacci examples/fibonacci/fibonacci.bin
```

## Build All Examples

```bash
./examples/build_all.sh
```

This will build all examples and create `.bin` files ready for proving.

## Performance Comparison

| Example | Pure RISC-V Cycles | With Delegation | Speedup |
|---------|-------------------|-----------------|---------|
| fibonacci | ~100 | ~100 | 1x (no delegation) |
| keccak | ~10M | ~100 | 100,000x |
| sha256 | ~8M | ~80 | 100,000x |
| ecrecover | ~10M | ~100 | 100,000x |
| memory-test | ~500 | ~500 | 1x (no delegation) |

## Delegation Benefits

ZP1's delegation framework provides massive speedups:

1. **Cryptographic Operations**: 50,000-100,000x faster
2. **Trace Size**: Reduced from millions to hundreds of rows
3. **Proof Time**: Proportionally reduced
4. **Ethereum Viability**: Makes block proving practical

## Writing Your Own Examples

1. Copy `guest-hello` as a template
2. Write your `main.rs` with `#![no_std]` and `#![no_main]`
3. Use syscalls for delegation (see `keccak` example)
4. Build for `riscv32im-unknown-none-elf` target
5. Extract binary with `cargo objcopy`

## Syscall Reference

### Available Syscalls

| Syscall | Number | Description |
|---------|--------|-------------|
| HALT | 0x00 | Terminate program |
| WRITE | 0x01 | Output data |
| READ | 0x02 | Input data |
| COMMIT | 0x03 | Commit to journal |
| KECCAK256 | 0x10 | Keccak-256 hash |
| SHA256 | 0x11 | SHA-256 hash |
| RIPEMD160 | 0x12 | RIPEMD-160 hash |
| BLAKE2B | 0x13 | Blake2b hash |
| ECRECOVER | 0x14 | Ethereum signature recovery |
| MODEXP | 0x15 | Modular exponentiation |

### Syscall Convention

```rust
unsafe {
    core::arch::asm!(
        "ecall",
        in("a7") syscall_number,  // Syscall ID
        in("a0") arg0,             // First argument
        in("a1") arg1,             // Second argument
        in("a2") arg2,             // Third argument
        options(nostack)
    );
}
```

## Troubleshooting

### "cannot find crate for `std`"
Make sure you have `#![no_std]` at the top of your file.

### "undefined reference to `_start`"
Add `#![no_main]` and define your own `_start` function.

### Binary too large
Use release mode with LTO:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Syscall not working
Check that:
1. Syscall number matches the table above
2. Using correct register conventions (a7, a0-a6)
3. Arguments are properly formatted (pointers, lengths)

## Further Reading

- [ZP1 User Guide](../docs/USER_GUIDE.md)
- [Architecture Overview](../docs/architecture.md)
- [Keccak Acceleration](../docs/KECCAK_ACCELERATION.md)
- [ECRECOVER Delegation](../docs/ECRECOVER_ACCELERATION.md)

## Inspiration

These examples are inspired by [ZKsync Airbender](https://github.com/matter-labs/zksync-airbender/tree/main/examples), adapted for ZP1's architecture and delegation model.
