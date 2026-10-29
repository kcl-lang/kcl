use anyhow::{Result, anyhow};
use kcl_evaluator::{CoverageState, Evaluator};
use std::collections::{HashMap, HashSet};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};

use kcl_ast::ast;
use kcl_config::{
    modfile::get_vendor_home,
    settings::{SettingsFile, SettingsPathBuf},
};
use kcl_error::{Diagnostic, Handler, RenderedError};
use kcl_primitives::IndexMap;
#[cfg(not(target_arch = "wasm32"))]
use kcl_runtime::kcl_plugin_init;
use kcl_runtime::val_plan::PlanOptions;
use kcl_runtime::{
    BacktraceFrame, Context, ContextBuffer, PanicInfo, RuntimePanicRecord, ValueRef,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::os::raw::c_char;

const RESULT_SIZE: usize = 2048 * 2048;
const KCL_DEBUG_ERROR_ENV_VAR: &str = "KCL_DEBUG_ERROR";

#[allow(non_camel_case_types)]
pub type kcl_char_t = c_char;
#[allow(non_camel_case_types)]
pub type kcl_size_t = i32;
#[allow(non_camel_case_types)]
pub type kcl_context_t = std::ffi::c_void;
#[allow(non_camel_case_types)]
pub type kcl_value_ref_t = std::ffi::c_void;

/// ExecProgramArgs denotes the configuration required to execute the KCL program.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExecProgramArgs {
    pub work_dir: Option<String>,
    pub k_filename_list: Vec<String>,
    /// -E key=value
    pub external_pkgs: Vec<ast::ExternalPkg>,
    pub k_code_list: Vec<String>,
    /// -D key=value
    pub args: Vec<ast::Argument>,
    /// -O override_spec
    pub overrides: Vec<String>,
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
    /// Output format selector ("yaml" or "json"). When `None`, both
    /// formats are generated (legacy behaviour, used when the caller
    /// did not specify a format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Name recorded in the `file` field of the emitted Source Map v3
    /// document, i.e. the generated artifact the map accompanies. When
    /// `None`, no source map is generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sourcemap_output: Option<String>,
    /// When `true`, the planner emits the `__kcl_info_meta__`
    /// side-channel marker so downstream emitters (CLI `--format xml`,
    /// kcl-go `XAMLString()`) can render decorated schema fields as
    /// XML attributes (vs. child elements).
    #[serde(default, skip_serializing_if = "is_false")]
    pub emit_attribute_metadata: bool,
    /// When true, the evaluator records `(filename, line)` hits for every
    /// top-level statement it walks and the runner drains those hits into
    /// [`ExecProgramResult::coverage`]. Default false — recording has a
    /// small cost so callers must opt in. Used by `kcl test --coverage`.
    /// `serde(default)` keeps older JSON fixtures (which predate this
    /// field) compatible with the current deserializer.
    #[serde(default)]
    pub coverage: bool,
}

