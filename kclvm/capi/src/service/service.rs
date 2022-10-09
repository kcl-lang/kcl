use std::{path::Path, string::String, time::SystemTime};

use crate::model::gpyrpc::*;

use kclvm::ValueRef;
use kclvm_parser::load_program;
use kclvm_query::apply_overrides;
use kclvm_query::override_file;
use protobuf_json_mapping::print_to_string_with_options;
use protobuf_json_mapping::PrintOptions;

/// Specific implementation of calling service
pub struct KclvmService {
    pub plugin_agent: u64,
}

impl Default for KclvmService {
    fn default() -> Self {
        Self { plugin_agent: 0 }
    }
}

impl KclvmService {
    /// Ping KclvmService ,return the same value as the parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::model::gpyrpc::*;
    /// let serv = &KclvmService { plugin_agent: 0 };
    /// let args = &Ping_Args {
    ///     value: "hello".to_string(),
    ///     ..Default::default()
    /// };
    /// let ping_result = serv.ping(args);
    /// assert_eq!(ping_result.value, "hello".to_string());
    /// ```
    ///
    pub fn ping(&self, args: &Ping_Args) -> Ping_Result {
        Ping_Result {
            value: (args.value.clone()),
            special_fields: (args.special_fields.clone()),
        }
    }

    /// Execute KCL file with args
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::model::gpyrpc::*;
    /// let serv = &KclvmService { plugin_agent: 0 };
    /// let args = &ExecProgram_Args {
    ///     work_dir: "./src/testdata".to_string(),
    ///     k_filename_list: vec!["./src/testdata".to_string()],
    ///     ..Default::default()
    /// };
    /// let exec_result = serv.exec_program(args).unwrap();
    /// println!("{}",exec_result.json_result);
    /// ```
    pub fn exec_program(&self, args: &ExecProgram_Args) -> Result<ExecProgram_Result, String> {
        // transform args to json
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
        // parse native_args from json string
        let native_args = kclvm_runner::ExecProgramArgs::from_str(args_json.as_str());
        let opts = native_args.get_load_program_options();
        let k_files = &native_args.k_filename_list;
        let mut kcl_paths = Vec::<String>::new();
        // join work_path with k_fiel_path
        for (_, file) in k_files.into_iter().enumerate() {
            match Path::new(args.work_dir.as_str()).join(file).to_str() {
                Some(str) => kcl_paths.push(String::from(str)),
                None => (),
            }
        }

        let kcl_paths_str = kcl_paths.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        let mut result = ExecProgram_Result::default();
        let mut program = load_program(&kcl_paths_str.as_slice(), Some(opts))?;

        if let Err(err) = apply_overrides(
            &mut program,
            &native_args.overrides,
            &[],
            native_args.print_override_ast,
        ) {
            return Err(err.to_string());
        }

        let start_time = SystemTime::now();
        let exec_result = kclvm_runner::execute(program, self.plugin_agent, &native_args);
        let escape_time = match SystemTime::now().duration_since(start_time) {
            Ok(dur) => dur.as_secs_f32(),
            Err(err) => return Err(err.to_string()),
        };
        result.escaped_time = escape_time.to_string();
        let json_result = match exec_result {
            Ok(res) => res,
            Err(res) => {
                if res.is_empty() {
                    return Ok(result);
                } else {
                    return Err(res);
                }
            }
        };
        let kcl_val = ValueRef::from_json(&json_result).unwrap();
        if let Some(val) = kcl_val.get_by_key("__kcl_PanicInfo__") {
            if val.is_truthy() {
                return Err(json_result);
            }
        }
        let (json_result, yaml_result) = kcl_val.plan();
        result.json_result = json_result;
        if !args.disable_yaml_result {
            result.yaml_result = yaml_result;
        }
        Ok(result)
    }

    /// Override KCL file with args
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::model::gpyrpc::*;
    /// let serv = &KclvmService { plugin_agent: 0 };
    /// let args = &OverrideFile_Args {
    ///     file: "./src/testdata/test.k".to_string(),
    ///     specs: vec!["alice.age=18".to_string()],
    ///     import_paths: vec![],
    ///     ..Default::default()
    /// };
    /// let override_result = serv.override_file(args).unwrap();
    /// assert!(override_result.result);
    /// ```
    ///
    ///  - test.k (after override)
    ///
    /// ```kcl
    /// schema Person:
    ///     age: int
    ///
    /// alice = Person {
    ///     age = 18
    /// }
    /// ```
    pub fn override_file(&self, args: &OverrideFile_Args) -> Result<OverrideFile_Result, String> {
        override_file(&args.file, &args.specs, &args.import_paths)
            .map_err(|err| err.to_string())
            .map(|result| OverrideFile_Result {
                result,
                ..Default::default()
            })
    }
}
