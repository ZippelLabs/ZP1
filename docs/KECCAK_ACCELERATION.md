# Keccak-256 Acceleration

## Overview

Keccak-256 is a critical precompile for Ethereum execution, used for state trie hashing, transaction hashing, and contract address derivation. Since ~20-30% of Ethereum execution time is spent hashing, running Keccak in pure RISC-V instructions would be catastrophically slow and generate enormous traces (100,000+ rows per hash).

**Solution**: We delegate Keccak operations to a specialized circuit that runs outside the main RISC-V trace.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ RISC-V Program                                      │
│   ...                                               │
│   li a0, input_ptr                                  │
│   li a1, input_len                                  │
│   li a2, output_ptr                                 │
│   li a7, 0x1000    # Keccak syscall number         │
│   ecall            # Delegated hash                 │
│   ...                                               │
└─────────────────────────────────────────────────────┘
          │
          │ Intercepted by Executor
          ▼
┌─────────────────────────────────────────────────────┐
│ Keccak Delegation Circuit                          │
│                                                      │
│  • Absorb Phase (1 row per 136-byte block)         │
│  • Permutation (24 rounds × 5 steps)                │
│  • Squeeze Phase (1 row for output)                 │
│                                                      │
│  Total: ~26 rows per hash (vs. 100,000+ in RISC-V) │
└─────────────────────────────────────────────────────┘
          │
          │ Linked via Lookup Argument
          ▼
┌─────────────────────────────────────────────────────┐
│ Main RISC-V Trace                                   │
│   Row 100: ECALL with Keccak flag                   │
│   Row 101: PC = PC + 4 (continue)                   │
└─────────────────────────────────────────────────────┘
```

## Usage

### From RISC-V Programs

```c
// Example: Hash a 32-byte message
uint8_t message[32] = {...};
uint8_t hash[32];

// Set up registers
register uint32_t a0 asm("a0") = (uint32_t)message;  // Input pointer
register uint32_t a1 asm("a1") = 32;                  // Input length
register uint32_t a2 asm("a2") = (uint32_t)hash;     // Output pointer
register uint32_t a7 asm("a7") = 0x1000;             // Keccak syscall

// Invoke delegated Keccak
asm volatile("ecall" : "+r"(a0) : "r"(a1), "r"(a2), "r"(a7));

// hash[] now contains Keccak256(message)
```

### From Rust Executor

```rust
use zp1_executor::Cpu;

let mut cpu = Cpu::new();
cpu.enable_tracing();

// Load program with Keccak call
cpu.load_program(0x1000, &program)?;

// Execute - Keccak calls are automatically delegated
while cpu.step()?.is_some() {}

// Trace includes delegation markers
let trace = cpu.take_trace().unwrap();
for row in &trace.rows {
    if let MemOp::Keccak256 { input_ptr, input_len, output_ptr } = row.mem_op {
        println!("Keccak hash: {} bytes at 0x{:x}", input_len, input_ptr);
    }
}
```

## Syscall Interface

### Registers

| Register | Purpose | Type |
|----------|---------|------|
| `a0` (x10) | Input pointer | u32 address |
| `a1` (x11) | Input length | u32 bytes |
| `a2` (x12) | Output pointer | u32 address (32 bytes required) |
| `a7` (x17) | Syscall number | `0x1000` (Keccak256) |

### Return Value

| Register | Value | Meaning |
|----------|-------|---------|
| `a0` | `0` | Success |
| `a0` | `non-zero` | Error (via trap) |

### Errors

- **Out of Bounds**: Input or output pointer is invalid
- **Alignment**: No alignment requirement (unlike memory ops)

## Implementation Details

### Trace Structure

Each Keccak-256 invocation generates:

1. **Absorption Rows**: `⌈input_len / 136⌉` rows
   - Each row represents XORing a 136-byte block into state
   - Records: input block, pre-state, post-state
2. **Permutation Rows**: 24 rows
   - One per Keccak-f[1600] round
   - Records: round number, round constant, state transitions
3. **Squeeze Row**: 1 row
   - Extracts first 32 bytes of state as output

**Total**: ~26 rows for typical 32-byte input (vs. ~100,000 if done in RISC-V)

### Field Representation

Keccak operates on 64-bit lanes, but we use M31 (31-bit field). Each u64 is split into 3 limbs:

```rust
limb0 = bits[0..30]   // 31 bits
limb1 = bits[31..61]  // 31 bits  
limb2 = bits[62..63]  // 2 bits
```

Reconstruction: `value = limb0 + (limb1 << 31) + (limb2 << 62)`

### Constraint System

The Keccak circuit enforces:

1. **Absorption**: `state' = state XOR pad(input_block)`
2. **Permutation**: 5 steps per round (θ, ρ, π, χ, ι)
3. **Round Constants**: Correct constants applied in ι step
4. **Rotation Amounts**: Correct shifts in ρ step
5. **Linking**: Lookup from main trace to Keccak trace

## Performance

### Without Delegation (Pure RISC-V)

- **Instructions per hash**: ~50,000 (SHA-3 in software)
- **Trace rows**: ~100,000 (with witnesses)
- **Prover time**: ~5 seconds per hash
- **Proof size**: ~500 KB per hash

### With Delegation (Specialized Circuit)

- **Trace rows**: ~26
- **Prover time**: ~0.1 seconds per hash
- **Proof size**: ~10 KB per hash
- **Speedup**: **50x faster**, **50x smaller**

## Testing

Run the Keccak test suite:

```bash
cargo test --package zp1-executor test_keccak
```

Test vectors:

```rust
keccak256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
keccak256("hello") = 1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
```

## Future Work

1. **Batch Proving**: Prove multiple Keccak calls in parallel
2. **GPU Acceleration**: Offload Keccak permutations to GPU
3. **Other Precompiles**: SHA-256, ECRECOVER, MODEXP
4. **State Proof Integration**: Link Keccak to Merkle Patricia Trie proofs

## References

- [NIST FIPS 202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf) - SHA-3 Standard
- [Keccak Team](https://keccak.team/keccak.html) - Original specification
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) - Section 4.3 (Keccak-256)
