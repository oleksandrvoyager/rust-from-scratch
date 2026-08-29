// Benchmark rc_refcell vs arena.

use criterion::{Criterion, criterion_group, criterion_main};
use lru_cache::Cache;
use lru_cache::{arena, rc_refcell};
use std::hint::black_box;

const CAPACITY: usize = 1_000;

fn fill<C: Cache<i32, i32>>() {
    let mut c = C::new(CAPACITY);
    for i in 0..CAPACITY as i32 {
        c.put(black_box(i), black_box(i));
    }
}

fn get_hits<C: Cache<i32, i32>>() {
    let mut c = C::new(CAPACITY);
    for i in 0..CAPACITY as i32 {
        c.put(i, i);
    }

    for i in 0..CAPACITY as i32 {
        black_box(c.get(black_box(&i)));
    }
}

fn put_with_eviction<C: Cache<i32, i32>>() {
    let mut c = C::new(CAPACITY);
    for i in 0..CAPACITY as i32 {
        c.put(i, i);
    }

    // Cache is now full: every further put evicts the current tail.
    for i in CAPACITY as i32..(CAPACITY as i32 * 2) {
        c.put(black_box(i), black_box(i));
    }
}

fn bench_fill(c: &mut Criterion) {
    let mut group = c.benchmark_group("fill");
    group.bench_function("arena", |b| b.iter(fill::<arena::LruCache<i32, i32>>));
    group.bench_function("rc_refcell", |b| {
        b.iter(fill::<rc_refcell::LruCache<i32, i32>>)
    });
    group.finish();
}

fn bench_get_hits(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_hits");
    group.bench_function("arena", |b| {
        b.iter(get_hits::<arena::LruCache<i32, i32>>)
    });
    group.bench_function("rc_refcell", |b| {
        b.iter(get_hits::<rc_refcell::LruCache<i32, i32>>)
    });
    group.finish();
}

fn bench_put_with_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_with_eviction");
    group.bench_function("arena", |b| {
        b.iter(put_with_eviction::<arena::LruCache<i32, i32>>)
    });
    group.bench_function("rc_refcell", |b| {
        b.iter(put_with_eviction::<rc_refcell::LruCache<i32, i32>>)
    });
    group.finish();
}

criterion_group!(benches, bench_fill, bench_get_hits, bench_put_with_eviction);
criterion_main!(benches);
