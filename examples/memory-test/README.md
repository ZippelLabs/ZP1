# Memory Operations Test

Comprehensive test of ZP1's memory model covering all load/store operations.

## What It Tests

1. **Word operations** (LW, SW) - 4 bytes
2. **Half-word operations** (LH, LHU, SH) - 2 bytes
3. **Byte operations** (LB, LBU, SB) - 1 byte
4. **Array operations** - Sequential memory access
5. **Arithmetic on memory** - Array sum
6. **Memory consistency** - Read-after-write

## Expected Results

| Address | Value | Description |
|---------|-------|-------------|
| 0x80000000 | 0xDEADBEEF | Word write test |
| 0x80000010 | 0xDEADBEEF | Word read-back |
| 0x80000004 | 0xCAFE | Half-word write |
| 0x80000014 | 0xCAFE | Half-word read-back |
| 0x80000006 | 0x42 | Byte write |
| 0x80000016 | 0x42 | Byte read-back |
| 0x80000050 | 90 (0x5A) | Array sum |

## Memory Model

ZP1 uses **LogUp argument** for memory consistency:
- Proves all loads return correct values
- Ensures memory is consistent across execution
- Efficient constraint checking

## Building

```bash
cargo build --release --target riscv32im-unknown-none-elf
cargo objcopy --release -- -O binary memory-test.bin
```

## Testing

```bash
cd /Users/zippellabs/Developer/zp1
cargo run --release -- prove memory-test examples/memory-test/memory-test.bin
```

This demonstrates:
- All RISC-V load/store instructions
- Memory consistency checking
- LogUp argument in action
- Proper trace generation for memory operations
