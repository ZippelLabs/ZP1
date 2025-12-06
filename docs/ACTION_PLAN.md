# ZP1 Action Plan - Critical Fixes
**Priority**: BLOCKER Issues First  
**Timeline**: 2-3 weeks for Phase 1

---

## Phase 1: Critical Soundness Fixes

### 1. Fix Verifier Fiat-Shamir Transcript [CRITICAL - 2 hours]

**File**: `crates/verifier/src/verify.rs:279-286`

**Current Bug**:
```rust
// Verifier absorbs trace_at_z_next, prover doesn't
for v in &proof.ood_values.trace_at_z_next {
    channel.absorb_felt(*v);  // ← REMOVE THIS
}
```

**Fix**:
```rust
// Match prover exactly - skip trace_at_z_next
for v in &proof.ood_values.trace_at_z {
    channel.absorb_felt(*v);
}
channel.absorb_felt(proof.ood_values.composition_at_z);
// Do NOT absorb trace_at_z_next
```

**OR** (better - these values should be bound):
Update prover to include them:
```rust
// In prover stark.rs after line 191
for &v in &ood_values.trace_at_z_next {
    self.channel.absorb_felt(v);
}
```

---

### 2. Add Domain Separator [CRITICAL - 5 minutes]

**File**: `crates/verifier/src/channel.rs:13`

**Current**:
```rust
pub fn new() -> Self {
    Self { hasher: Sha256::new() }
}
```

**Fix**:
```rust
pub fn new(domain_separator: &[u8]) -> Self {
    let mut hasher = Sha256::new();
    hasher.update(domain_separator);
    Self { hasher }
}
```

**Update call sites**:
```rust
// In verify.rs:262
let mut channel = VerifierChannel::new(b"zp1-stark-v1");
```

---

### 3. Add Public Input Binding [CRITICAL - 1 hour]

**Files**: 
- `crates/prover/src/stark.rs:180` (prover)
- `crates/verifier/src/verify.rs:260` (verifier)

**Prover Fix**:
```rust
pub fn prove(&mut self, trace_columns: Vec<Vec<M31>>) -> StarkProof {
    // NEW: Absorb public inputs FIRST
    for &public_input in &self.public_inputs {
        self.channel.absorb_felt(public_input);
    }
    
    // Phase 1: Trace Commitment
    let trace_commitment = self.commit_trace(&trace_columns);
    // ... rest unchanged
}
```

**Add field to StarkProver**:
```rust
pub struct StarkProver {
    config: StarkConfig,
    channel: ProverChannel,
    public_inputs: Vec<M31>,  // NEW
}
```

**Verifier Fix**:
```rust
pub fn verify(&self, proof: &StarkProof, public_inputs: &[M31]) -> VerifyResult<()> {
    let mut channel = VerifierChannel::new(b"zp1-stark-v1");
    
    // NEW: Absorb public inputs FIRST (matches prover)
    for &public_input in public_inputs {
        channel.absorb_felt(public_input);
    }
    
    // Step 1: Absorb trace commitment
    channel.absorb_commitment(&proof.trace_commitment);
    // ... rest unchanged
}
```

---

### 4. Enforce x0 = 0 Invariant [CRITICAL - 30 minutes]

**File**: `crates/air/src/cpu.rs:111-117`

**Current**:
```rust
pub fn register_x0_is_zero(
    _rd_idx: M31,
    _rd_val_lo: M31,
    _rd_val_hi: M31,
) -> M31 {
    M31::ZERO  // Placeholder
}
```

**Fix Option 1** (simple - check rd index):
```rust
pub fn register_x0_is_zero(
    rd_idx: M31,     // 0-31 register index
    rd_val_lo: M31,  // lower limb of value written
    rd_val_hi: M31,  // upper limb of value written
) -> M31 {
    // If rd == 0, then rd_val must be 0
    // Constraint: (rd_idx == 0) => (rd_val_lo == 0 AND rd_val_hi == 0)
    // Equivalent: rd_idx==0 is boolean flag
    let is_rd_zero = rd_idx;  // Assume rd_idx is 0 or 1 (needs enforcement elsewhere)
    
    // If rd_idx is 1 (writing to x0), both limbs must be 0
    is_rd_zero * rd_val_lo + is_rd_zero * rd_val_hi
}
```

**Better Fix** (using selector):
```rust
pub fn register_x0_is_zero_lo(is_write_x0: M31, rd_val_lo: M31) -> M31 {
    // When writing to x0, value must be 0
    is_write_x0 * rd_val_lo
}

pub fn register_x0_is_zero_hi(is_write_x0: M31, rd_val_hi: M31) -> M31 {
    is_write_x0 * rd_val_hi
}
```

---

### 5. Implement RAM Permutation Accumulator [CRITICAL - 4 hours]

**File**: `crates/air/src/memory.rs:35-43`

**Current**:
```rust
pub fn ram_permutation_check(...) -> M31 {
    // Placeholder for LogUp or grand product argument
    M31::ZERO
}
```

