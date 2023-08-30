use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_query::override_file;
use kclvm_tools::format::{format, FormatOptions};
use std::{
    fmt,
    time::{Duration, Instant},
};

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

pub struct StopWatch {
    time: Instant,
}

pub struct StopWatchSpan {
    pub time: Duration,
}

impl StopWatch {
    pub fn start() -> StopWatch {
        let time = Instant::now();
        StopWatch { time }
    }

    pub fn elapsed(&mut self) -> StopWatchSpan {
        let time = self.time.elapsed();

        StopWatchSpan { time }
    }
}

impl fmt::Display for StopWatchSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2?}", self.time)?;
        Ok(())
    }
}

/// Utility for writing benchmark tests.
///
/// If you need to benchmark the entire test, you can directly add the macro `#[bench_test]` like this:
/// ```
/// #[test]
/// #[bench_test]
/// fn benchmark_foo() {
///     actual_work(analysis)
/// }
/// ```
///
/// If you need to skip some preparation stages and only test some parts of test, you can use the `bench()` method.
/// A benchmark test looks like this:
///
/// ```
/// #[test]
/// fn benchmark_foo() {
///     let data = bench_fixture::some_fixture();
///     let analysis = some_setup();
///
///     {
///         let _b = bench("foo");
///         actual_work(analysis)
///     };
/// }
/// ```
///
///
pub fn bench(label: &'static str) -> impl Drop {
    struct Bencher {
        sw: StopWatch,
        label: &'static str,
    }

    impl Drop for Bencher {
        fn drop(&mut self) {
            eprintln!("{}: {}", self.label, self.sw.elapsed());
        }
    }

    Bencher {
        sw: StopWatch::start(),
        label,
    }
}
