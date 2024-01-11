//! [kclvm_tools::format] module mainly contains some functions of language formatting,
//! the main API function is `format`, which accepts a path to be formatted and
//! formatted options.
//!
//! The basic principle is to call the [kclvm_parser::parse_file] function to parse the
//! AST Module, and then use the AST printer [kclvm_tools::printer::print_ast_module]
//! to print it as source code string.
use anyhow::Result;
use kclvm_ast_pretty::print_ast_module;
use kclvm_driver::get_kcl_files;
use std::path::Path;

use kclvm_parser::{parse_file, parse_file_force_errors};

#[cfg(test)]
mod tests;

/// FormatOptions contains two options:
/// - is_stdout: whether to output the formatted result to stdout.
/// - recursively: whether to recursively traverse a folder and format all KCL files in it.
/// - omit_errors: whether to omit the parse errors when format the KCL code.
#[derive(Debug, Default)]
pub struct FormatOptions {
    pub is_stdout: bool,
    pub recursively: bool,
    pub omit_errors: bool,
}

/// Formats kcl file or directory path contains kcl files and
/// returns the changed file paths.
///
/// # Examples
///
/// ```no_run
/// use kclvm_tools::format::{format, FormatOptions};
///
/// // Format a single file.
/// format("path_to_a_single_file.k", &FormatOptions::default()).unwrap();
/// // Format a folder contains kcl files
/// format("path_to_a_folder", &FormatOptions::default()).unwrap();
/// ```
pub fn format<P: AsRef<Path>>(path: P, opts: &FormatOptions) -> Result<Vec<String>> {
    let mut changed_paths: Vec<String> = vec![];
    let path_ref = path.as_ref();
    if path_ref.is_dir() {
        for file in &get_kcl_files(path, opts.recursively)? {
            if format_file(file, opts)? {
                changed_paths.push(file.clone())
            }
        }
    } else if path_ref.is_file() {
        let file = path_ref.to_str().unwrap().to_string();
        if format_file(&file, opts)? {
            changed_paths.push(file)
        }
    }
    if opts.is_stdout {
        let n = changed_paths.len();
        println!(
            "KCL format done and {} {} formatted:",
            n,
            if n <= 1 { "file was" } else { "files were" }
        );
        for p in &changed_paths {
            println!("{}", p);
        }
    }
    Ok(changed_paths)
}

/// Formats a file and returns whether the file has been formatted and modified.
pub fn format_file(file: &str, opts: &FormatOptions) -> Result<bool> {
    let src = std::fs::read_to_string(file)?;
    let (source, is_formatted) = format_source(file, &src, opts)?;
    if opts.is_stdout {
        println!("{}", source);
    } else {
        std::fs::write(file, &source)?
    }
    Ok(is_formatted)
}

/// Formats a code source and returns the formatted source and
/// whether the source is changed.
pub fn format_source(file: &str, src: &str, opts: &FormatOptions) -> Result<(String, bool)> {
    let module = if opts.omit_errors {
        parse_file(file, Some(src.to_string()))?.module
    } else {
        parse_file_force_errors(file, Some(src.to_string()))?
    };
    let formatted_src = print_ast_module(&module);
    let is_formatted = src != formatted_src;
    Ok((formatted_src, is_formatted))
}
