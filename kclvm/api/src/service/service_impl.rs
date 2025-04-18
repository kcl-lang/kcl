use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::string::String;

use crate::gpyrpc::{self, *};

use kcl_language_server::rename;
use kclvm_ast::ast::SerializeProgram;
use kclvm_config::settings::build_settings_pathbuf;
use kclvm_loader::option::list_options;
use kclvm_loader::{load_packages_with_cache, LoadPackageOptions};
use kclvm_parser::entry::{canonicalize_input_file, get_normalized_k_files_from_paths};
use kclvm_parser::load_program;
use kclvm_parser::parse_single_file;
use kclvm_parser::KCLModuleCache;
use kclvm_parser::LoadProgramOptions;
use kclvm_parser::ParseSessionRef;
use kclvm_query::override_file;
use kclvm_query::query::CompilationOptions;
use kclvm_query::query::{get_full_schema_type, get_full_schema_type_under_path};
use kclvm_query::selector::{list_variables, ListOptions};
use kclvm_query::GetSchemaOption;
use kclvm_runner::exec_program;
#[cfg(feature = "llvm")]
use kclvm_runner::{build_program, exec_artifact};
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::resolver::scope::KCLScopeCache;
use kclvm_sema::resolver::Options;
use kclvm_tools::format::{format, format_source, FormatOptions};
use kclvm_tools::lint::lint_files;
use kclvm_tools::testing;
use kclvm_tools::testing::TestRun;
use kclvm_tools::vet::validator::validate;
use kclvm_tools::vet::validator::LoaderKind;
use kclvm_tools::vet::validator::ValidateOption;
use tempfile::NamedTempFile;

use super::into::*;
use super::ty::kcl_schema_ty_to_pb_ty;
use super::util::{transform_exec_para, transform_str_para};

/// Specific implementation of calling service
#[derive(Debug, Clone, Default)]
pub struct KclvmServiceImpl {
    pub plugin_agent: u64,
}

