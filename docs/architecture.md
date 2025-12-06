# ZP1 Architecture

## Overview
Circle STARK prover for RISC-V RV32IM over Mersenne31 ($p = 2^{31} - 1$).
- DEEP composition with FRI polynomial commitment
- LogUp lookup arguments for memory/register consistency
- AIR constraints for RV32IM instruction set
- Delegation circuits for BLAKE2s/BLAKE3, U256 ops

## Field & Constraints
- **M31 base field** ($2^{31} - 1$), quartic extension for security
- **Degree-2 constraints** for all AIR operations
- **16-bit limb decomposition** for range checks
- **Circle group domains** for FFT evaluation

## Execution Pipeline
1. **Execute**: RV32IM executor captures trace (pc, registers, memory, syscalls)
2. **Encode**: Convert trace to AIR columns with domain padding
3. **Prove**: STARK prover via DEEP FRI + LogUp + RAM permutation
4. **Verify**: Check polynomial commitments and constraint satisfaction

## Components
- **Executor** (`zp1-executor`): Deterministic RV32IM emulator, no MMU
- **AIR** (`zp1-air`): Constraint functions for all 47 RV32IM instructions
- **Prover** (`zp1-prover`): STARK with FRI, Merkle commitments, Fiat-Shamir transcript
- **Verifier**: Base + recursive proof verification
- **Delegation**: BLAKE2s/BLAKE3 circuits, U256 bigint ops (future)

## CPU AIR
**State per step**: pc, instr, opcode, rd/rs1/rs2, imm, flags, registers[32], memory ops

**Constraints** (74+ implemented):
- ALU: ADD, SUB, AND, OR, XOR, SLT, SLTU
- Shifts: SLL, SRL, SRA with bit decomposition
- Branches: BEQ, BNE, BLT, BGE, BLTU, BGEU
- Jumps: JAL, JALR with link register
- Memory: LW/SW fully constrained; LB/LBU/LH/LHU/SB/SH share value-consistency checks but still need byte/half extraction + masking wiring
- M-extension: MUL, MULH, MULHSU, MULHU, DIV, DIVU, REM, REMU
- Invariant: x0 = 0 enforced every step

## Memory Consistency
- **RAM permutation**: LogUp argument sorts by (addr, timestamp)
- **Read/write**: Consistency via grand product accumulation
- **Width handling**: Word proven; sub-word paths share value constraints pending extraction/masking wiring
- **Init table**: Program image + static data preloaded

## FRI Commitment
- **Domain**: Circle group, power-of-two sized with padding
- **Blowup**: 8x-16x for degree-2 constraints
- **DEEP**: Out-of-domain sampling with quotient polynomial
- **Folding**: Multi-round with Fiat-Shamir challenges

## Prover Pipeline
1. Trace ingestion
2. Low-degree extension (Circle FFT)
3. Constraint evaluation over domain
4. DEEP composition polynomial
5. FRI folding + Merkle commitments
6. Query openings

GPU support planned for FFT/Merkle operations.

## Implementation Status
**Completed** (90%):
- All RV32IM instruction constraints (47 ops)
- Fiat-Shamir transcript with domain separators
- Public input binding
- RAM permutation (LogUp)
- DEEP quotient verification
- x0 invariant enforcement

**In Progress** (10%):
- Full AIR integration with trace builder
- End-to-end prove/verify testing
- Performance optimization

See `PROGRESS.md` for detailed implementation tracking.
