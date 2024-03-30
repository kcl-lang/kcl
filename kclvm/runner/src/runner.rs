use anyhow::{anyhow, Result};
use kclvm_evaluator::Evaluator;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use kclvm_ast::ast;
use kclvm_config::{
    modfile::get_vendor_home,
    settings::{SettingsFile, SettingsPathBuf},
};
use kclvm_error::{Diagnostic, Handler};
use kclvm_query::r#override::parse_override_spec;
use kclvm_runtime::{kclvm_plugin_init, Context, FFIRunOptions, PanicInfo, RuntimePanicRecord};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::raw::c_char;

const RESULT_SIZE: usize = 2048 * 2048;
const KCL_DEBUG_ERROR_ENV_VAR: &str = "KCL_DEBUG_ERROR";

#[allow(non_camel_case_types)]
pub type kclvm_char_t = c_char;
#[allow(non_camel_case_types)]
pub type kclvm_size_t = i32;
#[allow(non_camel_case_types)]
pub type kclvm_context_t = std::ffi::c_void;
#[allow(non_camel_case_types)]
pub type kclvm_value_ref_t = std::ffi::c_void;

/// ExecProgramArgs denotes the configuration required to execute the KCL program.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExecProgramArgs {
    pub work_dir: Option<String>,
    pub k_filename_list: Vec<String>,
    /// -E key=value
    pub external_pkgs: Vec<ast::CmdExternalPkgSpec>,
    pub k_code_list: Vec<String>,
    /// -D key=value
    pub args: Vec<ast::CmdArgSpec>,
    /// -O override_spec
    pub overrides: Vec<ast::OverrideSpec>,
    /// -S path_selector
    pub path_selector: Vec<String>,
    pub disable_yaml_result: bool,
    /// Whether to apply overrides on the source code.
    pub print_override_ast: bool,
    /// -r --strict-range-check
    pub strict_range_check: bool,
    /// -n --disable-none
    pub disable_none: bool,
    /// -v --verbose
    pub verbose: i32,
    /// -d --debug
    pub debug: i32,
    /// yaml/json: sort keys
    pub sort_keys: bool,
    /// Show hidden attributes
    pub show_hidden: bool,
    /// Whether including schema type in JSON/YAML result
    pub include_schema_type_path: bool,
    /// Whether to compile only.
    pub compile_only: bool,
    /// plugin_agent is the address of plugin.
    #[serde(skip)]
    pub plugin_agent: u64,
    /// fast_eval denotes directly executing at the AST level to obtain
    /// the result without any form of compilation.
    #[serde(skip)]
    pub fast_eval: bool,
}

impl ExecProgramArgs {
    /// [`get_package_maps_from_external_pkg`] gets the package name to package path mapping.
    pub fn get_package_maps_from_external_pkg(&self) -> HashMap<String, String> {
        let mut package_maps = HashMap::new();
        for external_pkg in &self.external_pkgs {
            package_maps.insert(external_pkg.pkg_name.clone(), external_pkg.pkg_path.clone());
        }
        package_maps
    }

    /// [`set_external_pkg_from_package_maps`] sets the package name to package path mapping.
    pub fn set_external_pkg_from_package_maps(&mut self, package_maps: HashMap<String, String>) {
        self.external_pkgs = package_maps
            .iter()
            .map(|(pkg_name, pkg_path)| ast::CmdExternalPkgSpec {
                pkg_name: pkg_name.clone(),
                pkg_path: pkg_path.clone(),
            })
            .collect();
    }
}

/// ExecProgramResult denotes the running result of the KCL program.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExecProgramResult {
    pub json_result: String,
    pub yaml_result: String,
    pub log_message: String,
    pub err_message: String,
}

pub trait MapErrorResult {
    /// Map execute error message into the [`Result::Err`]
    fn map_err_to_result(self) -> Result<ExecProgramResult>
    where
        Self: Sized;
}

impl MapErrorResult for ExecProgramResult {
    /// Map execute error message into the [`Result::Err`]
    fn map_err_to_result(self) -> Result<ExecProgramResult>
    where
        Self: Sized,
    {
        if self.err_message.is_empty() {
            Ok(self)
        } else {
            Err(anyhow!(self.err_message))
        }
    }
}

impl MapErrorResult for Result<ExecProgramResult> {
    /// Map execute error message into the [`Result::Err`]
    fn map_err_to_result(self) -> Result<ExecProgramResult>
    where
        Self: Sized,
    {
        match self {
            Ok(result) => result.map_err_to_result(),
            Err(err) => Err(err),
        }
    }
}