impl From<&kclvm_query::selector::Variable> for Variable {
    fn from(var: &kclvm_query::selector::Variable) -> Self {
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

impl KclvmServiceImpl {
    /// Ping KclvmService, return the same value as the parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// let serv = KclvmServiceImpl::default();
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

    /// GetVersion KclvmService, return the kclvm service version information
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// let serv = KclvmServiceImpl::default();
    /// let args = &GetVersionArgs {
    ///     ..Default::default()
    /// };
    /// let get_version_result = serv.get_version(args).unwrap();
    /// assert!(get_version_result.version_info.to_string().contains("Version"), "{0}", get_version_result.version_info);
    /// ```
    ///
    pub fn get_version(&self, _args: &GetVersionArgs) -> anyhow::Result<GetVersionResult> {
        Ok(GetVersionResult {
            version: kclvm_version::VERSION.to_string(),
            checksum: kclvm_version::CHECK_SUM.to_string(),
            git_sha: kclvm_version::GIT_SHA.to_string(),
            version_info: kclvm_version::get_version_info(),
        })
    }

    /// Parse KCL program with entry files.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclvmServiceImpl::default();
    /// let args = &ParseFileArgs {
    ///     path: Path::new(".").join("src").join("testdata").join("parse").join("main.k").canonicalize().unwrap().display().to_string(),
    ///     ..Default::default()
    /// };
    /// let result = serv.parse_file(args).unwrap();
    /// assert_eq!(result.errors.len(), 0);
    /// assert_eq!(result.deps.len(), 2);
    /// ```
    pub fn parse_file(&self, args: &ParseFileArgs) -> anyhow::Result<ParseFileResult> {
        let file = canonicalize_input_file(&args.path, "");
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// use kclvm_utils::path::PathPrefix;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// assert_eq!(result.node_symbol_map.len(), 191);
    /// assert_eq!(result.symbol_node_map.len(), 191);
    /// assert_eq!(result.fully_qualified_name_map.len(), 202);
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
            kclvm_ast::ast::set_should_serialize_id(true);
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = KclvmServiceImpl::default();
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
                    required: o.required.clone(),
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = KclvmServiceImpl::default();
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

        return Ok(ListVariablesResult {
            variables: variable_list,
            unsupported_codes,
            parse_errors: select_res
                .parse_errors
                .into_iter()
                .map(|e| e.into_error())
                .collect(),
        });
    }

    /// Execute KCL file with arguments and return the JSON/YAML result.
    ///
    /// **Note that it is not thread safe when the llvm feature is enabled.**
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclvmServiceImpl::default();
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
    ///     ..Default::default()
    /// };
    /// let error = serv.exec_program(args).unwrap_err();
    /// assert!(error.to_string().contains("No input KCL files or paths"), "{error}");
    /// ```
    pub fn exec_program(&self, args: &ExecProgramArgs) -> anyhow::Result<ExecProgramResult> {
        // transform args to json
        let exec_args = transform_exec_para(&Some(args.clone()), self.plugin_agent)?;
        let sess = ParseSessionRef::default();
        let result = exec_program(sess, &exec_args)?;

        Ok(ExecProgramResult {
            json_result: result.json_result,
            yaml_result: result.yaml_result,
            log_message: result.log_message,
            err_message: result.err_message,
        })
    }

    /// Build the KCL program to an artifact.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclvmServiceImpl::default();
    /// let exec_args = ExecProgramArgs {
    ///     work_dir: Path::new(".").join("src").join("testdata").canonicalize().unwrap().display().to_string(),
    ///     k_filename_list: vec!["test.k".to_string()],
    ///     ..Default::default()
    /// };
    /// let artifact = serv.build_program(&BuildProgramArgs {
    ///     exec_args: Some(exec_args),
    ///     output: "".to_string(),
    /// }).unwrap();
    /// assert!(!artifact.path.is_empty());
    /// ```
    #[cfg(feature = "llvm")]
    pub fn build_program(&self, args: &BuildProgramArgs) -> anyhow::Result<BuildProgramResult> {
        let exec_args = transform_exec_para(&args.exec_args, self.plugin_agent)?;
        let artifact = build_program(
            ParseSessionRef::default(),
            &exec_args,
            transform_str_para(&args.output),
        )?;
        Ok(BuildProgramResult {
            path: artifact.get_path().to_string(),
        })
    }

    /// Execute the KCL artifact with arguments and return the JSON/YAML result.
    ///
    /// ***Note that it is not thread safe when the llvm feature is enabled.*
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// // File case
    /// let serv = KclvmServiceImpl::default();
    /// let exec_args = ExecProgramArgs {
    ///     work_dir: Path::new(".").join("src").join("testdata").canonicalize().unwrap().display().to_string(),
    ///     k_filename_list: vec!["test.k".to_string()],
    ///     ..Default::default()
    /// };
    /// let artifact = serv.build_program(&BuildProgramArgs {
    ///     exec_args: Some(exec_args.clone()),
    ///     output: "./lib".to_string(),
    /// }).unwrap();
    /// assert!(!artifact.path.is_empty());
    /// let exec_result = serv.exec_artifact(&ExecArtifactArgs {
    ///     exec_args: Some(exec_args),
    ///     path: artifact.path,
    /// }).unwrap();
    /// assert_eq!(exec_result.err_message, "");
    /// assert_eq!(exec_result.yaml_result, "alice:\n  age: 18");
    /// ```
    #[cfg(feature = "llvm")]
    pub fn exec_artifact(&self, args: &ExecArtifactArgs) -> anyhow::Result<ExecProgramResult> {
        let exec_args = transform_exec_para(&args.exec_args, self.plugin_agent)?;
        let result = exec_artifact(&args.path, &exec_args)?;
        Ok(ExecProgramResult {
            json_result: result.json_result,
            yaml_result: result.yaml_result,
            log_message: result.log_message,
            err_message: result.err_message,
        })
    }

    /// Override KCL file with args
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// use kclvm_ast::MAIN_PKG;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// # use std::path::PathBuf;
    /// # use std::fs;
    /// #
    /// # let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
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
                    result.info.push(TestCaseInfo {
                        name: name.clone(),
                        error: info
                            .error
                            .as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_default(),
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
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    /// use std::path::Path;
    /// use std::fs::remove_dir_all;
    ///
    /// let serv = KclvmServiceImpl::default();
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
        use kclvm_driver::client::ModClient;
        use std::path::Path;
        let mut client = ModClient::new(&args.manifest_path)?;
        if args.vendor {
            client.set_vendor(&Path::new(&args.manifest_path).join("vendor"));
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
