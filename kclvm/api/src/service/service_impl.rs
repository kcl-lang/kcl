use std::collections::HashMap;
use std::io::Write;
use std::string::String;
use std::sync::Arc;

use crate::gpyrpc::*;

use anyhow::anyhow;
use kclvm_config::settings::build_settings_pathbuf;
use kclvm_driver::canonicalize_input_files;
use kclvm_parser::ParseSession;
use kclvm_query::get_schema_type;
use kclvm_query::override_file;
use kclvm_runner::exec_program;
use kclvm_tools::format::{format, format_source, FormatOptions};
use kclvm_tools::lint::lint_files;
use kclvm_tools::vet::validator::validate;
use kclvm_tools::vet::validator::LoaderKind;
use kclvm_tools::vet::validator::ValidateOption;
use tempfile::NamedTempFile;

use super::into::IntoLoadSettingsFiles;
use super::ty::kcl_schema_ty_to_pb_ty;
use super::util::transform_str_para;

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
    /// assert!(error.contains("Cannot find the kcl file"), "{error}");
    ///
    /// let args = &ExecProgramArgs {
    ///     k_filename_list: vec![],
    ///     ..Default::default()
    /// };
    /// let error = serv.exec_program(args).unwrap_err();
    /// assert!(error.contains("No input KCL files or paths"), "{error}");
    /// ```
    pub fn exec_program(&self, args: &ExecProgramArgs) -> Result<ExecProgramResult, String> {
        // transform args to json
        let args_json = serde_json::to_string(args).unwrap();

        let sess = Arc::new(ParseSession::default());
        let result = exec_program(
            sess,
            &kclvm_runner::ExecProgramArgs::from_str(args_json.as_str()),
        )?;

        Ok(ExecProgramResult {
            json_result: result.json_result,
            yaml_result: result.yaml_result,
            escaped_time: result.escaped_time,
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
        let (formatted, _) = format_source(&args.source)?;
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
        // Write some test data to the first handle.
        file.write_all(args.data.as_bytes())?;
        let file_path = file.path().to_string_lossy().to_string();
        let (success, err_message) = match validate(ValidateOption::new(
            transform_str_para(&args.schema),
            args.attribute_name.clone(),
            file_path,
            match args.format.to_lowercase().as_str() {
                "yaml" | "yml" => LoaderKind::YAML,
                "json" => LoaderKind::JSON,
                _ => LoaderKind::JSON,
            },
            None,
            transform_str_para(&args.code),
        )) {
            Ok(success) => (success, "".to_string()),
            Err(err) => (false, err),
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
}
