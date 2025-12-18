# ZP1 Implementation Status

Last updated: Dec 2024

## What's Done

- RV32IM execution (47 instructions)
- STARK prover with FRI
- M31/QM31 field arithmetic
- LogUp memory consistency
- Bitwise lookup tables
- Metal GPU kernels (NTT, LDE, Merkle)
- Precompiles: Keccak, SHA256, Blake2b, ECRECOVER, Ed25519, RIPEMD160

## What's Not Done

### Roadmap to Parity

To become a functional, production-grade zkVM, the following components are required:

1.  **Execution Engine (Critical)**
    - **Current**: `generate_trace` is a stub; `guest_executor` is unimplemented.
    - **Needed**: A full RISC-V interpreter that executes ELF binaries and records the 77-column trace row-by-row.

2.  **Guest Toolchain**
    - **Current**: No standard way to build guests.
    - **Needed**: A CLI tool (e.g., `cargo-zp1`) to compile Rust to `riscv32im` ELFs with the correct memory map and entry point.

3.  **Verifier Logic (Critical)**
    - **Current**: `verify_constraint_consistency` is a placeholder.
    - **Needed**: Full evaluation of AIR constraints at the challenge point $z$ in the verifier.

4.  **Recursion**
    - **Current**: Missing.
    - **Needed**: A recursion circuit to verify ZP1 proofs within ZP1, enabling large program proving via continuations.

5.  **On-Chain Verifier**
    - **Current**: Missing.
    

### Blocking Issues

**Guest Executor** (`ethereum/src/guest_executor.rs:53, 80`)
- `execute_guest()` and `build_guest()` not implemented
- Blocks Ethereum block proving
- Needs cargo-zp1 or build.rs integration

### Known TODOs

| File | Line | Issue |
|------|------|-------|
| `primitives/src/circle.rs` | 660 | Circle FFT is O(n²), should be O(n log n) |
| `ethereum/src/prover.rs` | 67, 215 | EVM→RISC-V trace conversion incomplete |
| `ethereum/src/aggregation.rs` | 87, 149 | Recursive proof aggregation not done |
| `cli/src/main.rs` | 305, 411 | Transaction proving, full verification |
| `prover/src/recursion.rs` | 105, 318 | Placeholder proofs |

### Placeholders (Work Required)

**AIR Constraints** (`air/src/cpu.rs`)
- Lines 547, 569, 608, 630, 654, 677: Load/store byte extraction
- Lines 715, 735: Alignment checks assume pre-verified
- Lines 778-966: Test helpers, not production code

**Verifier** (`verifier/src/verify.rs`)
- Lines 371, 395: Constraint verification is incomplete

**Delegation**
- `ed25519.rs:147`: Stub when ed25519-dalek missing
- `secp256r1.rs:171`: Stub when p256 missing

**GPU**
- `gpu/cuda.rs:318, 745`: CUDA detection and device naming are stubs

## Optimization Opportunities

### High Impact

1. **Circle FFT butterfly** - Code exists in `FastCircleFFT` struct, just not wired up. Would give O(n log n) instead of O(n²).

2. **Use Plonky3 SIMD** - `p3_interop.rs` has `p3_fast_dft()` ready. 4-8x speedup.

3. **CUDA integration** - Shaders written, runtime missing.

### Medium Impact

4. Merkle multi-proof batching (`commitment.rs:295`)
5. Precompute lookup tables instead of per-proof
6. Parallel constraint evaluation with Rayon

### Low Priority

7. SoA memory layout for traces
8. GPU FRI folding

## How to Find Issues

```bash
# Find unimplemented
grep -rn "unimplemented!" crates/

# Find TODOs
grep -rn "// TODO" crates/

# Find placeholders
grep -rn "placeholder" crates/

# Find stubs  
grep -rn "stub" crates/
```

## Testing After Fixes

```bash
cargo test --workspace
cargo bench -p zp1-prover
cargo bench -p zp1-primitives
```
