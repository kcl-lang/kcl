use crate::state::LanguageServerState;
use clap::{builder::Str, Command};
use lsp_server::Connection;
use lsp_types::InitializeParams;

#[allow(dead_code)]
/// Runs the main loop of the language server. This will receive requests and handle them.
pub(crate) fn main_loop(
    connection: Connection,
    initialize_params: InitializeParams,
) -> anyhow::Result<()> {
    LanguageServerState::new(connection.sender, initialize_params).run(connection.receiver)
}

#[allow(dead_code)]
/// Get the kcl language server CLI application.
pub(crate) fn app() -> Command {
    Command::new("kcl-language-server")
        .version(Str::from(kclvm_version::get_version_info()))
        .about("KCL language server CLI.")
        .subcommand(Command::new("version").about("Show the KCL language server version"))
}
