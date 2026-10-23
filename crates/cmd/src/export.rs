use std::io::Write;

use anyhow::{Result, bail};
use clap::ArgMatches;
use kcl_api::{
    gpyrpc::{GeneratePythonArgs, GenerateRustArgs, ParseProgramArgs},
    service::service_impl::KclServiceImpl,
};

pub fn export_command<W: Write>(matches: &ArgMatches, writer: &mut W) -> Result<()> {
    let target = matches
        .get_one::<String>("target")
        .map(String::as_str)
        .unwrap_or_default();

    let paths: Vec<String> = matches
        .get_many::<String>("input")
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let parse_args = Some(ParseProgramArgs {
        paths,
        ..Default::default()
    });

    let source = match target {
        "rust" => {
            KclServiceImpl::default()
                .generate_rust(&GenerateRustArgs { parse_args })?
                .rust_source
        }
        "python" => {
            KclServiceImpl::default()
                .generate_python(&GeneratePythonArgs { parse_args })?
                .python_source
        }
        _ => bail!(
            "export target {target:?} is not implemented yet; available targets: rust, python"
        ),
    };

    match matches.get_one::<String>("output") {
        Some(path) => std::fs::write(path, source)?,
        None => write!(writer, "{}", source)?,
    }
    Ok(())
}
