use kcl_error::Position as KCLPos;
use kcl_sema::{
    builtin::BUILTIN_DECORATORS,
    core::global_state::GlobalState,
    ty::{ANY_TYPE_STR, FunctionType, Type},
};
use lsp_types::{Hover, HoverContents, MarkedString};

use crate::goto_def::find_def;

enum MarkedStringType {
    String,
    LanguageString,
}

/// Returns a short text describing element at position.
/// Specifically, the doc for schema and schema attr(todo)
pub fn hover(kcl_pos: &KCLPos, gs: &GlobalState) -> Option<lsp_types::Hover> {
    let mut docs: Vec<(String, MarkedStringType)> = vec![];

    let def = find_def(kcl_pos, gs, true);
    if let Some(def_ref) = def
        && let Some(obj) = gs.get_symbols().get_symbol(def_ref)
    {
        match def_ref.get_kind() {
            kcl_sema::core::symbol::SymbolKind::Schema => {
                if let Some(ty) = &obj.get_sema_info().ty {
                    // Build hover content for schema definition
                    // Schema Definition hover
                    // ```
                    // pkg
                    // ----------------
                    // schema Foo(Base)[param: type]:
                    //     attr1: type
                    //     attr2? type = defalut_value
                    // -----------------
                    // doc
                    // ```
                    let schema_ty = ty.into_schema_type();
                    let (pkgpath, rest_sign) = schema_ty.schema_ty_signature_str();
                    if !pkgpath.is_empty() {
                        docs.push((pkgpath.clone(), MarkedStringType::String));
                    }

                    // The attr of schema_ty does not contain the attrs from inherited base schema.
                    // Use the api provided by GlobalState to get all attrs
                    let module_info = gs.get_packages().get_module_info(&kcl_pos.filename);
                    let schema_attrs = obj.get_all_attributes(gs.get_symbols(), module_info);
                    let mut attrs: Vec<String> = vec![];
                    for schema_attr in schema_attrs {
                        if let kcl_sema::core::symbol::SymbolKind::Attribute =
                            schema_attr.get_kind()
                        {
                            let attr = gs.get_symbols().get_symbol(schema_attr).unwrap();
                            let name = attr.get_name();
                            let attr_symbol =
                                gs.get_symbols().get_attr_symbol(schema_attr).unwrap();
                            let default_value_content = match attr_symbol.get_default_value() {
                                Some(s) => format!(" = {}", s),
                                None => "".to_string(),
                            };
                            let attr_ty_str = match &attr.get_sema_info().ty {
                                Some(ty) => ty_hover_content(ty),
                                None => ANY_TYPE_STR.to_string(),
                            };
                            attrs.push(format!(
                                "    {}{}: {}{}",
                                name,
                                if attr_symbol.is_optional() { "?" } else { "" },
                                attr_ty_str,
                                default_value_content
                            ));
                        }
                    }

                    let merged_doc = format!("{}\n{}", rest_sign.clone(), attrs.join("\n"));
                    docs.push((merged_doc, MarkedStringType::LanguageString));

                    if !schema_ty.doc.is_empty() {
                        docs.push((schema_ty.doc.clone(), MarkedStringType::String));

                        // Add examples to the hover content
                        if !schema_ty.examples.is_empty() {
                            let examples = schema_ty
                                .examples
                                .values()
                                .map(|example| format!("{}\n", example.value))
                                .collect::<Vec<String>>()
                                .join("\n");
                            docs.push((examples, MarkedStringType::LanguageString));
                        }
                    }
                }
            }
            kcl_sema::core::symbol::SymbolKind::Attribute => {
                let sema_info = obj.get_sema_info();
                let attr_symbol = gs.get_symbols().get_attr_symbol(def_ref).unwrap();
                let default_value_content = match attr_symbol.get_default_value() {
                    Some(s) => format!(" = {}", s),
                    None => "".to_string(),
                };
                if let Some(ty) = &sema_info.ty {
                    docs.push((
                        format!(
                            "{}: {}{}",
                            &obj.get_name(),
                            ty.ty_hint(),
                            default_value_content
                        ),
                        MarkedStringType::LanguageString,
                    ));
                    if let Some(doc) = &sema_info.doc
                        && !doc.is_empty()
                    {
                        docs.push((doc.clone(), MarkedStringType::String));
                    }
                }
            }
            kcl_sema::core::symbol::SymbolKind::Value
            | kcl_sema::core::symbol::SymbolKind::Function => {
                if let Some(ty) = &obj.get_sema_info().ty {
                    match &ty.kind {
                        kcl_sema::ty::TypeKind::Function(func_ty) => {
                            docs.append(&mut build_func_hover_content(
                                func_ty.clone(),
                                obj.get_name().clone(),
                            ));
                        }
                        _ => {
                            docs.push((
                                format!("{}: {}", &obj.get_name(), ty.ty_str()),
                                MarkedStringType::LanguageString,
                            ));
                        }
                    }
                }
            }
            kcl_sema::core::symbol::SymbolKind::Expression => return None,
            kcl_sema::core::symbol::SymbolKind::Comment => return None,
            kcl_sema::core::symbol::SymbolKind::Decorator => {
                match BUILTIN_DECORATORS.get(&obj.get_name()) {
                    Some(ty) => {
                        let mut hover_content =
                            build_func_hover_content(ty.into_func_type(), obj.get_name().clone());

                        docs.append(&mut hover_content);
                    }
                    None => todo!(),
                }
            }
            _ => {
                let ty_str = match &obj.get_sema_info().ty {
                    Some(ty) => ty.ty_str(),
                    None => "".to_string(),
                };
                docs.push((
                    format!("{}: {}", &obj.get_name(), ty_str),
                    MarkedStringType::LanguageString,
                ));
            }
        }
    }
    docs_to_hover(docs)
}

