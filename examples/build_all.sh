#!/usr/bin/env bash
# Build all ZP1 examples (ELF binaries)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔨 Building all ZP1 examples..."
echo ""

# Check prerequisites
if ! rustup target list | grep -q "riscv32im-unknown-none-elf (installed)"; then
    echo "❌ RISC-V target not installed"
    echo "Run: rustup target add riscv32im-unknown-none-elf"
    exit 1
fi

# Build each example
EXAMPLES=("fibonacci" "keccak" "sha256" "ecrecover" "memory-test" "blake2b" "json-parser" "merkle-proof" "password-hash" "ed25519-verify" "rsa-verify" "eth-header" "ripemd160" "wordle" "chess-checkmate" "range-proof" "waldo-proof" "sudoku" "age-proof" "voting" "regex-match" "hash-chain" "hello-zkvm" "nullifier" "commitment")

for example in "${EXAMPLES[@]}"; do
    if [ -d "$example" ]; then
        echo "📦 Building $example..."
        
        # Build the ELF (from workspace root for shared target/)
        cargo build --release --target riscv32im-unknown-none-elf -p "$example" 2>&1 | grep -v "warning:" || true
        
        ELF_PATH="$SCRIPT_DIR/target/riscv32im-unknown-none-elf/release/$example"
        
        # Show ELF size
        if [ -f "$ELF_PATH" ]; then
            SIZE=$(wc -c < "$ELF_PATH")
            echo "   ✓ Built $example ELF ($SIZE bytes)"
        fi
        echo ""
    fi
done

echo "✅ All examples built successfully!"
echo ""
echo "Run an example from the root directory (ZP1):"
echo "  cargo run --release -- prove --bin fibonacci"