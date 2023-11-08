use std::path::Path;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use walkdir::WalkDir;

use kclvm_parser::{load_program, ParseSession};
use kclvm_runner::{execute, runner::ExecProgramArgs};

const EXEC_DATA_PATH: &str = "./src/exec_data/";

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("refactor kclvm-runner", |b| {
        b.iter(|| {
            let prev_hook = std::panic::take_hook();
            // disable print panic info
            std::panic::set_hook(Box::new(|_| {}));
            let result = std::panic::catch_unwind(|| {
                for file in get_files(EXEC_DATA_PATH, false, true, ".k") {
                    exec(&file).unwrap();
                }
            });
            assert!(result.is_ok());
            std::panic::set_hook(prev_hook);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn exec(file: &str) -> Result<String, String> {
    let mut args = ExecProgramArgs::default();
    args.k_filename_list.push(file.to_string());
    let opts = args.get_load_program_options();
    let sess = Arc::new(ParseSession::default());
    // Load AST program
    let program = load_program(sess.clone(), &[file], Some(opts), None).unwrap();
    // Resolve ATS, generate libs, link libs and execute.
    execute(sess, program, &args)
}

/// Get kcl files from path.
fn get_files<P: AsRef<Path>>(
    path: P,
    recursively: bool,
    sorted: bool,
    suffix: &str,
) -> Vec<String> {
    let mut files = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(suffix) && (recursively || entry.depth() == 1) {
                files.push(file.to_string())
            }
        }
    }
    if sorted {
        files.sort();
    }
    files
}
