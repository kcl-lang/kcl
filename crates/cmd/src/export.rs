use std::io::Write;

use anyhow::{Result, bail};
use clap::ArgMatches;
use kcl_api::{
    gpyrpc::{
        GenerateGoArgs, GenerateJavaArgs, GeneratePythonArgs, GenerateRustArgs, ParseProgramArgs,
    },
    service::service_impl::KclServiceImpl,
};

// TODO: once the TypeScript export target is added (alongside the existing
// rust/python/go/java), document the `export` subcommand (targets,
// --package, --output) on the user-facing docs site:
// https://github.com/kcl-lang/kcl-lang.io

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
        "go" => {
            let package = matches
                .get_one::<String>("package")
                .cloned()
                .unwrap_or_default();
            KclServiceImpl::default()
                .generate_go(&GenerateGoArgs {
                    parse_args,
                    package,
                })?
                .go_source
        }
        "java" => {
            let package = matches
                .get_one::<String>("package")
                .cloned()
                .unwrap_or_default();
            KclServiceImpl::default()
                .generate_java(&GenerateJavaArgs {
                    parse_args,
                    package,
                })?
                .java_source
        }
        _ => bail!(
            "export target {target:?} is not implemented yet; available targets: rust, python, go, java"
        ),
    };

    match matches.get_one::<String>("output") {
        Some(path) => std::fs::write(path, source)?,
        None => write!(writer, "{}", source)?,
    }
    Ok(())
}