fn ty_hover_content(ty: &Type) -> String {
    ty.ty_hint()
}

// Convert doc to Marked String. This function will convert docs to Markedstrings
fn convert_doc_to_marked_string(doc: &(String, MarkedStringType)) -> MarkedString {
    match doc.1 {
        MarkedStringType::String => MarkedString::String(doc.0.clone()),
        MarkedStringType::LanguageString => {
            MarkedString::LanguageString(lsp_types::LanguageString {
                language: "KCL".to_owned(),
                value: doc.0.clone(),
            })
        }
    }
}

// Convert docs to Hover. This function will convert to
// None, Scalar or Array according to the number of positions
fn docs_to_hover(docs: Vec<(String, MarkedStringType)>) -> Option<lsp_types::Hover> {
    let mut all_docs: Vec<MarkedString> = Vec::new();

    for doc in docs {
        all_docs.push(convert_doc_to_marked_string(&doc));
    }

    match all_docs.len() {
        0 => None,
        1 => Some(Hover {
            contents: HoverContents::Scalar(all_docs.remove(0)),
            range: None,
        }),
        _ => Some(Hover {
            contents: HoverContents::Array(all_docs),
            range: None,
        }),
    }
}

// Build hover content for function call
// ```
// pkg
// -----------------
// function func_name(arg1: type, arg2: type, ..) -> type
// -----------------
// doc
// ```
fn build_func_hover_content(
    func_ty: FunctionType,
    name: String,
) -> Vec<(String, MarkedStringType)> {
    let mut docs: Vec<(String, MarkedStringType)> = vec![];
    if let Some(ty) = &func_ty.self_ty {
        let self_ty = format!("{}\n\n", ty.ty_str());
        docs.push((self_ty, MarkedStringType::String));
    }

    let mut sig = format!("function {}(", name);
    if func_ty.params.is_empty() {
        sig.push(')');
    } else {
        for (i, p) in func_ty.params.iter().enumerate() {
            let default_value = match &p.default_value {
                Some(s) => format!(" = {}", s),
                None => "".to_string(),
            };
            sig.push_str(&format!("{}: {}{}", p.name, p.ty.ty_str(), default_value));

            if i != func_ty.params.len() - 1 {
                sig.push_str(", ");
            }
        }
        sig.push(')');
    }
    sig.push_str(&format!(" -> {}", func_ty.return_ty.ty_str()));
    docs.push((sig, MarkedStringType::LanguageString));

    if !func_ty.doc.is_empty() {
        docs.push((
            func_ty.doc.clone().replace('\n', "\n\n"),
            MarkedStringType::String,
        ));
    }
    docs
}

#[cfg(test)]
mod tests {
    use crate::hover::MarkedStringType;
    use crate::hover::docs_to_hover;

    use kcl_error::Position as KCLPos;
    use lsp_types::MarkedString;
    use proc_macro_crate::bench_test;

    use crate::tests::compile_test_file;

    use super::hover;

