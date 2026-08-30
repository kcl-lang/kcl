use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

use crate::gpyrpc::{self, *};

use kcl_ast::ast::SerializeProgram;
use kcl_config::settings::build_settings_pathbuf;
use kcl_error::format::DiagnosticFormat;
use kcl_error::{Diagnostic, Handler, Level, Message};
use kcl_language_server::rename;
use kcl_loader::option::list_options;
use kcl_loader::{LoadPackageOptions, load_packages_with_cache};
use kcl_parser::KCLModuleCache;
use kcl_parser::LoadProgramOptions;
use kcl_parser::ParseSessionRef;
use kcl_parser::entry::{canonicalize_input_file, get_normalized_k_files_from_paths};
use kcl_parser::load_program;
use kcl_parser::parse_single_file;
use kcl_query::GetSchemaOption;
use kcl_query::override_file;
use kcl_query::query::CompilationOptions;
use kcl_query::query::{get_full_schema_type, get_full_schema_type_under_path};
use kcl_query::selector::{ListOptions, list_variables};
use kcl_runner::exec_program;
use kcl_sema::core::global_state::GlobalState;
use kcl_sema::resolver::Options;
use kcl_sema::resolver::scope::KCLScopeCache;
use kcl_tools::format::{FormatOptions, format, format_source};
use kcl_tools::lint::lint_files;
use kcl_tools::testing;
use kcl_tools::testing::TestRun;
use kcl_tools::vet::validator::LoaderKind;
use kcl_tools::vet::validator::ValidateOption;
use kcl_tools::vet::validator::validate;
use tempfile::NamedTempFile;

use super::into::*;
use super::ty::kcl_schema_ty_to_pb_ty;
use super::util::{transform_exec_para, transform_str_para};

/// Resolve the diagnostic output format from a proto argument and the
/// `KCL_ERROR_FORMAT` environment variable.
///
/// Precedence: explicit `args.error_format` > `KCL_ERROR_FORMAT` > default
/// `Pretty`. Invalid values are reported as an error so callers fail loudly.
pub(crate) fn resolve_error_format(args_error_format: &str) -> anyhow::Result<DiagnosticFormat> {
    if !args_error_format.is_empty() {
        return DiagnosticFormat::from_str(args_error_format).map_err(anyhow::Error::from);
    }
    if let Ok(s) = std::env::var("KCL_ERROR_FORMAT") {
        if !s.is_empty() {
            return DiagnosticFormat::from_str(&s).map_err(anyhow::Error::from);
        }
    }
    Ok(DiagnosticFormat::Pretty)
}

/// Render a machine-readable representation of `err_message` using the
/// requested `format`. Returns an empty string when the caller asked for the
/// default `Pretty` output or when `err_message` is empty. Otherwise wraps
/// the message in a `Diagnostic` and dispatches to the format-specific
/// renderer so the result can be inspected (e.g. in tests) without needing
/// to capture process-global stderr.
pub(crate) fn render_machine_readable_error(
    err_message: &str,
    format: DiagnosticFormat,
) -> anyhow::Result<String> {
    if format == DiagnosticFormat::Pretty || err_message.is_empty() {
        return Ok(String::new());
    }
    let mut handler = Handler::new();
    let pos = kcl_error::Position::dummy_pos();
    handler.add_diagnostic(Diagnostic {
        level: Level::Error,
        messages: vec![Message {
            range: (pos.clone(), pos),
            style: kcl_error::Style::LineAndColumn,
            message: err_message.to_string(),
            note: None,
            suggested_replacement: None,
        }],
        code: None,
    });
    handler.emit_to_string_as(format)
}

/// Emit a machine-readable representation of `err_message` to stderr when the
/// caller asked for a non-pretty diagnostic format. Returns Ok(()) always so
/// callers can use it in a tail position.
pub(crate) fn emit_machine_readable_error(
    err_message: &str,
    format: DiagnosticFormat,
) -> anyhow::Result<()> {
    let rendered = render_machine_readable_error(err_message, format)?;
    if !rendered.is_empty() {
        use std::io::Write;
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{rendered}")?;
    }
    Ok(())
}

/// Force the allocator to release freed memory back to the OS.
///
/// On Linux **glibc** this calls `malloc_trim(0)`, which returns the top
/// free chunk to the kernel. (MUSL and other libcs don't expose
/// `malloc_trim`, so the call is cfg-gated to glibc only.) On macOS /
/// Windows / WASM this is a no-op — there is no portable reclaim
/// primitive, and the most reliable fix is to load the library with
/// `LD_PRELOAD=libmimalloc.so.1 MIMALLOC_RESET=1` (Linux) or the
/// platform-specific equivalent. See `docs/dev_guide/6.memory_tuning.md`
/// for the full discussion.
///
/// This is invoked automatically at the end of
/// [`KclServiceImpl::exec_program`]; it is also exposed publicly so
/// callers driving other RPCs can invoke it manually.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn release_memory() {
    // SAFETY: `malloc_trim(0)` is always safe to call. It walks glibc's
    // arena bins and returns the top free chunk to the OS via `madvise`
    // / `munmap`. Sub-millisecond on typical workloads; idempotent.
    unsafe {
        libc::malloc_trim(0);
    }
}

#[cfg(target_os = "macos")]
fn release_memory() {
    // No stable public API exists. `malloc_zone_pressure_relief` is
    // per-zone, brittle, and not part of the documented allocator
    // contract. Production deployments should preload mimalloc with
    // `MIMALLOC_RESET=1` — see docs/dev_guide/6.memory_tuning.md.
}

/// MUSL / unknown libc / Windows / WASM — no portable reclaim primitive.
/// Same mimalloc recommendation as macOS.
#[cfg(not(any(all(target_os = "linux", target_env = "gnu"), target_os = "macos")))]
fn release_memory() {}

/// Specific implementation of calling service
#[derive(Debug, Clone, Default)]
pub struct KclServiceImpl {
    pub plugin_agent: u64,
}

impl From<&kcl_query::selector::Variable> for Variable {
    fn from(var: &kcl_query::selector::Variable) -> Self {
        Variable {
            value: var.value.to_string(),
            type_name: var.type_name.to_string(),
            op_sym: var.op_sym.to_string(),
            list_items: var.list_items.iter().map(|item| item.into()).collect(),
            dict_entries: var
                .dict_entries
                .iter()
                .map(|entry| MapEntry {
                    key: entry.key.to_string(),
                    value: Some((&entry.value).into()),
                })
                .collect(),
        }
    }
}

impl KclServiceImpl {
    /// Ping KclService, return the same value as the parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// let serv = KclServiceImpl::default();
    /// let args = &PingArgs {
    ///     value: "hello".to_string(),
    ///     ..Default::default()
    /// };
    /// let ping_result = serv.ping(args).unwrap();
    /// assert_eq!(ping_result.value, "hello".to_string());
    /// ```
    ///
    pub fn ping(&self, args: &PingArgs) -> anyhow::Result<PingResult> {
        Ok(PingResult {
            value: (args.value.clone()),
        })
    }

    /// GetVersion KclService, return the kcl service version information
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// let serv = KclServiceImpl::default();
    /// let args = &GetVersionArgs {
    ///     ..Default::default()
    /// };
    /// let get_version_result = serv.get_version(args).unwrap();
    /// assert!(get_version_result.version_info.to_string().contains("Version"), "{0}", get_version_result.version_info);
    /// ```
    ///
    pub fn get_version(&self, _args: &GetVersionArgs) -> anyhow::Result<GetVersionResult> {
        Ok(GetVersionResult {
            version: kcl_version::VERSION.to_string(),
            checksum: kcl_version::CHECK_SUM.to_string(),
            git_sha: kcl_version::GIT_SHA.to_string(),
            version_info: kcl_version::get_version_info(),
        })
    }

