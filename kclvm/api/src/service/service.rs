use std::{path::Path, string::String, time::SystemTime};

use crate::model::gpyrpc::*;

use kclvm_parser::load_program;
use protobuf_json_mapping::print_to_string_with_options;
use protobuf_json_mapping::PrintOptions;

//Specific implementation of calling service
pub struct KclvmService {
    //Store the error information of the last call
    pub kclvm_service_err_buffer: String,
}

impl Default for KclvmService {
    fn default() -> Self {
        Self {
            kclvm_service_err_buffer: "\0".to_string(),
        }
    }
}

impl KclvmService {
    pub fn ping(&self, args: &Ping_Args) -> Ping_Result {
        Ping_Result {
            value: (args.value.clone()),
            special_fields: (args.special_fields.clone()),
        }
    }

    pub fn exec_program(&self, args: &ExecProgram_Args) -> Result<ExecProgram_Result, String> {
        //transform args to json
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
        //parse native_args from json string
        let native_args = kclvm_runner::ExecProgramArgs::from_str(args_json.as_str());
        let plgin_agent = 0;
        let opts = native_args.get_load_program_options();
        let k_files = &native_args.k_filename_list;
        let mut kcl_paths = Vec::<String>::new();
        //join work_path with k_fiel_path
        for (_, file) in k_files.into_iter().enumerate() {
            match Path::new(args.work_dir.as_str()).join(file).to_str() {
                Some(str) => kcl_paths.push(String::from(str)),
                None => (),
            }
        }

        let kcl_paths_str = kcl_paths.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        let program = load_program(&kcl_paths_str.as_slice(), Some(opts))?;
        let start_time = SystemTime::now();
        let json_result = kclvm_runner::execute(program, plgin_agent, &native_args)?;
        let escape_time = match SystemTime::now().duration_since(start_time) {
            Ok(dur) => dur.as_secs_f32(),
            Err(err) => return Err(err.to_string()),
        };
        let mut result = ExecProgram_Result::default();
        result.json_result = json_result.clone();
        result.escaped_time = escape_time.to_string();
        if !args.disable_yaml_result {
            //if diable_yaml_result == flase ,transfrom json_result to yaml_result
            let yaml_value = match serde_json::from_str::<serde_yaml::Value>(json_result.as_str()) {
                Ok(val) => val,
                Err(err) => return Err(err.to_string()),
            };
            let yaml_result = match serde_yaml::to_string(&yaml_value) {
                Ok(str) => str,
                Err(err) => return Err(err.to_string()),
            };
            //remove yaml prefix
            let yaml_prefix = "---\n";
            if yaml_result.starts_with(yaml_prefix) {
                result.yaml_result =
                    String::from(&yaml_result[yaml_prefix.len()..yaml_result.len()]);
            } else {
                result.yaml_result = yaml_result;
            }
        }
        Ok(result)
    }
}
