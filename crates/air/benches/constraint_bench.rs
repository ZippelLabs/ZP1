//! Benchmarks for AIR constraint evaluation.
//!
//! Run with: cargo bench -p zp1-air
//!
//! This benchmark compares bit-based vs lookup-based bitwise constraints.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use zp1_primitives::M31;
use zp1_air::cpu::CpuAir;
use zp1_air::rv32im::{CpuTraceRow, ConstraintEvaluator};

/// Create a test row for bitwise AND operation.
fn create_and_row() -> CpuTraceRow {
    let mut row = CpuTraceRow::default();
    
    // AND: 0x12345678 & 0x0F0F0F0F = 0x02040608
    row.is_and = M31::ONE;
    
    // rs1 = 0x12345678
    row.rs1_val_lo = M31::new(0x5678);
    row.rs1_val_hi = M31::new(0x1234);
    
    // rs2 = 0x0F0F0F0F
    row.rs2_val_lo = M31::new(0x0F0F);
    row.rs2_val_hi = M31::new(0x0F0F);
    
    // Result = 0x02040608
    row.rd_val_lo = M31::new(0x0608);
    row.rd_val_hi = M31::new(0x0204);
    
    // Bit decomposition for bit-based constraints
    let rs1: u32 = 0x12345678;
    let rs2: u32 = 0x0F0F0F0F;
    let result = rs1 & rs2;
    
    for i in 0..32 {
        row.rs1_bits[i] = M31::new(((rs1 >> i) & 1) as u32);
        row.rs2_bits[i] = M31::new(((rs2 >> i) & 1) as u32);
        row.and_bits[i] = M31::new(((result >> i) & 1) as u32);
    }
    
    // Byte decomposition for lookup-based constraints
    row.rs1_bytes[0] = M31::new(0x78);
    row.rs1_bytes[1] = M31::new(0x56);
    row.rs1_bytes[2] = M31::new(0x34);
    row.rs1_bytes[3] = M31::new(0x12);
    
    row.rs2_bytes[0] = M31::new(0x0F);
    row.rs2_bytes[1] = M31::new(0x0F);
    row.rs2_bytes[2] = M31::new(0x0F);
    row.rs2_bytes[3] = M31::new(0x0F);
    
    row.and_result_bytes[0] = M31::new(0x08);
    row.and_result_bytes[1] = M31::new(0x06);
    row.and_result_bytes[2] = M31::new(0x04);
    row.and_result_bytes[3] = M31::new(0x02);
    
    row
}

/// Create a test row for bitwise XOR operation.
fn create_xor_row() -> CpuTraceRow {
    let mut row = CpuTraceRow::default();
    
    // XOR: 0xAAAAAAAA ^ 0x55555555 = 0xFFFFFFFF
    row.is_xor = M31::ONE;
    
    row.rs1_val_lo = M31::new(0xAAAA);
    row.rs1_val_hi = M31::new(0xAAAA);
    
    row.rs2_val_lo = M31::new(0x5555);
    row.rs2_val_hi = M31::new(0x5555);
    
    row.rd_val_lo = M31::new(0xFFFF);
    row.rd_val_hi = M31::new(0xFFFF);
    
    // Bit decomposition
    let rs1: u32 = 0xAAAAAAAA;
    let rs2: u32 = 0x55555555;
    let result = rs1 ^ rs2;
    
    for i in 0..32 {
        row.rs1_bits[i] = M31::new(((rs1 >> i) & 1) as u32);
        row.rs2_bits[i] = M31::new(((rs2 >> i) & 1) as u32);
        row.xor_bits[i] = M31::new(((result >> i) & 1) as u32);
    }
    
    // Byte decomposition
    row.rs1_bytes[0] = M31::new(0xAA);
    row.rs1_bytes[1] = M31::new(0xAA);
    row.rs1_bytes[2] = M31::new(0xAA);
    row.rs1_bytes[3] = M31::new(0xAA);
    
    row.rs2_bytes[0] = M31::new(0x55);
    row.rs2_bytes[1] = M31::new(0x55);
    row.rs2_bytes[2] = M31::new(0x55);
    row.rs2_bytes[3] = M31::new(0x55);
    
    row.xor_result_bytes[0] = M31::new(0xFF);
    row.xor_result_bytes[1] = M31::new(0xFF);
    row.xor_result_bytes[2] = M31::new(0xFF);
    row.xor_result_bytes[3] = M31::new(0xFF);
    
    row
}

fn u8_bits(byte: u32) -> [M31; 8] {
    std::array::from_fn(|i| M31::new((byte >> i) & 1))
}

