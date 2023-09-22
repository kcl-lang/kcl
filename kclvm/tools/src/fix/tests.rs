use std::fs;

use crate::lint::lint_files;

use super::fix;

#[test]
fn test_lint() {
    let (errors, warnings) = lint_files(
        &[
            "./src/fix/test_data/fix_import.k",
            "./src/fix/test_data/unused_import.k",
        ],
        None,
    );
    assert_eq!(errors.len(), 0);
    let mut diags = vec![];
    diags.extend(warnings);

    match fix(diags) {
        Ok(_) => {
            let src = fs::read_to_string("./src/fix/test_data/fix_import.k").unwrap();
            #[cfg(target_os = "windows")]
            assert_eq!(src, "import math\r\n\r\na = math.pow(1, 1)".to_string());
            #[cfg(not(target_os = "windows"))]
            assert_eq!(src, "import math\n\na = math.pow(1, 1)".to_string());
            fs::write(
                "./src/fix/test_data/fix_import.k",
                r#"import regex
import math
import regex

a = math.pow(1, 1)"#,
            )
            .unwrap();
            let src = fs::read_to_string("./src/fix/test_data/unused_import.k").unwrap();
            assert_eq!(src, "".to_string());
            fs::write("./src/fix/test_data/unused_import.k", r#"import math"#).unwrap();
        }
        Err(e) => panic!("fix failed: {:?}", e),
    }
}
