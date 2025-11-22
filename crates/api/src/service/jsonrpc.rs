use crate::gpyrpc::*;
use crate::service::service_impl::KclServiceImpl;
use core::fmt::Display;
use jsonrpc_stdio_server::ServerBuilder;
use jsonrpc_stdio_server::jsonrpc_core::{Error, ErrorCode, IoHandler, Params};
use serde::Serialize;
const KCL_SERVER_ERROR_CODE: i64 = 0x4B434C; // the ASCII code of "KCL"

/// Start a json rpc server via Stdin/Stdout
#[tokio::main]
pub async fn start_stdio_server() -> Result<(), anyhow::Error> {
    let mut io = IoHandler::default();
    // KclService
    register_kcl_service(&mut io);
    // BuiltinService
    register_builtin_service(&mut io);
    let server = ServerBuilder::new(io).build();
    server.await;
    Ok(())
}

macro_rules! catch {
    ($serv:expr, $args:expr, $serv_name:ident) => {{
        let prev_hook = std::panic::take_hook();

        // disable print panic info
        std::panic::set_hook(Box::new(|_info| {}));
        let result = std::panic::catch_unwind(|| to_json_result(&$serv.$serv_name(&$args)));
        std::panic::set_hook(prev_hook);
        match result {
            Ok(result) => result,
            Err(panic_err) => {
                let err_message = kcl_error::err_to_str(panic_err);
                Err(Error {
                    code: ErrorCode::from(KCL_SERVER_ERROR_CODE),
                    message: err_message,
                    data: None,
                })
            }
        }
    }};
}

/// Transform the [`Result<V, E>`]  into [`Result<serde_json::Value,jsonrpc_core::Error>`]
#[inline]
fn to_json_result<V, E>(val: &Result<V, E>) -> Result<serde_json::Value, Error>
where
    V: Serialize,
    E: Display,
{
    match val {
        Ok(val) => Ok(serde_json::to_value(val).unwrap()),
        Err(err) => Err(Error {
            code: ErrorCode::from(KCL_SERVER_ERROR_CODE),
            message: err.to_string(),
            data: None,
        }),
    }
}

fn register_kcl_service(io: &mut IoHandler) {
    io.add_method("KclService.Ping", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: PingArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, ping))
    });
    io.add_method("KclService.GetVersion", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: GetVersionArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, get_version))
    });
    io.add_method("KclService.ParseFile", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: ParseFileArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, parse_file))
    });
    io.add_method("KclService.ParseProgram", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: ParseProgramArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, parse_program))
    });
    io.add_method("KclService.LoadPackage", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: LoadPackageArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, load_package))
    });
    io.add_method("KclService.ListOptions", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: ParseProgramArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, list_options))
    });
    io.add_method("KclService.ListVariables", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: ListVariablesArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, list_variables))
    });
    io.add_method("KclService.ExecProgram", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: ExecProgramArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, exec_program))
    });
    io.add_method("KclService.OverrideFile", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: OverrideFileArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, override_file))
    });
    io.add_method("KclService.GetSchemaTypeMapping", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: GetSchemaTypeMappingArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, get_schema_type_mapping))
    });
    io.add_method(
        "KclService.GetSchemaTypeMappingUnderPath",
        |params: Params| {
            let kcl_service_impl = KclServiceImpl::default();
            let args: GetSchemaTypeMappingArgs = match params.parse() {
                Ok(val) => val,
                Err(err) => return futures::future::ready(Err(err)),
            };
            futures::future::ready(catch!(
                kcl_service_impl,
                args,
                get_schema_type_mapping_under_path
            ))
        },
    );
    io.add_method("KclService.FormatCode", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: FormatCodeArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, format_code))
    });
    io.add_method("KclService.FormatPath", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: FormatPathArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, format_path))
    });
    io.add_method("KclService.LintPath", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: LintPathArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, lint_path))
    });
    io.add_method("KclService.ValidateCode", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: ValidateCodeArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, validate_code))
    });
    io.add_method("KclService.LoadSettingsFiles", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: LoadSettingsFilesArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, load_settings_files))
    });
    io.add_method("KclService.Rename", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: RenameArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, rename))
    });
    io.add_method("KclService.RenameCode", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: RenameCodeArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, rename_code))
    });
    io.add_method("KclService.Test", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: TestArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, test))
    });
    io.add_method("KclService.UpdateDependencies", |params: Params| {
        let kcl_service_impl = KclServiceImpl::default();
        let args: UpdateDependenciesArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kcl_service_impl, args, update_dependencies))
    });
}

fn register_builtin_service(io: &mut IoHandler) {
    io.add_sync_method("BuiltinService.Ping", |params: Params| {
        let args: PingArgs = params.parse()?;
        let result = PingResult { value: args.value };
        Ok(serde_json::to_value(result).unwrap())
    });
    io.add_sync_method("BuiltinService.ListMethod", |_params: Params| {
        let result = ListMethodResult {
            method_name_list: vec![
                "KclService.Ping".to_owned(),
                "KclService.GetVersion".to_owned(),
                "KclService.ParseFile".to_owned(),
                "KclService.ParseProgram".to_owned(),
                "KclService.ExecProgram".to_owned(),
                "KclService.BuildProgram".to_owned(),
                "KclService.ExecArtifact".to_owned(),
                "KclService.OverrideFile".to_owned(),
                "KclService.GetSchemaType".to_owned(),
                "KclService.GetFullSchemaType".to_owned(),
                "KclService.GetSchemaTypeMapping".to_owned(),
                "KclService.FormatCode".to_owned(),
                "KclService.FormatPath".to_owned(),
                "KclService.LintPath".to_owned(),
                "KclService.ValidateCode".to_owned(),
                "KclService.LoadSettingsFiles".to_owned(),
                "KclService.Rename".to_owned(),
                "KclService.RenameCode".to_owned(),
                "KclService.Test".to_owned(),
                "KclService.UpdateDependencies".to_owned(),
                "BuiltinService.Ping".to_owned(),
                "BuiltinService.PingListMethod".to_owned(),
            ],
        };
        Ok(serde_json::to_value(result).unwrap())
    });
}
