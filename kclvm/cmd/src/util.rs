use anyhow::bail;
use anyhow::Result;
use clap::ArgMatches;
use kclvm_driver::arguments::parse_key_value_pair;
use std::collections::HashMap;

#[inline]
pub(crate) fn strings_from_matches(matches: &ArgMatches, key: &str) -> Option<Vec<String>> {
    matches.values_of(key).map(|files| {
        files
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
    })
}

#[inline]
pub(crate) fn hashmaps_from_matches(
    matches: &ArgMatches,
    key: &str,
) -> Option<Result<HashMap<String, String>>> {
    matches.values_of(key).map(|files| {
        files
            .into_iter()
            .map(|s| match parse_key_value_pair(s) {
                Ok(pair) => Ok((pair.key, pair.value)),
                Err(err) => {
                    bail!("Invalid arguments format '-E, --external', use'kclvm_cli run --help' for more help.")
                }
            })
            .collect::<Result<HashMap<String, String>>>()
    })
}

#[inline]
pub(crate) fn string_from_matches(matches: &ArgMatches, key: &str) -> Option<String> {
    matches.value_of(key).map(|v| v.to_string())
}

#[inline]
pub(crate) fn bool_from_matches(matches: &ArgMatches, key: &str) -> Option<bool> {
    let occurrences = matches.occurrences_of(key);
    if occurrences > 0 {
        Some(true)
    } else {
        None
    }
}

#[inline]
pub(crate) fn u32_from_matches(matches: &ArgMatches, key: &str) -> Option<u32> {
    let occurrences = matches.occurrences_of(key);
    if occurrences > 0 {
        Some(occurrences as u32)
    } else {
        None
    }
}
