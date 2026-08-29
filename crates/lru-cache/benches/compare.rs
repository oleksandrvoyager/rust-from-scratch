// Benchmark rc_refcell vs arena, scaled across several sizes.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use lru_cache::Cache;
use lru_cache::{arena, rc_refcell};
use std::hint::black_box;

const SIZES: [usize; 4] = [100, 1_000, 10_000, 100_000];

fn fill<C: Cache<usize, usize>>(size: usize) {
    let mut c = C::new(size);
    for i in 0..size {
        c.put(black_box(i), black_box(i));
    }
}

fn get_hits<C: Cache<usize, usize>>(size: usize) {
    let mut c = C::new(size);
    for i in 0..size {
        c.put(i, i);
    }

    for i in 0..size {
        black_box(c.get(black_box(&i)));
    }
}

fn put_with_eviction<C: Cache<usize, usize>>(size: usize) {
    let mut c = C::new(size);
    for i in 0..size {
        c.put(i, i);
    }

    // Cache is now full: every further put evicts the current tail.
    for i in size..(size * 2) {
        c.put(black_box(i), black_box(i));
    }
}

fn bench_fill(c: &mut Criterion) {
    let mut group = c.benchmark_group("fill");
    for &size in &SIZES {
        group.bench_with_input(BenchmarkId::new("arena", size), &size, |b, &size| {
            b.iter(|| fill::<arena::LruCache<usize, usize>>(size));
        });
        group.bench_with_input(BenchmarkId::new("rc_refcell", size), &size, |b, &size| {
            b.iter(|| fill::<rc_refcell::LruCache<usize, usize>>(size));
        });
    }
    group.finish();
}

fn bench_get_hits(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_hits");
    for &size in &SIZES {
        group.bench_with_input(BenchmarkId::new("arena", size), &size, |b, &size| {
            b.iter(|| get_hits::<arena::LruCache<usize, usize>>(size));
        });
        group.bench_with_input(BenchmarkId::new("rc_refcell", size), &size, |b, &size| {
            b.iter(|| get_hits::<rc_refcell::LruCache<usize, usize>>(size));
        });
    }
    group.finish();
}

fn bench_put_with_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_with_eviction");
    for &size in &SIZES {
        group.bench_with_input(BenchmarkId::new("arena", size), &size, |b, &size| {
            b.iter(|| put_with_eviction::<arena::LruCache<usize, usize>>(size));
        });
        group.bench_with_input(BenchmarkId::new("rc_refcell", size), &size, |b, &size| {
            b.iter(|| put_with_eviction::<rc_refcell::LruCache<usize, usize>>(size));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_fill, bench_get_hits, bench_put_with_eviction);
criterion_main!(benches);
