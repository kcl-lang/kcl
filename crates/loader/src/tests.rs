use crate::option::list_options;
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
load_package_snapshot! {builtin_call_0, r#"print("hello world")"#}
load_package_snapshot! {builtin_call_1, r#"a = option("key", type="str", required=True)"#}
load_package_snapshot! {builtin_call_2, r#"opt = option

a = opt("key", type="str", required=True)
"#}

#[macro_export]
macro_rules! list_options_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            let options = list_options(&LoadPackageOptions {
                paths: vec!["test.k".to_string()],
                load_opts: Some(LoadProgramOptions {
                    k_code_list: vec![$src.to_string()],
                    ..Default::default()
                }),
                load_builtin: false,
                ..Default::default()
            })
            .unwrap();
            insta::assert_snapshot!(format!("{:#?}", options));
        }
    };
}
list_options_snapshot! {list_options_0, r#"a = option("key", type="int")"#}
list_options_snapshot! {list_options_1, r#"opt = option

a = opt("key1", type="str", required=True)
b = option("key2", type="int")
"#}
list_options_snapshot! {list_options_2, r#"
a = option("key1", type="str", required=True, default="value", help="help me")
if True:
    b = option("key2")
"#}
list_options_snapshot! {list_options_3, r#"
a = option("key1", type="int", required=False, default=123, help="help me")
"#}
