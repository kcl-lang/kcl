//! Complete for KCL
//! Now supports code completion in trigger mode (triggered when user enters `.`, `:` and `=`), schema attr and global variables
//! and the content of the completion includes:
//! + variable
//! + schema attr name
//! + dot(.)
//!     + import path
//!     + schema attr
//!     + builtin function(str function)
//!     + definitions in pkg
//!     + system module functions
//! + assign(=, :)
//!     + schema attr value
//!     + variable value
//! + new line
//!     + schema init

use std::io;
use std::{fs, path::Path};

use crate::goto_def::find_def_with_gs;
use indexmap::IndexSet;
use kclvm_ast::ast::{self, ImportStmt, Program, Stmt};
use kclvm_ast::MAIN_PKG;
use kclvm_config::modfile::KCL_FILE_EXTENSION;
use kclvm_sema::core::global_state::GlobalState;

use kclvm_driver::get_real_path_from_external;
use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::{BUILTIN_FUNCTIONS, STANDARD_SYSTEM_MODULES};
use kclvm_sema::core::package::ModuleInfo;
use kclvm_sema::core::symbol::SymbolKind;
use kclvm_sema::resolver::doc::{parse_doc_string, Doc};
use kclvm_sema::ty::{FunctionType, SchemaType, Type, TypeKind};
use lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

use crate::util::{inner_most_expr_in_stmt, is_in_docstring, is_in_schema_expr};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum KCLCompletionItemKind {
    Function,
    Variable,
    File,
    Dir,
    Schema,
    SchemaAttr,
    Module,
    Doc,
}

impl From<KCLCompletionItemKind> for CompletionItemKind {
    fn from(val: KCLCompletionItemKind) -> Self {
        match val {
            KCLCompletionItemKind::Function => CompletionItemKind::FUNCTION,
            KCLCompletionItemKind::Variable => CompletionItemKind::VARIABLE,
            KCLCompletionItemKind::File => CompletionItemKind::FILE,
            KCLCompletionItemKind::Schema => CompletionItemKind::CLASS,
            KCLCompletionItemKind::SchemaAttr => CompletionItemKind::FIELD,
            KCLCompletionItemKind::Module => CompletionItemKind::MODULE,
            KCLCompletionItemKind::Dir => CompletionItemKind::FOLDER,
            KCLCompletionItemKind::Doc => CompletionItemKind::SNIPPET,
        }
    }
}

/// Abstraction of CompletionItem in KCL
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub(crate) struct KCLCompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub kind: Option<KCLCompletionItemKind>,
    pub insert_text: Option<String>,
}