fn u16_bits(half: u32) -> [M31; 16] {
    std::array::from_fn(|i| M31::new((half >> i) & 1))
}

fn u32_to_limbs(value: u32) -> (M31, M31) {
    (M31::new(value & 0xFFFF), M31::new(value >> 16))
}

fn u32_to_bytes(value: u32) -> ([M31; 4], [[M31; 8]; 4]) {
    let bytes = [
        value & 0xFF,
        (value >> 8) & 0xFF,
        (value >> 16) & 0xFF,
        (value >> 24) & 0xFF,
    ];
    (
        std::array::from_fn(|i| M31::new(bytes[i])),
        std::array::from_fn(|i| u8_bits(bytes[i])),
    )
}

fn u32_to_halves(value: u32) -> ([M31; 2], [[M31; 16]; 2]) {
    let halves = [value & 0xFFFF, (value >> 16) & 0xFFFF];
    (
        std::array::from_fn(|i| M31::new(halves[i])),
        std::array::from_fn(|i| u16_bits(halves[i])),
    )
}

fn bench_and_bit_based(c: &mut Criterion) {
    let row = create_and_row();
    
    c.bench_function("AND_bit_based", |b| {
        b.iter(|| ConstraintEvaluator::and_constraint(black_box(&row)))
    });
}

fn bench_and_lookup_based(c: &mut Criterion) {
    let row = create_and_row();
    
    c.bench_function("AND_lookup_based", |b| {
        b.iter(|| ConstraintEvaluator::and_constraint_lookup(black_box(&row)))
    });
}

fn bench_xor_bit_based(c: &mut Criterion) {
    let row = create_xor_row();
    
    c.bench_function("XOR_bit_based", |b| {
        b.iter(|| ConstraintEvaluator::xor_constraint(black_box(&row)))
    });
}

fn bench_xor_lookup_based(c: &mut Criterion) {
    let row = create_xor_row();
    
    c.bench_function("XOR_lookup_based", |b| {
        b.iter(|| ConstraintEvaluator::xor_constraint_lookup(black_box(&row)))
    });
}

fn bench_bitwise_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bitwise_Constraints");
    
    let and_row = create_and_row();
    let xor_row = create_xor_row();
    
    group.bench_function("AND/bit_based", |b| {
        b.iter(|| ConstraintEvaluator::and_constraint(black_box(&and_row)))
    });
    
    group.bench_function("AND/lookup_based", |b| {
        b.iter(|| ConstraintEvaluator::and_constraint_lookup(black_box(&and_row)))
    });
    
    group.bench_function("XOR/bit_based", |b| {
        b.iter(|| ConstraintEvaluator::xor_constraint(black_box(&xor_row)))
    });
    
    group.bench_function("XOR/lookup_based", |b| {
        b.iter(|| ConstraintEvaluator::xor_constraint_lookup(black_box(&xor_row)))
    });
    
    group.finish();
}