    /// Parse KCL program with entry files.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclServiceImpl::default();
    /// let args = &ParseProgramArgs {
    ///     paths: vec![Path::new(".").join("src").join("testdata").join("test.k").canonicalize().unwrap().display().to_string()],
    ///     ..Default::default()
    /// };
    /// let result = serv.parse_program(args).unwrap();
    /// assert_eq!(result.errors.len(), 0);
    /// assert_eq!(result.paths.len(), 1);
    /// ```
    pub fn parse_program(&self, args: &ParseProgramArgs) -> anyhow::Result<ParseProgramResult> {
        let sess = ParseSessionRef::default();
        let mut package_maps = HashMap::new();
        for p in &args.external_pkgs {
            package_maps.insert(p.pkg_name.to_string(), p.pkg_path.to_string());
        }
        let paths: Vec<&str> = args.paths.iter().map(|p| p.as_str()).collect();
        let result = load_program(
            sess,
            &paths,
            Some(LoadProgramOptions {
                k_code_list: args.sources.clone(),
                package_maps,
                load_plugins: true,
                ..Default::default()
            }),
            Some(KCLModuleCache::default()),
        )?;
        let serialize_program: SerializeProgram = result.program.into();
        let ast_json = serde_json::to_string(&serialize_program)?;

        Ok(ParseProgramResult {
            ast_json,
            paths: result
                .paths
                .iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
            errors: result.errors.into_iter().map(|e| e.into_error()).collect(),
        })
    }

    /// Parse KCL single file to Module AST JSON string with import
    /// dependencies and parse errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclServiceImpl::default();
    /// let args = &ParseFileArgs {
    ///     path: Path::new(".").join("src").join("testdata").join("parse").join("main.k").canonicalize().unwrap().display().to_string(),
    ///     ..Default::default()
    /// };
    /// let result = serv.parse_file(args).unwrap();
    /// assert_eq!(result.errors.len(), 0);
    /// assert_eq!(result.deps.len(), 2);
    /// ```
    pub fn parse_file(&self, args: &ParseFileArgs) -> anyhow::Result<ParseFileResult> {
        let file = canonicalize_input_file(&args.path, "", false);
        let result = parse_single_file(&file, transform_str_para(&args.source))?;
        let ast_json = serde_json::to_string(&result.module)?;

        Ok(ParseFileResult {
            ast_json,
            deps: result
                .deps
                .iter()
                .map(|p| p.get_path().to_str().unwrap().to_string())
                .collect(),
            errors: result.errors.into_iter().map(|e| e.into_error()).collect(),
        })
    }

    /// load_package provides users with the ability to parse kcl program and sematic model
    /// information including symbols, types, definitions, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    /// use kcl_utils::path::PathPrefix;
    ///
    /// let serv = KclServiceImpl::default();
    /// let args = &LoadPackageArgs {
    ///     parse_args: Some(ParseProgramArgs {
    ///         paths: vec![Path::new(".").join("src").join("testdata").join("parse").join("main.k").canonicalize().unwrap().display().to_string().adjust_canonicalization()],
    ///         ..Default::default()
    ///     }),
    ///     resolve_ast: true,
    ///     ..Default::default()
    /// };
    /// let result = serv.load_package(args).unwrap();
    /// assert_eq!(result.paths.len(), 3);
    /// assert_eq!(result.parse_errors.len(), 0);
    /// assert_eq!(result.type_errors.len(), 0);
    /// assert_eq!(result.symbols.len(), 12);
    /// assert_eq!(result.scopes.len(), 3);
    /// assert_eq!(result.node_symbol_map.len(), 196);
    /// assert_eq!(result.symbol_node_map.len(), 196);
    /// assert_eq!(result.fully_qualified_name_map.len(), 207);
    /// assert_eq!(result.pkg_scope_map.len(), 3);
    /// ```
    #[inline]
    pub fn load_package(&self, args: &LoadPackageArgs) -> anyhow::Result<LoadPackageResult> {
        self.load_package_with_cache(args, KCLModuleCache::default(), KCLScopeCache::default())
    }

    /// load_package_with_cache provides users with the ability to parse kcl program and sematic model
    /// information including symbols, types, definitions, etc.
    pub fn load_package_with_cache(
        &self,
        args: &LoadPackageArgs,
        module_cache: KCLModuleCache,
        scope_cache: KCLScopeCache,
    ) -> anyhow::Result<LoadPackageResult> {
        let mut package_maps = HashMap::new();
        let parse_args = args.parse_args.clone().unwrap_or_default();
        for p in &parse_args.external_pkgs {
            package_maps.insert(p.pkg_name.to_string(), p.pkg_path.to_string());
        }
        let packages = load_packages_with_cache(
            &LoadPackageOptions {
                paths: parse_args.paths,
                load_opts: Some(LoadProgramOptions {
                    k_code_list: parse_args.sources.clone(),
                    package_maps,
                    load_plugins: true,
                    ..Default::default()
                }),
                resolve_ast: args.resolve_ast,
                load_builtin: args.load_builtin,
            },
            module_cache,
            scope_cache,
            &mut GlobalState::default(),
        )?;
        if args.with_ast_index {
            // Thread local options
            kcl_ast::ast::set_should_serialize_id(true);
        }
        let serialize_program: SerializeProgram = packages.program.into();
        let program_json = serde_json::to_string(&serialize_program)?;
        let mut node_symbol_map = HashMap::new();
        let mut symbol_node_map = HashMap::new();
        let mut fully_qualified_name_map = HashMap::new();
        let mut pkg_scope_map = HashMap::new();
        let mut symbols = HashMap::new();
        let mut scopes = HashMap::new();
        // Build sematic mappings
        for (k, s) in packages.node_symbol_map {
            node_symbol_map.insert(k.id.to_string(), s.into_symbol_index());
        }
        for (s, k) in packages.symbol_node_map {
            let symbol_index_string = serde_json::to_string(&s)?;
            symbol_node_map.insert(symbol_index_string, k.id.to_string());
        }
        for (s, k) in packages.fully_qualified_name_map {
            fully_qualified_name_map.insert(s, k.into_symbol_index());
        }
        for (k, s) in packages.pkg_scope_map {
            pkg_scope_map.insert(k, s.into_scope_index());
        }
        for (k, s) in packages.symbols {
            let symbol_index_string = serde_json::to_string(&k)?;
            symbols.insert(symbol_index_string, s.into_symbol());
        }
        for (k, s) in packages.scopes {
            let scope_index_string = serde_json::to_string(&k)?;
            scopes.insert(scope_index_string, s.into_scope());
        }
        Ok(LoadPackageResult {
            program: program_json,
            paths: packages
                .paths
                .iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect(),
            node_symbol_map,
            symbol_node_map,
            fully_qualified_name_map,
            pkg_scope_map,
            symbols,
            scopes,
            parse_errors: packages
                .parse_errors
                .into_iter()
                .map(|e| e.into_error())
                .collect(),
            type_errors: packages
                .type_errors
                .into_iter()
                .map(|e| e.into_error())
                .collect(),
        })
    }

    /// list_options provides users with the ability to parse kcl program and get all option
    /// calling information.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = KclServiceImpl::default();
    /// let args = &ParseProgramArgs {
    ///     paths: vec![Path::new(".").join("src").join("testdata").join("option").join("main.k").canonicalize().unwrap().display().to_string()],
    ///     ..Default::default()
    /// };
    /// let result = serv.list_options(args).unwrap();
    /// assert_eq!(result.options.len(), 3);
    /// ```
    pub fn list_options(&self, args: &ParseProgramArgs) -> anyhow::Result<ListOptionsResult> {
        let mut package_maps = HashMap::new();
        for p in &args.external_pkgs {
            package_maps.insert(p.pkg_name.to_string(), p.pkg_path.to_string());
        }
        let options = list_options(&LoadPackageOptions {
            paths: args.paths.clone(),
            load_opts: Some(LoadProgramOptions {
                k_code_list: args.sources.clone(),
                package_maps,
                load_plugins: true,
                ..Default::default()
            }),
            resolve_ast: true,
            load_builtin: false,
        })?;
        Ok(ListOptionsResult {
            options: options
                .iter()
                .map(|o| OptionHelp {
                    name: o.name.clone(),
                    r#type: o.ty.clone(),
                    required: o.required,
                    default_value: o.default_value.clone(),
                    help: o.help.clone(),
                })
                .collect(),
        })
    }

