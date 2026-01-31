use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use hft_primitives::AtomicCounter;
use std::sync::{Arc, Mutex};
use std::thread;

fn bench_atomic_counter_single_thread(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_counter_single_thread");
    group.throughput(Throughput::Elements(1000000));

    group.bench_function("increment_1m", |b| {
        let counter = AtomicCounter::new();
        b.iter(|| {
            for _ in 0..1000000 {
                black_box(counter.increment());
            }
            counter.reset();
        });
    });

    group.finish();
}

fn bench_atomic_counter_multi_thread(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_counter_multi_thread");
    group.throughput(Throughput::Elements(800000));

    group.bench_function("8_threads_100k_each", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicCounter::new());
            let mut handles = vec![];

            for _ in 0..8 {
                let counter_clone = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for _ in 0..100000 {
                        counter_clone.increment();
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            black_box(counter.get());
        });
    });

    group.finish();
}

fn bench_mutex_counter_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("counter_comparison");
    group.throughput(Throughput::Elements(800000));

    group.bench_function("atomic_8_threads", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicCounter::new());
            let mut handles = vec![];

            for _ in 0..8 {
                let counter_clone = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for _ in 0..100000 {
                        counter_clone.increment();
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.bench_function("mutex_8_threads", |b| {
        b.iter(|| {
            let counter = Arc::new(Mutex::new(0usize));
            let mut handles = vec![];

            for _ in 0..8 {
                let counter_clone = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for _ in 0..100000 {
                        let mut c = counter_clone.lock().unwrap();
                        *c += 1;
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_atomic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_operations");
    group.throughput(Throughput::Elements(1000000));

    let counter = AtomicCounter::new();

    group.bench_function("increment", |b| {
        b.iter(|| {
            for _ in 0..1000000 {
                black_box(counter.increment());
            }
            counter.reset();
        });
    });

    group.bench_function("add_5", |b| {
        b.iter(|| {
            for _ in 0..1000000 {
                black_box(counter.add(5));
            }
            counter.reset();
        });
    });

    group.bench_function("get", |b| {
        b.iter(|| {
            for _ in 0..1000000 {
                black_box(counter.get());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_atomic_counter_single_thread,
    bench_atomic_counter_multi_thread,
    bench_mutex_counter_comparison,
    bench_atomic_operations
);
criterion_main!(benches);
