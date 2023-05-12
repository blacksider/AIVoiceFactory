use std::time::Duration;

use criterion::{black_box, Criterion, criterion_group, criterion_main, Throughput};

use project_lib::cypher::encrypt;

fn criterion_benchmark(c: &mut Criterion) {
    let key = black_box("01234567012345670123456701234567");
    let content = black_box("some text");
    let mut group = c.benchmark_group("encrypt_group");
    group.significance_level(0.1)
        .sample_size(500)
        .measurement_time(Duration::from_secs(5))
        .throughput(Throughput::Elements(100_000_000));
    group.bench_function("encrypt", |b| b.iter(|| encrypt::encrypt(key, content)));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