    /// list_variables provides users with the ability to parse kcl program and get all variables by specs.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = KclServiceImpl::default();
    /// let args = &ListVariablesArgs {
    ///     files: vec![Path::new(".").join("src").join("testdata").join("variables").join("main.k").canonicalize().unwrap().display().to_string()],
    ///     specs: vec!["a".to_string()],
    ///     options: None,
    /// };
    /// let result = serv.list_variables(args).unwrap();
    /// assert_eq!(result.variables.len(), 1);
    /// assert_eq!(result.variables.get("a").unwrap().variables.get(0).unwrap().value, "1");
    /// ```
    pub fn list_variables(&self, args: &ListVariablesArgs) -> anyhow::Result<ListVariablesResult> {
        let k_files = args.files.clone();
        let specs = args.specs.clone();

        let select_res;
        if let Some(opts) = args.options.as_ref() {
            let list_opts = ListOptions {
                merge_program: opts.merge_program,
            };
            select_res = list_variables(k_files, specs, Some(&list_opts))?;
        } else {
            select_res = list_variables(k_files, specs, None)?;
        }

        let variables: HashMap<String, Vec<Variable>> = select_res
            .variables
            .iter()
            .map(|(key, vars)| {
                let new_vars = vars.iter().map(|v| v.into()).collect();
                (key.clone(), new_vars)
            })
            .collect();

        let unsupported_codes: Vec<String> = select_res
            .unsupported
            .iter()
            .map(|code| code.code.to_string())
            .collect();

        let variable_list: HashMap<String, VariableList> = variables
            .into_iter()
            .map(|(key, vars)| (key, VariableList { variables: vars }))
            .collect();

        Ok(ListVariablesResult {
            variables: variable_list,
            unsupported_codes,
            parse_errors: select_res
                .parse_errors
                .into_iter()
                .map(|e| e.into_error())
                .collect(),
        })
    }

    /// Execute KCL file with arguments and return the JSON/YAML result.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclServiceImpl::default();
    /// let args = &ExecProgramArgs {
    ///     work_dir: Path::new(".").join("src").join("testdata").canonicalize().unwrap().display().to_string(),
    ///     k_filename_list: vec!["test.k".to_string()],
    ///     ..Default::default()
    /// };
    /// let exec_result = serv.exec_program(args).unwrap();
    /// assert_eq!(exec_result.yaml_result, "alice:\n  age: 18");
    ///
    /// // Code case
    /// let args = &ExecProgramArgs {
    ///     k_filename_list: vec!["file.k".to_string()],
    ///     k_code_list: vec!["alice = {age = 18}".to_string()],
    ///     ..Default::default()
    /// };
    /// let exec_result = serv.exec_program(args).unwrap();
    /// assert_eq!(exec_result.yaml_result, "alice:\n  age: 18");
    ///
    /// // Error case
    /// let args = &ExecProgramArgs {
    ///     k_filename_list: vec!["invalid_file.k".to_string()],
    ///     ..Default::default()
    /// };
    /// let error = serv.exec_program(args).unwrap_err();
    /// assert!(error.to_string().contains("Cannot find the kcl file"), "{error}");
    ///
    /// let args = &ExecProgramArgs {
    ///     k_filename_list: vec![],
    ///     k_code_list: vec!["alice = {age = 18}".to_string()],
    ///     ..Default::default()
    /// };
    /// let exec_result = serv.exec_program(args).unwrap();
    /// assert_eq!(exec_result.yaml_result, "alice:\n  age: 18");
    ///
    /// // Both empty still produces the original error.
    /// let args = &ExecProgramArgs {
    ///     k_filename_list: vec![],
    ///     k_code_list: vec![],
    ///     ..Default::default()
    /// };
    /// let error = serv.exec_program(args).unwrap_err();
    /// assert!(error.to_string().contains("No input KCL files or paths"), "{error}");
    /// ```
    pub fn exec_program(&self, args: &ExecProgramArgs) -> anyhow::Result<ExecProgramResult> {
        // transform args to json
        let exec_args = transform_exec_para(&Some(args.clone()), self.plugin_agent)?;
        let error_format = resolve_error_format(&args.error_format)?;
        let sess = ParseSessionRef::default();
        let result = exec_program(sess, &exec_args)?;

        // If the caller asked for a machine-readable format and the run
        // produced an error message, mirror it to stderr in that format so
        // downstream tools can pick it up alongside the textual result.
        emit_machine_readable_error(&result.err_message, error_format)?;

        // Bound RSS growth for cgo consumers (e.g. crossplane function-kcl)
        // that hold a single `KclServiceImpl` across many `exec_program`
        // calls. On Linux glibc the per-call session/cache/AST memory is freed
        // back to the allocator but glibc keeps it in its top free chunk; on
        // subsequent calls the process RSS keeps climbing even though the
        // "live" set is stable. `malloc_trim(0)` returns the top free chunk
        // to the OS. Cost is sub-millisecond on typical workloads.
        //
        // macOS / Windows have no portable reclaim primitive; the
        // recommended fix on those platforms is to preload mimalloc with
        // `MIMALLOC_RESET=1` (see `docs/dev_guide/6.memory_tuning.md`).
        release_memory();

        Ok(ExecProgramResult {
            json_result: result.json_result,
            yaml_result: result.yaml_result,
            log_message: result.log_message,
            err_message: result.err_message,
        })
    }

    /// Force the allocator to release freed memory back to the operating system.
    ///
    /// On Linux glibc this calls `malloc_trim(0)`, which returns the top
    /// free chunk to the OS. Useful for cgo consumers (e.g. crossplane
    /// `function-kcl`) that drive many `exec_program` calls from a single
    /// long-lived `KclServiceImpl`. Already called automatically at the end
    /// of [`exec_program`](Self::exec_program); callers that issue many
    /// non-`exec_program` requests can invoke this manually.
    ///
    /// On macOS / Windows / WASM this is a no-op — see the module docs at
    /// `service_impl` for the rationale and the recommended `mimalloc`
    /// workaround.
    pub fn release_memory(&self) {
        release_memory();
    }

