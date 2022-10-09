use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_query::override_file;
use kclvm_tools::format::{format, FormatOptions};

pub fn criterion_benchmark_override(c: &mut Criterion) {
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

pub fn criterion_benchmark_format(c: &mut Criterion) {
    c.bench_function("format", |b| {
        b.iter(|| {
            format("./benches/test_data/format.k", &FormatOptions::default()).unwrap();
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark_override,
    criterion_benchmark_format
);
criterion_main!(benches);
