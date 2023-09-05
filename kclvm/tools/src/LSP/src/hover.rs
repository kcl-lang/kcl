use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_error::Position as KCLPos;
use kclvm_sema::resolver::scope::{ProgramScope, ScopeObjectKind};
use lsp_types::{Hover, HoverContents, MarkedString};

use crate::goto_def::find_def;

/// Returns a short text describing element at position.
/// Specifically, the doc for schema and schema attr(todo)
pub(crate) fn hover(
    program: &Program,
    kcl_pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::Hover> {
    match program.pos_to_stmt(kcl_pos) {
        Some(node) => {
            let mut docs: IndexSet<String> = IndexSet::new();
            if let Some(def) = find_def(node, kcl_pos, prog_scope) {
                if let crate::goto_def::Definition::Object(obj) = def {
                    match obj.kind {
                        ScopeObjectKind::Definition => {
                            // Schema Definition hover
                            // ```
                            // pkg
                            // schema Foo(Base)
                            // -----------------
                            // doc
                            // -----------------
                            // Attributes:
                            // attr1: type
                            // attr2? type
                            // ```
                            let schema_ty = obj.ty.into_schema_type();
                            let base: String = if let Some(base) = schema_ty.base {
                                format!("({})", base.name)
                            } else {
                                "".to_string()
                            };
                            docs.insert(format!(
                                "{}\n\nschema {}{}",
                                schema_ty.pkgpath, schema_ty.name, base
                            ));
                            if !schema_ty.doc.is_empty() {
                                docs.insert(schema_ty.doc.clone());
                            }
                            let mut attrs = vec!["Attributes:".to_string()];
                            for (name, attr) in schema_ty.attrs {
                                attrs.push(format!(
                                    "{}{}:{}",
                                    name,
                                    if attr.is_optional { "?" } else { "" },
                                    format!(" {}", attr.ty.ty_str()),
                                ));
                            }
                            docs.insert(attrs.join("\n\n"));
                        }
                        // todo: hover ScopeObjectKind::Attribute optional, default value
                        _ => {
                            // Variable
                            // ```
                            // name: type
                            //```
                            docs.insert(format!("{}: {}", obj.name, obj.ty.ty_str()));
                            if let Some(doc) = obj.doc {
                                docs.insert(doc);
                            }
                        }
                    }
                }
            }
            docs_to_hover(docs)
        }
        None => None,
    }
}

// Convert docs to Hover. This function will convert to
// None, Scalar or Array according to the number of positions
fn docs_to_hover(docs: IndexSet<String>) -> Option<lsp_types::Hover> {
    match docs.len() {
        0 => None,
        1 => Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(docs[0].clone())),
            range: None,
        }),
        _ => Some(Hover {
            contents: HoverContents::Array(
                docs.iter()
                    .map(|doc| MarkedString::String(doc.clone()))
                    .collect(),
            ),
            range: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kclvm_error::Position as KCLPos;
    use lsp_types::MarkedString;
    use proc_macro_crate::bench_test;

    use crate::tests::compile_test_file;

    use super::hover;

    #[test]
    #[bench_test]
    fn schema_doc_hover_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test hover of schema doc: p = pkg.Person
        let pos = KCLPos {
            filename: file.clone(),
            line: 4,
            column: Some(11),
        };
        let got = hover(&program, &pos, &prog_scope).unwrap();
        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "pkg\n\nschema Person");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "hover doc test");
                }
                if let MarkedString::String(s) = vec[2].clone() {
                    assert_eq!(
                        s,
                        "Attributes:\n\n__settings__?: {str:any}\n\nname: str\n\nage: int"
                    );
                }
            }
            _ => unreachable!("test error"),
        }
        let pos = KCLPos {
            filename: file,
            line: 5,
            column: Some(7),
        };
        let got = hover(&program, &pos, &prog_scope).unwrap();
        match got.contents {
            lsp_types::HoverContents::Scalar(marked_string) => {
                if let MarkedString::String(s) = marked_string {
                    assert_eq!(s, "name: str");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn schema_doc_hover_test1() {
        let (file, program, prog_scope, _) = compile_test_file("src/test_data/hover_test/hover.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 15,
            column: Some(11),
        };
        let got = hover(&program, &pos, &prog_scope).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "__main__\n\nschema Person");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "hover doc test");
                }
                if let MarkedString::String(s) = vec[2].clone() {
                    assert_eq!(
                        s,
                        "Attributes:\n\n__settings__?: {str:any}\n\nname: str\n\nage?: int"
                    );
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn schema_attr_hover_test() {
        let (file, program, prog_scope, _) = compile_test_file("src/test_data/hover_test/hover.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 16,
            column: Some(11),
        };
        let got = hover(&program, &pos, &prog_scope).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "name: str");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "name doc test");
                }
            }
            _ => unreachable!("test error"),
        }

        let pos = KCLPos {
            filename: file.clone(),
            line: 17,
            column: Some(11),
        };
        let got = hover(&program, &pos, &prog_scope).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "age: int");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "age doc test");
                }
            }
            _ => unreachable!("test error"),
        }
    }
}
