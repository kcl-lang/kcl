use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_parser::load_program;
use kclvm_runner::{execute, runner::ExecProgramArgs};
use kclvm_tools::query::apply_overrides;

const TEST_CASE_PATH: &str = "./src/test_datas/init_check_order_0/main.k";

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("refactor kclvm-runner", |b| {
        b.iter(|| {
            after_refactor(TEST_CASE_PATH.to_string());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn after_refactor(k_path: String) {
    let mut args = ExecProgramArgs::default();
    args.k_filename_list.push(k_path);

    let plugin_agent = 0;

    let files = args.get_files();
    let opts = args.get_load_program_options();

    // load ast
    let mut program = load_program(&files, Some(opts)).unwrap();
    apply_overrides(&mut program, &args.overrides, &[]);

    // resolve ast, generate libs, link libs and execute.
    execute(program, plugin_agent, &args);
}
