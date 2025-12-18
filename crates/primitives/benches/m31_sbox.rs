//! Benchmarks for M31 field operations and Poseidon2 sbox candidates.
//!
//! This benchmark compares different sbox exponents to quantify the performance
//! difference between M31 (which requires x^5 or higher) and fields like
//! Koalabear/BabyBear that could use cheaper sboxes like x^3.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zp1_primitives::M31;

/// Compute x^3 (simulating cheaper sbox, e.g. Koalabear if valid)
#[inline]
fn sbox_cube(x: M31) -> M31 {
    let x2 = x * x;
    x2 * x
}

/// Compute x^5 (typical Poseidon2 sbox for M31)
#[inline]
fn sbox_fifth(x: M31) -> M31 {
    let x2 = x * x;
    let x4 = x2 * x2;
    x4 * x
}

/// Compute x^7 (alternative sbox)
#[inline]
fn sbox_seventh(x: M31) -> M31 {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    x6 * x
}

fn bench_field_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("M31 Field Operations");

    let a = M31::new(0x12345678);
    let b = M31::new(0x7654321);

    group.bench_function("add", |bench| bench.iter(|| black_box(a) + black_box(b)));

    group.bench_function("mul", |bench| bench.iter(|| black_box(a) * black_box(b)));

    group.bench_function("square", |bench| bench.iter(|| black_box(a).square()));

    group.bench_function("inverse", |bench| bench.iter(|| black_box(a).inv()));

    group.finish();
}

fn bench_sbox(c: &mut Criterion) {
    let mut group = c.benchmark_group("Poseidon2 Sbox Candidates");

    let x = M31::new(0x12345678);

    group.bench_function("x^3 (Koalabear proxy)", |bench| {
        bench.iter(|| sbox_cube(black_box(x)))
    });

    group.bench_function("x^5 (M31 Poseidon2)", |bench| {
        bench.iter(|| sbox_fifth(black_box(x)))
    });

    group.bench_function("x^7 (alternative)", |bench| {
        bench.iter(|| sbox_seventh(black_box(x)))
    });

    group.finish();
}

fn bench_batch_sbox(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Sbox (1000 elements)");

    // Create a batch of 1000 elements
    let batch: Vec<M31> = (0..1000u32).map(|i| M31::new(i + 1)).collect();

    for (name, sbox_fn) in [
        ("x^3", sbox_cube as fn(M31) -> M31),
        ("x^5", sbox_fifth as fn(M31) -> M31),
        ("x^7", sbox_seventh as fn(M31) -> M31),
    ] {
        group.bench_with_input(BenchmarkId::new(name, "1000"), &batch, |bench, data| {
            bench.iter(|| {
                data.iter()
                    .map(|&x| sbox_fn(black_box(x)))
                    .collect::<Vec<_>>()
            })
        });
    }

    group.finish();
}

fn bench_poseidon2_round_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Poseidon2 Round Simulation (width=12)");

    // Simulate a width-12 Poseidon2 state
    let state: [M31; 12] = core::array::from_fn(|i| M31::new((i + 1) as u32));

    // Simulate MDS matrix (simple mock - just sum all elements)
    fn mock_mds(state: [M31; 12]) -> [M31; 12] {
        let sum: M31 = state.iter().fold(M31::ZERO, |acc, &x| acc + x);
        core::array::from_fn(|i| state[i] + sum)
    }

    group.bench_function("round with x^3 sbox", |bench| {
        bench.iter(|| {
            let mut s = black_box(state);
            // Apply sbox to all elements
            for x in &mut s {
                *x = sbox_cube(*x);
            }
            // Apply MDS
            mock_mds(s)
        })
    });

    group.bench_function("round with x^5 sbox", |bench| {
        bench.iter(|| {
            let mut s = black_box(state);
            for x in &mut s {
                *x = sbox_fifth(*x);
            }
            mock_mds(s)
        })
    });

    group.bench_function("round with x^7 sbox", |bench| {
        bench.iter(|| {
            let mut s = black_box(state);
            for x in &mut s {
                *x = sbox_seventh(*x);
            }
            mock_mds(s)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_field_ops,
    bench_sbox,
    bench_batch_sbox,
    bench_poseidon2_round_simulation
);
criterion_main!(benches);