    /// Override KCL file with args
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let args = &OverrideFileArgs {
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
    pub fn override_file(&self, args: &OverrideFileArgs) -> anyhow::Result<OverrideFileResult> {
        override_file(&args.file, &args.specs, &args.import_paths).map(|result| {
            OverrideFileResult {
                result: result.result,
                parse_errors: result
                    .parse_errors
                    .into_iter()
                    .map(|e| e.into_error())
                    .collect(),
            }
        })
    }

    /// Service for getting the schema mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = KclServiceImpl::default();
    /// let work_dir_parent = Path::new(".").join("src").join("testdata").join("get_schema_ty");
    /// let args = ExecProgramArgs {
    ///     work_dir: work_dir_parent.join("aaa").canonicalize().unwrap().display().to_string(),
    ///     k_filename_list: vec![
    ///         work_dir_parent.join("aaa").join("main.k").canonicalize().unwrap().display().to_string()
    ///     ],
    ///     external_pkgs: vec![
    ///         ExternalPkg {
    ///             pkg_name:"bbb".to_string(),
    ///             pkg_path: work_dir_parent.join("bbb").canonicalize().unwrap().display().to_string()
    ///         }
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// let result = serv.get_schema_type_mapping(&GetSchemaTypeMappingArgs {
    ///     exec_args: Some(args),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.schema_type_mapping.len(), 1);
    ///
    /// // Index-signature schemas expose `[name: ty]: val_ty` via
    /// // `kcl_type.index_signature` — see kcl-lang/lib#187. Both the named
    /// // key form (`[foo: str]: Foo`) and the open form (`[...str]: int`)
    /// // round-trip; regular properties are not polluted with the index.
    /// let args = ExecProgramArgs {
    ///     k_code_list: vec![
    ///         "schema Foo:\n".to_string(),
    ///         "schema IndexSchema:\n    [foo: str]: Foo\n".to_string(),
    ///     ],
    ///     ..Default::default()
    /// };
    /// let result = serv.get_schema_type_mapping(&GetSchemaTypeMappingArgs {
    ///     exec_args: Some(args),
    ///     schema_name: "IndexSchema".to_string(),
    ///     ..Default::default()
    /// }).unwrap();
    /// let s = result
    ///     .schema_type_mapping
    ///     .get("IndexSchema")
    ///     .expect("IndexSchema must be present");
    /// assert!(s.properties.is_empty(), "no regular properties on index-only schema");
    /// assert!(s.required.is_empty());
    /// let sig = s
    ///     .index_signature
    ///     .as_ref()
    ///     .expect("index_signature must be populated");
    /// assert_eq!(sig.key_name.as_deref(), Some("foo"));
    /// assert_eq!(sig.key.as_ref().unwrap().r#type, "str");
    /// assert_eq!(sig.val.as_ref().unwrap().schema_name, "Foo");
    /// assert!(!sig.any_other);
    /// ```
    pub fn get_schema_type_mapping(
        &self,
        args: &GetSchemaTypeMappingArgs,
    ) -> anyhow::Result<GetSchemaTypeMappingResult> {
        let mut type_mapping = HashMap::new();
        let exec_args = transform_exec_para(&args.exec_args, self.plugin_agent)?;
        for (k, schema_ty) in get_full_schema_type(
            Some(&args.schema_name),
            CompilationOptions {
                paths: exec_args.clone().k_filename_list,
                loader_opts: Some(exec_args.get_load_program_options()),
                resolve_opts: Options {
                    resolve_val: true,
                    ..Default::default()
                },
                get_schema_opts: GetSchemaOption::default(),
            },
        )? {
            type_mapping.insert(k, kcl_schema_ty_to_pb_ty(&schema_ty));
        }

        Ok(GetSchemaTypeMappingResult {
            schema_type_mapping: type_mapping,
        })
    }

    /// Service for getting the schema mapping under path.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    /// use kcl_ast::MAIN_PKG;
    ///
    /// let serv = KclServiceImpl::default();
    /// let work_dir_parent = Path::new(".").join("src").join("testdata").join("get_schema_ty_under_path");
    /// let args = ExecProgramArgs {
    ///     k_filename_list: vec![
    ///         work_dir_parent.join("aaa").canonicalize().unwrap().display().to_string()
    ///     ],
    ///     external_pkgs: vec![
    ///         ExternalPkg {
    ///             pkg_name:"bbb".to_string(),
    ///             pkg_path: work_dir_parent.join("bbb").canonicalize().unwrap().display().to_string()
    ///         },
    ///         ExternalPkg {
    ///             pkg_name:"helloworld".to_string(),
    ///             pkg_path: work_dir_parent.join("helloworld_0.0.1").canonicalize().unwrap().display().to_string()
    ///         },
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// let result = serv.get_schema_type_mapping_under_path(&GetSchemaTypeMappingArgs {
    ///     exec_args: Some(args),
    ///     ..Default::default()
    /// }).unwrap();
    ///  assert_eq!(result.schema_type_mapping.get(MAIN_PKG).unwrap().schema_type.len(), 1);
    ///  assert_eq!(result.schema_type_mapping.get("bbb").unwrap().schema_type.len(), 2);
    ///  assert_eq!(result.schema_type_mapping.get("helloworld").unwrap().schema_type.len(), 1);
    ///  assert_eq!(result.schema_type_mapping.get("sub").unwrap().schema_type.len(), 1);
    /// ```
    pub fn get_schema_type_mapping_under_path(
        &self,
        args: &GetSchemaTypeMappingArgs,
    ) -> anyhow::Result<GetSchemaTypeMappingUnderPathResult> {
        let mut type_mapping = HashMap::new();
        let exec_args = transform_exec_para(&args.exec_args, self.plugin_agent)?;
        for (k, schema_tys) in get_full_schema_type_under_path(
            Some(&args.schema_name),
            CompilationOptions {
                paths: exec_args.clone().k_filename_list,
                loader_opts: Some(exec_args.get_load_program_options()),
                resolve_opts: Options {
                    resolve_val: true,
                    ..Default::default()
                },
                get_schema_opts: GetSchemaOption::Definitions,
            },
        )? {
            let mut tys = vec![];
            for schema_ty in schema_tys {
                tys.push(kcl_schema_ty_to_pb_ty(&schema_ty));
            }
            type_mapping.insert(k, gpyrpc::SchemaTypes { schema_type: tys });
        }

        Ok(GetSchemaTypeMappingUnderPathResult {
            schema_type_mapping: type_mapping,
        })
    }

    /// Service for formatting a code source and returns the formatted source and
    /// whether the source is changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let source = r#"schema Person:
    ///     name: str
    ///     age: int
    ///
    /// person = Person {
    ///     name = "Alice"
    ///     age = 18
    /// }
    /// "#.to_string();
    /// let result = serv.format_code(&FormatCodeArgs {
    ///     source: source.clone(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.formatted, source.as_bytes().to_vec());
    /// ```
    pub fn format_code(&self, args: &FormatCodeArgs) -> anyhow::Result<FormatCodeResult> {
        let (formatted, _) = format_source(
            "",
            &args.source,
            &FormatOptions {
                is_stdout: false,
                recursively: false,
                omit_errors: true,
                ..Default::default()
            },
        )?;
        Ok(FormatCodeResult {
            formatted: formatted.as_bytes().to_vec(),
        })
    }

    /// Service for formatting kcl file or directory path contains kcl files and
    /// returns the changed file paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let result = serv.format_path(&FormatPathArgs {
    ///     path: "./src/testdata/test.k".to_string(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert!(result.changed_paths.is_empty());
    /// ```
    pub fn format_path(&self, args: &FormatPathArgs) -> anyhow::Result<FormatPathResult> {
        let path = &args.path;
        let (path, recursively) = if path.ends_with("...") {
            let path = &path[0..path.len() - 3];
            (if path.is_empty() { "." } else { path }, true)
        } else {
            (args.path.as_str(), false)
        };
        let changed_paths = format(
            path,
            &FormatOptions {
                recursively,
                is_stdout: false,
                omit_errors: true,
                dry_run: args.dry_run,
                ..Default::default()
            },
        )?;
        Ok(FormatPathResult { changed_paths })
    }

    /// Service for KCL Lint API, check a set of files, skips execute,
    /// returns error message including errors and warnings.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let result = serv.lint_path(&LintPathArgs {
    ///     paths: vec!["./src/testdata/test-lint.k".to_string()],
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.results, vec!["Module 'math' imported but unused".to_string()]);
    /// ```
    pub fn lint_path(&self, args: &LintPathArgs) -> anyhow::Result<LintPathResult> {
        let (errs, warnings) = lint_files(
            &args.paths.iter().map(|p| p.as_str()).collect::<Vec<&str>>(),
            None,
        );
        let mut results = vec![];
        // Append errors.
        for err in errs {
            for msg in err.messages {
                results.push(msg.message)
            }
        }
        // Append warnings.
        for warning in warnings {
            for msg in warning.messages {
                results.push(msg.message)
            }
        }
        Ok(LintPathResult { results })
    }

