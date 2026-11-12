//! Criterion smoke benchmark.
//!
//! This benchmark is deliberately tiny: it exists so `cargo bench --no-run`
//! has a real target to compile, keeping the harness from rotting between
//! milestones. It measures public constants that every report path touches,
//! nothing more. Performance budgets (10 KB ≤ 150 µs, 5 MB ≤ 60 ms, ≥ 100 MB/s)
//! are owned by E9.4 and must not be duplicated here.

// Criterion's `criterion_group!` expands to an undocumented `fn benches()`;
// benchmarks are not part of the public API surface.
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn version_len(c: &mut Criterion) {
    c.bench_function("version_len", |b| {
        b.iter(|| black_box(fattura_core::VERSION.len()));
    });
}

fn report_schema_version_parse(c: &mut Criterion) {
    c.bench_function("report_schema_version_parse", |b| {
        b.iter(|| black_box(fattura_core::REPORT_SCHEMA_VERSION.parse::<u32>().ok()));
    });
}

criterion_group!(benches, version_len, report_schema_version_parse);
criterion_main!(benches);