    #[macro_export]
    macro_rules! hover_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr) => {
            #[test]
            fn $name() {
                let (file, _program, _, gs, _) = compile_test_file($file);

                let pos = KCLPos {
                    filename: file,
                    line: $line,
                    column: Some($column),
                };
                let res = hover(&pos, &gs);
                insta::assert_snapshot!(format!("{:#?}", res));
            }
        };
    }

    #[test]
    #[bench_test]
    fn test_docs_to_hover_multiple_docs() {
        // Given multiple documentation strings
        let docs = vec![
            (
                "Documentation string 1".to_string(),
                MarkedStringType::String,
            ),
            (
                "Documentation string 2".to_string(),
                MarkedStringType::String,
            ),
            (
                "Documentation string 3".to_string(),
                MarkedStringType::String,
            ),
        ];

        // When converting to hover content
        let hover = docs_to_hover(docs);

        // Then the result should be a Hover object with an Array of MarkedString::String
        assert!(hover.is_some());
        let hover = hover.unwrap();
        match hover.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 3);
                assert_eq!(
                    vec[0],
                    MarkedString::String("Documentation string 1".to_string())
                );
                assert_eq!(
                    vec[1],
                    MarkedString::String("Documentation string 2".to_string())
                );
                assert_eq!(
                    vec[2],
                    MarkedString::String("Documentation string 3".to_string())
                );
            }
            _ => panic!("Unexpected hover contents"),
        }
    }

    // p = pkg.Person
    hover_test_snapshot!(
        schema_doc_hover_test,
        "src/test_data/goto_def_test/goto_def.k",
        4,
        11
    );

    hover_test_snapshot!(
        schema_doc_hover_attr_test,
        "src/test_data/goto_def_test/goto_def.k",
        5,
        7
    );

    hover_test_snapshot!(
        schema_doc_hover_test1,
        "src/test_data/hover_test/hover.k",
        16,
        8
    );

    hover_test_snapshot!(
        schema_attr_hover_test,
        "src/test_data/hover_test/hover.k",
        17,
        7
    );

    hover_test_snapshot!(
        schema_optional_attr_hover_test,
        "src/test_data/hover_test/hover.k",
        18,
        7
    );

    hover_test_snapshot!(
        lambda_doc_hover_test,
        "src/test_data/hover_test/lambda.k",
        1,
        1
    );

    // base64.encode("1")
    hover_test_snapshot!(func_def_hover, "src/test_data/hover_test/hover.k", 22, 18);

    // "a".count()
    hover_test_snapshot!(
        str_func_def_hover,
        "src/test_data/hover_test/hover.k",
        23,
        14
    );

    // print(1)
    hover_test_snapshot!(
        builtin_func_def_hover,
        "src/test_data/hover_test/hover.k",
        25,
        4
    );

    hover_test_snapshot!(
        complex_select_hover,
        "src/test_data/hover_test/fib.k",
        14,
        22
    );

    hover_test_snapshot!(
        assignment_ty_in_lambda_hover,
        "src/test_data/hover_test/ty_in_lambda.k",
        3,
        8
    );

    hover_test_snapshot!(
        str_var_func_hover,
        "src/test_data/hover_test/hover.k",
        28,
        12
    );

    hover_test_snapshot!(
        import_pkg_hover,
        "src/test_data/hover_test/import_pkg.k",
        3,
        7
    );

    hover_test_snapshot!(
        expr_after_config_if_hover,
        "src/test_data/hover_test/hover.k",
        41,
        13
    );

    hover_test_snapshot!(
        schema_scope_variable_hover,
        "src/test_data/hover_test/fib.k",
        3,
        11
    );

    hover_test_snapshot!(
        decorator_hover,
        "src/test_data/hover_test/decorator.k",
        1,
        1
    );

    hover_test_snapshot!(
        schema_attr_decorator_hover,
        "src/test_data/hover_test/decorator.k",
        3,
        8
    );

    hover_test_snapshot!(
        inherit_schema_attr_hover,
        "src/test_data/hover_test/inherit.k",
        5,
        9
    );

    // n1: Name = {name = 1}
    hover_test_snapshot!(
        dict_key_in_schema,
        "src/test_data/hover_test/dict_key_in_schema/dict_key_in_schema.k",
        5,
        5
    );

    // n2 = Name{name: 1}
    hover_test_snapshot!(
        dict_key_in_schema_expr,
        "src/test_data/hover_test/dict_key_in_schema/dict_key_in_schema.k",
        9,
        5
    );

    // n3: Name = Name{name: 1}
    hover_test_snapshot!(
        dict_key_in_typed_schema_expr,
        "src/test_data/hover_test/dict_key_in_schema/dict_key_in_schema.k",
        13,
        5
    );

    hover_test_snapshot!(
        schema_doc_with_examples_hover_test,
        "src/test_data/hover_test/schema_with_examples.k",
        1,
        8
    );
}
