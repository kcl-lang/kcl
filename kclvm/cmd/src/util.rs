use clap::ArgMatches;
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
) -> Option<HashMap<String, String>> {
    matches.values_of(key).map(|files| {
        files
            .into_iter()
            .map(|s| {
                let parts: Vec<&str> = s.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    (key.to_string(), value.to_string())
                } else {
                    (String::default(), String::default())
                }
            })
            .collect::<HashMap<String, String>>()
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