impl ExecProgramArgs {
    /// Deserialize an instance of type [ExecProgramArgs] from a string of JSON text.
    pub fn from_str(s: &str) -> Self {
        if s.trim().is_empty() {
            return Default::default();
        }
        serde_json::from_str::<ExecProgramArgs>(s).expect(s)
    }

    /// Serialize the [ExecProgramArgs] structure as a String of JSON.
    pub fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    /// Get the input file list.
    pub fn get_files(&self) -> Vec<&str> {
        self.k_filename_list.iter().map(|s| s.as_str()).collect()
    }

    /// Get the [`kclvm_parser::LoadProgramOptions`] from the [`kclvm_runner::ExecProgramArgs`]
    pub fn get_load_program_options(&self) -> kclvm_parser::LoadProgramOptions {
        kclvm_parser::LoadProgramOptions {
            work_dir: self.work_dir.clone().unwrap_or_default(),
            vendor_dirs: vec![get_vendor_home()],
            package_maps: self.get_package_maps_from_external_pkg(),
            k_code_list: self.k_code_list.clone(),
            load_plugins: self.plugin_agent > 0,
            ..Default::default()
        }
    }
}

impl TryFrom<SettingsFile> for ExecProgramArgs {
    type Error = anyhow::Error;
    fn try_from(settings: SettingsFile) -> Result<Self, Self::Error> {
        let mut args = Self::default();
        if let Some(cli_configs) = settings.kcl_cli_configs {
            args.k_filename_list = cli_configs.files.unwrap_or_default();
            if args.k_filename_list.is_empty() {
                args.k_filename_list = cli_configs.file.unwrap_or_default();
            }
            args.strict_range_check = cli_configs.strict_range_check.unwrap_or_default();
            args.disable_none = cli_configs.disable_none.unwrap_or_default();
            args.verbose = cli_configs.verbose.unwrap_or_default() as i32;
            args.debug = cli_configs.debug.unwrap_or_default() as i32;
            args.sort_keys = cli_configs.sort_keys.unwrap_or_default();
            args.show_hidden = cli_configs.show_hidden.unwrap_or_default();
            args.fast_eval = cli_configs.fast_eval.unwrap_or_default();
            args.include_schema_type_path =
                cli_configs.include_schema_type_path.unwrap_or_default();
            for override_str in &cli_configs.overrides.unwrap_or_default() {
                args.overrides.push(parse_override_spec(override_str)?);
            }
            args.path_selector = cli_configs.path_selector.unwrap_or_default();
            args.set_external_pkg_from_package_maps(
                cli_configs.package_maps.unwrap_or(HashMap::default()),
            )
        }
        if let Some(options) = settings.kcl_options {
            args.args = options
                .iter()
                .map(|o| ast::CmdArgSpec {
                    name: o.key.to_string(),
                    value: o.value.to_string(),
                })
                .collect();
        }
        Ok(args)
    }
}

impl TryFrom<SettingsPathBuf> for ExecProgramArgs {
    type Error = anyhow::Error;
    fn try_from(s: SettingsPathBuf) -> Result<Self, Self::Error> {
        let mut args: ExecProgramArgs = s.settings().clone().try_into()?;
        args.work_dir = s.path().clone().map(|p| p.to_string_lossy().to_string());
        Ok(args)
    }
}

/// A public struct named [Artifact] which wraps around the native library [libloading::Library].
pub struct Artifact(libloading::Library, String);

pub trait ProgramRunner {
    /// Run with the arguments [ExecProgramArgs] and return the program execute result that
    /// contains the planning result and the evaluation errors if any.
    fn run(&self, args: &ExecProgramArgs) -> Result<ExecProgramResult>;
}

impl ProgramRunner for Artifact {
    fn run(&self, args: &ExecProgramArgs) -> Result<ExecProgramResult> {
        unsafe {
            LibRunner::lib_kclvm_plugin_init(&self.0, args.plugin_agent)?;
            LibRunner::lib_kcl_run(&self.0, args)
        }
    }
}

impl Artifact {
    #[inline]
    pub fn from_path<P: AsRef<OsStr>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_str().unwrap().to_string();
        let lib = unsafe { libloading::Library::new(&path)? };
        Ok(Self(lib, path))
    }

    #[inline]
    pub fn get_path(&self) -> &String {
        &self.1
    }
}

#[derive(Debug, Default)]
pub struct RunnerOptions {
    pub plugin_agent_ptr: u64,
}

pub struct LibRunner {
    opts: RunnerOptions,
}

impl LibRunner {
    /// New a runner using the lib path and options.
    pub fn new(opts: Option<RunnerOptions>) -> Self {
        Self {
            opts: opts.unwrap_or_default(),
        }
    }

