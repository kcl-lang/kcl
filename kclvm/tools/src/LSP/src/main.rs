use config::Config;
use lsp_server::Connection;
use state::LanguageServerState;

mod analysis;
mod capabilities;
mod completion;
mod config;
mod db;
mod dispatcher;
mod document_symbol;
mod from_lsp;
mod goto_def;
mod hover;
mod notification;
mod request;
mod state;
mod to_lsp;
mod util;

#[cfg(test)]
mod tests;

/// Runs the main loop of the language server. This will receive requests and handle them.
pub fn main_loop(connection: Connection, config: Config) -> anyhow::Result<()> {
    LanguageServerState::new(connection.sender, config).run(connection.receiver)
}

/// Main entry point for the language server
pub fn run_server() -> anyhow::Result<()> {
    // Setup IO connections
    let (connection, io_threads) = lsp_server::Connection::stdio();
    // Wait for a client to connect
    let (initialize_id, initialize_params) = connection.initialize_start()?;

    let initialize_params =
        util::from_json::<lsp_types::InitializeParams>("InitializeParams", initialize_params)?;

    let server_capabilities = capabilities::server_capabilities(&initialize_params.capabilities);

    let initialize_result = lsp_types::InitializeResult {
        capabilities: server_capabilities,
        server_info: Some(lsp_types::ServerInfo {
            name: String::from("kcl-language-server"),
            version: None,
        }),
    };

    let initialize_result = serde_json::to_value(initialize_result)
        .map_err(|_| anyhow::anyhow!("Initialize result error"))?;

    connection.initialize_finish(initialize_id, initialize_result)?;

    let config = Config::default();
    main_loop(connection, config)?;
    io_threads.join()?;
    Ok(())
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    Error,
}

/// Main entry point for the `kcl-language-server` executable.
fn main() -> Result<(), anyhow::Error> {
    let status: Result<ExitStatus, anyhow::Error> = {
        run_server().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(ExitStatus::Success)
    };
    match status.unwrap() {
        ExitStatus::Success => {}
        ExitStatus::Error => std::process::exit(1),
    };
    Ok(())
}
