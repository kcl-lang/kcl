use crate::main_loop::main_loop;
use config::Config;
use main_loop::app;

mod analysis;
mod capabilities;
mod completion;
mod config;
mod db;
mod dispatcher;
mod document_symbol;
mod error;
mod find_refs;
mod from_lsp;
mod goto_def;
mod hover;
mod main_loop;
mod notification;
mod quick_fix;
mod request;
mod semantic_token;
mod state;
mod to_lsp;
mod util;
mod word_index;

mod formatting;
#[cfg(test)]
mod tests;

/// Main entry point for the `kcl-language-server` executable.
fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let matches = app()
        .arg_required_else_help(false)
        .try_get_matches_from(args);
    match matches {
        Ok(arg_matches) => match arg_matches.subcommand() {
            Some(("version", _)) => {
                println!("{}", kclvm_version::get_version_info());
                Ok(())
            }
            Some((subcommand, _)) => Err(anyhow::anyhow!("unknown subcommand: {}", subcommand)),
            None => {
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
        },
        Err(e) => Err(e.into()),
    }
}

#[allow(dead_code)]
/// Main entry point for the language server
fn run_server() -> anyhow::Result<()> {
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
    main_loop(connection, config, initialize_params)?;
    io_threads.join()?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
enum ExitStatus {
    Success,
    Error,
}
