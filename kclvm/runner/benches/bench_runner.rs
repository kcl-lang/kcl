use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_parser::load_program;
use kclvm_runner::{execute, runner::ExecProgramArgs};

const TEST_CASE_PATH: &str = "/src/test_datas/init_check_order_0/main.k";

pub fn criterion_benchmark(c: &mut Criterion) {
    let kcl_path = &format!(
        "{}{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        TEST_CASE_PATH
    );
    let plugin_agent = 0;

    c.bench_function("load_program -> execute", |b| {
        b.iter(|| {
            let args = ExecProgramArgs::default();
            let opts = args.get_load_program_options();
            let program = load_program(&[kcl_path], Some(opts)).unwrap();
            execute(program, plugin_agent, &args).unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
