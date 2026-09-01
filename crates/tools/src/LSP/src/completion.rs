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
use kcl_ast::MAIN_PKG;
use kcl_ast::ast::{self, ImportStmt, Program, Stmt};
use kcl_config::modfile::{KCL_FILE_EXTENSION, KCL_FILE_SUFFIX};
use kcl_driver::toolchain::{Metadata, Toolchain, get_real_path_from_external};
use kcl_error::diagnostic::Range;
use kcl_primitives::{DefaultHashBuilder, IndexMap, IndexSet};
use kcl_sema::core::global_state::GlobalState;
use kcl_utils::pkgpath::rm_external_pkg_name;
use std::io;
use std::path::PathBuf;
use std::{fs, path::Path};
use walkdir::WalkDir;

use kcl_error::Position as KCLPos;
use kcl_sema::builtin::{BUILTIN_FUNCTIONS, STANDARD_SYSTEM_MODULES};
use kcl_sema::core::package::ModuleInfo;
use kcl_sema::core::scope::{LocalSymbolScopeKind, ScopeKind};
use kcl_sema::core::symbol::SymbolKind;
use kcl_sema::resolver::doc::{SchemaDoc, parse_schema_doc_string};
use kcl_sema::ty::{FunctionType, SchemaType, Type, TypeKind};
use kcl_utils::path::PathPrefix;
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
            '.' => completion_dot(program, pos, gs, tool, metadata.as_ref()),
            '=' | ':' => completion_assign(pos, gs),
            '\n' => completion_newline(program, pos, gs),
            _ => None,
        },
        None => {
            let mut completions: IndexSet<KCLCompletionItem> = Default::default();
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
                    kcl_sema::core::scope::ScopeKind::Local => {
                        if let Some(local_scope) = gs.get_scopes().try_get_local_scope(&scope)
                            && local_scope.get_kind()
                                == &kcl_sema::core::scope::LocalSymbolScopeKind::Lambda
                        {
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
                        }
                    }
                    kcl_sema::core::scope::ScopeKind::Root => {
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
                        if let Some(def) = gs.get_symbols().get_symbol(symbol_ref) {
                            let sema_info = def.get_sema_info();
                            let name = def.get_name();
                            if let Some(ty) = &sema_info.ty {
                                match symbol_ref.get_kind() {
                                    SymbolKind::Schema => {
                                        let schema_ty = ty.into_schema_type();
                                        // complete schema type
                                        completions
                                            .insert(schema_ty_to_type_complete_item(&schema_ty));
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
                                }
                            }
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
    metadata: Option<&Metadata>,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<KCLCompletionItem> = Default::default();

    // get pre position of trigger character '.'
    let pre_pos = KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: pos.column.map(|c| c.saturating_sub(1)),
    };

    if let Some(stmt) = program.pos_to_stmt(&pre_pos) {
        match stmt.node {
            Stmt::Import(stmt) => {
                return dot_completion_in_import_stmt(&stmt, pos, program, tool, metadata);
            }
            _ => {
                let (expr, _) = inner_most_expr_in_stmt(&stmt.node, pos, None);
                if let Some(node) = expr {
                    match node.node {
                        // if the complete trigger character in string, skip it
                        ast::Expr::StringLit(_) | ast::Expr::JoinedString(_) => {
                            return Some(into_completion_items(&items).into());
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

    if let Some(def_ref) = def
        && let Some(def) = gs.get_symbols().get_symbol(def_ref)
    {
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
                            TypeKind::Function(func_ty) => func_ty_complete_label(&name, func_ty),
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
                                TypeKind::Schema(_) => Some(KCLCompletionItemKind::SchemaAttr),
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
    Some(into_completion_items(&items).into())
}

/// Get completion items for trigger '=' or ':'
/// Now, just completion for schema attr value
fn completion_assign(pos: &KCLPos, gs: &GlobalState) -> Option<lsp_types::CompletionResponse> {
    let mut items = IndexSet::with_hasher(DefaultHashBuilder::default());
    if let Some(symbol_ref) = find_def(pos, gs, false)
        && let Some(symbol) = gs.get_symbols().get_symbol(symbol_ref)
        && let Some(def) = symbol.get_definition()
        && def.get_kind() == SymbolKind::Attribute
    {
        let sema_info = symbol.get_sema_info();
        if let Some(ty) = &sema_info.ty {
            items.extend(
                ty_complete_label_and_inser_text(
                    ty,
                    gs.get_packages().get_module_info(&pos.filename),
                )
                .iter()
                .map(|(label, insert_text)| KCLCompletionItem {
                    label: format!(" {}", label),
                    detail: Some(format!("{}: {}", symbol.get_name(), ty.ty_str())),
                    kind: Some(KCLCompletionItemKind::Variable),
                    documentation: sema_info.doc.clone(),
                    insert_text: Some(format!(" {}", insert_text)),
                    additional_text_edits: None,
                }),
            );
            return Some(into_completion_items(&items).into());
        }
    }
    None
}

fn completion_newline(
    program: &Program,
    pos: &KCLPos,
    gs: &GlobalState,
) -> Option<lsp_types::CompletionResponse> {
    let mut completions: IndexSet<KCLCompletionItem> = Default::default();

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
    if let Some(scope) = gs.look_up_scope(pos)
        && let ScopeKind::Local = scope.get_kind()
        && let Some(locol_scope) = gs.get_scopes().try_get_local_scope(&scope)
        && let LocalSymbolScopeKind::Config = locol_scope.get_kind()
        && let Some(defs) = gs.get_defs_within_scope(scope, pos)
    {
        for symbol_ref in defs {
            if let Some(def) = gs.get_symbols().get_symbol(symbol_ref) {
                let sema_info = def.get_sema_info();
                let name = def.get_name();
                if symbol_ref.get_kind() == SymbolKind::Attribute {
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
    let mut completions: IndexSet<KCLCompletionItem> = Default::default();
    // completion position not contained in import stmt
    // import <space>  <cursor>
    // |             | |  <- input `m` here for complete `math`
    // |<----------->| <- import stmt only contains this range, so we need to check the beginning of line
    let line_start_pos = &KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: Some(0),
    };

    if let Some(node) = program.pos_to_stmt(line_start_pos)
        && let Stmt::Import(_) = node.node
    {
        completions.extend(completion_import_builtin_pkg());
        completions.extend(completion_import_internal_pkg(program, line_start_pos));
        completions.extend(completion_import_external_pkg(metadata));
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
    _line_start_pos: &KCLPos,
) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = Default::default();
    if let Ok(entries) = fs::read_dir(program.root.clone()) {
        for entry in entries {
            // KCL `import` statements always target a *package* (a directory),
            // never a single `.k` file. A sibling file like `main.k` is therefore
            // never a valid completion for `import …`, and previously slipped in
            // here as a wrong completion suggestion (see kcl-lang/kcl#1736).
            // Only suggest directories that contain at least one `.k` file — those
            // are the sub-packages the user can actually `import`.
            //
            // Hidden directories (`.git`, `.vscode`, …) are never importable
            // packages, and scanning them for `.k` files on every completion
            // request is pure waste (`.git` alone can hold tens of thousands
            // of entries), so they are skipped outright.
            if let Ok(entry) = entry
                && let Ok(file_type) = entry.file_type()
                && file_type.is_dir()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| !name.starts_with('.'))
                && dir_contains_kcl_file(&entry.path())
                && let Some(name) = entry.file_name().to_str()
            {
                completions.insert(KCLCompletionItem {
                    label: name.to_string(),
                    detail: None,
                    documentation: None,
                    kind: Some(KCLCompletionItemKind::Dir),
                    insert_text: None,
                    additional_text_edits: None,
                });
            }
        }
    }
    completions
}

/// Returns whether the directory tree rooted at `path` contains at least one
/// `.k` file. Unlike collecting every file with `get_kcl_files(path, true)`,
/// this stops at the first match and never descends into hidden directories,
/// so import completion stays cheap on every keystroke even in workspaces
/// with large sibling directories.
fn dir_contains_kcl_file(path: &Path) -> bool {
    WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden_entry(e))
        .filter_map(|e| e.ok())
        .any(|e| e.path().is_file() && e.file_name().to_str().is_some_and(is_kcl_file_name))
}

fn is_hidden_entry(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|name| name.starts_with('.'))
}

fn is_kcl_file_name(file_name: &str) -> bool {
    file_name.ends_with(KCL_FILE_SUFFIX)
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
        None => Default::default(),
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
        schema.pkgpath.split('.').next_back().unwrap().to_string()
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
    metadata: Option<&Metadata>,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<KCLCompletionItem> = Default::default();
    let pkgpath = &stmt.path.node;
    let mut real_path =
        Path::new(&program.root).join(pkgpath.replace('.', std::path::MAIN_SEPARATOR_STR));
    if !real_path.exists() {
        // Prefer the workspace metadata cached by the compilation pipeline:
        // resolving via the toolchain runs `kcl mod metadata` (a subprocess)
        // synchronously in the completion request, which is far too slow to
        // repeat on every keystroke.
        real_path = metadata
            .and_then(|m| external_pkg_real_path(m, &stmt.pkg_name, pkgpath))
            .unwrap_or_else(|| {
                get_real_path_from_external(
                    tool,
                    &stmt.pkg_name,
                    pkgpath,
                    program.root.clone().into(),
                )
            });
    }
    if real_path.is_dir()
        && let Ok(entries) = fs::read_dir(real_path)
    {
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
            } else if path.is_file()
                && let Some(extension) = path.extension()
                && extension == KCL_FILE_EXTENSION
            {
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
    Some(into_completion_items(&items).into())
}

/// Resolves the on-disk path of `pkgpath` (e.g. `my_pkg.sub.dir`) inside the
/// external package `pkg_name`, using the metadata cached for the current
/// workspace. Returns `None` when the metadata knows nothing about
/// `pkg_name`, so the caller can fall back to querying the toolchain.
fn external_pkg_real_path(metadata: &Metadata, pkg_name: &str, pkgpath: &str) -> Option<PathBuf> {
    let mut real_path = metadata.packages.get(pkg_name)?.manifest_path.clone();
    let sub_path = rm_external_pkg_name(pkgpath).unwrap_or_default();
    sub_path.split('.').for_each(|s| real_path.push(s));
    Some(real_path)
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
                        format!("{}.", schema.pkgpath.split('.').next_back().unwrap())
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
    pkg.split('.').next_back().unwrap().to_string()
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
    let mut completions: IndexSet<KCLCompletionItem> = Default::default();
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
                completions.insert(schema_ty_to_value_complete_item(schema, has_import));
            }
        }
    }
    completions
}

#[cfg(test)]
mod tests {
    use crate::{
        completion::{
            KCLCompletionItem, KCLCompletionItemKind, completion, func_ty_complete_insert_text,
            func_ty_complete_label, into_completion_items,
        },
        tests::{compile_test_file, compile_test_file_and_metadata},
    };
    use kcl_ast::ast::{self, ImportStmt, Program};
    use kcl_driver::toolchain::{self, Metadata, Package, Toolchain};
    use kcl_error::Position as KCLPos;
    use kcl_primitives::IndexSet;
    use kcl_sema::builtin::{
        BUILTIN_FUNCTIONS, MATH_FUNCTION_TYPES, STANDARD_SYSTEM_MODULES, STRING_MEMBER_FUNCTIONS,
    };
    use lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, InsertTextFormat};
    use proc_macro_crate::bench_test;

    #[test]
    #[bench_test]
    fn var_completion_test() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/completion/completion.k");

        // test completion for var
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 26,
            column: Some(1),
        };

        let tool = toolchain::default();
        let got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        let mut got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let mut expected_labels: Vec<String> = vec![
            "", // generate from error recovery of "pkg."
            "subpkg",
            "math",
            "Person",
            "Person1{}",
            "Person{}",
            "P",
            "P{}",
            "p",
            "p1",
            "p2",
            "p3",
            "p4",
            "aaaa",
            "Config",
            "Config{}",
            "n",
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

        let got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/completion/completion.k");

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 12,
            column: Some(7),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
            filename: file.clone(),
            line: 30,
            column: Some(11),
        };

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["a"];
        assert_eq!(got_labels, expected_labels);

        // test completion for string union type
        let pos = KCLPos {
            filename: file.clone(),
            line: 36,
            column: Some(30),
        };

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_func_type()))
            .collect();
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test_without_dot() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/without_dot/completion.k");

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 12,
            column: Some(7),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
            filename: file.clone(),
            line: 30,
            column: Some(11),
        };

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["a"];
        assert_eq!(got_labels, expected_labels);

        // test completion for str union types
        let pos = KCLPos {
            filename: file.clone(),
            line: 36,
            column: Some(30),
        };

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
    }

    #[test]
    #[bench_test]
    fn import_builtin_package() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/import/builtin/builtin_pkg.k");
        let mut items: IndexSet<KCLCompletionItem> = Default::default();

        // test completion for builtin packages
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(8),
        };

        let tool = toolchain::default();
        let got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
                "template",
                "runtime",
                "base32",
            ]
            .iter()
            .map(|name| KCLCompletionItem {
                label: name.to_string(),
                kind: Some(KCLCompletionItemKind::Module),
                detail: None,
                documentation: None,
                insert_text: None,
                additional_text_edits: None,
            })
            .collect::<IndexSet<KCLCompletionItem>>(),
        );
        let expect: CompletionResponse = into_completion_items(&items).into();
        assert_eq!(got, expect);
    }

    #[test]
    #[bench_test]
    fn attr_value_completion() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/assign/completion.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(6),
        };

        let tool = toolchain::default();
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        let got_labels: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" sub.Person1{}"];
        assert_eq!(got_labels, expected_labels);

        let got_insert_test: Vec<String> = match &got {
            CompletionResponse::Array(arr) => arr
                .iter()
                .map(|item| item.clone().insert_text.unwrap().clone())
                .collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_insert_test: Vec<&str> = vec![" {$1}"];
        assert_eq!(got_insert_test, expected_insert_test);
    }

    #[test]
    #[bench_test]
    fn schema_sig_completion() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/schema/schema/schema.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 7,
            column: Some(5),
        };

        let tool = toolchain::default();
        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                assert_eq!(
                    arr.iter().find(|item| item.label == "Person(b){}").unwrap(),
                    &CompletionItem {
                        label: "Person(b){}".to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some(
                            "__main__\n\nschema Person[b: int](Base):\nAttributes:\nc: int"
                                .to_string()
                        ),
                        documentation: Some(lsp_types::Documentation::String("".to_string())),
                        insert_text: Some("Person(${1:b}){$0}".to_string()),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn schema_docstring_newline_completion() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/newline/docstring_newline.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 3,
            column: Some(4),
        };
        let tool = toolchain::default();
        let mut got =
            completion(Some('\n'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/lit_str/lit_str.k");

        // test complete str functions when at the end of literal str
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(10),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();

        match &got {
            CompletionResponse::Array(arr) => {
                assert!(
                    arr.iter()
                        .all(|item| item.kind == Some(CompletionItemKind::FUNCTION))
                )
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        match got {
            CompletionResponse::Array(arr) => assert!(arr.is_empty()),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 3,
            column: Some(2),
        };
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert!(
                    arr.iter()
                        .all(|item| item.kind == Some(CompletionItemKind::FUNCTION))
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        };
    }

    #[test]
    fn schema_ty_attr_complete() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/schema_ty_attr/schema_ty_attr.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 13,
            column: Some(2),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/schema/schema_pos/schema_pos.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 6,
            column: Some(16),
        };

        let tool = toolchain::default();
        let got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/lit_str/lit_str.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 4,
            column: Some(4),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();

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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/missing_expr/missing_expr.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 10,
            column: Some(16),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/check/check.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 4,
            column: Some(10),
        };

        let tool = toolchain::default();
        let got = completion(Some(':'), &program, &pos, &gs, &tool, None, &schema_map);
        assert!(got.is_none());

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 5,
            column: Some(9),
        };

        let got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/lit_str/lit_str.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 6,
            column: Some(28),
        };

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let tool = toolchain::default();
        let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/schema/schema/schema.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 18,
            column: Some(15),
        };

        let tool = toolchain::default();
        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
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

        let tool = toolchain::default();
        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                assert!(labels.contains(&"name".to_string()));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    #[bench_test]
    fn nested_1_test() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/nested/nested_1/nested_1.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 9,
            column: Some(9),
        };
        let tool = toolchain::default();

        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();

        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                insta::assert_snapshot!(format!("{:?}", labels));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    #[bench_test]
    fn nested_2_test() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/nested/nested_2/nested_2.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 9,
            column: Some(9),
        };

        let tool = toolchain::default();

        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();

        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                insta::assert_snapshot!(format!("{:?}", labels));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }
    #[test]
    #[bench_test]
    fn nested_3_test() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/nested/nested_3/nested_3.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 10,
            column: Some(13),
        };

        let tool = toolchain::default();
        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();

        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                insta::assert_snapshot!(format!("{:?}", labels));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    #[bench_test]
    fn nested_4_test() {
        let (file, program, _, gs, schema_map) =
            compile_test_file("src/test_data/completion_test/dot/nested/nested_4/nested_4.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 9,
            column: Some(9),
        };

        let tool = toolchain::default();

        let mut got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();

        match &mut got {
            CompletionResponse::Array(arr) => {
                let labels: Vec<String> = arr.iter().map(|item| item.label.clone()).collect();
                insta::assert_snapshot!(format!("{:?}", labels));
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

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

    completion_label_without_builtin_func_test_snapshot!(
        schema_attr_newline_completion_2,
        "src/test_data/completion_test/newline/schema/schema_2/schema_2.k",
        13,
        8,
        Some('\n')
    );

    completion_label_without_system_pkg_test_snapshot!(
        import_internal_pkg_test,
        "src/test_data/completion_test/import/internal/main.k",
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

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "{}-{}-{}",
            prefix,
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn import_stmt_node(path: &str) -> ImportStmt {
        ImportStmt {
            path: ast::Node {
                id: Default::default(),
                node: path.to_string(),
                filename: String::new(),
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 1,
            },
            rawpath: path.to_string(),
            name: path.to_string(),
            asname: None,
            pkg_name: path.split('.').next().unwrap().to_string(),
        }
    }

    /// Import completion must only suggest importable sub-packages: directories
    /// holding at least one `.k` file (at any depth). Hidden directories such as
    /// `.git` are never importable and must be skipped without being walked.
    #[test]
    fn import_internal_pkg_completion_skips_hidden_and_empty_dirs() {
        let base = unique_temp_dir("kcl-lsp-import-completion");

        // pkg_a: a `.k` file at the top level -> importable.
        std::fs::create_dir_all(base.join("pkg_a")).unwrap();
        std::fs::write(base.join("pkg_a/main.k"), "x = 1\n").unwrap();
        // pkg_b: a `.k` file only at a deeper level -> still importable.
        std::fs::create_dir_all(base.join("pkg_b/nested")).unwrap();
        std::fs::write(base.join("pkg_b/nested/deep.k"), "x = 1\n").unwrap();
        // pkg_c: no `.k` file at all -> not importable.
        std::fs::create_dir_all(base.join("pkg_c")).unwrap();
        std::fs::write(base.join("pkg_c/readme.txt"), "not kcl\n").unwrap();
        // Hidden directories, even with `.k` files, are never importable.
        std::fs::create_dir_all(base.join(".hidden")).unwrap();
        std::fs::write(base.join(".hidden/secret.k"), "x = 1\n").unwrap();
        std::fs::create_dir_all(base.join(".git/objects")).unwrap();
        std::fs::write(base.join(".git/objects/abcd"), "blob\n").unwrap();

        let program = Program {
            root: base.display().to_string(),
            ..Default::default()
        };
        let pos = KCLPos {
            filename: String::new(),
            line: 1,
            column: Some(0),
        };
        let mut labels: Vec<String> = super::completion_import_internal_pkg(&program, &pos)
            .iter()
            .map(|item| item.label.clone())
            .collect();
        labels.sort();

        assert_eq!(labels, vec!["pkg_a".to_string(), "pkg_b".to_string()]);
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A `Toolchain` that fails every call: completion must never reach it when
    /// the workspace metadata is already cached (a real call would spawn
    /// `kcl mod metadata` as a subprocess inside the request handler).
    struct NoCallToolchain;

    impl Toolchain for NoCallToolchain {
        fn fetch_metadata(&self, _manifest_path: std::path::PathBuf) -> anyhow::Result<Metadata> {
            panic!("fetch_metadata must not be called when the metadata cache is available");
        }

        fn update_dependencies(&self, _manifest_path: std::path::PathBuf) -> anyhow::Result<()> {
            panic!("update_dependencies must not be called during completion");
        }
    }

    /// Dot completion inside an `import external_pkg.<cursor>` statement must
    /// resolve the package location from the cached workspace metadata instead
    /// of querying the toolchain on every keystroke.
    #[test]
    fn import_dot_completion_resolves_external_pkg_from_cached_metadata() {
        let ws_root = unique_temp_dir("kcl-lsp-import-ws");
        let pkg_root = unique_temp_dir("kcl-lsp-import-external");
        std::fs::create_dir_all(pkg_root.join("sub")).unwrap();
        std::fs::write(pkg_root.join("sub/model.k"), "x = 1\n").unwrap();
        std::fs::write(pkg_root.join("util.k"), "x = 1\n").unwrap();

        let metadata = Metadata {
            packages: [(
                "my_pkg".to_string(),
                Package {
                    name: "my_pkg".to_string(),
                    manifest_path: pkg_root.clone(),
                },
            )]
            .into_iter()
            .collect(),
        };
        let program = Program {
            root: ws_root.display().to_string(),
            ..Default::default()
        };
        let pos = KCLPos {
            filename: String::new(),
            line: 1,
            column: Some(0),
        };

        // `import my_pkg.<cursor>`: completes the sub-packages/modules of my_pkg.
        let stmt = import_stmt_node("my_pkg");
        let res = super::dot_completion_in_import_stmt(
            &stmt,
            &pos,
            &program,
            &NoCallToolchain,
            Some(&metadata),
        )
        .unwrap();
        let mut labels: Vec<String> = match res {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        labels.sort();
        assert_eq!(labels, vec!["sub".to_string(), "util".to_string()]);

        // `import my_pkg.sub.<cursor>`: resolves the sub-package directory from
        // the cached metadata and completes its modules.
        let stmt = import_stmt_node("my_pkg.sub");
        let res = super::dot_completion_in_import_stmt(
            &stmt,
            &pos,
            &program,
            &NoCallToolchain,
            Some(&metadata),
        )
        .unwrap();
        let labels: Vec<String> = match res {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        assert_eq!(labels, vec!["model".to_string()]);

        let _ = std::fs::remove_dir_all(&ws_root);
        let _ = std::fs::remove_dir_all(&pkg_root);
    }
}
