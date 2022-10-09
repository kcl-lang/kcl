use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
