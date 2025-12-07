# Fibonacci Example

Simple example that computes the 10th Fibonacci number.

## Expected Output

The 10th Fibonacci number is **55**.

## Building

```bash
cargo build --release --target riscv32im-unknown-none-elf
cargo objcopy --release -- -O binary fibonacci.bin
```

## Testing with ZP1

```bash
cd /Users/zippellabs/Developer/zp1
cargo run --release -- prove fibonacci examples/fibonacci/fibonacci.bin
```

## What It Does

1. Computes F(10) iteratively
2. Stores result (55) at memory address 0x80000000
3. Total instructions: ~100 cycles

This demonstrates:
- Basic RV32IM instruction execution
- Simple arithmetic operations
- Memory writes
- Clean program termination
