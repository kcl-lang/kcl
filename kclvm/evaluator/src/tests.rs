use crate::Evaluator;
use kclvm_loader::{load_packages, LoadPackageOptions};
use kclvm_parser::LoadProgramOptions;

#[macro_export]
macro_rules! evaluator_snapshot {
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
            let evaluator = Evaluator::new(&p.program);
            insta::assert_snapshot!(format!("{:#?}", evaluator.run().unwrap().1));
        }
    };
}

evaluator_snapshot! {assign_stmt_0, "a = 1"}
evaluator_snapshot! {assign_stmt_1, "a = 1 + 1"}
evaluator_snapshot! {assign_stmt_2, "a = (1 + 2)"}
evaluator_snapshot! {assign_stmt_3, r#"a = 1
b = a + 1
"#}
