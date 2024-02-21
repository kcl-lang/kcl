use crate::{load_packages, LoadPackageOptions};
use kclvm_parser::LoadProgramOptions;

#[macro_export]
macro_rules! load_package_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            let p = load_packages(&LoadPackageOptions {
                paths: vec!["test.k".to_string()],
                load_opts: Some(LoadProgramOptions {
                    k_code_list: vec![$src.to_string()],
                    ..Default::default()
                }),
                load_builtin: false,
                ..Default::default()
            })
            .unwrap();
            insta::assert_snapshot!(format!("{:#?}", p.symbols.values()));
        }
    };
}

load_package_snapshot! {assign_stmt_0, "a = 1"}
load_package_snapshot! {assign_stmt_1, "a = 1 + 1"}
load_package_snapshot! {assign_stmt_2, "a = (1 + 1)"}

load_package_snapshot! {import_stmt_0, r#"import math

a = math.log(10)
"#}
load_package_snapshot! {import_stmt_1, r#"import pkg

a = pkg.a
"#}