/// Computes completions at the given position.
pub(crate) fn completion(
    trigger_character: Option<char>,
    program: &Program,
    pos: &KCLPos,
    gs: &GlobalState,
) -> Option<lsp_types::CompletionResponse> {
    match trigger_character {
        Some(c) => match c {
            '.' => completion_dot(program, pos, gs),
            '=' | ':' => completion_assign(pos, gs),
            '\n' => completion_newline(program, pos, gs),
            _ => None,
        },
        None => {
            let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
            // Complete builtin pkgs if in import stmt
            completions.extend(completion_import_builtin_pkg(program, pos));
            if !completions.is_empty() {
                return Some(into_completion_items(&completions).into());
            }

            // Complete import pkgs name
            if let Some(pkg_info) = gs.get_packages().get_module_info(&pos.filename) {
                completions.extend(pkg_info.get_imports().keys().map(|key| KCLCompletionItem {
                    label: key.clone(),
                    detail: None,
                    documentation: None,
                    kind: Some(KCLCompletionItemKind::Module),
                    insert_text: None,
                }));
            }

            if let Some(scope) = gs.look_up_scope(pos) {
                // Complete builtin functions in root scope and lambda
                match scope.get_kind() {
                    kclvm_sema::core::scope::ScopeKind::Local => {
                        if let Some(local_scope) = gs.get_scopes().try_get_local_scope(&scope) {
                            match local_scope.get_kind() {
                                kclvm_sema::core::scope::LocalSymbolScopeKind::Lambda => {
                                    completions.extend(BUILTIN_FUNCTIONS.iter().map(
                                        |(name, ty)| KCLCompletionItem {
                                            label: func_ty_complete_label(
                                                name,
                                                &ty.into_func_type(),
                                            ),
                                            detail: Some(
                                                ty.into_func_type().func_signature_str(name),
                                            ),
                                            documentation: ty.ty_doc(),
                                            kind: Some(KCLCompletionItemKind::Function),
                                            insert_text: Some(func_ty_complete_insert_text(
                                                name,
                                                &ty.into_func_type(),
                                            )),
                                        },
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                    kclvm_sema::core::scope::ScopeKind::Root => {
                        completions.extend(BUILTIN_FUNCTIONS.iter().map(|(name, ty)| {
                            KCLCompletionItem {
                                label: func_ty_complete_label(name, &ty.into_func_type()),
                                detail: Some(ty.into_func_type().func_signature_str(name)),
                                documentation: ty.ty_doc(),
                                kind: Some(KCLCompletionItemKind::Function),
                                insert_text: Some(func_ty_complete_insert_text(
                                    name,
                                    &ty.into_func_type(),
                                )),
                            }
                        }));
                    }
                }

                // Complete all usable symbol obj in inner most scope
                if let Some(defs) = gs.get_all_defs_in_scope(scope) {
                    for symbol_ref in defs {
                        match gs.get_symbols().get_symbol(symbol_ref) {
                            Some(def) => {
                                let sema_info = def.get_sema_info();
                                let name = def.get_name();
                                match &sema_info.ty {
                                    Some(ty) => match symbol_ref.get_kind() {
                                        SymbolKind::Schema => {
                                            let schema_ty = ty.into_schema_type();
                                            // complete schema type
                                            completions.insert(schema_ty_to_type_complete_item(
                                                &schema_ty,
                                            ));
                                            // complete schema value
                                            completions.insert(schema_ty_to_value_complete_item(
                                                &schema_ty,
                                            ));
                                        }
                                        SymbolKind::Package => {
                                            completions.insert(KCLCompletionItem {
                                                label: name,
                                                detail: Some(ty.ty_str()),
                                                documentation: sema_info.doc.clone(),
                                                kind: Some(KCLCompletionItemKind::Module),
                                                insert_text: None,
                                            });
                                        }
                                        _ => {
                                            let detail = match &ty.kind {
                                                TypeKind::Function(func_ty) => {
                                                    func_ty.func_signature_str(&name)
                                                }
                                                _ => ty.ty_str(),
                                            };
                                            completions.insert(KCLCompletionItem {
                                                label: name,
                                                detail: Some(detail),
                                                documentation: sema_info.doc.clone(),
                                                kind: type_to_item_kind(ty),
                                                insert_text: None,
                                            });
                                        }
                                    },
                                    None => {}
                                }
                            }
                            None => {}
                        }
                    }
                }
            }

            Some(into_completion_items(&completions).into())
        }
    }
}

fn completion_dot(
    program: &Program,
    pos: &KCLPos,
    gs: &GlobalState,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();

    // get pre position of trigger character '.'
    let pre_pos = KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: pos.column.map(|c| if c >= 1 { c - 1 } else { 0 }),
    };

    if let Some(stmt) = program.pos_to_stmt(&pre_pos) {
        match stmt.node {
            Stmt::Import(stmt) => return completion_import(&stmt, pos, program),
            _ => {
                let (expr, _) = inner_most_expr_in_stmt(&stmt.node, pos, None);
                if let Some(node) = expr {
                    match node.node {
                        // if the complete trigger character in string, skip it
                        ast::Expr::StringLit(_) | ast::Expr::JoinedString(_) => {
                            return Some(into_completion_items(&items).into())
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // look_up_exact_symbol
    let mut def = find_def_with_gs(&pre_pos, gs, true);
    if def.is_none() {
        def = find_def_with_gs(pos, gs, false);
    }

    match def {
        Some(def_ref) => {
            if let Some(def) = gs.get_symbols().get_symbol(def_ref) {
                let module_info = gs.get_packages().get_module_info(&pos.filename);
                let attrs = def.get_all_attributes(gs.get_symbols(), module_info);
                for attr in attrs {
                    let attr_def = gs.get_symbols().get_symbol(attr);
                    if let Some(attr_def) = attr_def {
                        let sema_info = attr_def.get_sema_info();
                        let name = attr_def.get_name();
                        match &sema_info.ty {
                            Some(attr_ty) => {
                                let label: String = match &attr_ty.kind {
                                    TypeKind::Function(func_ty) => {
                                        func_ty_complete_label(&name, func_ty)
                                    }
                                    _ => name.clone(),
                                };
                                let insert_text = match &attr_ty.kind {
                                    TypeKind::Function(func_ty) => {
                                        Some(func_ty_complete_insert_text(&name, func_ty))
                                    }
                                    _ => None,
                                };
                                let kind = match &def.get_sema_info().ty {
                                    Some(symbol_ty) => match &symbol_ty.kind {
                                        TypeKind::Schema(_) => {
                                            Some(KCLCompletionItemKind::SchemaAttr)
                                        }
                                        _ => type_to_item_kind(attr_ty),
                                    },
                                    None => type_to_item_kind(attr_ty),
                                };
                                let documentation = match &sema_info.doc {
                                    Some(doc) => {
                                        if doc.is_empty() {
                                            None
                                        } else {
                                            Some(doc.clone())
                                        }
                                    }
                                    None => None,
                                };
                                items.insert(KCLCompletionItem {
                                    label,
                                    detail: Some(format!("{}: {}", name, attr_ty.ty_str())),
                                    documentation,
                                    kind,
                                    insert_text,
                                });
                            }
                            None => {
                                items.insert(KCLCompletionItem {
                                    label: name,
                                    detail: None,
                                    documentation: None,
                                    kind: None,
                                    insert_text: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }
    Some(into_completion_items(&items).into())
}

/// Get completion items for trigger '=' or ':'
/// Now, just completion for schema attr value
fn completion_assign(pos: &KCLPos, gs: &GlobalState) -> Option<lsp_types::CompletionResponse> {
    let mut items = IndexSet::new();
    if let Some(symbol_ref) = find_def_with_gs(pos, gs, false) {
        if let Some(symbol) = gs.get_symbols().get_symbol(symbol_ref) {
            if let Some(def) = symbol.get_definition() {
                match def.get_kind() {
                    SymbolKind::Attribute => {
                        let sema_info = symbol.get_sema_info();
                        match &sema_info.ty {
                            Some(ty) => {
                                items.extend(
                                    ty_complete_label(
                                        ty,
                                        gs.get_packages().get_module_info(&pos.filename),
                                    )
                                    .iter()
                                    .map(|label| {
                                        KCLCompletionItem {
                                            label: format!(" {}", label),
                                            detail: Some(format!(
                                                "{}: {}",
                                                symbol.get_name(),
                                                ty.ty_str()
                                            )),
                                            kind: Some(KCLCompletionItemKind::Variable),
                                            documentation: sema_info.doc.clone(),
                                            insert_text: None,
                                        }
                                    }),
                                );
                                return Some(into_completion_items(&items).into());
                            }
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

fn completion_newline(
    program: &Program,
    pos: &KCLPos,
    gs: &GlobalState,
) -> Option<lsp_types::CompletionResponse> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();

    if let Some((doc, schema)) = is_in_docstring(program, pos) {
        let doc = parse_doc_string(&doc.node);
        if doc.summary.is_empty() && doc.attrs.is_empty() && doc.examples.is_empty() {
            // empty docstring, provide total completion
            let doc_parsed = Doc::new_from_schema_stmt(&schema);
            let label = doc_parsed.to_doc_string();
            // generate docstring from doc
            completions.insert(KCLCompletionItem {
                label,
                detail: Some("generate docstring".to_string()),
                documentation: Some(format!("docstring for {}", schema.name.node.clone())),
                kind: Some(KCLCompletionItemKind::Doc),
                insert_text: None,
            });
        }
        return Some(into_completion_items(&completions).into());
    }

    // todo: judge based on scope kind instead of `is_in_schema_expr`
    if let Some(_) = is_in_schema_expr(program, pos) {
        // Complete schema attr when input newline in schema
        if let Some(scope) = gs.look_up_scope(pos) {
            if let Some(defs) = gs.get_all_defs_in_scope(scope) {
                for symbol_ref in defs {
                    match gs.get_symbols().get_symbol(symbol_ref) {
                        Some(def) => {
                            let sema_info = def.get_sema_info();
                            let name = def.get_name();
                            match symbol_ref.get_kind() {
                                SymbolKind::Attribute => {
                                    completions.insert(KCLCompletionItem {
                                        label: name.clone(),
                                        detail: sema_info
                                            .ty
                                            .as_ref()
                                            .map(|ty| format!("{}: {}", name, ty.ty_str())),
                                        documentation: match &sema_info.doc {
                                            Some(doc) => {
                                                if doc.is_empty() {
                                                    None
                                                } else {
                                                    Some(doc.clone())
                                                }
                                            }
                                            None => None,
                                        },
                                        kind: Some(KCLCompletionItemKind::SchemaAttr),
                                        insert_text: None,
                                    });
                                }
                                _ => {}
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
    Some(into_completion_items(&completions).into())
}

fn completion_import_builtin_pkg(program: &Program, pos: &KCLPos) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
    // completion position not contained in import stmt
    // import <space>  <cursor>
    // |             | |  <- input `m` here for complete `math`
    // |<----------->| <- import stmt only contains this range, so we need to check the beginning of line
    let line_start_pos = &KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: Some(0),
    };

    if let Some(node) = program.pos_to_stmt(line_start_pos) {
        if let Stmt::Import(_) = node.node {
            completions.extend(STANDARD_SYSTEM_MODULES.iter().map(|s| KCLCompletionItem {
                label: s.to_string(),
                detail: None,
                documentation: None,
                kind: Some(KCLCompletionItemKind::Module),
                insert_text: None,
            }))
        }
    }
    completions
}

/// Complete schema value
///
/// ```no_check
/// #[cfg(not(test))]
/// p = P<cursor>
/// ```
/// complete to
/// ```no_check
/// #[cfg(not(test))]
/// p = Person(param1, param2){}<cursor>
/// ```
fn schema_ty_to_value_complete_item(schema_ty: &SchemaType) -> KCLCompletionItem {
    let param = schema_ty.func.params.clone();
    let label = format!(
        "{}{}{}",
        schema_ty.name.clone(),
        if param.is_empty() {
            "".to_string()
        } else {
            format!(
                "({})",
                param
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        },
        "{}"
    );
    let detail = {
        let mut details = vec![];
        details.push(schema_ty.schema_ty_signature_str());
        details.push("Attributes:".to_string());
        for (name, attr) in &schema_ty.attrs {
            details.push(format!(
                "{}{}: {}",
                name,
                if attr.is_optional { "?" } else { "" },
                attr.ty.ty_str(),
            ));
        }
        details.join("\n")
    };
    let insert_text = format!(
        "{}{}{}",
        schema_ty.name.clone(),
        if param.is_empty() {
            "".to_string()
        } else {
            format!(
                "({})",
                param
                    .iter()
                    .enumerate()
                    .map(|(idx, p)| format!("${{{}:{}}}", idx + 1, p.name.clone()))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        },
        "{}"
    );
    KCLCompletionItem {
        label,
        detail: Some(detail),
        documentation: Some(schema_ty.doc.clone()),
        kind: Some(KCLCompletionItemKind::Schema),
        insert_text: Some(insert_text),
    }
}

/// Complete schema type
///
/// ```no_check
/// #[cfg(not(test))]
/// p: P<cursor>
/// ```
/// complete to
/// ```no_check
/// #[cfg(not(test))]
/// p: Person
/// ```
fn schema_ty_to_type_complete_item(schema_ty: &SchemaType) -> KCLCompletionItem {
    let detail = {
        let mut details = vec![];
        details.push(schema_ty.schema_ty_signature_str());
        details.push("Attributes:".to_string());
        for (name, attr) in &schema_ty.attrs {
            details.push(format!(
                "{}{}: {}",
                name,
                if attr.is_optional { "?" } else { "" },
                attr.ty.ty_str(),
            ));
        }
        details.join("\n")
    };
    KCLCompletionItem {
        label: schema_ty.name.clone(),
        detail: Some(detail),
        documentation: Some(schema_ty.doc.clone()),
        kind: Some(KCLCompletionItemKind::Schema),
        insert_text: None,
    }
}

fn completion_import(
    stmt: &ImportStmt,
    _pos: &KCLPos,
    program: &Program,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();
    let pkgpath = &stmt.path.node;
    let mut real_path =
        Path::new(&program.root).join(pkgpath.replace('.', std::path::MAIN_SEPARATOR_STR));
    if !real_path.exists() {
        real_path =
            get_real_path_from_external(&stmt.pkg_name, pkgpath, program.root.clone().into());
    }
    if real_path.is_dir() {
        if let Ok(entries) = fs::read_dir(real_path) {
            let mut entries = entries
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()
                .unwrap();
            entries.sort();
            for path in entries {
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                if path.is_dir() {
                    items.insert(KCLCompletionItem {
                        label: filename,
                        detail: None,
                        documentation: None,
                        kind: Some(KCLCompletionItemKind::Dir),
                        insert_text: None,
                    });
                } else if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == KCL_FILE_EXTENSION {
                            items.insert(KCLCompletionItem {
                                label: path
                                    .with_extension("")
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                                detail: None,
                                documentation: None,
                                kind: Some(KCLCompletionItemKind::File),
                                insert_text: None,
                            });
                        }
                    }
                }
            }
        }
    }
    Some(into_completion_items(&items).into())
}

fn ty_complete_label(ty: &Type, module: Option<&ModuleInfo>) -> Vec<String> {
    match &ty.kind {
        TypeKind::Bool => vec!["True".to_string(), "False".to_string()],
        TypeKind::BoolLit(b) => {
            vec![if *b {
                "True".to_string()
            } else {
                "False".to_string()
            }]
        }
        TypeKind::IntLit(i) => vec![i.to_string()],
        TypeKind::FloatLit(f) => vec![f.to_string()],
        TypeKind::Str => vec![r#""""#.to_string()],
        TypeKind::StrLit(s) => vec![format!("{:?}", s)],
        TypeKind::List(_) => vec!["[]".to_string()],
        TypeKind::Dict(_) => vec!["{}".to_string()],
        TypeKind::Union(types) => types
            .iter()
            .flat_map(|ty| ty_complete_label(ty, module))
            .collect(),
        TypeKind::Schema(schema) => {
            vec![format!(
                "{}{}{}",
                if schema.pkgpath.is_empty() || schema.pkgpath == MAIN_PKG {
                    "".to_string()
                } else if let Some(m) = module {
                    format!("{}.", pkg_real_name(&schema.pkgpath, m))
                } else {
                    format!("{}.", schema.pkgpath.split('.').last().unwrap())
                },
                schema.name,
                "{}"
            )]
        }
        _ => vec![],
    }
}

/// get pkg_path real name: as_name if not none or pkg last name
fn pkg_real_name(pkg: &String, module: &ModuleInfo) -> String {
    let imports = module.get_imports();
    for (name, import_info) in imports {
        if &import_info.get_fully_qualified_name() == pkg {
            return name;
        }
    }
    pkg.split('.').last().unwrap().to_string()
}

fn func_ty_complete_label(func_name: &String, _func_type: &FunctionType) -> String {
    format!("{}(…)", func_name,)
}

fn func_ty_complete_insert_text(func_name: &String, func_type: &FunctionType) -> String {
    format!(
        "{}({})",
        func_name,
        func_type
            .params
            .iter()
            .enumerate()
            .map(|(idx, param)| format!("${{{}:{}}}", idx + 1, param.name.clone()))
            .collect::<Vec<String>>()
            .join(", "),
    )
}
fn type_to_item_kind(ty: &Type) -> Option<KCLCompletionItemKind> {
    match ty.kind {
        TypeKind::Bool
        | TypeKind::BoolLit(_)
        | TypeKind::Int
        | TypeKind::IntLit(_)
        | TypeKind::Float
        | TypeKind::FloatLit(_)
        | TypeKind::Str
        | TypeKind::StrLit(_)
        | TypeKind::List(_)
        | TypeKind::Dict(_)
        | TypeKind::Union(_)
        | TypeKind::NumberMultiplier(_)
        | TypeKind::Named(_) => Some(KCLCompletionItemKind::Variable),
        TypeKind::Schema(_) => Some(KCLCompletionItemKind::Schema),
        TypeKind::Function(_) => Some(KCLCompletionItemKind::Function),
        TypeKind::Module(_) => Some(KCLCompletionItemKind::Module),
        TypeKind::Void | TypeKind::None | TypeKind::Any => None,
    }
}

pub(crate) fn into_completion_items(items: &IndexSet<KCLCompletionItem>) -> Vec<CompletionItem> {
    items
        .iter()
        .map(|item| CompletionItem {
            label: item.label.clone(),
            detail: item.detail.clone(),
            documentation: item
                .documentation
                .clone()
                .map(lsp_types::Documentation::String),
            kind: item.kind.clone().map(|kind| kind.into()),
            insert_text: item.insert_text.clone(),
            insert_text_format: if item.insert_text.is_some() {
                Some(InsertTextFormat::SNIPPET)
            } else {
                None
            },
            ..Default::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;
    use kclvm_error::Position as KCLPos;
    use kclvm_sema::builtin::{BUILTIN_FUNCTIONS, MATH_FUNCTION_TYPES, STRING_MEMBER_FUNCTIONS};
    use lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, InsertTextFormat};
    use proc_macro_crate::bench_test;

    use crate::{
        completion::{
            completion, func_ty_complete_insert_text, func_ty_complete_label,
            into_completion_items, KCLCompletionItem, KCLCompletionItemKind,
        },
        tests::compile_test_file,
    };

    #[test]
    #[bench_test]
    fn var_completion_test() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/completion.k");

        // test completion for var
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 26,
            column: Some(1),
        };

        let got = completion(None, &program, &pos, &gs).unwrap();
        let mut got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let mut expected_labels: Vec<String> = vec![
            "", // generate from error recovery of "pkg."
            "subpkg", "math", "Person", "Person{}", "P", "P{}", "p", "p1", "p2", "p3", "p4",
            "aaaa",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        expected_labels.extend(
            BUILTIN_FUNCTIONS
                .iter()
                .map(|(name, func)| func_ty_complete_label(name, &func.into_func_type())),
        );
        got_labels.sort();
        expected_labels.sort();

        assert_eq!(got_labels, expected_labels);

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 24,
            column: Some(4),
        };

        let got = completion(None, &program, &pos, &gs).unwrap();
        let mut got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        expected_labels = ["", "age", "math", "name", "subpkg"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        got_labels.sort();
        expected_labels.sort();
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/completion.k");

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 12,
            column: Some(7),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["name", "age"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(12),
        };

        // test completion for str builtin function
        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let got_insert_text: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr
                .iter()
                .map(|item| item.insert_text.clone().unwrap())
                .collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_insert_text: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_insert_text(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_insert_text, expected_insert_text);

        // test completion for import pkg path
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["file1", "file2", "subpkg"];
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg' schema
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 16,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["Person1"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 19,
            column: Some(5),
        };
        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = MATH_FUNCTION_TYPES
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        // test completion for literal str builtin function
        let pos = KCLPos {
            filename: file.clone(),
            line: 21,
            column: Some(4),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(11),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["a"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test_without_dot() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/without_dot/completion.k");

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 12,
            column: Some(7),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["name", "age"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(12),
        };

        // test completion for str builtin function
        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg path
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["file1", "file2", "subpkg"];
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg' schema
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 16,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["Person1"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 19,
            column: Some(5),
        };
        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = MATH_FUNCTION_TYPES
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let got_insert_text: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr
                .iter()
                .map(|item| item.insert_text.clone().unwrap())
                .collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_insert_text: Vec<String> = MATH_FUNCTION_TYPES
            .iter()
            .map(|(name, ty)| func_ty_complete_insert_text(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_insert_text, expected_insert_text);

        // test completion for literal str builtin function
        let pos = KCLPos {
            filename: file.clone(),
            line: 21,
            column: Some(4),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let got_insert_text: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr
                .iter()
                .map(|item| item.insert_text.clone().unwrap())
                .collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_insert_text: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_insert_text(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_insert_text, expected_insert_text);

        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(11),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["a"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn import_builtin_package() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/import/builtin_pkg.k");
        let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();

        // test completion for builtin packages
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(8),
        };

        let got = completion(None, &program, &pos, &gs).unwrap();
        let _got_labels: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        items.extend(
            [
                "collection",
                "net",
                "manifests",
                "math",
                "datetime",
                "regex",
                "yaml",
                "json",
                "crypto",
                "base64",
                "units",
                "file",
            ]
            .iter()
            .map(|name| KCLCompletionItem {
                label: name.to_string(),
                kind: Some(KCLCompletionItemKind::Module),
                detail: None,
                documentation: None,
                insert_text: None,
            })
            .collect::<IndexSet<KCLCompletionItem>>(),
        );
        let expect: CompletionResponse = into_completion_items(&items).into();
        assert_eq!(got, expect);
    }

    #[test]
    #[bench_test]
    fn attr_value_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/assign/completion.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(6),
        };

        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" True", " False"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 16,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" \"abc\"", " \"def\""];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 18,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" []"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 20,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" 1"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 22,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" True"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 24,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" {}"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 26,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" sub.Person1{}"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn schema_sig_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/schema/schema.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 7,
            column: Some(5),
        };

        let mut got = completion(None, &program, &pos, &gs).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                assert_eq!(
                    arr.iter().find(|item| item.label == "Person(b){}").unwrap(),
                    &CompletionItem {
                        label: "Person(b){}".to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some(
                            "__main__\n\nschema Person[b: int](Base)\nAttributes:\nc: int"
                                .to_string()
                        ),
                        documentation: Some(lsp_types::Documentation::String("".to_string())),
                        insert_text: Some("Person(${1:b}){}".to_string()),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn schema_attr_newline_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/newline/newline.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 8,
            column: Some(4),
        };

        let mut got = completion(Some('\n'), &program, &pos, &gs).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                arr.sort_by(|a, b| a.label.cmp(&b.label));
                assert_eq!(
                    arr[1],
                    CompletionItem {
                        label: "c".to_string(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some("c: int".to_string()),
                        documentation: None,
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }

        // not complete in schema stmt
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 5,
            column: Some(4),
        };
        let got = completion(Some('\n'), &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert!(arr.is_empty())
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn schema_docstring_newline_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/newline/docstring_newline.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 3,
            column: Some(4),
        };

        let mut got = completion(Some('\n'), &program, &pos, &gs).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                arr.sort_by(|a, b| a.label.cmp(&b.label));
                assert_eq!(
                    arr[0],
                    CompletionItem {
                        label: "\n\nAttributes\n----------\nname: \nworkloadType: \nreplica: \n\nExamples\n--------\n".to_string(),
                        detail: Some("generate docstring".to_string()),
                        kind: Some(CompletionItemKind::SNIPPET),
                        documentation: Some(lsp_types::Documentation::String("docstring for Server".to_string())),
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn str_dot_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/lit_str/lit_str.k");

        // test complete str functions when at the end of literal str
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(10),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();

        match &got {
            CompletionResponse::Array(arr) => {
                assert!(arr
                    .iter()
                    .all(|item| item.kind == Some(CompletionItemKind::FUNCTION)))
            }
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let got_labels: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let got_insert_text: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr
                .iter()
                .map(|item| item.insert_text.clone().unwrap())
                .collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_insert_text: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_insert_text(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_insert_text, expected_insert_text);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 2,
            column: Some(6),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        assert_eq!(got_labels, expected_labels);

        // not complete inside literal str
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 2,
            column: Some(5),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => assert!(arr.is_empty()),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        // not complete inside literal str
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(8),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => assert!(arr.is_empty()),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 3,
            column: Some(2),
        };
        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert!(arr
                    .iter()
                    .all(|item| item.kind == Some(CompletionItemKind::FUNCTION)))
            }
            CompletionResponse::List(_) => panic!("test failed"),
        };
    }

    #[test]
    fn schema_ty_attr_complete() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/schema_ty_attr/schema_ty_attr.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 13,
            column: Some(2),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(
                    arr[0],
                    CompletionItem {
                        label: "name".to_string(),
                        detail: Some("name: Name".to_string()),
                        kind: Some(CompletionItemKind::FIELD),
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn schema_end_pos() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/schema/schema_pos/schema_pos.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 6,
            column: Some(16),
        };

        let got = completion(None, &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(arr.len(), 4);
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                assert!(labels.contains(&"min".to_string()));
                assert!(labels.contains(&"max".to_string()));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn comment_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/lit_str/lit_str.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 4,
            column: Some(4),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();

        match &got {
            CompletionResponse::Array(arr) => {
                assert_eq!(arr.len(), 0)
            }
            CompletionResponse::List(_) => panic!("test failed"),
        };
    }

    #[test]
    #[bench_test]
    fn missing_expr_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/missing_expr/missing_expr.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 10,
            column: Some(16),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(arr.len(), 2);
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                assert!(labels.contains(&"cpu".to_string()));
                assert!(labels.contains(&"memory".to_string()));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    #[bench_test]
    fn check_scope_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/check/check.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 4,
            column: Some(10),
        };

        let got = completion(Some(':'), &program, &pos, &gs);
        assert!(got.is_none());

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 5,
            column: Some(9),
        };

        let got = completion(None, &program, &pos, &gs).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(arr.len(), 3);
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                assert!(labels.contains(&"name".to_string()));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    #[bench_test]
    fn join_str_inner_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/dot/lit_str/lit_str.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 6,
            column: Some(28),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match &got {
            CompletionResponse::Array(arr) => {
                assert!(arr.is_empty())
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 7,
            column: Some(27),
        };

        let got = completion(Some('.'), &program, &pos, &gs).unwrap();
        match &got {
            CompletionResponse::Array(arr) => {
                assert!(arr.is_empty())
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    #[bench_test]
    fn schema_type_attr_completion() {
        let (file, program, _, gs) =
            compile_test_file("src/test_data/completion_test/schema/schema.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 18,
            column: Some(15),
        };

        let mut got = completion(None, &program, &pos, &gs).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                assert!(labels.contains(&"name".to_string()));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 19,
            column: Some(21),
        };

        let mut got = completion(None, &program, &pos, &gs).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                assert!(labels.contains(&"name".to_string()));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }
}
