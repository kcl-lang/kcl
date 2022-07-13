use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_parser::load_program;
use kclvm_sema::resolver::*;
use kclvm_sema::ty::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sup", |b| {
        b.iter(|| {
            let types = vec![
                Type::int_lit(1),
                Type::INT,
                Type::union(&[Type::STR, Type::dict(Type::STR, Type::STR)]),
                Type::dict(Type::ANY, Type::ANY),
            ];
            sup(&types);
        })
    });
}

pub fn criterion_benchmark_resolver(c: &mut Criterion) {
    let mut program = load_program(&["./src/resolver/test_data/import.k"], None).unwrap();
    c.bench_function("resolver", |b| b.iter(|| resolve_program(&mut program)));
}

criterion_group!(benches, criterion_benchmark, criterion_benchmark_resolver);
criterion_main!(benches);
