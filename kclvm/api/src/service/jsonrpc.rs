use crate::gpyrpc::*;
use crate::service::service_impl::KclvmServiceImpl;
use jsonrpc_stdio_server::jsonrpc_core::{IoHandler, Params};
use jsonrpc_stdio_server::ServerBuilder;

use super::util::result_to_json_value;

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

fn register_kclvm_service(io: &mut IoHandler) {
    io.add_sync_method("KclvmService.Ping", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: PingArgs = params.parse()?;
        Ok(result_to_json_value(&kclvm_service_impl.ping(&args)))
    });
    io.add_sync_method("KclvmService.ExecProgram", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: ExecProgramArgs = params.parse()?;
        Ok(result_to_json_value(
            &kclvm_service_impl.exec_program(&args),
        ))
    });
    io.add_sync_method("KclvmService.OverrideFile", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: OverrideFileArgs = params.parse()?;
        Ok(result_to_json_value(
            &kclvm_service_impl.override_file(&args),
        ))
    });
    io.add_sync_method("KclvmService.GetSchemaTypeMapping", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: GetSchemaTypeMappingArgs = params.parse()?;
        Ok(result_to_json_value(
            &kclvm_service_impl.get_schema_type_mapping(&args),
        ))
    });
    io.add_sync_method("KclvmService.FormatCode", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: FormatCodeArgs = params.parse()?;
        Ok(result_to_json_value(&kclvm_service_impl.format_code(&args)))
    });
    io.add_sync_method("KclvmService.FormatPath", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: FormatPathArgs = params.parse()?;
        Ok(result_to_json_value(&kclvm_service_impl.format_path(&args)))
    });
    io.add_sync_method("KclvmService.LintPath", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: LintPathArgs = params.parse()?;
        Ok(result_to_json_value(&kclvm_service_impl.lint_path(&args)))
    });
    io.add_sync_method("KclvmService.ValidateCode", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: ValidateCodeArgs = params.parse()?;
        Ok(result_to_json_value(
            &kclvm_service_impl.validate_code(&args),
        ))
    });
    io.add_sync_method("KclvmService.LoadSettingsFiles", |params: Params| {
        let kclvm_service_impl = KclvmServiceImpl::default();
        let args: LoadSettingsFilesArgs = params.parse()?;
        Ok(result_to_json_value(
            &kclvm_service_impl.load_settings_files(&args),
        ))
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
