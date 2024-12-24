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

use crate::goto_def::{find_def, find_symbol};
use crate::to_lsp::lsp_pos;
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast::{self, ImportStmt, Program, Stmt};
use kclvm_ast::MAIN_PKG;
use kclvm_config::modfile::KCL_FILE_EXTENSION;
use kclvm_driver::toolchain::{get_real_path_from_external, Metadata, Toolchain};
use kclvm_error::diagnostic::Range;
use kclvm_parser::get_kcl_files;
use kclvm_sema::core::global_state::GlobalState;
use std::io;
use std::{fs, path::Path};

use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::{BUILTIN_FUNCTIONS, STANDARD_SYSTEM_MODULES};
use kclvm_sema::core::package::ModuleInfo;
use kclvm_sema::core::scope::{LocalSymbolScopeKind, ScopeKind};
use kclvm_sema::core::symbol::SymbolKind;
use kclvm_sema::resolver::doc::{parse_schema_doc_string, SchemaDoc};
use kclvm_sema::ty::{FunctionType, SchemaType, Type, TypeKind};
use kclvm_utils::path::PathPrefix;
use lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

use crate::util::{inner_most_expr_in_stmt, is_in_docstring};

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

#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Abstraction of CompletionItem in KCL
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub(crate) struct KCLCompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub kind: Option<KCLCompletionItemKind>,
    pub insert_text: Option<String>,
    pub additional_text_edits: Option<Vec<TextEdit>>,
}

