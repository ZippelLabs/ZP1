# ZP1 User Guide

Complete guide for using the ZP1 RISC-V STARK prover.

---

## Quick Start

### Installation

```bash
git clone https://github.com/this-vishalsingh/zp1
cd zp1
cargo build --release
```

### Your First Proof

Create a simple RISC-V program (`hello.s`):

```assembly
.section .text
.globl _start

_start:
    # Load value 10 into register x1
    addi x1, x0, 10
    
    # Load value 20 into register x2
    addi x2, x0, 20
    
    # Add x1 + x2 -> x3
    add x3, x1, x2
    
    # Exit (syscall 93)
    addi a7, x0, 93
    ecall
```

Compile to ELF:

```bash
riscv32-unknown-elf-as -march=rv32im -o hello.o hello.s
riscv32-unknown-elf-ld -o hello.elf hello.o
```

Generate proof:

```bash
./target/release/zp1 prove hello.elf --output proof.json
```

Verify proof:

```bash
./target/release/zp1 verify proof.json
```

---

## Writing Provable Programs

### Supported Instructions

ZP1 supports the complete RV32IM instruction set:

#### Arithmetic (10 instructions)
- `ADD`, `SUB`, `ADDI` - Addition and subtraction
- `AND`, `OR`, `XOR`, `ANDI`, `ORI`, `XORI` - Bitwise logic
- `LUI`, `AUIPC` - Upper immediate operations

#### Shifts (6 instructions)
- `SLL`, `SRL`, `SRA` - Register-register shifts
- `SLLI`, `SRLI`, `SRAI` - Immediate shifts

#### Comparisons (4 instructions)
- `SLT`, `SLTU` - Set less than (signed/unsigned)
- `SLTI`, `SLTIU` - Set less than immediate

#### Branches (6 instructions)
- `BEQ`, `BNE` - Branch if equal/not equal
- `BLT`, `BGE`, `BLTU`, `BGEU` - Branch comparisons

#### Jumps (2 instructions)
- `JAL` - Jump and link
- `JALR` - Jump and link register

#### Memory (8 instructions)
- `LB`, `LH`, `LW`, `LBU`, `LHU` - Load byte/halfword/word
- `SB`, `SH`, `SW` - Store byte/halfword/word

#### M-Extension (8 instructions)
- `MUL`, `MULH`, `MULHSU`, `MULHU` - Multiply
- `DIV`, `DIVU` - Division
- `REM`, `REMU` - Remainder

### Limitations

**Not Supported:**
- CSR (Control/Status Register) instructions
- Floating point operations (F/D extensions)
- Atomic operations (A extension)
- Interrupts and traps

**Memory Model:**
- Direct physical addressing (no MMU)
- 16 MB default memory size
- Word and halfword accesses must be aligned

### Example: Fibonacci

```assembly
.section .text
.globl _start

_start:
    # Initialize: fib(0) = 0, fib(1) = 1
    addi x1, x0, 0      # x1 = 0
    addi x2, x0, 1      # x2 = 1
    addi x3, x0, 10     # x3 = n (compute 10 terms)
    
loop:
    # Check if done
    beq x3, x0, done
    
    # fib(n) = fib(n-1) + fib(n-2)
    add x4, x1, x2      # x4 = x1 + x2
    
    # Shift: x1 = x2, x2 = x4
    addi x1, x2, 0
    addi x2, x4, 0
    
    # Decrement counter
    addi x3, x3, -1
    jal x0, loop

done:
    # Result in x2
    addi a7, x0, 93
    ecall
```

---

## CLI Reference

### Commands

#### `prove`

Generate a STARK proof from an ELF binary.

```bash
zp1 prove <ELF_FILE> [OPTIONS]
```

**Options:**
- `--output <FILE>` - Output proof file (default: `proof.json`)
- `--queries <N>` - Number of FRI queries for security (default: 20)

**Example:**
```bash
zp1 prove program.elf --output my_proof.json --queries 30
```

#### `verify`

Verify a STARK proof.

```bash
zp1 verify <PROOF_FILE>
```

**Example:**
```bash
zp1 verify my_proof.json
```

#### `run`

Execute a program without generating a proof (for debugging).

```bash
zp1 run <ELF_FILE> [OPTIONS]
```

