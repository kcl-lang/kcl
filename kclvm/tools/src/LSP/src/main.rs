use main_loop::_main;

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
mod main_loop;
mod notification;
mod quick_fix;
mod request;
mod state;
mod to_lsp;
mod util;

mod formatting;
#[cfg(test)]
mod tests;

/// Main entry point for the `kcl-language-server` executable.
fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    _main(args)
}
