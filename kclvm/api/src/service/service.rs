use std::{path::Path, string::String, time::SystemTime};

use crate::model::gpyrpc::*;

use kclvm_parser::load_program;
use protobuf_json_mapping::print_to_string_with_options;
use protobuf_json_mapping::PrintOptions;

#[derive(Default)]
pub struct KclvmService {}

impl KclvmService {
    pub fn ping(&self, args: &Ping_Args) -> Ping_Result {
        Ping_Result {
            value: (args.value.clone()),
            special_fields: (args.special_fields.clone()),
        }
    }

    pub fn exec_program(&self, args: &ExecProgram_Args) -> ExecProgram_Result {
        let args_json = print_to_string_with_options(
            args,
            &PrintOptions {
                enum_values_int: true,
                proto_field_name: true,
                always_output_default_values: true,
                _future_options: (),
            },
        )
        .unwrap();
        let native_args = kclvm_runner::ExecProgramArgs::from_str(args_json.as_str());
        let plgin_agent = 0;
        let opts = native_args.get_load_program_options();
        let k_files = &native_args.k_filename_list;
        let kcl_paths: Vec<String> = k_files
            .iter()
            .map(|file| {
                String::from(
                    Path::new(args.work_dir.as_str())
                        .join(file)
                        .to_str()
                        .unwrap(),
                )
            })
            .collect();
        let kcl_paths_str: Vec<&str> = kcl_paths.iter().map(|s| s.as_str()).collect();
        let program = load_program(&kcl_paths_str.as_slice(), Some(opts)).unwrap();
        let start_time = SystemTime::now();
        let json_result = kclvm_runner::execute(program, plgin_agent, &native_args).unwrap();
        let escape_time = SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_secs_f64();
        let mut result = ExecProgram_Result::default();
        result.json_result = json_result.clone();
        result.escaped_time = escape_time.to_string();
        if !args.disable_yaml_result {
            let yaml_result = serde_yaml::to_string(
                &serde_json::from_str::<serde_yaml::Value>(json_result.as_str()).unwrap(),
            )
            .unwrap();
            let yaml_prefix = "---\n";
            if yaml_result.starts_with(yaml_prefix) {
                result.yaml_result =
                    String::from(&yaml_result[yaml_prefix.len()..yaml_result.len()]);
            } else {
                result.yaml_result = yaml_result;
            }
        }
        result
    }
}