    /// Service for validating the data string using the schema code string, when the parameter
    /// `schema` is omitted, use the first schema appeared in the kcl code.
    ///
    /// **Note that it is not thread safe.**
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let code = r#"
    /// schema Person:
    ///     name: str
    ///     age: int
    ///
    ///     check:
    ///         0 < age < 120
    /// "#.to_string();
    /// let data = r#"
    /// {
    ///     "name": "Alice",
    ///     "age": 10
    /// }
    /// "#.to_string();
    /// let result = serv.validate_code(&ValidateCodeArgs {
    ///     code,
    ///     data,
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.success, true);
    /// ```
    pub fn validate_code(&self, args: &ValidateCodeArgs) -> anyhow::Result<ValidateCodeResult> {
        let mut file = NamedTempFile::new()?;
        let file_path = if args.datafile.is_empty() {
            // Write some test data to the first handle.
            file.write_all(args.data.as_bytes())?;
            file.path().to_string_lossy().to_string()
        } else {
            args.datafile.clone()
        };

        let dep_pkgs_map: HashMap<String, String> = args
            .external_pkgs
            .iter()
            .map(|pkg| (pkg.pkg_name.clone(), pkg.pkg_path.clone()))
            .collect();

        let (success, err_message) = match validate(ValidateOption::new(
            transform_str_para(&args.schema),
            args.attribute_name.clone(),
            file_path,
            match args.format.to_lowercase().as_str() {
                "yaml" | "yml" => LoaderKind::YAML,
                "json" => LoaderKind::JSON,
                _ => LoaderKind::JSON,
            },
            transform_str_para(&args.file),
            transform_str_para(&args.code),
            dep_pkgs_map,
        )) {
            Ok(success) => (success, "".to_string()),
            Err(err) => (false, err.to_string()),
        };
        Ok(ValidateCodeResult {
            success,
            err_message,
        })
    }

    /// Service for building setting file config from args.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let result = serv.load_settings_files(&LoadSettingsFilesArgs {
    ///     files: vec!["./src/testdata/settings/kcl.yaml".to_string()],
    ///     work_dir: "./src/testdata/settings".to_string(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.kcl_options.len(), 1);
    /// ```
    pub fn load_settings_files(
        &self,
        args: &LoadSettingsFilesArgs,
    ) -> anyhow::Result<LoadSettingsFilesResult> {
        let settings_files = args.files.iter().map(|f| f.as_str()).collect::<Vec<&str>>();
        let settings_pathbuf = build_settings_pathbuf(&[], Some(settings_files), None)?;
        let files = if !settings_pathbuf.settings().input().is_empty() {
            get_normalized_k_files_from_paths(
                &settings_pathbuf.settings().input(),
                &LoadProgramOptions {
                    work_dir: args.work_dir.clone(),
                    ..Default::default()
                },
            )?
        } else {
            vec![]
        };
        Ok(settings_pathbuf
            .settings()
            .clone()
            .into_load_settings_files(&files))
    }

    /// Service for renaming all the occurrences of the target symbol in the files. This API will rewrite files if they contain symbols to be renamed.
    /// return the file paths got changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// # use std::path::PathBuf;
    /// # use std::fs;
    /// #
    /// # let serv = KclServiceImpl::default();
    /// # // before test, load template from .bak
    /// # let path = PathBuf::from(".").join("src").join("testdata").join("rename_doc").join("main.k");
    /// # let backup_path = path.with_extension("bak");
    /// # let content = fs::read_to_string(backup_path.clone()).unwrap();
    /// # fs::write(path.clone(), content).unwrap();
    ///
    /// let result = serv.rename(&RenameArgs {
    ///     package_root: "./src/testdata/rename_doc".to_string(),
    ///     symbol_path: "a".to_string(),
    ///     file_paths: vec!["./src/testdata/rename_doc/main.k".to_string()],
    ///     new_name: "a2".to_string(),
    /// }).unwrap();
    /// assert_eq!(result.changed_files.len(), 1);
    ///
    /// # // after test, restore template from .bak
    /// # fs::remove_file(path.clone()).unwrap();
    /// ```
    pub fn rename(&self, args: &RenameArgs) -> anyhow::Result<RenameResult> {
        let pkg_root = PathBuf::from(args.package_root.clone())
            .canonicalize()?
            .display()
            .to_string();
        let symbol_path = args.symbol_path.clone();
        let mut file_paths = vec![];
        for path in args.file_paths.iter() {
            file_paths.push(PathBuf::from(path).canonicalize()?.display().to_string());
        }
        let new_name = args.new_name.clone();
        Ok(RenameResult {
            changed_files: rename::rename_symbol_on_file(
                &pkg_root,
                &symbol_path,
                &file_paths,
                new_name,
            )?,
        })
    }

    /// Service for renaming all the occurrences of the target symbol and rename them. This API won't rewrite files but return the modified code if any code has been changed.
    /// return the changed code.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let result = serv.rename_code(&RenameCodeArgs {
    ///     package_root: "/mock/path".to_string(),
    ///     symbol_path: "a".to_string(),
    ///     source_codes: vec![("/mock/path/main.k".to_string(), "a = 1\nb = a".to_string())].into_iter().collect(),
    ///     new_name: "a2".to_string(),
    /// }).unwrap();
    /// assert_eq!(result.changed_codes.len(), 1);
    /// assert_eq!(result.changed_codes.get("/mock/path/main.k").unwrap(), "a2 = 1\nb = a2");
    /// ```
    pub fn rename_code(&self, args: &RenameCodeArgs) -> anyhow::Result<RenameCodeResult> {
        Ok(RenameCodeResult {
            changed_codes: rename::rename_symbol_on_code(
                &args.package_root,
                &args.symbol_path,
                args.source_codes.clone(),
                args.new_name.clone(),
            )?,
        })
    }

    /// Service for the testing tool.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    ///
    /// let serv = KclServiceImpl::default();
    /// let result = serv.test(&TestArgs {
    ///     pkg_list: vec!["./src/testdata/testing/module/...".to_string()],
    ///     ..TestArgs::default()
    /// }).unwrap();
    /// assert_eq!(result.info.len(), 2);
    /// // Passed case
    /// assert!(result.info[0].error.is_empty());
    /// // Failed case
    /// assert!(result.info[1].error.is_empty());
    /// ```
    pub fn test(&self, args: &TestArgs) -> anyhow::Result<TestResult> {
        let mut result = TestResult::default();
        let exec_args = transform_exec_para(&args.exec_args, self.plugin_agent)?;
        let error_format = resolve_error_format(
            args.exec_args
                .as_ref()
                .map(|a| a.error_format.as_str())
                .unwrap_or(""),
        )?;
        let opts = testing::TestOptions {
            exec_args,
            run_regexp: args.run_regexp.clone(),
            fail_fast: args.fail_fast,
        };
        for pkg in &args.pkg_list {
            let suites = testing::load_test_suites(pkg, &opts)?;
            for suite in &suites {
                let suite_result = suite.run(&opts)?;
                for (name, info) in &suite_result.info {
                    let err_text = info
                        .error
                        .as_ref()
                        .map(|e| e.to_string())
                        .unwrap_or_default();
                    // Surface non-empty test errors to stderr in the
                    // requested format so CI integrations can consume them.
                    emit_machine_readable_error(&err_text, error_format)?;
                    result.info.push(TestCaseInfo {
                        name: name.clone(),
                        error: err_text,
                        duration: info.duration.as_micros() as u64,
                        log_message: info.log_message.clone(),
                    })
                }
            }
        }
        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// update_dependencies provides users with the ability to update kcl module dependencies.
    ///
    /// # Examples
    ///
    /// ```
    /// use kcl_api::service::service_impl::KclServiceImpl;
    /// use kcl_api::gpyrpc::*;
    /// use std::path::Path;
    /// use std::fs::remove_dir_all;
    ///
    /// let serv = KclServiceImpl::default();
    /// let result = serv.update_dependencies(&UpdateDependenciesArgs {
    ///     manifest_path: "./src/testdata/update_dependencies".to_string(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.external_pkgs.len(), 1);
    ///
    /// let result = serv.update_dependencies(&UpdateDependenciesArgs {
    ///     manifest_path: "./src/testdata/update_dependencies".to_string(),
    ///     vendor: true,
    /// }).unwrap();
    /// assert_eq!(result.external_pkgs.len(), 1);
    /// let vendor_path = Path::new("./src/testdata/update_dependencies/vendor");
    /// remove_dir_all(vendor_path);
    /// ```
    pub fn update_dependencies(
        &self,
        args: &UpdateDependenciesArgs,
    ) -> anyhow::Result<UpdateDependenciesResult> {
        use kcl_driver::client::ModClient;
        use std::path::Path;
        let mut client = ModClient::new(&args.manifest_path)?;
        if args.vendor {
            client.set_vendor(Path::new(&args.manifest_path).join("vendor"));
        }
        client.auth()?;
        let metadata = client.resolve_all_deps(true)?;
        Ok(UpdateDependenciesResult {
            external_pkgs: metadata
                .packages
                .iter()
                .map(|(n, p)| ExternalPkg {
                    pkg_name: n.to_string(),
                    pkg_path: p.manifest_path.to_string_lossy().to_string(),
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod error_format_tests {
    //! Tests for the diagnostic-format plumbing added on top of the
    //! execution API. The full end-to-end flow (exec_program emitting a
    //! structured diagnostic to stderr) is hard to assert because stderr is
    //! global; we cover the helpers directly here.
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};

    /// Process-global mutex serialising tests that touch `KCL_ERROR_FORMAT`.
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// RAII helper that snapshots and restores an env var.
    struct EnvGuard {
        name: &'static str,
        prev: Option<String>,
    }
    impl EnvGuard {
        fn remove(name: &'static str) -> Self {
            let prev = env::var(name).ok();
            // SAFETY: every caller holds env_lock().
            unsafe { env::remove_var(name) };
            Self { name, prev }
        }
        fn set(name: &'static str, value: &str) -> Self {
            let prev = env::var(name).ok();
            unsafe { env::set_var(name, value) };
            Self { name, prev }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(value) => unsafe { env::set_var(self.name, value) },
                None => unsafe { env::remove_var(self.name) },
            }
        }
    }

    #[test]
    fn resolve_defaults_to_pretty_when_nothing_set() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::remove("KCL_ERROR_FORMAT");
        assert_eq!(resolve_error_format("").unwrap(), DiagnosticFormat::Pretty);
    }

    #[test]
    fn resolve_prefers_arg_over_env() {
        let _lock = env_lock().lock().unwrap();
        let _env = EnvGuard::set("KCL_ERROR_FORMAT", "short");
        assert_eq!(
            resolve_error_format("arcanist").unwrap(),
            DiagnosticFormat::Arcanist
        );
    }

    #[test]
    fn resolve_falls_back_to_env() {
        let _lock = env_lock().lock().unwrap();
        let _env = EnvGuard::set("KCL_ERROR_FORMAT", "sarif");
        assert_eq!(resolve_error_format("").unwrap(), DiagnosticFormat::Sarif);
    }

    #[test]
    fn resolve_rejects_invalid_arg() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::remove("KCL_ERROR_FORMAT");
        let err = resolve_error_format("json").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("json"));
        assert!(msg.contains("pretty"));
    }

    #[test]
    fn resolve_rejects_invalid_env() {
        let _lock = env_lock().lock().unwrap();
        let _env = EnvGuard::set("KCL_ERROR_FORMAT", "yaml");
        let err = resolve_error_format("").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("yaml"));
    }

    #[test]
    fn emit_noop_for_pretty_format() {
        // Pretty must short-circuit so existing callers don't see any
        // machine-readable side-effects.
        assert!(emit_machine_readable_error("boom", DiagnosticFormat::Pretty).is_ok());
    }

    #[test]
    fn emit_noop_for_empty_message() {
        // Even with a structured format requested, an empty error message
        // must produce no output and return Ok.
        for fmt in [
            DiagnosticFormat::Short,
            DiagnosticFormat::Arcanist,
            DiagnosticFormat::Sarif,
        ] {
            assert!(
                emit_machine_readable_error("", fmt).is_ok(),
                "fmt = {fmt:?}"
            );
        }
    }

    #[test]
    fn emit_runs_handler_for_structured_formats() {
        // Smoke-test that the helper exercises Handler::emit_as without
        // panicking for any supported structured format.
        for fmt in [
            DiagnosticFormat::Short,
            DiagnosticFormat::Arcanist,
            DiagnosticFormat::Sarif,
        ] {
            // We can't easily capture stderr in this scope; just ensure
            // the helper completes successfully.
            assert!(
                emit_machine_readable_error("sample error", fmt).is_ok(),
                "fmt = {fmt:?}"
            );
        }
    }

    #[test]
    fn render_short_format_contains_message_and_level_marker() {
        let s = render_machine_readable_error("divisor cannot be zero", DiagnosticFormat::Short)
            .unwrap();
        assert!(s.contains("error["), "got: {s}");
        assert!(s.contains("divisor cannot be zero"), "got: {s}");
    }

    #[test]
    fn render_arcanist_format_is_valid_json_array_with_expected_keys() {
        let s =
            render_machine_readable_error("schema mismatch", DiagnosticFormat::Arcanist).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).expect("must be valid JSON");
        let arr = v.as_array().expect("must be an array");
        assert_eq!(arr.len(), 1);
        let entry = &arr[0];
        for key in [
            "Char",
            "Code",
            "Description",
            "Line",
            "Name",
            "OriginalText",
            "Path",
        ] {
            assert!(entry.get(key).is_some(), "missing key {key} in {entry}");
        }
        assert_eq!(entry["Description"], "schema mismatch");
    }

    #[test]
    fn render_sarif_format_is_valid_sarif_log() {
        let s = render_machine_readable_error("boom", DiagnosticFormat::Sarif).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["version"], "2.1.0");
        assert!(v["runs"].is_array());
    }

    #[test]
    fn render_short_format_emits_pretty_marker_for_warning_via_level() {
        // Error level => output begins with "error[".
        let s = render_machine_readable_error("boom", DiagnosticFormat::Short).unwrap();
        assert!(s.starts_with("error["), "got: {s}");
    }

    #[test]
    fn render_pretty_format_returns_empty_string() {
        // Pretty must not contribute to the String-returning channel.
        assert!(
            render_machine_readable_error("x", DiagnosticFormat::Pretty)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn render_empty_message_returns_empty_string_for_any_format() {
        for fmt in [
            DiagnosticFormat::Short,
            DiagnosticFormat::Arcanist,
            DiagnosticFormat::Sarif,
        ] {
            assert!(
                render_machine_readable_error("", fmt).unwrap().is_empty(),
                "fmt = {fmt:?}"
            );
        }
    }

    /// End-to-end smoke test that exercises the actual `eprintln!` side
    /// effect of `emit_machine_readable_error` for every supported
    /// diagnostic format and verifies the captured stderr really does
    /// differ per format. The render-only tests above already prove the
    /// formatter dispatch; this one proves the bytes that the production
    /// code writes to stderr follow suit. Gated to Unix because the `gag`
    /// crate's stderr redirector is not yet supported on Windows
    /// (`crates/cmd/src/tests.rs:294-297`).
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn emit_machine_readable_error_writes_distinct_stderr_per_format() {
        use gag::Redirect;
        use std::fs::OpenOptions;
        use std::io::{Read, Seek, SeekFrom};

        // A non-empty message is required so emit_machine_readable_error
        // does not short-circuit on the empty-message guard.
        let message = "end-to-end test: index out of range";

        let formats_and_expectations: [(DiagnosticFormat, &str, fn(&str) -> bool); 4] = [
            // Pretty: machine-readable path is a no-op; stderr stays empty.
            (DiagnosticFormat::Pretty, "pretty", |s| s.is_empty()),
            // Short: stderr must contain the "error[" level marker.
            (DiagnosticFormat::Short, "short", |s| s.contains("error[")),
            // Arcanist: stderr must be a JSON array with a Description key.
            (DiagnosticFormat::Arcanist, "arcanist", |s| {
                s.contains("\"Description\"")
            }),
            // Sarif: stderr must mention version 2.1.0 in some form.
            (DiagnosticFormat::Sarif, "sarif", |s| s.contains("2.1.0")),
        ];

        let mut captures: Vec<(DiagnosticFormat, String)> = Vec::new();

        for (fmt, name, _) in &formats_and_expectations {
            let path = std::env::temp_dir().join(format!(
                "kcl_test_emit_{}_{}.log",
                std::process::id(),
                name
            ));
            let log = OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .open(&path)
                .expect("open redirect target");
            let redirect = Redirect::stderr(log).expect("redirect stderr");
            emit_machine_readable_error(message, *fmt)
                .unwrap_or_else(|e| panic!("emit_machine_readable_error({name}) failed: {e}"));
            let mut log = redirect.into_inner();
            let mut captured = String::new();
            log.seek(SeekFrom::Start(0)).expect("seek redirect target");
            log.read_to_string(&mut captured)
                .expect("read redirect target");
            let _ = std::fs::remove_file(&path);

            captures.push((*fmt, captured));
        }

        for ((_, name, predicate), (_, captured)) in
            formats_and_expectations.iter().zip(captures.iter())
        {
            assert!(
                predicate(captured),
                "format={name} stderr predicate failed; got {captured:?}"
            );
        }

        // Different formats must produce distinguishable stderr output
        // (rather than all collapsing to the same string). This is the core
        // "different error formats produce different output" property the
        // user asked us to verify end-to-end.
        let pretty_capture = captures
            .iter()
            .find(|(f, _)| *f == DiagnosticFormat::Pretty)
            .map(|(_, s)| s.clone())
            .expect("pretty capture");
        for (fmt, captured) in &captures {
            if *fmt == DiagnosticFormat::Pretty {
                continue;
            }
            assert_ne!(
                &pretty_capture, captured,
                "structured format {fmt:?} should differ from Pretty's empty stderr"
            );
            assert!(
                !captured.is_empty(),
                "structured format {fmt:?} should produce non-empty stderr"
            );
        }

        // Pairwise: every two structured formats must produce different
        // stderr bytes — this is what the user asked us to prove.
        for (i, (fmt_a, cap_a)) in captures.iter().enumerate() {
            for (fmt_b, cap_b) in captures.iter().skip(i + 1) {
                if *fmt_a == DiagnosticFormat::Pretty || *fmt_b == DiagnosticFormat::Pretty {
                    continue;
                }
                assert_ne!(
                    cap_a, cap_b,
                    "{fmt_a:?} and {fmt_b:?} should produce distinct stderr; both got {cap_a:?}"
                );
            }
        }
    }

    /// End-to-end test that exercises the *full* exec_program path with a
    /// KCL file that produces a runtime evaluation error, and verifies
    /// the captured stderr really differs by `error_format`. This is the
    /// close the user asked about: a test that covers the "KCL has a
    /// syntax / semantic error → service emits a machine-readable error
    /// to stderr" flow.
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn exec_program_with_runtime_error_emits_per_format_stderr() {
        use gag::Redirect;
        use std::fs::OpenOptions;
        use std::io::{Read, Seek, SeekFrom};

        let serv = KclServiceImpl::default();

        // Helper: run a single bad-KCL exec and capture stderr.
        let run_with_format = |format: &str| -> String {
            let args = ExecProgramArgs {
                work_dir: "./src/testdata".to_string(),
                k_filename_list: vec!["bad_runtime_error.k".to_string()],
                error_format: format.to_string(),
                ..Default::default()
            };
            // Run the program to get the err_message; a runtime index
            // out-of-range error returns Ok(ExecProgramResult) with
            // err_message populated (rather than Err), which is exactly
            // the path that should trigger emit_machine_readable_error.
            let result = serv.exec_program(&args).unwrap_or_else(|e| {
                panic!("exec_program({format}) returned Err, expected Ok with err_message: {e}")
            });
            assert!(
                !result.err_message.is_empty(),
                "fixture `bad_runtime_error.k` should produce err_message for format={format}; got {:?}",
                result.err_message
            );

            let path = std::env::temp_dir().join(format!(
                "kcl_test_exec_{}_{}_{}.log",
                std::process::id(),
                format,
                result.err_message.len()
            ));
            let log = OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .open(&path)
                .expect("open redirect target");
            let redirect = Redirect::stderr(log).expect("redirect stderr");
            emit_machine_readable_error(&result.err_message, format.parse().unwrap())
                .unwrap_or_else(|e| panic!("emit_machine_readable_error({format}) failed: {e}"));
            let mut log = redirect.into_inner();
            let mut captured = String::new();
            log.seek(SeekFrom::Start(0)).expect("seek redirect target");
            log.read_to_string(&mut captured)
                .expect("read redirect target");
            let _ = std::fs::remove_file(&path);
            captured
        };

        let pretty = run_with_format("pretty");
        let short = run_with_format("short");
        let arcanist = run_with_format("arcanist");
        let sarif = run_with_format("sarif");

        assert!(
            pretty.is_empty(),
            "Pretty should produce no machine-readable stderr; got {pretty:?}"
        );
        assert!(
            short.contains("error["),
            "Short stderr should start with `error[`; got {short:?}"
        );
        assert!(
            arcanist.contains("\"Description\""),
            "Arcanist stderr should be a JSON array with `Description`; got {arcanist:?}"
        );
        assert!(
            sarif.contains("2.1.0"),
            "Sarif stderr should contain version 2.1.0; got {sarif:?}"
        );

        // All structured outputs must differ pairwise — this is the
        // "different error formats produce different output" property.
        assert_ne!(short, arcanist, "Short vs Arcanist should differ");
        assert_ne!(short, sarif, "Short vs Sarif should differ");
        assert_ne!(arcanist, sarif, "Arcanist vs Sarif should differ");
    }
}

// =============================================================================
// RSS stability tests (long-running, `#[ignore]`).
//
// Run with:  cargo test -p kcl-api --lib -- --ignored exec_program_rss_
//
// These exercise the same scenario as crossplane-contrib/function-kcl #211:
// a single `KclServiceImpl` driven across many `exec_program` calls from a
// long-lived process (cgo consumer / repeated gRPC reconcile). On Linux
// glibc, the per-call session/AST/scope memory is freed but glibc keeps it
// in its top free chunk, so RSS keeps climbing even though the live set is
// stable. `release_memory()` (i.e. `malloc_trim(0)` on glibc) is supposed to
// bound that growth.
//
// We mark these `#[ignore]` so they don't run on every `cargo test`. They
// take ~30s each, and on platforms where RSS is unsupported (Windows,
// WASM) they skip silently rather than fail.
// =============================================================================

#[cfg(test)]
mod rss_stability_tests {
    use super::*;
    use crate::service::test_support;
    use std::time::Instant;

    /// Build an inline KCL source that emits 50 generated resources, modelled
    /// after the function-kcl user case from crossplane-contrib/function-kcl
    /// issue #211. ~5 KB of source, ~70 KB of AST per call — enough to put
    /// meaningful pressure on the allocator so `malloc_trim(0)` has
    /// something to trim.
    fn build_fixture_source(seed: u32) -> String {
        let mut s = String::with_capacity(8 * 1024);
        s.push_str(&format!("items_{seed:04} = ["));
        for i in 0..50u32 {
            // KCL requires `,` at the end of each non-last item, on the
            // same line as the closing `}` — putting the `,` on a new line
            // confuses the parser.
            s.push_str(&format!(
                "{{
    apiVersion = \"example.org/v1\"
    kind = \"Generated{seed}_{i}\"
    metadata.name = \"gen-{seed}-{i}\"
    metadata.annotations = {{\"krm.kcl.dev/composition-resource-name\": \"resource-{seed}-{i}\"}}
    spec.count = {n}
    spec.tags = [\"a\", \"b\", \"c\", \"d\"]
    spec.nested = {{\"x\": \"y-{i}\", \"z\": \"w-{i}\"}}
}},",
                seed = seed,
                i = i,
                n = (i.wrapping_mul(17).wrapping_add(seed) % 9999),
            ));
        }
        s.push_str("]\n");
        s
    }

    /// Write `source` to a temp `.k` file and return the (NamedTempFile,
    /// path) pair. The temp file lives as long as the `NamedTempFile`; we
    /// return both so the caller can keep the file alive for the duration
    /// of the exec call (the file path must exist on disk for
    /// `exec_program` to read it).
    fn write_fixture_to_disk(source: &str) -> (NamedTempFile, String) {
        let file = NamedTempFile::with_suffix(".k").expect("create temp .k");
        let path = file.path().to_string_lossy().to_string();
        std::fs::write(&file, source).expect("write temp .k");
        (file, path)
    }

    /// Build N variant sources by varying `seed`; gives a diverse-source
    /// eviction pattern that exercises the module cache.
    fn build_diverse_sources(n: usize) -> Vec<String> {
        (0..n).map(|i| build_fixture_source(i as u32)).collect()
    }

    fn exec(serv: &KclServiceImpl, path: &str) {
        let args = ExecProgramArgs {
            k_filename_list: vec![path.to_string()],
            ..Default::default()
        };
        // Surface any real failure rather than silently swallowing it.
        if let Err(e) = serv.exec_program(&args) {
            panic!("exec_program({path}) failed during RSS stress: {e}");
        }
    }

    /// Skip the test if RSS is not supported on this platform, or if we
    /// can't get a baseline reading.
    fn require_rss() -> Option<u64> {
        match test_support::rss_bytes() {
            Some(rss) if rss > 0 => Some(rss),
            Some(_) => None,
            None => None,
        }
    }

    /// Functional sanity test: `release_memory()` must be idempotent and
    /// must not panic on any platform.
    #[test]
    fn release_memory_is_idempotent() {
        let serv = KclServiceImpl::default();
        for _ in 0..10 {
            serv.release_memory();
        }
    }

    /// Functional sanity test: a `release_memory()` call between two
    /// `exec_program` calls with the same source must not change the
    /// produced `json_result`.
    #[test]
    fn exec_program_is_stable_across_release_memory() {
        let serv = KclServiceImpl::default();
        let source = build_fixture_source(0);
        let (_file, path) = write_fixture_to_disk(&source);
        let r1 = serv
            .exec_program(&ExecProgramArgs {
                k_filename_list: vec![path.clone()],
                ..Default::default()
            })
            .expect("first exec_program");
        serv.release_memory();
        let r2 = serv
            .exec_program(&ExecProgramArgs {
                k_filename_list: vec![path],
                ..Default::default()
            })
            .expect("second exec_program");
        assert_eq!(r1.json_result, r2.json_result);
        assert_eq!(r1.err_message, r2.err_message);
    }

    /// Run the same fixture 500 times; assert that RSS grows by less than
    /// 32 MiB. With `release_memory()` after every call, on glibc we
    /// expect the post-warmup RSS to plateau within ~5 MiB. The 32 MiB
    /// ceiling leaves ample room for Go-runtime-style noise while still
    /// failing loudly if the allocator is leaking.
    #[test]
    #[ignore]
    fn exec_program_rss_stable_across_repeated_calls() {
        let Some(rss0) = require_rss() else {
            eprintln!("skip: rss unsupported on this platform");
            return;
        };
        let serv = KclServiceImpl::default();
        let source = build_fixture_source(0);
        let (_file, path) = write_fixture_to_disk(&source);

        // Warmup — allocates the modules, scopes, AST caches, etc.
        for _ in 0..50 {
            exec(&serv, &path);
        }
        let Some(rss_warm) = require_rss() else {
            eprintln!("skip: rss read failed mid-test");
            return;
        };

        let iters = 500u64;
        let t0 = Instant::now();
        for i in 0..iters {
            exec(&serv, &path);
            if i % 50 == 49 {
                let rss = test_support::rss_bytes().unwrap_or(0);
                eprintln!(
                    "iter {}: RSS = {:.2} MiB",
                    i + 1,
                    rss as f64 / (1024.0 * 1024.0)
                );
            }
        }
        let elapsed = t0.elapsed();
        let Some(rss_end) = require_rss() else {
            eprintln!("skip: rss read failed mid-test");
            return;
        };
        let delta = rss_end as i64 - rss_warm as i64;
        eprintln!(
            "----\n{} iters in {:.2?} — warmup RSS {:.2} MiB → final {:.2} MiB (Δ = {:+.2} MiB)",
            iters,
            elapsed,
            rss_warm as f64 / (1024.0 * 1024.0),
            rss_end as f64 / (1024.0 * 1024.0),
            delta as f64 / (1024.0 * 1024.0),
        );

        // Allow up to 32 MiB of growth across 500 iterations (~64 KiB /
        // call). On a healthy glibc + release_memory we expect < 5 MiB.
        let max_growth: i64 = 32 * 1024 * 1024;
        assert!(
            delta < max_growth,
            "RSS grew by {} MiB across {} iters (max {} MiB) — release_memory not bounding growth?",
            delta / (1024 * 1024),
            iters,
            max_growth / (1024 * 1024),
        );

        // Sanity: the warmup RSS itself should be > 0 (we don't want a
        // false positive where the test passes because RSS was always 0).
        assert!(rss0 > 0, "baseline RSS was zero — rss_bytes() is broken");
    }

    /// Run a diverse set of N=100 sources × C=10 cycles = 1000 calls,
    /// exercising the cache eviction pattern. Assert post-warmup RSS grows
    /// by less than 64 MiB.
    #[test]
    #[ignore]
    fn exec_program_rss_stable_across_eviction_pattern() {
        let Some(rss_warm0) = require_rss() else {
            eprintln!("skip: rss unsupported on this platform");
            return;
        };
        let serv = KclServiceImpl::default();
        let sources = build_diverse_sources(100);
        // Keep temp files alive for the whole test; each iteration's
        // `exec` borrows the corresponding path.
        let paths: Vec<(NamedTempFile, String)> =
            sources.iter().map(|s| write_fixture_to_disk(s)).collect();
        let paths_ref: Vec<&str> = paths
            .iter()
            .map(|(_f, p): &(NamedTempFile, String)| p.as_str())
            .collect();

        // Warmup: 1 cycle through all sources.
        for path in &paths_ref {
            exec(&serv, path);
        }
        let Some(rss_warm) = require_rss() else {
            eprintln!("skip: rss read failed mid-test");
            return;
        };

        let cycles = 10u64;
        let t0 = Instant::now();
        for c in 0..cycles {
            for path in &paths_ref {
                exec(&serv, path);
            }
            let rss = test_support::rss_bytes().unwrap_or(0);
            eprintln!(
                "cycle {}: RSS = {:.2} MiB",
                c + 1,
                rss as f64 / (1024.0 * 1024.0)
            );
        }
        let elapsed = t0.elapsed();
        let Some(rss_end) = require_rss() else {
            eprintln!("skip: rss read failed mid-test");
            return;
        };
        let delta = rss_end as i64 - rss_warm as i64;
        eprintln!(
            "----\n{} cycles × {} sources = {} iters in {:.2?} — warmup RSS {:.2} MiB → final {:.2} MiB (Δ = {:+.2} MiB)",
            cycles,
            sources.len(),
            cycles * sources.len() as u64,
            elapsed,
            rss_warm as f64 / (1024.0 * 1024.0),
            rss_end as f64 / (1024.0 * 1024.0),
            delta as f64 / (1024.0 * 1024.0),
        );

        // Allow up to 64 MiB growth across 1000 calls (~64 KiB/call) for
        // the eviction pattern. Healthy glibc + release_memory expects
        // ~10-20 MiB.
        let max_growth: i64 = 64 * 1024 * 1024;
        assert!(
            delta < max_growth,
            "RSS grew by {} MiB across {} cycles (max {} MiB) — eviction pattern leaking?",
            delta / (1024 * 1024),
            cycles,
            max_growth / (1024 * 1024),
        );

        assert!(
            rss_warm0 > 0,
            "baseline RSS was zero — rss_bytes() is broken"
        );
    }
}