#[inline]
fn is_false(b: &bool) -> bool {
    !*b
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
            .map(|(pkg_name, pkg_path)| ast::ExternalPkg {
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
    /// Source Map v3 JSON mapping generated YAML lines back to the `.k`
    /// statements that produced them. Only populated when the caller sets
    /// [`ExecProgramArgs::sourcemap_output`].
    pub sourcemap: Option<String>,
    /// Per-run line hits collected when [`ExecProgramArgs::coverage`] was
    /// `true`. Maps `filename -> line -> hits`. Empty otherwise.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub coverage: HashMap<String, HashMap<u64, u64>>,
    /// The structured diagnostics `err_message` was rendered from.
    ///
    /// Callers that re-render the error in a machine-readable format
    /// (arcanist JSON, SARIF) need these to report real source positions
    /// rather than the position-less text in `err_message` (issue #2216).
    /// Never serialized: this is an in-process carrier only.
    #[serde(skip)]
    pub diagnostics: Vec<Diagnostic>,
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
            // `RenderedError` displays as `err_message`, so callers still see
            // the same text; the diagnostics travel along for callers that
            // re-render the error in a machine-readable format.
            Err(anyhow!(RenderedError::new(
                self.err_message,
                self.diagnostics
            )))
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

/// The result of one call made by [`crate::exec_program_calls`].
#[derive(Debug, Default, Clone)]
pub struct ExecCallResult {
    /// Name of the called function.
    pub name: String,
    /// The log, the error and the coverage of the evaluation plus the
    /// call. The JSON and YAML results are always empty.
    pub result: ExecProgramResult,
    /// How long the call took, not counting the evaluation it ran in.
    pub duration: Duration,
}

impl ExecProgramArgs {
    /// Deserialize an instance of type [ExecProgramArgs] from a string of JSON text.
    pub fn from_json(s: &str) -> Self {
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

    /// Get the [`kcl_parser::LoadProgramOptions`] from the [`kcl_runner::ExecProgramArgs`]
    pub fn get_load_program_options(&self) -> kcl_parser::LoadProgramOptions {
        kcl_parser::LoadProgramOptions {
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
            args.format = cli_configs.format.clone().filter(|s| !s.is_empty());
            args.sourcemap_output = cli_configs
                .sourcemap_output
                .clone()
                .filter(|s| !s.is_empty());
            args.emit_attribute_metadata = cli_configs.emit_attribute_metadata.unwrap_or_default();
            for override_str in cli_configs.overrides.unwrap_or_default() {
                args.overrides.push(override_str);
            }
            args.path_selector = cli_configs.path_selector.unwrap_or_default();
            args.set_external_pkg_from_package_maps(
                cli_configs.package_maps.unwrap_or(HashMap::default()),
            )
        }
        if let Some(options) = settings.kcl_options {
            args.args = options
                .iter()
                .map(|o| ast::Argument {
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

#[derive(Debug, Default)]
pub struct RunnerOptions {
    pub plugin_agent_ptr: u64,
    /// When set, packages absent from this set skip pass-3 statement
    /// evaluation during import (issue #1758). `None` preserves legacy
    /// behaviour where every imported package's body is evaluated.
    /// Each entry is a pkgpath prefixed with `'@'` (i.e.
    /// `kcl_runtime::PKG_PATH_PREFIX`) so it can be compared against
    /// the prefix-stripped strings the evaluator carries internally.
    pub referenced_pkgs: Option<HashSet<String>>,
}

impl RunnerOptions {
    /// Clone the configured referenced-packages set, if any. `None`
    /// here means the evaluator should fall back to legacy behaviour
    /// (every imported package's body is evaluated).
    pub(crate) fn referenced_pkgs_set(&self) -> Option<HashSet<String>> {
        self.referenced_pkgs.clone()
    }
}

pub trait ProgramRunner {
    /// Run with the arguments [ExecProgramArgs] and return the program execute result that
    /// contains the planning result and the evaluation errors if any.
    fn run(&self, args: &ExecProgramArgs) -> Result<ExecProgramResult>;
}

thread_local! {
    pub static KCL_RUNTIME_PANIC_RECORD: RefCell<RuntimePanicRecord> = RefCell::new(RuntimePanicRecord::default());
    static RECORDING_PANICS: Cell<bool> = const { Cell::new(false) };
}

static ONCE_PANIC_HOOK: Lazy<()> = Lazy::new(|| {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info: &std::panic::PanicHookInfo| {
        // This hook runs for every panic in the process, so it must not panic.
        let _ = KCL_RUNTIME_PANIC_RECORD.try_with(|record| {
            let Ok(mut record) = record.try_borrow_mut() else {
                return;
            };
            record.kcl_panic_info = true;
            record.message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<&String>() {
                (*s).clone()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                (*s).clone()
            } else {
                "unknown runtime error".to_string()
            };
            if let Some(location) = info.location() {
                record.rust_file = location.file().to_string();
                record.rust_line = location.line() as i32;
                record.rust_col = location.column() as i32;
            }
        });
        if !RECORDING_PANICS.get() {
            prev_hook(info);
        }
    }));
});

/// Run `f` like [`std::panic::catch_unwind`] without printing its panic.
///
/// The panic hook is process-global, so it is installed once. It records
/// every panic into the thread-local [`KCL_RUNTIME_PANIC_RECORD`] and prints
/// it through the hook it replaced only on threads outside this function. A
/// hook set after the first call replaces this one for the whole process, so
/// panics inside this function then reach that hook and are not recorded.
pub fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    Lazy::force(&ONCE_PANIC_HOOK);
    let recording = RECORDING_PANICS.replace(true);
    let result = std::panic::catch_unwind(f);
    RECORDING_PANICS.set(recording);
    result
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
        let ctx = Rc::new(RefCell::new(args_to_ctx(program, args)));
        // Coverage state is created up front so it can outlive the
        // `catch_unwind` closure — the closure moves the evaluator by value,
        // so we drain the shared state after the closure returns. When
        // `args.coverage` is false the state is `None` and the evaluator
        // short-circuits on its own.
        let coverage_state = if args.coverage {
            Some(CoverageState::new())
        } else {
            None
        };
        let mut evaluator = Evaluator::new_with_runtime_ctx(program, ctx.clone());
        evaluator.set_coverage_state(coverage_state.clone());
        if let Some(pkgs) = self.opts.referenced_pkgs_set() {
            evaluator.set_referenced_pkgs(pkgs);
        }
        // Drop panics recorded on this thread before this run.
        KCL_RUNTIME_PANIC_RECORD.take();
        let evaluator_result = catch_unwind_silent(|| {
            self.init_plugin_agent();
            evaluator.run()
        });
        let mut result = ExecProgramResult {
            log_message: ctx.borrow().log_message.clone(),
            ..Default::default()
        };
        match evaluator_result {
            Ok(r) => match r {
                Ok((json, yaml)) => {
                    // Honour the requested output format: only the requested
                    // encoder ran; the other side stays empty. When the
                    // caller didn't ask for a specific format, both fields
                    // are populated (legacy behaviour).
                    match args.format.as_deref() {
                        Some("json") => result.json_result = json,
                        Some("yaml") => result.yaml_result = yaml,
                        _ => {
                            result.json_result = json;
                            result.yaml_result = yaml;
                        }
                    }
                    // The map indexes generated YAML lines, so it is only
                    // meaningful when the YAML encoder actually ran.
                    if let Some(file) = &args.sourcemap_output
                        && !result.yaml_result.is_empty()
                    {
                        result.sourcemap = evaluator
                            .source_map(&result.yaml_result, file)
                            .to_json()
                            .ok();
                    }
                }
                Err(err) => {
                    result.err_message = err.to_string();
                }
            },
            Err(err) => {
                result.err_message = panic_err_message(&ctx, err);
            }
        }
        render_runtime_error(&mut result);
        // Free all value references at runtime. This is because the runtime context marks
        // all KCL objects and holds their copies, so it is necessary to actively GC them.
        ctx.borrow().gc();
        // Drain collected coverage hits into the result. We snapshot the
        // recorded `(filename, line, hits)` triples, group by filename, and
        // merge counts so multiple test runs that share a file sum
        // correctly. This runs after `ctx.borrow().gc()` so it observes
        // only what the evaluator actually walked.
        if let Some(state) = coverage_state.as_ref() {
            result.coverage = coverage_by_file(state.drain());
        }
        Ok(result)
    }

    /// Evaluate the program once, then call the global functions `names`
    /// one after another in that evaluation. See
    /// [`crate::exec_program_calls`].
    pub fn run_calls(
        &self,
        program: &ast::Program,
        args: &ExecProgramArgs,
        names: &[String],
        stop_on_error: bool,
    ) -> Result<Vec<ExecCallResult>> {
        let mut shared = self.eval_shared(program, args);
        let eval_hits = shared.drain_coverage();
        // Plan once instead of after every call, which would cost as much
        // as the evaluation for a program with large values. A call
        // changes what planning reports only if it sets the output, which
        // is checked after the call, or mutates a value in place, which
        // planning does not see: like the later calls that see such a
        // mutation, that is a documented difference.
        let mut plan_err_message = String::new();
        // Whether the next call needs a freshly evaluated program.
        let mut stale = false;
        if shared.err_message.is_empty() {
            KCL_RUNTIME_PANIC_RECORD.take();
            let plan = catch_unwind_silent(|| shared.evaluator.plan());
            plan_err_message = step_err_message(&shared.ctx, plan.map(Ok));
            stale = !plan_err_message.is_empty();
        }
        let mut results = Vec::with_capacity(names.len());
        for name in names {
            // A call that panicked may have left the evaluator midway
            // through a lambda or a schema, and a call that failed may
            // have changed values that the next call must not see;
            // evaluate the program again rather than undo either by hand.
            if stale {
                shared.ctx.borrow().gc();
                shared = self.eval_shared(program, args);
                shared.drain_coverage();
                stale = false;
            }
            let mut result = ExecProgramResult::default();
            let mut duration = Duration::ZERO;
            if shared.err_message.is_empty() {
                shared.rest.restore(&mut shared.ctx.borrow_mut());
                let depths = shared.evaluator.stack_depths();
                let objects = shared.ctx.borrow().objects.len();
                KCL_RUNTIME_PANIC_RECORD.take();
                let start = Instant::now();
                let call = catch_unwind_silent(|| {
                    // A plugin that an earlier call ran may have evaluated
                    // another program with another plugin agent.
                    self.init_plugin_agent();
                    shared.evaluator.call_global(name)
                });
                duration = start.elapsed();
                // The record also shows a panic that `runtime.catch`
                // stopped inside the call, unless an evaluation nested in
                // the call, e.g. one a plugin ran, cleared it; the stacks
                // show what such a panic left behind either way.
                stale = !matches!(call, Ok(Ok(_)))
                    || KCL_RUNTIME_PANIC_RECORD.with(|record| record.borrow().kcl_panic_info)
                    || shared.evaluator.stack_depths() != depths;
                result.log_message = shared.ctx.borrow().log_message.clone();
                result.err_message = step_err_message(&shared.ctx, call);
                if result.err_message.is_empty() {
                    // Planning reads the output and the options a call
                    // set, e.g. with `manifests.yaml_stream`, so plan again
                    // for such a call, whether or not the program planned.
                    let replan = {
                        let ctx = shared.ctx.borrow();
                        ctx.buffer.custom_manifests_output
                            != shared.rest.buffer.custom_manifests_output
                            || ctx.plan_opts != shared.rest.plan_opts
                    };
                    if replan {
                        KCL_RUNTIME_PANIC_RECORD.take();
                        let plan = catch_unwind_silent(|| shared.evaluator.plan());
                        result.err_message = step_err_message(&shared.ctx, plan.map(Ok));
                    } else {
                        result.err_message = plan_err_message.clone();
                    }
                }
                // A call that failed, even only to plan, leaves the next
                // call none of its changes, as when it runs on its own.
                stale = stale || !result.err_message.is_empty();
                if !stale {
                    shared.free_objects_from(objects);
                }
            } else {
                result.log_message = shared.rest.log_message.clone();
                result.err_message = shared.err_message.clone();
            }
            render_runtime_error(&mut result);
            if args.coverage {
                result.coverage =
                    coverage_by_file(eval_hits.iter().cloned().chain(shared.drain_coverage()));
            }
            let failed = !result.err_message.is_empty();
            results.push(ExecCallResult {
                name: name.clone(),
                result,
                duration,
            });
            if failed && stop_on_error {
                break;
            }
        }
        shared.ctx.borrow().gc();
        Ok(results)
    }

    /// Evaluate the program for [`FastRunner::run_calls`], recording the
    /// error of the evaluation instead of returning it.
    fn eval_shared<'p>(&self, program: &'p ast::Program, args: &ExecProgramArgs) -> SharedEval<'p> {
        let ctx = Rc::new(RefCell::new(args_to_ctx(program, args)));
        let coverage_state = if args.coverage {
            Some(CoverageState::new())
        } else {
            None
        };
        let mut evaluator = Evaluator::new_with_runtime_ctx(program, ctx.clone());
        evaluator.set_coverage_state(coverage_state.clone());
        if let Some(pkgs) = self.opts.referenced_pkgs_set() {
            evaluator.set_referenced_pkgs(pkgs);
        }
        KCL_RUNTIME_PANIC_RECORD.take();
        let eval = catch_unwind_silent(|| {
            self.init_plugin_agent();
            evaluator.eval()
        });
        let err_message = step_err_message(&ctx, eval);
        let rest = ContextRest::save(&ctx.borrow());
        SharedEval {
            ctx,
            evaluator,
            coverage_state,
            rest,
            err_message,
        }
    }

    /// Hand the plugin agent, if any, to the runtime.
    fn init_plugin_agent(&self) {
        if self.opts.plugin_agent_ptr > 0 {
            #[cfg(not(target_arch = "wasm32"))]
            unsafe {
                let plugin_method: extern "C-unwind" fn(
                    method: *const c_char,
                    args: *const c_char,
                    kwargs: *const c_char,
                ) -> *const c_char = std::mem::transmute(self.opts.plugin_agent_ptr);
                kcl_plugin_init(plugin_method);
            }
        }
    }
}

/// A program evaluated once by [`FastRunner::run_calls`].
struct SharedEval<'p> {
    ctx: Rc<RefCell<Context>>,
    evaluator: Evaluator<'p>,
    coverage_state: Option<CoverageState>,
    /// The context as the evaluation left it, restored before every call.
    rest: ContextRest,
    /// The raw error of the evaluation, empty when it succeeded.
    err_message: String,
}

impl SharedEval<'_> {
    /// Take the coverage hits recorded since the last drain.
    fn drain_coverage(&self) -> Vec<(String, u64, u64)> {
        self.coverage_state
            .as_ref()
            .map(CoverageState::drain)
            .unwrap_or_default()
    }

    /// Free the values that the runtime boxed after its first `len`.
    ///
    /// The runtime boxes the arguments and the result of every builtin
    /// call and frees the boxes only with the context, so without this,
    /// the boxes of every call would live until the last call ends. Once
    /// a call has returned, nothing points to the boxes it made. They are
    /// freed without `kcl_value_delete`, which clears the closure of a
    /// boxed function value that the program may still use.
    fn free_objects_from(&self, len: usize) {
        let mut ctx = self.ctx.borrow_mut();
        for ptr in ctx.objects.drain(len..) {
            // SAFETY: `new_mut_ptr` boxed every object, and an object
            // drained from the set isn't freed again by `Context::gc`.
            unsafe { kcl_runtime::free_mut_ptr(ptr as *mut ValueRef) };
        }
    }
}

/// The parts of a [`Context`] that a call of [`FastRunner::run_calls`]
/// changes and that must not reach the next call: what the call printed,
/// where it failed, the schema instances it created and the output and
/// planning options it set.
struct ContextRest {
    log_message: String,
    panic_info: PanicInfo,
    backtrace: Vec<BacktraceFrame>,
    instances: IndexMap<String, IndexMap<String, Vec<ValueRef>>>,
    buffer: ContextBuffer,
    plan_opts: PlanOptions,
}

impl ContextRest {
    fn save(ctx: &Context) -> Self {
        Self {
            log_message: ctx.log_message.clone(),
            panic_info: ctx.panic_info.clone(),
            backtrace: ctx.backtrace.clone(),
            instances: ctx.instances.clone(),
            buffer: ctx.buffer.clone(),
            plan_opts: ctx.plan_opts.clone(),
        }
    }

    fn restore(&self, ctx: &mut Context) {
        ctx.log_message.clone_from(&self.log_message);
        ctx.panic_info.clone_from(&self.panic_info);
        ctx.backtrace.clone_from(&self.backtrace);
        ctx.instances.clone_from(&self.instances);
        ctx.buffer.clone_from(&self.buffer);
        ctx.plan_opts.clone_from(&self.plan_opts);
    }
}

/// The raw error message of an evaluation step that [`catch_unwind_silent`]
/// ran in `ctx`: the error it returned or the panic it raised. Empty when
/// the step succeeded.
fn step_err_message<T>(ctx: &RefCell<Context>, result: std::thread::Result<Result<T>>) -> String {
    match result {
        Ok(Ok(_)) => String::new(),
        Ok(Err(err)) => err.to_string(),
        Err(err) => panic_err_message(ctx, err),
    }
}

/// The runtime panic info of `ctx`, as JSON, for a panic that
/// [`catch_unwind_silent`] caught with the payload `err`.
fn panic_err_message(ctx: &RefCell<Context>, err: Box<dyn Any + Send>) -> String {
    KCL_RUNTIME_PANIC_RECORD.with(|record| {
        let record = record.borrow();
        ctx.borrow_mut().set_panic_info(&record);
    });
    // The payload is this run's own panic, whereas the record is
    // written by a process-global hook the host may have replaced.
    let message = kcl_error::err_to_str(err);
    if !message.is_empty() {
        ctx.borrow_mut().panic_info.message = message;
    }
    ctx.borrow()
        .get_panic_info_json_string()
        .unwrap_or_default()
}

/// Wrap runtime JSON Panic error string into diagnostic style string.
fn render_runtime_error(result: &mut ExecProgramResult) {
    if !result.err_message.is_empty() && std::env::var(KCL_DEBUG_ERROR_ENV_VAR).is_err() {
        let diag =
            <PanicInfo as Into<Diagnostic>>::into(PanicInfo::from(result.err_message.as_str()));
        // Keep the structured form around so callers can re-render the
        // error in a machine-readable format without losing the source
        // positions that `emit_to_string` bakes into the text.
        result.diagnostics.push(diag.clone());
        result.err_message = match Handler::default().add_diagnostic(diag).emit_to_string() {
            Ok(msg) => msg,
            Err(err) => err.to_string(),
        };
    }
}

/// Group coverage hits `(filename, line, hits)` by filename, summing the
/// hits of a line that occurs more than once.
fn coverage_by_file(
    hits: impl IntoIterator<Item = (String, u64, u64)>,
) -> HashMap<String, HashMap<u64, u64>> {
    let mut by_file: HashMap<String, HashMap<u64, u64>> = HashMap::new();
    for (filename, line, hits) in hits {
        *by_file
            .entry(filename)
            .or_default()
            .entry(line)
            .or_default() += hits;
    }
    by_file
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
    ctx.plan_opts.format = args.format.clone();
    ctx.plan_opts.emit_attribute_metadata = args.emit_attribute_metadata;
    for arg in &args.args {
        ctx.builtin_option_init(&arg.name, &arg.value);
    }
    ctx.set_kcl_workdir(&args.work_dir.clone().unwrap_or_default());
    ctx.set_kcl_module_path(&program.root);
    ctx
}

#[repr(C)]
pub struct Buffer(Vec<u8>, i32);

impl Buffer {
    #[inline]
    pub fn make() -> Self {
        let buffer = vec![0u8; RESULT_SIZE];
        Self(buffer, RESULT_SIZE as i32 - 1)
    }

    #[inline]
    pub fn to_string(&self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.0[0..self.1 as usize].to_vec())?)
    }

    #[inline]
    pub fn mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr() as *mut c_char
    }

    #[inline]
    pub fn mut_len(&mut self) -> &mut i32 {
        &mut self.1
    }
}
