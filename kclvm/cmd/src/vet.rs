use anyhow::Result;
use clap::ArgMatches;
use kclvm_tools::vet::validator::{validate, LoaderKind, ValidateOption};

use crate::util::string_from_matches;

/// Run the KCL vet command.
pub fn vet_command(matches: &ArgMatches) -> Result<()> {
    let data_file = matches.value_of("data_file");
    let kcl_file = matches.value_of("kcl_file");
    match (data_file, kcl_file) {
        (Some(data_file), Some(kcl_file)) => {
            validate(ValidateOption::new(
                string_from_matches(matches, "schema"),
                string_from_matches(matches, "attribute_name").unwrap_or_default(),
                data_file.to_string(),
                match string_from_matches(matches, "format") {
                    Some(format) => match format.to_lowercase().as_str() {
                        "json" => LoaderKind::JSON,
                        "yaml" => LoaderKind::YAML,
                        _ => {
                            return Err(anyhow::anyhow!(
                                "Invalid data format, expected JSON or YAML"
                            ))
                        }
                    },
                    // Default loader kind is JSON,
                    None => LoaderKind::JSON,
                },
                Some(kcl_file.to_string()),
                None,
            ))
            .map_err(|err| anyhow::anyhow!(err))?;
            Ok(())
        }
        _ => Err(anyhow::anyhow!("No input data file or kcl file")),
    }
}
