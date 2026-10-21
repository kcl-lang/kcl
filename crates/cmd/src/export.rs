use std::io::Write;

use anyhow::{Result, bail};
use clap::ArgMatches;
use kcl_api::{
    gpyrpc::{GenerateRustArgs, ParseProgramArgs},
    service::service_impl::KclServiceImpl,
};

pub fn export_command<W: Write>(matches: &ArgMatches, writer: &mut W) -> Result<()> {
    let target = matches
        .get_one::<String>("target")
        .map(String::as_str)
        .unwrap_or_default();
    if target != "rust" {
        bail!("export target {target:?} is not implemented yet; available target: rust");
    }

    let paths = matches
        .get_many::<String>("input")
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let result = KclServiceImpl::default().generate_rust(&GenerateRustArgs {
        parse_args: Some(ParseProgramArgs {
            paths,
            ..Default::default()
        }),
    })?;

    match matches.get_one::<String>("output") {
        Some(path) => std::fs::write(path, result.rust_source)?,
        None => write!(writer, "{}", result.rust_source)?,
    }
    Ok(())
}
