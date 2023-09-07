use crate::config::Config;
use crate::state::LanguageServerState;
use lsp_server::Connection;

/// Runs the main loop of the language server. This will receive requests and handle them.
pub(crate) fn main_loop(connection: Connection, config: Config) -> anyhow::Result<()> {
    LanguageServerState::new(connection.sender, config).run(connection.receiver)
}
