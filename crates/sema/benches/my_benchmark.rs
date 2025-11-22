use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_sema::ty::*;

use std::sync::Arc;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sup", |b| {
        b.iter(|| {
            let types = vec![
                Arc::new(Type::int_lit(1)),
                Arc::new(Type::INT),
                Arc::new(Type::union(&[
                    Arc::new(Type::STR),
                    Arc::new(Type::dict(Arc::new(Type::STR), Arc::new(Type::STR))),
                ])),
                Arc::new(Type::dict(Arc::new(Type::ANY), Arc::new(Type::ANY))),
            ];
            sup(&types);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