/// Computes completions at the given position.
pub fn completion(
    trigger_character: Option<char>,
    program: &Program,
    pos: &KCLPos,
    gs: &GlobalState,
    tool: &dyn Toolchain,
    metadata: Option<Metadata>,
    schema_map: &IndexMap<String, Vec<SchemaType>>,
) -> Option<lsp_types::CompletionResponse> {
    match trigger_character {
        Some(c) => match c {
            '.' => completion_dot(program, pos, gs, tool),
            '=' | ':' => completion_assign(pos, gs),
            '\n' => completion_newline(program, pos, gs),
            _ => None,
        },
        None => {
            let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
            // Complete builtin pkgs if in import stmt
            completions.extend(completion_import_stmt(program, pos, metadata));
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
                    additional_text_edits: None,
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
                                            additional_text_edits: None,
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
                                additional_text_edits: None,
                            }
                        }));
                        // Complete all schema def in gs if in main pkg
                        if program.get_main_files().contains(&pos.filename) {
                            completions.extend(unimport_schemas(&pos.filename, gs, schema_map));
                        }
                    }
                }

                // Complete all usable symbol obj in inner most scope
                if let Some(defs) = gs.get_all_defs_in_scope(scope, pos) {
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
                                                &schema_ty, true,
                                            ));
                                        }
                                        SymbolKind::Package => {
                                            completions.insert(KCLCompletionItem {
                                                label: name,
                                                detail: Some(ty.ty_str()),
                                                documentation: sema_info.doc.clone(),
                                                kind: Some(KCLCompletionItemKind::Module),
                                                insert_text: None,
                                                additional_text_edits: None,
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
                                                additional_text_edits: None,
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
    tool: &dyn Toolchain,
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
            Stmt::Import(stmt) => return dot_completion_in_import_stmt(&stmt, pos, program, tool),
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
    let mut symbol = find_symbol(&pre_pos, gs, true);
    if symbol.is_none() {
        symbol = find_symbol(pos, gs, false);
    }
    let def = match symbol {
        Some(symbol_ref) => {
            if let SymbolKind::Unresolved = symbol_ref.get_kind() {
                let unresolved_symbol = gs.get_symbols().get_unresolved_symbol(symbol_ref).unwrap();
                if unresolved_symbol.is_type() {
                    return Some(into_completion_items(&items).into());
                }
            }
            match gs.get_symbols().get_symbol(symbol_ref) {
                Some(symbol) => symbol.get_definition(),
                None => None,
            }
        }
        None => None,
    };

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
                                    additional_text_edits: None,
                                });
                            }
                            None => {
                                items.insert(KCLCompletionItem {
                                    label: name,
                                    detail: None,
                                    documentation: None,
                                    kind: None,
                                    insert_text: None,
                                    additional_text_edits: None,
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
    if let Some(symbol_ref) = find_def(pos, gs, false) {
        if let Some(symbol) = gs.get_symbols().get_symbol(symbol_ref) {
            if let Some(def) = symbol.get_definition() {
                match def.get_kind() {
                    SymbolKind::Attribute => {
                        let sema_info = symbol.get_sema_info();
                        match &sema_info.ty {
                            Some(ty) => {
                                items.extend(
                                    ty_complete_label_and_inser_text(
                                        ty,
                                        gs.get_packages().get_module_info(&pos.filename),
                                    )
                                    .iter()
                                    .map(
                                        |(label, insert_text)| KCLCompletionItem {
                                            label: format!(" {}", label),
                                            detail: Some(format!(
                                                "{}: {}",
                                                symbol.get_name(),
                                                ty.ty_str()
                                            )),
                                            kind: Some(KCLCompletionItemKind::Variable),
                                            documentation: sema_info.doc.clone(),
                                            insert_text: Some(format!(" {}", insert_text)),
                                            additional_text_edits: None,
                                        },
                                    ),
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
        let doc = parse_schema_doc_string(&doc.node);
        if doc.summary.is_empty() && doc.attrs.is_empty() && doc.examples.is_empty() {
            // empty docstring, provide total completion
            let doc_parsed = SchemaDoc::new_from_schema_stmt(&schema);
            let label = doc_parsed.to_doc_string();
            // generate docstring from doc
            completions.insert(KCLCompletionItem {
                label,
                detail: Some("generate docstring".to_string()),
                documentation: Some(format!("docstring for {}", schema.name.node.clone())),
                kind: Some(KCLCompletionItemKind::Doc),
                insert_text: None,
                additional_text_edits: None,
            });
        }
        return Some(into_completion_items(&completions).into());
    }

    // Complete schema attr when input newline in schema
    if let Some(scope) = gs.look_up_scope(pos) {
        if let ScopeKind::Local = scope.get_kind() {
            if let Some(locol_scope) = gs.get_scopes().try_get_local_scope(&scope) {
                if let LocalSymbolScopeKind::Config = locol_scope.get_kind() {
                    if let Some(defs) = gs.get_defs_within_scope(scope, pos) {
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
                                                additional_text_edits: None,
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
        }
    }

    Some(into_completion_items(&completions).into())
}

fn completion_import_stmt(
    program: &Program,
    pos: &KCLPos,
    metadata: Option<Metadata>,
) -> IndexSet<KCLCompletionItem> {
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
            completions.extend(completion_import_builtin_pkg());
            completions.extend(completion_import_internal_pkg(program, line_start_pos));
            completions.extend(completion_import_external_pkg(metadata));
        }
    }
    completions
}

fn completion_import_builtin_pkg() -> IndexSet<KCLCompletionItem> {
    STANDARD_SYSTEM_MODULES
        .iter()
        .map(|s| KCLCompletionItem {
            label: s.to_string(),
            detail: None,
            documentation: None,
            kind: Some(KCLCompletionItemKind::Module),
            insert_text: None,
            additional_text_edits: None,
        })
        .collect()
}

fn completion_import_internal_pkg(
    program: &Program,
    line_start_pos: &KCLPos,
) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
    if let Ok(entries) = fs::read_dir(program.root.clone()) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    // internal pkgs
                    if file_type.is_dir() {
                        if let Ok(files) = get_kcl_files(entry.path(), true) {
                            // skip folder if without kcl file
                            if files.is_empty() {
                                continue;
                            }
                        } else {
                            continue;
                        }
                        if let Some(name) = entry.file_name().to_str() {
                            completions.insert(KCLCompletionItem {
                                label: name.to_string(),
                                detail: None,
                                documentation: None,
                                kind: Some(KCLCompletionItemKind::Dir),
                                insert_text: None,
                                additional_text_edits: None,
                            });
                        }
                    } else {
                        // internal module
                        let path = entry.path();
                        if path.to_str().unwrap_or("").adjust_canonicalization()
                            == line_start_pos.filename.adjust_canonicalization()
                        {
                            continue;
                        }
                        if let Some(extension) = path.extension() {
                            if extension == KCL_FILE_EXTENSION {
                                if let Some(name) = path.file_stem() {
                                    if let Some(name) = name.to_str() {
                                        completions.insert(KCLCompletionItem {
                                            label: name.to_string(),
                                            detail: None,
                                            documentation: None,
                                            kind: Some(KCLCompletionItemKind::Module),
                                            insert_text: None,
                                            additional_text_edits: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    completions
}

fn completion_import_external_pkg(metadata: Option<Metadata>) -> IndexSet<KCLCompletionItem> {
    match metadata {
        Some(metadata) => metadata
            .packages
            .keys()
            .map(|name| KCLCompletionItem {
                label: name.to_string(),
                detail: None,
                documentation: None,
                kind: Some(KCLCompletionItemKind::Dir),
                insert_text: None,
                additional_text_edits: None,
            })
            .collect(),
        None => IndexSet::new(),
    }
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
/// import pkg
/// p = pkg.Person(param1, param2){<cursor>}
/// ```
fn schema_ty_to_value_complete_item(schema_ty: &SchemaType, has_import: bool) -> KCLCompletionItem {
    let schema = schema_ty.clone();
    let param = schema_ty.func.params.clone();
    let pkg_path_last_name = if schema.pkgpath.is_empty() || schema.pkgpath == MAIN_PKG {
        "".to_string()
    } else {
        format!("{}", schema.pkgpath.split('.').last().unwrap())
    };
    let need_import = !pkg_path_last_name.is_empty() && !has_import;

    let label = format!(
        "{}{}{}{}",
        schema.name,
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
        "{}",
        if need_import {
            format!("(import {})", schema.pkgpath)
        } else {
            "".to_string()
        },
    );

    // `pkg_path.schema_name{<cursor>}` or `schema_name{<cursor>}`
    let insert_text = format!(
        "{}{}{}{}{}",
        pkg_path_last_name,
        if pkg_path_last_name.is_empty() {
            ""
        } else {
            "."
        },
        schema.name,
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
        "{$0}"
    );

    // insert `import pkg`
    let additional_text_edits = if need_import {
        Some(vec![TextEdit {
            range: (KCLPos::dummy_pos(), KCLPos::dummy_pos()),
            new_text: format!("import {}\n", schema.pkgpath),
        }])
    } else {
        None
    };

    let detail = {
        let mut details = vec![];
        let (pkgpath, rest_sign) = schema_ty.schema_ty_signature_str();
        details.push(format!("{}\n\n{}", pkgpath, rest_sign));
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
        label,
        detail: Some(detail),
        documentation: Some(schema_ty.doc.clone()),
        kind: Some(KCLCompletionItemKind::Schema),
        insert_text: Some(insert_text),
        additional_text_edits,
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
        let (pkgpath, rest_sign) = schema_ty.schema_ty_signature_str();
        details.push(format!("{}\n\n{}", pkgpath, rest_sign));
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
        additional_text_edits: None,
    }
}

fn dot_completion_in_import_stmt(
    stmt: &ImportStmt,
    _pos: &KCLPos,
    program: &Program,
    tool: &dyn Toolchain,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();
    let pkgpath = &stmt.path.node;
    let mut real_path =
        Path::new(&program.root).join(pkgpath.replace('.', std::path::MAIN_SEPARATOR_STR));
    if !real_path.exists() {
        real_path =
            get_real_path_from_external(tool, &stmt.pkg_name, pkgpath, program.root.clone().into());
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
                        additional_text_edits: None,
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
                                additional_text_edits: None,
                            });
                        }
                    }
                }
            }
        }
    }
    Some(into_completion_items(&items).into())
}

fn ty_complete_label_and_inser_text(
    ty: &Type,
    module: Option<&ModuleInfo>,
) -> Vec<(String, String)> {
    match &ty.kind {
        TypeKind::Bool => vec![
            ("True".to_string(), "True".to_string()),
            ("False".to_string(), "False".to_string()),
        ],
        TypeKind::BoolLit(b) => {
            vec![if *b {
                ("True".to_string(), "True".to_string())
            } else {
                ("False".to_string(), "False".to_string())
            }]
        }
        TypeKind::IntLit(i) => vec![(i.to_string(), i.to_string())],
        TypeKind::FloatLit(f) => vec![(f.to_string(), f.to_string())],
        TypeKind::Str => vec![(r#""""#.to_string(), r#""""#.to_string())],
        TypeKind::StrLit(s) => vec![(format!("{:?}", s), format!("{:?}", s))],
        TypeKind::List(_) => vec![("[]".to_string(), "[$1]".to_string())],
        TypeKind::Dict(_) => vec![("{}".to_string(), "{$1}".to_string())],
        TypeKind::Union(types) => types
            .iter()
            .flat_map(|ty| ty_complete_label_and_inser_text(ty, module))
            .collect(),
        TypeKind::Schema(schema) => {
            vec![(
                format!(
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
                ),
                "{$1}".to_string(), // `$1`` is used to determine the cursor position after completion
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
        .map(|item| {
            let additional_text_edits = match &item.additional_text_edits {
                Some(edits) => {
                    let mut res = vec![];
                    for edit in edits {
                        res.push(lsp_types::TextEdit {
                            range: lsp_types::Range {
                                start: lsp_pos(&edit.range.0),
                                end: lsp_pos(&edit.range.1),
                            },
                            new_text: edit.new_text.clone(),
                        })
                    }

                    Some(res)
                }
                None => None,
            };

            CompletionItem {
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
                additional_text_edits,

                ..Default::default()
            }
        })
        .collect()
}

fn unimport_schemas(
    filename: &str,
    gs: &GlobalState,
    schema_map: &IndexMap<String, Vec<SchemaType>>,
) -> IndexSet<KCLCompletionItem> {
    let module = gs.get_packages().get_module_info(filename);
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
    for (_, schemas) in schema_map {
        for schema in schemas {
            let has_import = match module {
                Some(m) => m
                    .get_imports()
                    .iter()
                    .any(|(_, info)| info.get_fully_qualified_name() == schema.pkgpath),
                None => false,
            };
            if schema.pkgpath != MAIN_PKG {
                completions.insert(schema_ty_to_value_complete_item(&schema, has_import));
            }
        }
    }
    completions
}

#[cfg(test)]
mod tests {
    use crate::{
        completion::{completion, func_ty_complete_label},
        tests::{compile_test_file, compile_test_file_and_metadata},
    };
    use kclvm_driver::toolchain;
    use kclvm_error::Position as KCLPos;
    use kclvm_sema::builtin::{BUILTIN_FUNCTIONS, STANDARD_SYSTEM_MODULES};
    use lsp_types::CompletionResponse;
    #[macro_export]
    macro_rules! completion_label_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr, $trigger: expr) => {
            #[test]
            fn $name() {
                let (file, program, _, gs, schema_map) = compile_test_file($file);

                let pos = KCLPos {
                    filename: file.clone(),
                    line: $line,
                    column: Some($column),
                };
                let tool = toolchain::default();

                let mut got =
                    completion($trigger, &program, &pos, &gs, &tool, None, &schema_map).unwrap();

                let got_labels = match &mut got {
                    CompletionResponse::Array(arr) => {
                        let mut labels: Vec<String> =
                            arr.iter().map(|item| item.label.clone()).collect();
                        labels.sort();
                        labels
                    }
                    CompletionResponse::List(_) => panic!("test failed"),
                };
                insta::assert_snapshot!(format!("{:?}", got_labels));
            }
        };
    }

    #[macro_export]
    macro_rules! completion_label_without_builtin_func_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr, $trigger: expr) => {
            #[test]
            fn $name() {
                let (file, program, _, gs, schema_map) = compile_test_file($file);

                let pos = KCLPos {
                    filename: file.clone(),
                    line: $line,
                    column: Some($column),
                };
                let tool = toolchain::default();

                let mut got =
                    completion($trigger, &program, &pos, &gs, &tool, None, &schema_map).unwrap();

                let got_labels = match &mut got {
                    CompletionResponse::Array(arr) => {
                        let mut labels: Vec<String> =
                            arr.iter().map(|item| item.label.clone()).collect();
                        labels.sort();
                        let builtin_func_lables: Vec<String> = BUILTIN_FUNCTIONS
                            .iter()
                            .map(|(name, func)| {
                                func_ty_complete_label(name, &func.into_func_type())
                            })
                            .collect();
                        let labels: Vec<String> = labels
                            .iter()
                            .filter(|label| !builtin_func_lables.contains(label))
                            .map(|label| label.clone())
                            .collect();

                        labels
                    }
                    CompletionResponse::List(_) => panic!("test failed"),
                };
                insta::assert_snapshot!(format!("{:?}", got_labels));
            }
        };
    }

    #[macro_export]
    macro_rules! completion_label_without_system_pkg_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr, $trigger: expr) => {
            #[test]
            fn $name() {
                let (file, program, _, gs, metadata, schema_map) =
                    compile_test_file_and_metadata($file);
                let pos = KCLPos {
                    filename: file.clone(),
                    line: $line,
                    column: Some($column),
                };
                let tool = toolchain::default();
                let mut got =
                    completion($trigger, &program, &pos, &gs, &tool, metadata, &schema_map)
                        .unwrap();
                let got_labels = match &mut got {
                    CompletionResponse::Array(arr) => {
                        let mut labels: Vec<String> =
                            arr.iter().map(|item| item.label.clone()).collect();
                        labels.sort();
                        let labels: Vec<String> = labels
                            .iter()
                            .filter(|label| !STANDARD_SYSTEM_MODULES.contains(&label.as_str()))
                            .cloned()
                            .collect();

                        labels
                    }
                    CompletionResponse::List(_) => panic!("test failed"),
                };
                insta::assert_snapshot!(format!("{:?}", got_labels));
            }
        };
    }

    completion_label_without_builtin_func_test_snapshot!(
        var_completion_labels,
        "src/test_data/completion_test/dot/completion/completion.k",
        26,
        1,
        None
    );

    completion_label_test_snapshot!(
        schema_attr_completion_labels,
        "src/test_data/completion_test/dot/completion/completion.k",
        24,
        4,
        None
    );

    completion_label_test_snapshot!(
        dot_schema_attr_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        12,
        7,
        Some('.')
    );

    completion_label_test_snapshot!(
        dot_str_builtin_func_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        14,
        12,
        Some('.')
    );

    completion_label_test_snapshot!(
        import_pkg_path_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        1,
        12,
        Some('.')
    );

    completion_label_test_snapshot!(
        import_pkg_schema_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        16,
        12,
        Some('.')
    );

    completion_label_test_snapshot!(
        math_func_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        19,
        5,
        Some('.')
    );

    completion_label_test_snapshot!(
        literal_str_builtin_func_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        21,
        4,
        Some('.')
    );

    completion_label_test_snapshot!(
        single_schema_attr_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        30,
        11,
        Some('.')
    );

    completion_label_test_snapshot!(
        string_union_type_completion,
        "src/test_data/completion_test/dot/completion/completion.k",
        36,
        30,
        Some('.')
    );

    completion_label_without_builtin_func_test_snapshot!(
        var_completion_labels_without_dot,
        "src/test_data/completion_test/without_dot/completion.k",
        26,
        1,
        None
    );

    completion_label_without_system_pkg_test_snapshot!(
        system_pkg_labels,
        "src/test_data/completion_test/without_dot/completion.k",
        36,
        5,
        Some('.')
    );

    completion_label_test_snapshot!(
        basic_completion_labels,
        "src/test_data/completion_test/without_dot/completion.k",
        12,
        7,
        Some('.')
    );

    completion_label_test_snapshot!(
        import_builtin_package_test,
        "src/test_data/completion_test/import/builtin/builtin_pkg.k",
        1,
        8,
        None
    );

    completion_label_test_snapshot!(
        attr_value_completion_true_false,
        "src/test_data/completion_test/assign/completion.k",
        14,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        attr_value_completion_strings,
        "src/test_data/completion_test/assign/completion.k",
        16,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        attr_value_completion_list,
        "src/test_data/completion_test/assign/completion.k",
        18,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        attr_value_completion_integer,
        "src/test_data/completion_test/assign/completion.k",
        20,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        attr_value_completion_boolean,
        "src/test_data/completion_test/assign/completion.k",
        22,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        attr_value_completion_dict,
        "src/test_data/completion_test/assign/completion.k",
        24,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        attr_value_completion_schema,
        "src/test_data/completion_test/assign/completion.k",
        26,
        6,
        Some(':')
    );

    completion_label_test_snapshot!(
        schema_sig_completion_test,
        "src/test_data/completion_test/schema/schema/schema.k",
        7,
        5,
        None
    );

    completion_label_test_snapshot!(
        schema_docstring_newline_test,
        "src/test_data/completion_test/newline/docstring_newline.k",
        3,
        4,
        Some('\n')
    );

    completion_label_test_snapshot!(
        str_dot_completion_test_end_of_literal,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        1,
        10,
        Some('.')
    );

    completion_label_test_snapshot!(
        str_dot_completion_test_second_line_end,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        2,
        6,
        Some('.')
    );

    completion_label_test_snapshot!(
        str_dot_completion_test_inside_literal_1,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        2,
        5,
        Some('.')
    );

    completion_label_test_snapshot!(
        str_dot_completion_test_inside_literal_2,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        1,
        8,
        Some('.')
    );

    completion_label_test_snapshot!(
        str_dot_completion_test_third_line,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        3,
        2,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_ty_attr_complete_test,
        "src/test_data/completion_test/dot/schema_ty_attr/schema_ty_attr.k",
        13,
        2,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_end_pos_test,
        "src/test_data/completion_test/schema/schema_pos/schema_pos.k",
        6,
        16,
        None
    );

    completion_label_test_snapshot!(
        comment_completion_test,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        4,
        4,
        Some('.')
    );

    completion_label_test_snapshot!(
        missing_expr_completion_test,
        "src/test_data/completion_test/dot/missing_expr/missing_expr.k",
        10,
        16,
        Some('.')
    );

    completion_label_test_snapshot!(
        check_scope_completion_test_part1,
        "src/test_data/completion_test/check/check.k",
        4,
        10,
        Some(':')
    );

    completion_label_test_snapshot!(
        check_scope_completion_test_part2,
        "src/test_data/completion_test/check/check.k",
        5,
        9,
        None
    );

    completion_label_test_snapshot!(
        join_str_inner_completion_test_part1,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        6,
        28,
        Some('.')
    );

    completion_label_test_snapshot!(
        join_str_inner_completion_test_part2,
        "src/test_data/completion_test/dot/lit_str/lit_str.k",
        7,
        27,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_type_attr_completion_test_part1,
        "src/test_data/completion_test/schema/schema/schema.k",
        18,
        15,
        None
    );

    completion_label_test_snapshot!(
        schema_type_attr_completion_test_part2,
        "src/test_data/completion_test/schema/schema/schema.k",
        19,
        21,
        None
    );

    completion_label_test_snapshot!(
        nested_1_test,
        "src/test_data/completion_test/dot/nested/nested_1/nested_1.k",
        9,
        9,
        None
    );

    completion_label_test_snapshot!(
        nested_2_test,
        "src/test_data/completion_test/dot/nested/nested_2/nested_2.k",
        9,
        9,
        None
    );

    completion_label_test_snapshot!(
        nested_3_test,
        "src/test_data/completion_test/dot/nested/nested_3/nested_3.k",
        10,
        13,
        None
    );

    completion_label_test_snapshot!(
        nested_4_test,
        "src/test_data/completion_test/dot/nested/nested_4/nested_4.k",
        9,
        9,
        None
    );

    completion_label_without_builtin_func_test_snapshot!(
        lambda_1,
        "src/test_data/completion_test/lambda/lambda_1/lambda_1.k",
        8,
        5,
        None
    );

    completion_label_without_builtin_func_test_snapshot!(
        schema_attr_newline_completion_0,
        "src/test_data/completion_test/newline/schema/schema_0/schema_0.k",
        8,
        4,
        Some('\n')
    );

    completion_label_without_builtin_func_test_snapshot!(
        schema_attr_newline_completion_0_1,
        "src/test_data/completion_test/newline/schema/schema_0/schema_0.k",
        5,
        4,
        Some('\n')
    );

    completion_label_without_builtin_func_test_snapshot!(
        schema_attr_newline_completion_1,
        "src/test_data/completion_test/newline/schema/schema_1/schema_1.k",
        10,
        4,
        Some('\n')
    );

    completion_label_without_system_pkg_test_snapshot!(
        import_internal_pkg_test,
        "src/test_data/completion_test/import/internal/main.k",
        1,
        8,
        None
    );

    completion_label_without_system_pkg_test_snapshot!(
        import_external_pkg_test,
        "src/test_data/completion_test/import/external/external_1/main.k",
        1,
        8,
        None
    );

    completion_label_without_builtin_func_test_snapshot!(
        func_return_ty_1,
        "src/test_data/completion_test/dot/func_return/func_return_1/func_return_1.k",
        4,
        8,
        Some('.')
    );

    completion_label_without_builtin_func_test_snapshot!(
        func_return_ty_2,
        "src/test_data/completion_test/dot/func_return/func_return_2/func_return_2.k",
        8,
        12,
        Some('.')
    );

    completion_label_without_builtin_func_test_snapshot!(
        func_return_ty_3,
        "src/test_data/completion_test/dot/func_return/func_return_3/func_return_3.k",
        3,
        2,
        Some('.')
    );

    completion_label_test_snapshot!(
        func_doc_completion,
        "src/test_data/completion_test/schema_doc/schema_doc.k",
        7,
        14,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_attr_in_right,
        "src/test_data/completion_test/schema/schema/schema.k",
        23,
        11,
        None
    );

    completion_label_test_snapshot!(
        schema_def_1,
        "src/test_data/completion_test/schema_def/schema_def.k",
        10,
        22,
        None
    );

    completion_label_test_snapshot!(
        schema_def_2,
        "src/test_data/completion_test/schema_def/schema_def.k",
        12,
        5,
        None
    );

    completion_label_test_snapshot!(
        schema_def_3,
        "src/test_data/completion_test/schema_def/schema_def.k",
        13,
        8,
        None
    );

    completion_label_test_snapshot!(
        schema_def_4,
        "src/test_data/completion_test/schema_def/schema_def.k",
        3,
        12,
        None
    );

    completion_label_test_snapshot!(
        schema_attr_ty_0,
        "src/test_data/completion_test/dot/schema_attr_ty/schema_attr_ty.k",
        5,
        13,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_attr_ty_1,
        "src/test_data/completion_test/dot/schema_attr_ty/schema_attr_ty.k",
        6,
        14,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_attr_ty_2,
        "src/test_data/completion_test/dot/schema_attr_ty/schema_attr_ty.k",
        7,
        18,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_attr_ty_3,
        "src/test_data/completion_test/dot/schema_attr_ty/schema_attr_ty.k",
        8,
        17,
        Some('.')
    );

    completion_label_test_snapshot!(
        schema_attr_ty_4,
        "src/test_data/completion_test/dot/schema_attr_ty/schema_attr_ty.k",
        10,
        15,
        Some('.')
    );

    completion_label_test_snapshot!(
        complete_after_compare_expr_1,
        "src/test_data/completion_test/dot/special_expr/compare.k",
        2,
        23,
        Some('.')
    );

    completion_label_without_builtin_func_test_snapshot!(
        complete_unimport_schemas,
        "src/test_data/completion_test/unimport/unimport/main.k",
        1,
        1,
        None
    );
}