    /// Run kcl library with exec arguments.
    pub fn run(&self, lib_path: &str, args: &ExecProgramArgs) -> Result<ExecProgramResult> {
        unsafe {
            let lib = libloading::Library::new(std::path::PathBuf::from(lib_path).canonicalize()?)?;
            Self::lib_kclvm_plugin_init(&lib, self.opts.plugin_agent_ptr)?;
            Self::lib_kcl_run(&lib, args)
        }
    }
}

impl LibRunner {
    unsafe fn lib_kclvm_plugin_init(
        lib: &libloading::Library,
        plugin_method_ptr: u64,
    ) -> Result<()> {
        // get kclvm_plugin_init
        let kclvm_plugin_init: libloading::Symbol<
            unsafe extern "C" fn(
                fn_ptr: extern "C" fn(
                    method: *const i8,
                    args_json: *const i8,
                    kwargs_json: *const i8,
                ) -> *const i8,
            ),
        > = lib.get(b"kclvm_plugin_init")?;

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
        Ok(())
    }

    unsafe fn lib_kcl_run(
        lib: &libloading::Library,
        args: &ExecProgramArgs,
    ) -> Result<ExecProgramResult> {
        let kcl_run: libloading::Symbol<
            unsafe extern "C" fn(
                kclvm_main_ptr: u64, // main.k => kclvm_main
                option_len: kclvm_size_t,
                option_keys: *const *const kclvm_char_t,
                option_values: *const *const kclvm_char_t,
                opts: FFIRunOptions,
                path_selector: *const *const kclvm_char_t,
                json_result_buffer_len: *mut kclvm_size_t,
                json_result_buffer: *mut kclvm_char_t,
                yaml_result_buffer_len: *mut kclvm_size_t,
                yaml_result_buffer: *mut kclvm_char_t,
                err_buffer_len: *mut kclvm_size_t,
                err_buffer: *mut kclvm_char_t,
                log_buffer_len: *mut kclvm_size_t,
                log_buffer: *mut kclvm_char_t,
            ) -> kclvm_size_t,
        > = lib.get(b"_kcl_run")?;

        // The lib main function
        let kclvm_main: libloading::Symbol<u64> = lib.get(b"kclvm_main")?;
        let kclvm_main_ptr = kclvm_main.into_raw().into_raw() as u64;

        // CLI configs option len
        let option_len = args.args.len() as kclvm_size_t;
        // CLI configs option keys
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
        let option_keys = p_argv.as_ptr();
        // CLI configs option values
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
        let option_values = p_argv.as_ptr();
        // path selectors
        let cstr_argv: Vec<_> = args
            .path_selector
            .iter()
            .map(|arg| std::ffi::CString::new(arg.as_str()).unwrap())
            .collect();
        let mut p_argv: Vec<_> = cstr_argv
            .iter() // do NOT into_iter()
            .map(|arg| arg.as_ptr())
            .collect();
        p_argv.push(std::ptr::null());
        let path_selector = p_argv.as_ptr();

        let opts = FFIRunOptions {
            strict_range_check: args.strict_range_check as i32,
            disable_none: args.disable_none as i32,
            disable_schema_check: 0,
            disable_empty_list: 0,
            sort_keys: args.sort_keys as i32,
            show_hidden: args.show_hidden as i32,
            debug_mode: args.debug,
            include_schema_type_path: args.include_schema_type_path as i32,
        };
        let mut json_buffer = Buffer::make();
        let mut yaml_buffer = Buffer::make();
        let mut log_buffer = Buffer::make();
        let mut err_buffer = Buffer::make();
        // Input the main function, options and return the exec result
        // including JSON and YAML result, log message and error message.
        kcl_run(
            kclvm_main_ptr,
            option_len,
            option_keys,
            option_values,
            opts,
            path_selector,
            json_buffer.mut_len(),
            json_buffer.mut_ptr(),
            yaml_buffer.mut_len(),
            yaml_buffer.mut_ptr(),
            err_buffer.mut_len(),
            err_buffer.mut_ptr(),
            log_buffer.mut_len(),
            log_buffer.mut_ptr(),
        );
        // Convert runtime result to ExecProgramResult
        let mut result = ExecProgramResult {
            yaml_result: yaml_buffer.to_string()?,
            json_result: json_buffer.to_string()?,
            log_message: log_buffer.to_string()?,
            err_message: err_buffer.to_string()?,
        };
        // Wrap runtime JSON Panic error string into diagnostic style string.
        if !result.err_message.is_empty() && std::env::var(KCL_DEBUG_ERROR_ENV_VAR).is_err() {
            result.err_message = match Handler::default()
                .add_diagnostic(<PanicInfo as Into<Diagnostic>>::into(PanicInfo::from(
                    result.err_message.as_str(),
                )))
                .emit_to_string()
            {
                Ok(msg) => msg,
                Err(err) => err.to_string(),
            };
        }
        Ok(result)
    }
}

