use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use crate::model::gpyrpc::*;

use super::into::IntoLoadSettingsFiles;
use anyhow::anyhow;
use kclvm_config::settings::build_settings_pathbuf;
use kclvm_driver::canonicalize_input_files;
use kclvm_parser::ParseSession;
use kclvm_query::{get_schema_type, override_file};
use kclvm_runner::{exec_program, ExecProgramArgs};
use kclvm_tools::format::{format, format_source, FormatOptions};
use kclvm_tools::lint::lint_files;
use kclvm_tools::vet::validator::{validate, LoaderKind, ValidateOption};
use protobuf_json_mapping::print_to_string_with_options;
use protobuf_json_mapping::PrintOptions;
use tempfile::NamedTempFile;

use super::ty::kcl_schema_ty_to_pb_ty;
use super::util::transform_str_para;

/// Specific implementation of calling service
#[derive(Default)]
pub struct KclvmService {
    pub plugin_agent: u64,
}

impl KclvmService {
    /// Ping KclvmService, return the same value as the parameter
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    /// let serv = &KclvmService { plugin_agent: 0 };
    /// let args = &Ping_Args {
    ///     value: "hello".to_string(),
    ///     ..Default::default()
    /// };
    /// let ping_result = serv.ping(args).unwrap();
    /// assert_eq!(ping_result.value, "hello".to_string());
    /// ```
    ///
    pub fn ping(&self, args: &Ping_Args) -> anyhow::Result<Ping_Result> {
        Ok(Ping_Result {
            value: (args.value.clone()),
            special_fields: (args.special_fields.clone()),
        })
    }

    /// Execute KCL file with args.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    /// use std::path::Path;
    ///
    /// let serv = &KclvmService { plugin_agent: 0 };
    /// let args = &ExecProgram_Args {
    ///     work_dir: Path::new(".").join("src").join("testdata").canonicalize().unwrap().display().to_string(),
    ///     k_filename_list: vec!["test.k".to_string()],
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

        let sess = Arc::new(ParseSession::default());
        let result = exec_program(
            sess,
            &ExecProgramArgs::from_str(args_json.as_str()),
            self.plugin_agent,
        )?;

        Ok(ExecProgram_Result {
            json_result: result.json_result,
            yaml_result: result.yaml_result,
            escaped_time: result.escaped_time,
            ..Default::default()
        })
    }

    /// Override KCL file with args
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
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

    /// Service for getting the schema mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
    /// let serv = KclvmService::default();
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
    /// let result = serv.get_schema_type_mapping(&GetSchemaTypeMapping_Args {
    ///     file,
    ///     code,
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.schema_type_mapping.len(), 2);
    /// ```
    pub fn get_schema_type_mapping(
        &self,
        args: &GetSchemaTypeMapping_Args,
    ) -> anyhow::Result<GetSchemaTypeMapping_Result> {
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

        Ok(GetSchemaTypeMapping_Result {
            schema_type_mapping: type_mapping,
            ..Default::default()
        })
    }

    /// Service for formatting a code source and returns the formatted source and
    /// whether the source is changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
    /// let serv = KclvmService::default();
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
    /// let result = serv.format_code(&FormatCode_Args {
    ///     source: source.clone(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.formatted, source.as_bytes().to_vec());
    /// ```
    pub fn format_code(&self, args: &FormatCode_Args) -> anyhow::Result<FormatCode_Result> {
        let (formatted, _) = format_source(&args.source)?;
        Ok(FormatCode_Result {
            formatted: formatted.as_bytes().to_vec(),
            ..Default::default()
        })
    }

    /// Service for formatting kcl file or directory path contains kcl files and
    /// returns the changed file paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
    /// let serv = KclvmService::default();
    /// let result = serv.format_path(&FormatPath_Args {
    ///     path: "./src/testdata/test.k".to_string(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert!(result.changed_paths.is_empty());
    /// ```
    pub fn format_path(&self, args: &FormatPath_Args) -> anyhow::Result<FormatPath_Result> {
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
                ..Default::default()
            },
        )?;
        Ok(FormatPath_Result {
            changed_paths,
            ..Default::default()
        })
    }

    /// Service for KCL Lint API, check a set of files, skips execute,
    /// returns error message including errors and warnings.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
    /// let serv = KclvmService::default();
    /// let result = serv.lint_path(&LintPath_Args {
    ///     paths: vec!["./src/testdata/test-lint.k".to_string()],
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.results, vec!["Module 'math' imported but unused".to_string()]);
    /// ```
    pub fn lint_path(&self, args: &LintPath_Args) -> anyhow::Result<LintPath_Result> {
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
        Ok(LintPath_Result {
            results,
            ..Default::default()
        })
    }

    /// Service for validating the data string using the schema code string, when the parameter
    /// `schema` is omitted, use the first schema appeared in the kcl code.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
    /// let serv = KclvmService::default();
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
    /// let result = serv.validate_code(&ValidateCode_Args {
    ///     code,
    ///     data,
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.success, true);
    /// ```
    pub fn validate_code(&self, args: &ValidateCode_Args) -> anyhow::Result<ValidateCode_Result> {
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
        Ok(ValidateCode_Result {
            success,
            err_message,
            ..Default::default()
        })
    }

    /// Service for building setting file config from args.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_capi::service::service::KclvmService;
    /// use kclvm_capi::model::gpyrpc::*;
    ///
    /// let serv = KclvmService::default();
    /// let result = serv.load_settings_files(&LoadSettingsFiles_Args {
    ///     files: vec!["./src/testdata/settings/kcl.yaml".to_string()],
    ///     work_dir: "./src/testdata/settings".to_string(),
    ///     ..Default::default()
    /// }).unwrap();
    /// assert_eq!(result.kcl_options.len(), 1);
    /// ```
    pub fn load_settings_files(
        &self,
        args: &LoadSettingsFiles_Args,
    ) -> anyhow::Result<LoadSettingsFiles_Result> {
        let settings_files = args.files.iter().map(|f| f.as_str()).collect::<Vec<&str>>();
        let settings_pathbuf =
            build_settings_pathbuf(&[], None, Some(settings_files), false, false)?;
        let files = if !settings_pathbuf.settings().input().is_empty() {
            canonicalize_input_files(&settings_pathbuf.settings().input(), args.work_dir.clone())
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
