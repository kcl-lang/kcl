use crate::parse_file_force_errors;
use regex::Regex;

#[test]
fn test_parse_file_not_found() {
    match parse_file_force_errors("The file path is invalid", None) {
        Ok(_) => {
            panic!("unreachable")
        }
        Err(err_msg) => {
            assert!(
                Regex::new(r"^Failed to load KCL file 'The file path is invalid'. Because.*")
                    .unwrap()
                    .is_match(&err_msg.to_string())
            );
        }
    }
}
