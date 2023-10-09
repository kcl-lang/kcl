use crate::config::Config;
use crate::state::LanguageServerState;
use clap::Command;
use lsp_server::Connection;
use lsp_types::InitializeParams;

#[allow(dead_code)]
/// Runs the main loop of the language server. This will receive requests and handle them.
pub(crate) fn main_loop(
    connection: Connection,
    config: Config,
    initialize_params: InitializeParams,
) -> anyhow::Result<()> {
    LanguageServerState::new(connection.sender, config, initialize_params).run(connection.receiver)
}

#[allow(dead_code)]
/// Get the kcl language server CLI application.
pub(crate) fn app() -> Command {
    Command::new("kcl-language-server")
        .version(kclvm_version::VERSION)
        .about("KCL language server CLI.")
        .subcommand(Command::new("version").about("Show the KCL language server version"))
}
