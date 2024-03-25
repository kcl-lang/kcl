use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::string::String;

use crate::gpyrpc::*;

use anyhow::anyhow;
use kcl_language_server::rename;
use kclvm_config::settings::build_settings_pathbuf;
use kclvm_driver::canonicalize_input_files;
use kclvm_loader::option::list_options;
use kclvm_loader::{load_packages, LoadPackageOptions};
use kclvm_parser::load_program;
use kclvm_parser::parse_file;
use kclvm_parser::KCLModuleCache;
use kclvm_parser::LoadProgramOptions;
use kclvm_parser::ParseSessionRef;
use kclvm_query::get_schema_type;
use kclvm_query::override_file;
use kclvm_query::query::get_full_schema_type;
use kclvm_query::query::CompilationOptions;
use kclvm_query::GetSchemaOption;
use kclvm_runner::{build_program, exec_artifact, exec_program};
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
        let ast_json = serde_json::to_string(&result.program)?;

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
        let result = parse_file(&args.path, transform_str_para(&args.source))?;
        let ast_json = serde_json::to_string(&result.module)?;

        Ok(ParseFileResult {
            ast_json,
            deps: result
                .deps
                .iter()
                .map(|p| p.to_str().unwrap().to_string())
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
    ///
    /// let serv = KclvmServiceImpl::default();
    /// let args = &LoadPackageArgs {
    ///     parse_args: Some(ParseProgramArgs {
    ///         paths: vec![Path::new(".").join("src").join("testdata").join("parse").join("main.k").canonicalize().unwrap().display().to_string()],
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
    /// assert_eq!(result.node_symbol_map.len(), 167);
    /// assert_eq!(result.symbol_node_map.len(), 167);
    /// assert_eq!(result.fully_qualified_name_map.len(), 175);
    /// assert_eq!(result.pkg_scope_map.len(), 3);
    /// ```
    pub fn load_package(&self, args: &LoadPackageArgs) -> anyhow::Result<LoadPackageResult> {
        let mut package_maps = HashMap::new();
        let parse_args = args.parse_args.clone().unwrap_or_default();
        for p in &parse_args.external_pkgs {
            package_maps.insert(p.pkg_name.to_string(), p.pkg_path.to_string());
        }
        let packages = load_packages(&LoadPackageOptions {
            paths: parse_args.paths,
            load_opts: Some(LoadProgramOptions {
                k_code_list: parse_args.sources.clone(),
                package_maps,
                load_plugins: true,
                ..Default::default()
            }),
            resolve_ast: args.resolve_ast,
            load_builtin: args.load_builtin,
        })?;
        if args.with_ast_index {
            // Thread local options
            kclvm_ast::ast::set_should_serialize_id(true);
        }
        let program_json = serde_json::to_string(&packages.program)?;
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

    /// Execute KCL file with args. **Note that it is not thread safe.**
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

    /// Execute the KCL artifact with args. **Note that it is not thread safe.**
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
    pub fn override_file(&self, args: &OverrideFileArgs) -> Result<OverrideFileResult, String> {
        override_file(&args.file, &args.specs, &args.import_paths)
            .map_err(|err| err.to_string())
            .map(|result| OverrideFileResult { result })
    }

    /// Service for getting the schema type list.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
    /// let file = "schema.k".to_string();
    /// let code = r#"
    /// schema Person:
    ///     name: str
    ///     age: int
    ///
    /// person = Person {
    ///     name = "Alice"
    ///     age = 18
    /// }
    /// "#.to_string();
    /// let result = serv.get_schema_type(&GetSchemaTypeArgs {
    ///     file,
    ///     code,
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.schema_type_list.len(), 2);
    /// ```
    pub fn get_schema_type(&self, args: &GetSchemaTypeArgs) -> anyhow::Result<GetSchemaTypeResult> {
        let mut type_list = Vec::new();
        for (_k, schema_ty) in get_schema_type(
            &args.file,
            if args.code.is_empty() {
                None
            } else {
                Some(&args.code)
            },
            if args.schema_name.is_empty() {
                None
            } else {
                Some(&args.schema_name)
            },
            Default::default(),
        )? {
            type_list.push(kcl_schema_ty_to_pb_ty(&schema_ty));
        }

        Ok(GetSchemaTypeResult {
            schema_type_list: type_list,
        })
    }

    /// Service for getting the full schema type list.
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
    ///         CmdExternalPkgSpec{
    ///             pkg_name:"bbb".to_string(),
    ///             pkg_path: work_dir_parent.join("bbb").canonicalize().unwrap().display().to_string()
    ///         }
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// let result = serv.get_full_schema_type(&GetFullSchemaTypeArgs {
    ///     exec_args: Some(args),
    ///     schema_name: "a".to_string()
    /// }).unwrap();
    /// assert_eq!(result.schema_type_list.len(), 1);
    /// ```
    pub fn get_full_schema_type(
        &self,
        args: &GetFullSchemaTypeArgs,
    ) -> anyhow::Result<GetSchemaTypeResult> {
        let mut type_list = Vec::new();
        let exec_args = transform_exec_para(&args.exec_args, self.plugin_agent)?;
        for (_k, schema_ty) in get_full_schema_type(
            Some(&args.schema_name),
            CompilationOptions {
                k_files: exec_args.clone().k_filename_list,
                loader_opts: Some(exec_args.get_load_program_options()),
                resolve_opts: Options {
                    resolve_val: true,
                    ..Default::default()
                },
                get_schema_opts: GetSchemaOption::default(),
            },
        )? {
            type_list.push(kcl_schema_ty_to_pb_ty(&schema_ty));
        }

        Ok(GetSchemaTypeResult {
            schema_type_list: type_list,
        })
    }

    /// Service for getting the schema mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_api::service::service_impl::KclvmServiceImpl;
    /// use kclvm_api::gpyrpc::*;
    ///
    /// let serv = KclvmServiceImpl::default();
    /// let file = "schema.k".to_string();
    /// let code = r#"
    /// schema Person:
    ///     name: str
    ///     age: int
    ///
    /// person = Person {
    ///     name = "Alice"
    ///     age = 18
    /// }
    /// "#.to_string();
    /// let result = serv.get_schema_type_mapping(&GetSchemaTypeMappingArgs {
    ///     file,
    ///     code,
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.schema_type_mapping.len(), 2);
    /// ```
    pub fn get_schema_type_mapping(
        &self,
        args: &GetSchemaTypeMappingArgs,
    ) -> anyhow::Result<GetSchemaTypeMappingResult> {
        let mut type_mapping = HashMap::new();
        for (k, schema_ty) in get_schema_type(
            &args.file,
            if args.code.is_empty() {
                None
            } else {
                Some(&args.code)
            },
            if args.schema_name.is_empty() {
                None
            } else {
                Some(&args.schema_name)
            },
            Default::default(),
        )? {
            type_mapping.insert(k, kcl_schema_ty_to_pb_ty(&schema_ty));
        }

        Ok(GetSchemaTypeMappingResult {
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
    ///
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
            canonicalize_input_files(
                &settings_pathbuf.settings().input(),
                args.work_dir.clone(),
                false,
            )
            .map_err(|e| anyhow!(e))?
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
}
