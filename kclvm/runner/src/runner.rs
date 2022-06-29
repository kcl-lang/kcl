use serde::{Deserialize, Serialize};

use kclvm_ast::ast;

#[allow(non_camel_case_types)]
pub type kclvm_char_t = i8;
#[allow(non_camel_case_types)]
pub type kclvm_size_t = i32;
#[allow(non_camel_case_types)]
pub type kclvm_context_t = std::ffi::c_void;
#[allow(non_camel_case_types)]
pub type kclvm_value_ref_t = std::ffi::c_void;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExecProgramArgs {
    pub work_dir: Option<String>,
    pub k_filename_list: Vec<String>,
    pub k_code_list: Vec<String>,

    pub args: Vec<ast::CmdArgSpec>,
    pub overrides: Vec<ast::OverrideSpec>,

    pub disable_yaml_result: bool,
    pub print_override_ast: bool,

    // -r --strict-range-check
    pub strict_range_check: bool,

    // -n --disable-none
    pub disable_none: bool,
    // -v --verbose
    pub verbose: i32,

    // -d --debug
    pub debug: i32,

    // yaml/json: sort keys
    pub sort_keys: bool,
    // include schema type path in JSON/YAML result
    pub include_schema_type_path: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExecProgramResult {
    pub json_result: String,
    pub yaml_result: String,

    pub escaped_time: String,
}

impl ExecProgramArgs {
    pub fn from_str(s: &str) -> Self {
        if s.trim().is_empty() {
            return Default::default();
        }
        serde_json::from_str::<ExecProgramArgs>(s).expect(s)
    }
    pub fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    pub fn get_files(&self) -> Vec<&str> {
        self.k_filename_list.iter().map(|s| s.as_str()).collect()
    }

    pub fn get_load_program_options(&self) -> kclvm_parser::LoadProgramOptions {
        kclvm_parser::LoadProgramOptions {
            work_dir: self.work_dir.clone().unwrap_or("".to_string()).clone(),
            k_code_list: self.k_code_list.clone(),
            cmd_args: self.args.clone(),
            cmd_overrides: self.overrides.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct KclvmRunnerOptions {
    pub plugin_agent_ptr: u64,
}

pub struct KclvmRunner {
    opts: KclvmRunnerOptions,
    lib: libloading::Library,
}

impl KclvmRunner {
    pub fn new(dylib_path: &str, opts: Option<KclvmRunnerOptions>) -> Self {
        let lib = unsafe {
            libloading::Library::new(std::path::PathBuf::from(dylib_path).canonicalize().unwrap())
                .unwrap()
        };
        Self {
            opts: opts.unwrap_or_default(),
            lib,
        }
    }

    pub fn run(&self, args: &ExecProgramArgs) -> Result<String, String> {
        unsafe {
            Self::dylib_kclvm_plugin_init(&self.lib, self.opts.plugin_agent_ptr);
            Self::dylib_kcl_run(&self.lib, &args)
        }
    }
}

impl KclvmRunner {
    unsafe fn dylib_kclvm_plugin_init(lib: &libloading::Library, plugin_method_ptr: u64) {
        // get kclvm_plugin_init
        let kclvm_plugin_init: libloading::Symbol<
            unsafe extern "C" fn(
                fn_ptr: extern "C" fn(
                    method: *const i8,
                    args_json: *const i8,
                    kwargs_json: *const i8,
                ) -> *const i8,
            ),
        > = lib.get(b"kclvm_plugin_init").unwrap();

        // get plugin_method
        let plugin_method_ptr = plugin_method_ptr;
        let plugin_method_ptr = (plugin_method_ptr as *const u64) as *const ()
            as *const extern "C" fn(
                method: *const i8,
                args: *const i8,
                kwargs: *const i8,
            ) -> *const i8;
        let plugin_method: extern "C" fn(
            method: *const i8,
            args: *const i8,
            kwargs: *const i8,
        ) -> *const i8 = std::mem::transmute(plugin_method_ptr);

        // register plugin agent
        kclvm_plugin_init(plugin_method);
    }

    unsafe fn dylib_kcl_run(
        lib: &libloading::Library,
        args: &ExecProgramArgs,
    ) -> Result<String, String> {
        let kcl_run: libloading::Symbol<
            unsafe extern "C" fn(
                kclvm_main_ptr: u64, // main.k => kclvm_main
                option_len: kclvm_size_t,
                option_keys: *const *const kclvm_char_t,
                option_values: *const *const kclvm_char_t,
                strict_range_check: i32,
                disable_none: i32,
                disable_schema_check: i32,
                list_option_mode: i32,
                debug_mode: i32,
                result_buffer_len: kclvm_size_t,
                result_buffer: *mut kclvm_char_t,
                warn_buffer_len: kclvm_size_t,
                warn_buffer: *mut kclvm_char_t,
            ) -> kclvm_size_t,
        > = lib.get(b"_kcl_run").unwrap();

        let kclvm_main: libloading::Symbol<u64> = lib.get(b"kclvm_main").unwrap();
        let kclvm_main_ptr = kclvm_main.into_raw().into_raw() as u64;

        let option_len = args.args.len() as kclvm_size_t;

        let cstr_argv: Vec<_> = args
            .args
            .iter()
            .map(|arg| std::ffi::CString::new(arg.name.as_str()).unwrap())
            .collect();

        let mut p_argv: Vec<_> = cstr_argv
            .iter() // do NOT into_iter()
            .map(|arg| arg.as_ptr())
            .collect();
        p_argv.push(std::ptr::null());

        let p: *const *const kclvm_char_t = p_argv.as_ptr();
        let option_keys = p;

        let cstr_argv: Vec<_> = args
            .args
            .iter()
            .map(|arg| std::ffi::CString::new(arg.value.as_str()).unwrap())
            .collect();

        let mut p_argv: Vec<_> = cstr_argv
            .iter() // do NOT into_iter()
            .map(|arg| arg.as_ptr())
            .collect();
        p_argv.push(std::ptr::null());

        let p: *const *const kclvm_char_t = p_argv.as_ptr();
        let option_values = p;

        let strict_range_check = args.strict_range_check as i32;
        let disable_none = args.disable_none as i32;
        let disable_schema_check = 0; // todo
        let list_option_mode = 0; // todo
        let debug_mode = args.debug as i32;

        let mut result = vec![0u8; 1024 * 1024];
        let result_buffer_len = result.len() as i32 - 1;
        let result_buffer = result.as_mut_ptr() as *mut i8;

        let mut warn_data = vec![0u8; 1024 * 1024];
        let warn_buffer_len = warn_data.len() as i32 - 1;
        let warn_buffer = warn_data.as_mut_ptr() as *mut i8;

        let n = kcl_run(
            kclvm_main_ptr,
            option_len,
            option_keys,
            option_values,
            strict_range_check,
            disable_none,
            disable_schema_check,
            list_option_mode,
            debug_mode,
            result_buffer_len,
            result_buffer,
            warn_buffer_len,
            warn_buffer,
        );

        if n > 0 {
            let return_len = n;
            let s = std::str::from_utf8(&result[0..return_len as usize]).unwrap();
            Ok(s.to_string())
        } else {
            let return_len = 0 - n;
            let s = std::str::from_utf8(&warn_data[0..return_len as usize]).unwrap();
            Err(s.to_string())
        }
    }
}
