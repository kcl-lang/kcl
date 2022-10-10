use std::rc::Rc;

use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_sema::ty::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sup", |b| {
        b.iter(|| {
            let types = vec![
                Rc::new(Type::int_lit(1)),
                Rc::new(Type::INT),
                Rc::new(Type::union(&[
                    Rc::new(Type::STR),
                    Rc::new(Type::dict(Rc::new(Type::STR), Rc::new(Type::STR))),
                ])),
                Rc::new(Type::dict(Rc::new(Type::ANY), Rc::new(Type::ANY))),
            ];
            sup(&types);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