**Fix** (LogUp variant):
```rust
pub fn ram_permutation_running_sum(
    addr: M31,
    value: M31,
    timestamp_lo: M31,
    timestamp_hi: M31,
    is_write: M31,
    alpha: QM31,  // Challenge from Fiat-Shamir
    beta: QM31,   // Second challenge
    prev_sum: QM31,  // Running sum from previous row
) -> QM31 {
    // Fingerprint: α^4·addr + α^3·value + α^2·ts_lo + α·ts_hi + is_write + β
    let mut fp = QM31::from(addr);
    fp = fp * alpha + QM31::from(value);
    fp = fp * alpha + QM31::from(timestamp_lo);
    fp = fp * alpha + QM31::from(timestamp_hi);
    fp = fp * alpha + QM31::from(is_write);
    fp = fp + beta;
    
    // LogUp: sum_{i} 1 / (fingerprint_i)
    // Accumulator: sum' = sum + 1/fp
    prev_sum + fp.inv()
}

// Boundary constraint: first row sum = 0
// Final constraint: execution_sum = address_sorted_sum
```

**Add to columns**:
```rust
// In trace.rs
pub struct MemoryColumns {
    pub addr: Vec<M31>,
    pub value: Vec<M31>,
    pub timestamp: Vec<M31>,
    pub is_write: Vec<M31>,
    pub logup_sum: Vec<QM31>,  // NEW: running sum
}
```

---

### 6. Implement Load/Store Value Constraints [CRITICAL - 6 hours]

**File**: `crates/air/src/rv32im.rs` (new functions)

**Add Load Constraints**:
```rust
/// LB: Load byte (sign-extended)
pub fn load_byte_signed(
    mem_byte: M31,      // Byte value from memory (0-255)
    rd_val_lo: M31,     // Lower 16-bit limb of rd
    rd_val_hi: M31,     // Upper 16-bit limb of rd
) -> [M31; 2] {
    // mem_byte is 8-bit, need sign extension to 32-bit
    // If bit 7 is set, extend with 0xFF; else 0x00
    
    let bit7 = /* extract bit 7 from mem_byte */;
    let sign_extend = bit7 * M31::new(0xFFFFFF00);
    
    let expected_value = sign_extend + mem_byte;
    let (expected_lo, expected_hi) = decompose_u32(expected_value);
    
    [
        rd_val_lo - expected_lo,
        rd_val_hi - expected_hi,
    ]
}

/// LBU: Load byte (zero-extended)
pub fn load_byte_unsigned(
    mem_byte: M31,
    rd_val_lo: M31,
    rd_val_hi: M31,
) -> [M31; 2] {
    // Zero extension: upper 24 bits are 0
    [
        rd_val_lo - mem_byte,  // Lower limb = byte value
        rd_val_hi,              // Upper limb must be 0
    ]
}

// Similar for LH/LHU (halfword), LW (word)
```

**Add Store Constraints**:
```rust
/// SB: Store byte
pub fn store_byte(
    rs2_val_lo: M31,    // Lower limb of source register
    mem_byte: M31,      // Byte written to memory
) -> M31 {
    // Extract lower 8 bits of rs2
    let byte_mask = M31::new(0xFF);
    let expected_byte = rs2_val_lo & byte_mask;
    mem_byte - expected_byte
}

// Similar for SH (halfword), SW (word)
```

---

### 7. Implement Bitwise Operation Constraints [CRITICAL - 8 hours]

**File**: `crates/air/src/rv32im.rs:240-290`

**Strategy**: Use bit decomposition + lookup tables

**Step 1**: Add bit decomposition columns to trace:
```rust
// In trace.rs
pub struct TraceColumns {
    // ... existing fields ...
    
    // Bit decomposition for ALU ops (when needed)
    pub alu_bits_a: Vec<[M31; 32]>,  // 32 bits of operand A
    pub alu_bits_b: Vec<[M31; 32]>,  // 32 bits of operand B
    pub alu_bits_result: Vec<[M31; 32]>,  // 32 bits of result
}
```

**Step 2**: Constrain bit decomposition:
```rust
pub fn bit_decomposition_constraint(
    value_lo: M31,  // Lower 16-bit limb
    value_hi: M31,  // Upper 16-bit limb
    bits: &[M31; 32],  // Individual bits
) -> Vec<M31> {
    let mut constraints = Vec::new();
    
    // Each bit must be 0 or 1
    for &bit in bits {
        constraints.push(bit * (bit - M31::ONE));  // bit*(bit-1) = 0
    }
    
    // Bits must reconstruct the value
    let mut reconstructed = M31::ZERO;
    let mut power = M31::ONE;
    for &bit in bits {
        reconstructed = reconstructed + bit * power;
        power = power * M31::new(2);
    }
    
    let (recon_lo, recon_hi) = /* split reconstructed into limbs */;
    constraints.push(value_lo - recon_lo);
    constraints.push(value_hi - recon_hi);
    
    constraints
}
```

