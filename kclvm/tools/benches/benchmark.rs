use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_tools::query::override_file;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("override", |b| {
        b.iter(|| {
            override_file(
                "./benches/test_data/simple.k",
                &["config.image=\"image/image:v1\"".to_string()],
                &["pkg.to.path".to_string()],
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
