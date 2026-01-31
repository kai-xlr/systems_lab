use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use hft_primitives::LockFreeRingBuffer;
use std::sync::Arc;
use std::thread;

fn bench_single_threaded_send(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_single_threaded");
    group.throughput(Throughput::Elements(10000));

    group.bench_function("send_10k", |b| {
        let queue = LockFreeRingBuffer::new(16384);
        b.iter(|| {
            for i in 0..10000 {
                black_box(queue.send(i).ok());
            }
            // Drain to reset
            while queue.receive().is_some() {}
        });
    });

    group.finish();
}

fn bench_single_threaded_receive(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_single_threaded");
    group.throughput(Throughput::Elements(10000));

    group.bench_function("receive_10k", |b| {
        let queue = LockFreeRingBuffer::new(16384);
        // Pre-fill
        for i in 0..10000 {
            queue.send(i).unwrap();
        }

        b.iter(|| {
            for _ in 0..10000 {
                black_box(queue.receive());
            }
            // Refill
            for i in 0..10000 {
                queue.send(i).unwrap();
            }
        });
    });

    group.finish();
}

fn bench_spsc_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_spsc");
    group.throughput(Throughput::Elements(100000));

    group.bench_function("spsc_100k", |b| {
        b.iter(|| {
            let queue = Arc::new(LockFreeRingBuffer::new(16384));
            let queue_producer = Arc::clone(&queue);
            let queue_consumer = Arc::clone(&queue);

            let producer = thread::spawn(move || {
                for i in 0..100000 {
                    while queue_producer.send(i).is_err() {
                        // Spin if full
                    }
                }
            });

            let consumer = thread::spawn(move || {
                let mut received = 0;
                while received < 100000 {
                    if queue_consumer.receive().is_some() {
                        received += 1;
                    }
                }
            });

            producer.join().unwrap();
            consumer.join().unwrap();
        });
    });

    group.finish();
}

fn bench_different_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_sizes");

    for size in [64, 256, 1024, 4096, 16384].iter() {
        group.bench_with_input(format!("size_{}", size), size, |b, &size| {
            let queue = LockFreeRingBuffer::new(size);
            b.iter(|| {
                for i in 0..1000 {
                    black_box(queue.send(i).ok());
                }
                while queue.receive().is_some() {}
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_threaded_send,
    bench_single_threaded_receive,
    bench_spsc_throughput,
    bench_different_sizes
);
criterion_main!(benches);