**Step 3**: Bitwise operation constraints:
```rust
pub fn bitwise_and(
    bits_a: &[M31; 32],
    bits_b: &[M31; 32],
    bits_result: &[M31; 32],
) -> Vec<M31> {
    let mut constraints = Vec::with_capacity(32);
    for i in 0..32 {
        // result[i] = a[i] AND b[i] = a[i] * b[i]
        constraints.push(bits_result[i] - bits_a[i] * bits_b[i]);
    }
    constraints
}

pub fn bitwise_or(
    bits_a: &[M31; 32],
    bits_b: &[M31; 32],
    bits_result: &[M31; 32],
) -> Vec<M31> {
    let mut constraints = Vec::with_capacity(32);
    for i in 0..32 {
        // result[i] = a[i] OR b[i] = a[i] + b[i] - a[i]*b[i]
        constraints.push(bits_result[i] - (bits_a[i] + bits_b[i] - bits_a[i] * bits_b[i]));
    }
    constraints
}

pub fn bitwise_xor(
    bits_a: &[M31; 32],
    bits_b: &[M31; 32],
    bits_result: &[M31; 32],
) -> Vec<M31> {
    let mut constraints = Vec::with_capacity(32);
    for i in 0..32 {
        // result[i] = a[i] XOR b[i] = a[i] + b[i] - 2*a[i]*b[i]
        let two = M31::new(2);
        constraints.push(bits_result[i] - (bits_a[i] + bits_b[i] - two * bits_a[i] * bits_b[i]));
    }
    constraints
}
```

**Alternative**: Delegate to lookup tables (more efficient):
```rust
// Precompute 8-bit lookup tables: AND/OR/XOR for all 256×256 combinations
// Then decompose 32-bit ops into 4× 8-bit ops with lookups
```

---

### 8. Implement DEEP Quotient Verification [CRITICAL - 4 hours]

**File**: `crates/verifier/src/verify.rs:360-380`

**Current**:
```rust
fn verify_constraint_consistency(...) {
    if query.trace_values.is_empty() { return Err(...); }
    // No actual verification!
}
```

**Fix**:
```rust
fn verify_deep_quotient(
    &self,
    query: &QueryProof,
    oods_point: QM31,
    ood_values: &OodValues,
    deep_alphas: &[M31],
) -> VerifyResult<()> {
    // Evaluate domain point at query index
    let domain_point = self.get_domain_point(query.index);
    
    // DEEP quotient: sum_i alpha_i * (f_i(X) - f_i(z)) / (X - z)
    let mut expected_deep = QM31::ZERO;
    let denom = domain_point - oods_point;
    let denom_inv = denom.inv();
    
    // Trace columns contribution
    for (col_idx, &trace_val) in query.trace_values.iter().enumerate() {
        let numerator = QM31::from(trace_val) - QM31::from(ood_values.trace_at_z[col_idx]);
        let contribution = QM31::from(deep_alphas[col_idx]) * numerator * denom_inv;
        expected_deep = expected_deep + contribution;
    }
    
    // Composition polynomial contribution
    let comp_numerator = QM31::from(query.composition_value) 
                       - QM31::from(ood_values.composition_at_z);
    let comp_alpha_idx = query.trace_values.len();
    let comp_contribution = QM31::from(deep_alphas[comp_alpha_idx]) 
                          * comp_numerator * denom_inv;
    expected_deep = expected_deep + comp_contribution;
    
    // Compare with claimed FRI value
    if expected_deep != query.fri_value {
        return Err(VerifyError::DeepQuotientMismatch {
            index: query.index,
            expected: expected_deep,
            claimed: query.fri_value,
        });
    }
    
    Ok(())
}
```

---

## Phase 1 Summary

**Total Estimated Time**: 26 hours (~3-4 days)

**Critical Fixes**:
1. ✅ Fiat-Shamir transcript match (2 hours)
2. ✅ Domain separator (5 minutes)
3. ✅ Public input binding (1 hour)
4. ✅ x0 = 0 enforcement (30 minutes)
5. ✅ RAM permutation (4 hours)
6. ✅ Load/store constraints (6 hours)
7. ✅ Bitwise operations (8 hours)
8. ✅ DEEP quotient verification (4 hours)

**After Phase 1**:
- System can prove and verify basic RISC-V programs
- Memory integrity is proven
- Core ALU and memory operations constrained
- Verifier is functionally correct

**Still Missing** (Phase 2):
- M-extension delegation
- Shift operations
- Remaining I-type instructions
- Integration testing
- Performance optimization

---

## Next Steps

1. **Create GitHub Issues** for each fix above
2. **Assign Priority Labels**: P0 (blocker), P1 (high), P2 (medium)
3. **Set Milestones**: Phase 1 (Week 1-2), Phase 2 (Week 3-4)
4. **Daily Standups**: Track progress on critical path
5. **Code Review**: All PRs require review before merge
6. **Testing**: Add test for each fix before marking complete

---

*End of Action Plan*
