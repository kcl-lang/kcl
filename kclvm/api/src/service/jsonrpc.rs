use crate::gpyrpc::*;
use crate::service::service_impl::KclvmServiceImpl;
use core::fmt::Display;
use jsonrpc_stdio_server::jsonrpc_core::{Error, ErrorCode, IoHandler, Params};
use jsonrpc_stdio_server::ServerBuilder;
use serde::Serialize;
const KCLVM_SERVER_ERROR_CODE: i64 = 0x4B434C; // the ASCII code of "KCL"

/// Start a json rpc server via Stdin/Stdout
#[tokio::main]
pub async fn start_stdio_server() -> Result<(), anyhow::Error> {
    let mut io = IoHandler::default();
    // KclvmService
    register_kclvm_service(&mut io);
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
                let err_message = kclvm_error::err_to_str(panic_err);
                Err(Error {
                    code: ErrorCode::from(KCLVM_SERVER_ERROR_CODE),
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
            code: ErrorCode::from(KCLVM_SERVER_ERROR_CODE),
            message: err.to_string(),
            data: None,
        }),
    }
}

fn register_kclvm_service(io: &mut IoHandler) {
    io.add_method("KclvmService.Ping", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: PingArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, ping))
    });
    io.add_method("KclvmService.ExecProgram", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: ExecProgramArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, exec_program))
    });
    io.add_method("KclvmService.OverrideFile", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: OverrideFileArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, override_file))
    });
    io.add_method("KclvmService.GetSchemaType", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: GetSchemaTypeArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, get_schema_type))
    });
    io.add_method("KclvmService.GetSchemaTypeMapping", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: GetSchemaTypeMappingArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, get_schema_type_mapping))
    });
    io.add_method("KclvmService.FormatCode", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: FormatCodeArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, format_code))
    });
    io.add_method("KclvmService.FormatPath", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: FormatPathArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, format_path))
    });
    io.add_method("KclvmService.LintPath", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: LintPathArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, lint_path))
    });
    io.add_method("KclvmService.ValidateCode", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: ValidateCodeArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, validate_code))
    });
    io.add_method("KclvmService.LoadSettingsFiles", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: LoadSettingsFilesArgs = match params.parse() {
            Ok(val) => val,
            Err(err) => return futures::future::ready(Err(err)),
        };
        futures::future::ready(catch!(kclvm_service_impl, args, load_settings_files))
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
                "KclvmService.Ping".to_owned(),
                "KclvmService.ExecProgram".to_owned(),
                "KclvmService.OverrideFile".to_owned(),
                "KclvmService.GetSchemaTypeMapping".to_owned(),
                "KclvmService.FormatCode".to_owned(),
                "KclvmService.FormatPath".to_owned(),
                "KclvmService.LintPath".to_owned(),
                "KclvmService.ValidateCode".to_owned(),
                "KclvmService.LoadSettingsFiles".to_owned(),
                "BuiltinService.Ping".to_owned(),
                "BuiltinService.PingListMethod".to_owned(),
            ],
        };
        Ok(serde_json::to_value(result).unwrap())
    });
}
