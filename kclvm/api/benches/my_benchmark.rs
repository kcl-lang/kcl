use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kclvm_api::model::gpyrpc::*;
use kclvm_api::service::service::KclvmService;
use kclvm_api::service::util::*;
use std::fs;
use std::path::Path;
const TEST_DATA_PATH: &str = "./src/testdata";

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("exec_program", |b| {
        b.iter(|| {
            let input_path = Path::new(TEST_DATA_PATH).join("exec-program.json");
            let input = fs::read_to_string(&input_path)
                .expect(format!("Something went wrong reading {}", input_path.display()).as_str());
            let args = parse_message_from_json::<ExecProgram_Args>(&input);
            let serv = KclvmService::default();
            let prev_hook = std::panic::take_hook();
            // disable print panic info
            std::panic::set_hook(Box::new(|_| {}));
            let except_result_path = Path::new(TEST_DATA_PATH).join("exec-program.response.json");
            let except_result_json = std::fs::read_to_string(&except_result_path).expect(
                format!(
                    "Something went wrong reading {}",
                    except_result_path.display()
                )
                .as_str(),
            );
            let except_result = parse_message_from_json::<ExecProgram_Result>(&except_result_json);
            let result = std::panic::catch_unwind(|| {
                let result = serv.exec_program(&args).unwrap();
                assert_eq!(result.json_result, except_result.json_result);
                assert_eq!(result.yaml_result, except_result.yaml_result);
            });
            std::panic::set_hook(prev_hook);
            assert!(result.is_ok());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
