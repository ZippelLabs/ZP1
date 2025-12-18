# ZP1 Specification (Skeleton)

> Purpose: working blueprint to guide implementation and reviews. Keep this file authoritative; update as decisions land. ASCII-only.

## 1. Goals and Non-Goals
- Goals: deterministic zkVM for RISC-V–like programs; proving and verifying with delegation/aggregation; soundness-first with documented performance targets.
- Non-goals (current phase): full Linux syscalls, floating point, rich OS services; anything not explicitly listed as supported.

## 2. Threat Model and Trust Assumptions
- Attacker controls guest inputs and proofs; cannot break chosen cryptographic assumptions.
- Declare trusted setup (if any), randomness source, hardware assumptions (CPU/GPU), and who is trusted in delegation.

## 3. Architecture Overview
- Flow: guest program → zkVM execution → trace → prover → proof → verifier (optionally aggregated) → consumer/on-chain.
- Artifacts: execution trace, commitments, public inputs/outputs, proof object, verification key(s).

## 4. Instruction Set and Semantics
- ISA scope (MVP): integer ops, branches, loads/stores (aligned), no FP; explicit overflow/sign/zero-extension rules.
- Syscalls/host calls: ABI, determinism requirements, allowed side effects, error codes.

## 5. Memory and I/O Model
- Little-endian; alignment rules; address space limits; metering (gas/step limits); copy semantics.
- I/O channels (if any): format, buffering rules, determinism constraints.

## 6. Constraint System
- Field choice and rationale; hash/commitment primitives; lookup/permutation arguments.
- Mapping table: VM behaviors/opcodes → constraints; range/boolean constraints; lookup domain sizes; memory consistency constraints.

## 7. Transcript and Fiat-Shamir
- Domain separators per phase; challenge ordering; binding of all public inputs; prohibition of challenge reuse.

## 8. Prover Pipeline
- Stages: trace gen → witness gen → commitments → polynomial/IOP → proof encoding.
- Parallelism model (CPU/GPU); caching/chunking strategy; expected complexity and resource bounds.

## 9. Verifier
- Input validation; proof parsing; challenge recomputation; batching/aggregation handling; failure modes.

## 10. Delegation and Aggregation
- Instance encoding format; aggregation order rules; recursion/accumulation scheme; verification key handling; size/latency targets.

## 11. Parameters
- Security level (bits); field modulus; hash variants and personalization; polynomial degrees; table sizes; commitment schemes.

## 12. Public Inputs and Outputs
- Exact serialization: endianness, lengths, domain tags; versioning fields; determinism rules.

## 13. Error Handling and Limits
- Bounded allocations, panic policy, DOS guards (time/memory/input sizes); timeout behavior; invalid-proof handling.

## 14. Testing and Vectors
- Per-component: unit/property/fuzz; golden vectors for opcodes, transcripts, proofs (valid and invalid/tampered).
- CI gates: minimal green targets and expansion plan.

## 15. Tooling and Build
- Toolchain pin (`rust-toolchain.toml`); feature flags (e.g., `wip` to gate incomplete code); reproducible build steps.
- Canonical commands: `cargo check -p <nucleus>`; `cargo test -p <nucleus>`.

## 16. Deployment Targets
- On-chain/off-chain verifier integration; ABI; gas/fee benchmarks; upgrade/versioning strategy.

## 17. Observability
- Logging levels; metrics (timings, proof sizes, memory); debug trace hooks.

## 18. Roadmap and Milestones
- Phase 0: Tooling/guards — pin toolchain; add `wip` gating; slim workspace to nucleus (e.g., `primitives`, minimal `zkvm`).
- Phase 1: Minimal ISA + trace — implement a handful of opcodes with full semantics/constraints; add tests/vectors; keep green `cargo check/test` on nucleus.
- Phase 2: Transcript + PI binding — finalize domain separators/serialization; add negative tests for tampered inputs/proofs.
- Phase 3: Prover skeleton — end-to-end placeholder proof with real transcript/commitments; log timings/sizes.
- Phase 4: Aggregation/delegation — instance encoding + sanity checks; batch verification path.
- Phase 5: Hardening — fuzz/property tests, DOS guards, reproducible builds; expand ISA/constraints.

## 19. Open Decisions (fill as you go)
- Field choice, hash/commitment scheme, ISA surface for MVP, memory limits, batching strategy, on-chain verifier ABI.