/// Benchmark evaluating many rows (simulating trace evaluation)
fn bench_batch_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch_Evaluation");
    
    for num_rows in [100, 1000, 10000] {
        let rows: Vec<CpuTraceRow> = (0..num_rows).map(|_| create_and_row()).collect();
        
        group.bench_with_input(
            BenchmarkId::new("bit_based", num_rows),
            &rows,
            |b, rows| {
                b.iter(|| {
                    let mut sum = M31::ZERO;
                    for row in rows.iter() {
                        sum = sum + ConstraintEvaluator::and_constraint(black_box(row));
                    }
                    sum
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("lookup_based", num_rows),
            &rows,
            |b, rows| {
                b.iter(|| {
                    let mut sum = M31::ZERO;
                    for row in rows.iter() {
                        sum = sum + ConstraintEvaluator::and_constraint_lookup(black_box(row));
                    }
                    sum
                })
            },
        );
    }
    
    group.finish();
}

fn bench_memory_gadgets(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory_Gadgets");

    let mem = 0x1234F678u32;
    let (mem_lo, mem_hi) = u32_to_limbs(mem);
    let (mem_bytes, mem_byte_bits) = u32_to_bytes(mem);
    let (mem_halves, mem_half_bits) = u32_to_halves(mem);

    let byte_offset = M31::ONE;
    let byte_offset_bits = [M31::ONE, M31::ZERO];
    let byte_bits = u8_bits(0xF6);
    let byte_selectors = (mem_bytes[1], mem_bytes[3]);
    let rd_lbu_lo = M31::new(0xF6);
    let rd_lbu_hi = M31::ZERO;
    let rd_lb_lo = M31::new(0xFFF6);
    let rd_lb_hi = M31::new(0xFFFF);

    group.bench_function("LB", |b| {
        b.iter(|| {
            CpuAir::load_byte_constraint(
                black_box(mem_lo),
                black_box(mem_hi),
                black_box(byte_offset),
                black_box(rd_lb_lo),
                black_box(rd_lb_hi),
                black_box(&mem_bytes),
                black_box(&mem_byte_bits),
                black_box(&byte_offset_bits),
                black_box(&byte_bits),
                black_box(byte_selectors),
            )
        })
    });

    group.bench_function("LBU", |b| {
        b.iter(|| {
            CpuAir::load_byte_unsigned_constraint(
                black_box(mem_lo),
                black_box(mem_hi),
                black_box(byte_offset),
                black_box(rd_lbu_lo),
                black_box(rd_lbu_hi),
                black_box(&mem_bytes),
                black_box(&mem_byte_bits),
                black_box(&byte_offset_bits),
                black_box(&byte_bits),
                black_box(byte_selectors),
            )
        })
    });

    let half_offset = M31::ONE;
    let half_bits = u16_bits(0x1234);
    let rd_lh_lo = M31::new(0x1234);
    let rd_lh_hi = M31::ZERO;

    group.bench_function("LH", |b| {
        b.iter(|| {
            CpuAir::load_halfword_constraint(
                black_box(mem_lo),
                black_box(mem_hi),
                black_box(half_offset),
                black_box(rd_lh_lo),
                black_box(rd_lh_hi),
                black_box(&mem_halves),
                black_box(&mem_half_bits),
                black_box(&half_bits),
            )
        })
    });

    group.bench_function("LHU", |b| {
        b.iter(|| {
            CpuAir::load_halfword_unsigned_constraint(
                black_box(mem_lo),
                black_box(mem_hi),
                black_box(half_offset),
                black_box(rd_lh_lo),
                black_box(rd_lh_hi),
                black_box(&mem_halves),
                black_box(&mem_half_bits),
                black_box(&half_bits),
            )
        })
    });

    let old_mem = 0x1234F678u32;
    let new_byte_mem = 0x1234AB78u32;
    let (old_lo, old_hi) = u32_to_limbs(old_mem);
    let (new_byte_lo, new_byte_hi) = u32_to_limbs(new_byte_mem);
    let (old_bytes, old_byte_bits) = u32_to_bytes(old_mem);
    let (new_bytes, new_byte_bits) = u32_to_bytes(new_byte_mem);
    let byte_to_store = M31::new(0xAB);
    let byte_to_store_bits = u8_bits(0xAB);
    let byte_offset_selectors = [M31::ZERO, M31::ONE, M31::ZERO, M31::ZERO];

    group.bench_function("SB", |b| {
        b.iter(|| {
            CpuAir::store_byte_constraint(
                black_box(old_lo),
                black_box(old_hi),
                black_box(new_byte_lo),
                black_box(new_byte_hi),
                black_box(byte_to_store),
                black_box(&byte_to_store_bits),
                black_box(byte_offset),
                black_box(&old_bytes),
                black_box(&old_byte_bits),
                black_box(&new_bytes),
                black_box(&new_byte_bits),
                black_box(&byte_offset_bits),
                black_box(&byte_offset_selectors),
            )
        })
    });

    let new_half_mem = 0xABCDF678u32;
    let (new_half_lo, new_half_hi) = u32_to_limbs(new_half_mem);
    let (old_halves, old_half_bits) = u32_to_halves(old_mem);
    let (new_halves, new_half_bits) = u32_to_halves(new_half_mem);
    let half_to_store = M31::new(0xABCD);
    let half_to_store_bits = u16_bits(0xABCD);

    group.bench_function("SH", |b| {
        b.iter(|| {
            CpuAir::store_halfword_constraint(
                black_box(old_lo),
                black_box(old_hi),
                black_box(new_half_lo),
                black_box(new_half_hi),
                black_box(half_to_store),
                black_box(&half_to_store_bits),
                black_box(half_offset),
                black_box(&old_halves),
                black_box(&old_half_bits),
                black_box(&new_halves),
                black_box(&new_half_bits),
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_and_bit_based,
    bench_and_lookup_based,
    bench_xor_bit_based,
    bench_xor_lookup_based,
    bench_bitwise_comparison,
    bench_batch_evaluation,
    bench_memory_gadgets,
);
criterion_main!(benches);
