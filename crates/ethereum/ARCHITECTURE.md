# Ethereum Integration - Guest Program Architecture

## Overview

Following the industry standard approach used by **SP1**, **Risc0**, and **OpenVM**, our Ethereum integration executes EVM transactions **inside the zkVM guest program**, not on the host.

## Directory Structure

```
crates/ethereum/
├── src/                    # Host code (prepares data, invokes guest)
│   ├── lib.rs
│   ├── prover.rs           # Orchestrates guest execution
│   ├── fetcher.rs          # Fetches blocks from RPC
│   ├── guest_executor.rs   # Guest program integration
│   ├── evm.rs              # Legacy (direct execution - deprecated)
│   └── ...
└── guest/                  # Guest code (runs inside zkVM)
    ├── Cargo.toml
    └── src/
        └── main.rs         # EVM execution with revm
```

## Architecture

### Industry Standard Pattern

This is how the leading zkVMs integrate Ethereum:

```
┌─────────────────────────────────────────────────┐
│ Host (Outside zkVM)                             │
│                                                 │
│  1. Fetch block/transaction data from RPC      │
│  2. Prepare execution witness (pre-state)      │
│  3. Load guest program ELF                     │
│  4. Execute guest in zkVM with inputs          │
│  5. Extract outputs from journal               │
│  6. Generate/verify proof                      │
│                                                 │
└─────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────┐
│ Guest (Inside zkVM - RISC-V)                    │
│                                                 │
│  1. Read transaction data from host            │
│  2. Read pre-state (accounts, storage)         │
│  3. Execute transaction using revm             │
│  4. Calculate state changes                    │
│  5. Commit results to journal                  │
│     - Success/failure                          │
│     - Gas used                                 │
│     - Return data                              │
│     - State root                               │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Why Guest Execution?

1. **Provable**: Everything the guest does is proven by the zkVM
2. **Trustless**: EVM execution is cryptographically verified
3. **Standard**: This is how SP1/Risc0/OpenVM all work
4. **Efficient**: State transitions are proven, not just computed

### Comparison: Host vs Guest Execution

| Aspect | Host Execution (Wrong) | Guest Execution (Correct) |
|--------|----------------------|-------------------------|
| **Location** | Runs on host machine | Runs inside zkVM |
| **Provability** | Must trace separately | Automatically proven |
| **Trust** | Must trust host | Cryptographically verified |
| **Industry** | Not used by leaders | SP1, Risc0, OpenVM standard |

## Implementation Status

### ✅ Completed
- Guest program structure (`guest/Cargo.toml`, `guest/src/main.rs`)
- Guest execution logic with revm
- Host integration points (`guest_executor.rs`)
- Architecture documentation

### 🚧 In Progress
- Guest compilation to RISC-V
- zkVM I/O integration (reading inputs, writing outputs)
- State provider for pre-state data

### 📋 TODO
- Build system for guest compilation
- Integration with zp1-executor for guest invocation
- State witness fetching (like Zeth's execution witness)
- Block-level proving (multiple transactions)

## How to Use (Once Complete)

```rust
use zp1_ethereum::guest_executor::execute_tx_in_guest;

// Fetch transaction
let tx = fetcher.fetch_transaction(tx_hash).await?;

// Execute in guest (inside zkVM, produces proof)
let result = execute_tx_in_guest(&tx).await?;

// Result is now cryptographically proven
assert!(result.success);
```

## References

- **Zeth**: https://github.com/risc0/zeth (Risc0's Ethereum block prover)
- **SP1 Reth**: https://github.com/succinctlabs/sp1 (SP1 examples with reth)
- **Revm**: https://github.com/bluealloy/revm (The EVM we use)

## Migration Path

1. **Phase 1** (Current): Guest program created, host uses legacy `evm.rs`
2. **Phase 2** (Next): Compile guest to RISC-V, integrate with zp1-executor
3. **Phase 3** (Future): Deprecate `evm.rs`, all execution via guest
4. **Phase 4** (Advanced): Block-level proving like Zeth