**Options:**
- `--max-steps <N>` - Maximum execution steps (default: 10000)
- `--trace <FILE>` - Output execution trace

---

## Library Usage

### Basic Workflow

```rust
use zp1_executor::{Cpu, ElfLoader};
use zp1_trace::TraceColumns;
use zp1_prover::{StarkProver, StarkConfig};
use zp1_primitives::M31;

// 1. Load and execute program
let elf_data = std::fs::read("program.elf")?;
let loader = ElfLoader::parse(&elf_data)?;

let mut cpu = Cpu::new();
cpu.enable_tracing();
loader.load_into_memory(&mut cpu.memory)?;
cpu.pc = loader.entry_point();

// Execute
while cpu.step()?.is_some() {
    // Execution continues
}

// 2. Get execution trace
let trace = cpu.take_trace().unwrap();
let mut columns = TraceColumns::from_execution_trace(&trace);
columns.pad_to_power_of_two();

// 3. Configure and create prover
let config = StarkConfig {
    log_trace_len: columns.len().trailing_zeros() as usize,
    blowup_factor: 8,
    num_queries: 20,
    fri_folding_factor: 2,
    security_bits: 100,
};

let mut prover = StarkProver::new(config);

// 4. Generate proof
let trace_cols = columns.to_columns();
let public_inputs: Vec<M31> = vec![]; // Optional public inputs
let proof = prover.prove(trace_cols, &public_inputs);

// 5. Proof is ready!
println!("Proof generated with {} query proofs", proof.query_proofs.len());
```

### Custom AIR Constraints

If you need to add custom constraints:

```rust
use zp1_air::{CpuTraceRow, ConstraintEvaluator};
use zp1_primitives::M31;

// Evaluate constraints for a single row
let row = CpuTraceRow::from_slice(&columns);
let constraints = ConstraintEvaluator::evaluate_all(&row);

// All constraints should evaluate to zero
for (i, constraint) in constraints.iter().enumerate() {
    assert_eq!(*constraint, M31::ZERO, "Constraint {} failed", i);
}
```

---

## Performance Tuning

### Trace Size vs Proof Time

| Rows | Prove Time | Memory | Proof Size |
|------|-----------|--------|------------|
| 16   | ~1.2s     | 50 MB  | ~12 KB     |
| 64   | ~5.3s     | 120 MB | ~45 KB     |
| 256  | ~28s      | 350 MB | ~180 KB    |
| 1024 | ~4.8m     | 1.2 GB | ~720 KB    |

### Optimization Tips

1. **Reduce Trace Length**
   - Minimize loop iterations
   - Use efficient algorithms
   - Avoid unnecessary memory operations

2. **Adjust Security Parameters**
   - Lower `num_queries` for faster proving (reduces security)
   - Typical range: 15-30 queries
   - Default 20 provides ~100 bits of security

3. **Memory Alignment**
   - Use word-aligned memory accesses when possible
   - Avoid unaligned byte/halfword operations

4. **Batch Processing**
   - Generate multiple proofs in parallel
   - Use separate prover instances per thread

---

## Troubleshooting

### Common Errors

**"Unaligned memory access"**
- Memory loads/stores must be aligned
- Words (4 bytes) must be 4-byte aligned
- Halfwords (2 bytes) must be 2-byte aligned

**"Invalid instruction"**
- CSR instructions are not supported
- Check that your program uses only RV32IM instructions

**"Execution exceeded max steps"**
- Your program has an infinite loop
- Increase `--max-steps` if legitimate

**"Proof verification failed"**
- Trace may have been modified
- Check that prover and verifier use same configuration

### Debug Mode

Run with trace output to see execution:

```bash
zp1 run program.elf --trace execution.json
```

This outputs each instruction executed with register state.

---

## Examples

See the `tests/` directory for complete examples:

- **Counting**: Simple loop counter
- **Fibonacci**: Recursive computation
- **Arithmetic**: All ALU operations

---

## Additional Resources

- **Architecture**: See `docs/architecture.md`
- **Progress**: See `docs/PROGRESS.md`
- **API Docs**: Run `cargo doc --open`

---

## Support

For issues and questions:
- GitHub Issues: https://github.com/this-vishalsingh/zp1/issues
- Documentation: https://github.com/this-vishalsingh/zp1/tree/main/docs
