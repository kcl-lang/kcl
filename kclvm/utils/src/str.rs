use strsim::levenshtein;

/// [`str_distance`] will return the closest string in `pools` to `input` if it is within `min_distance`.
/// It calculates the Levenshtein distance between `input` and each string in `pools` and returns the closest one.
///
/// # Examples
///
/// ```
/// use kclvm_utils::str::find_closest_strs;
///
/// let input = "hello".to_string();
/// let pools = vec!["hello".to_string(), "world".to_string()];
/// let closest = find_closest_strs(input.to_string(), pools, None);
/// assert_eq!(closest, vec!["hello"]);
///
/// let pools = vec!["good".to_string(), "world".to_string()];
/// let closest = find_closest_strs(input, pools, None);
/// assert_eq!(closest, Vec::<String>::new());
/// ```
pub fn find_closest_strs(
    input: String,
    pools: Vec<String>,
    min_distance: Option<usize>,
) -> Vec<String> {
    if pools.contains(&input) {
        return vec![input];
    } else {
        let mut closests = vec![];
        // If the input is not in the pool, the minimum distance is half the length of the input.
        let mut min_distance = min_distance.unwrap_or((input.len() + 1) / 2);
        for s in pools {
            let distance = levenshtein(&input, &s);
            let has_intersection = input.chars().any(|c| s.contains(c));

            if distance <= min_distance && has_intersection {
                closests.push(s);
                min_distance = distance;
            }
        }
        return closests;
    }
}

#[test]
fn test_str_distance() {
    let input = "hello".to_string();
    let pools = vec!["hello".to_string(), "world".to_string()];
    let closest = find_closest_strs(input.to_string(), pools, None);
    assert_eq!(closest, vec!["hello".to_string()]);

    let input = "hello".to_string();
    let pools = vec!["helo".to_string(), "world".to_string()];
    let closest = find_closest_strs(input.to_string(), pools, None);
    assert_eq!(closest, vec!["helo".to_string()]);

    let input = "hello".to_string();
    let pools = vec!["good".to_string(), "world".to_string()];
    let closest = find_closest_strs(input.to_string(), pools, None);
    assert_eq!(closest, Vec::<String>::new());

    let input = "h".to_string();
    let pools = vec!["e".to_string(), "hh".to_string()];
    let closest = find_closest_strs(input.to_string(), pools, None);
    assert_eq!(closest, vec!["hh".to_string()]);
}