thread_local! {
    static KCL_RUNTIME_PANIC_RECORD: RefCell<RuntimePanicRecord>  = RefCell::new(RuntimePanicRecord::default())
}

pub struct FastRunner {
    opts: RunnerOptions,
}

impl FastRunner {
    /// New a runner using the lib path and options.
    pub fn new(opts: Option<RunnerOptions>) -> Self {
        Self {
            opts: opts.unwrap_or_default(),
        }
    }

    /// Run kcl library with exec arguments.
    pub fn run(&self, program: &ast::Program, args: &ExecProgramArgs) -> Result<ExecProgramResult> {
        let ctx = Rc::new(RefCell::new(args_to_ctx(&program, args)));
        let evaluator = Evaluator::new_with_runtime_ctx(&program, ctx.clone());
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
            KCL_RUNTIME_PANIC_RECORD.with(|record| {
                let mut record = record.borrow_mut();
                record.kcl_panic_info = true;
                record.message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = info.payload().downcast_ref::<&String>() {
                    (*s).clone()
                } else if let Some(s) = info.payload().downcast_ref::<String>() {
                    (*s).clone()
                } else {
                    "".to_string()
                };
                if let Some(location) = info.location() {
                    record.rust_file = location.file().to_string();
                    record.rust_line = location.line() as i32;
                    record.rust_col = location.column() as i32;
                }
            })
        }));
        let evaluator_result = std::panic::catch_unwind(|| {
            if self.opts.plugin_agent_ptr > 0 {
                unsafe {
                    let plugin_method: extern "C" fn(
                        method: *const i8,
                        args: *const c_char,
                        kwargs: *const c_char,
                    ) -> *const c_char = std::mem::transmute(self.opts.plugin_agent_ptr);
                    kclvm_plugin_init(plugin_method);
                }
            }
            evaluator.run()
        });
        std::panic::set_hook(prev_hook);
        KCL_RUNTIME_PANIC_RECORD.with(|record| {
            let record = record.borrow();
            ctx.borrow_mut().set_panic_info(&record);
        });
        let mut result = ExecProgramResult {
            log_message: ctx.borrow().log_message.clone(),
            ..Default::default()
        };
        let is_err = evaluator_result.is_err();
        match evaluator_result {
            Ok(r) => match r {
                Ok((json, yaml)) => {
                    result.json_result = json;
                    result.yaml_result = yaml;
                }
                Err(err) => {
                    result.err_message = err.to_string();
                }
            },
            Err(err) => {
                result.err_message = if is_err {
                    ctx.borrow()
                        .get_panic_info_json_string()
                        .unwrap_or_default()
                } else {
                    kclvm_error::err_to_str(err)
                };
            }
        }
        // Wrap runtime JSON Panic error string into diagnostic style string.
        if !result.err_message.is_empty() && std::env::var(KCL_DEBUG_ERROR_ENV_VAR).is_err() {
            result.err_message = match Handler::default()
                .add_diagnostic(<PanicInfo as Into<Diagnostic>>::into(PanicInfo::from(
                    result.err_message.as_str(),
                )))
                .emit_to_string()
            {
                Ok(msg) => msg,
                Err(err) => err.to_string(),
            };
        }
        Ok(result)
    }
}

pub(crate) fn args_to_ctx(program: &ast::Program, args: &ExecProgramArgs) -> Context {
    let mut ctx = Context::new();
    ctx.cfg.strict_range_check = args.strict_range_check;
    ctx.cfg.debug_mode = args.debug != 0;
    ctx.plan_opts.disable_none = args.disable_none;
    ctx.plan_opts.show_hidden = args.show_hidden;
    ctx.plan_opts.sort_keys = args.sort_keys;
    ctx.plan_opts.include_schema_type_path = args.include_schema_type_path;
    ctx.plan_opts.query_paths = args.path_selector.clone();
    for arg in &args.args {
        ctx.builtin_option_init(&arg.name, &arg.value);
    }
    ctx.set_kcl_workdir(&args.work_dir.clone().unwrap_or_default());
    ctx.set_kcl_module_path(&program.root);
    ctx
}

#[repr(C)]
struct Buffer(Vec<u8>, i32);

impl Buffer {
    #[inline]
    fn make() -> Self {
        let buffer = vec![0u8; RESULT_SIZE];
        Self(buffer, RESULT_SIZE as i32 - 1)
    }

    #[inline]
    fn to_string(&self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.0[0..self.1 as usize].to_vec())?)
    }

    #[inline]
    fn mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr() as *mut c_char
    }

    #[inline]
    fn mut_len(&mut self) -> &mut i32 {
        &mut self.1
    }
}
