use std::path::Path;

use kclvm_ast::ast::Program;
use kclvm_ast::MAIN_PKG;
use kclvm_sema::resolver::scope::ProgramScope;
use kclvm_sema::resolver::scope::Scope;
use kclvm_sema::resolver::scope::ScopeKind;
use kclvm_sema::resolver::scope::ScopeObject;
use kclvm_sema::resolver::scope::ScopeObjectKind;
use lsp_types::Range;
use lsp_types::{DocumentSymbol, DocumentSymbolResponse, SymbolKind};

use crate::to_lsp::lsp_pos;

pub(crate) fn document_symbol(
    file: &str,
    _program: &Program,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::DocumentSymbolResponse> {
    let mut documentsymbols: Vec<DocumentSymbol> = vec![];
    let scope = prog_scope.scope_map.get(MAIN_PKG).unwrap().borrow();
    // Get variable in scope
    for obj in scope.elems.values().filter(|obj| {
        if let Ok(canonicalized_path) = Path::new(&obj.borrow().start.filename).canonicalize() {
            // skip schema definition
            canonicalized_path.eq(Path::new(file))
                && obj.borrow().kind != ScopeObjectKind::Definition
        } else {
            false
        }
    }) {
        documentsymbols.push(scope_obj_to_document_symbol(obj.borrow().clone()));
    }
    // Get schema definition in scope
    for child in scope.children.iter().filter(|child| {
        if let Ok(canonicalized_path) = Path::new(&child.borrow().start.filename).canonicalize() {
            canonicalized_path.eq(Path::new(file))
        } else {
            false
        }
    }) {
        if let Some(symbol) = schema_scope_to_document_symbol(child.borrow().clone()) {
            documentsymbols.push(symbol)
        }
    }
    Some(DocumentSymbolResponse::Nested(documentsymbols))
}

#[allow(deprecated)]
fn schema_scope_to_document_symbol(scope: Scope) -> Option<DocumentSymbol> {
    if let ScopeKind::Schema(schema_name) = &scope.kind {
        let range = Range {
            start: lsp_pos(&scope.start),
            end: lsp_pos(&scope.end),
        };
        Some(DocumentSymbol {
            name: schema_name.clone(),
            kind: SymbolKind::STRUCT,
            range,
            selection_range: range,
            children: Some(
                scope
                    .elems
                    .iter()
                    .map(|(_, obj)| scope_obj_to_document_symbol(obj.borrow().clone()))
                    .collect(),
            ),
            detail: Some("schema".to_string()),
            tags: None,
            deprecated: None,
        })
    } else {
        None
    }
}

#[allow(deprecated)]
fn scope_obj_to_document_symbol(obj: ScopeObject) -> DocumentSymbol {
    let kind = scope_obj_kind_to_document_symbol_kind(obj.kind);
    let range = Range {
        start: lsp_pos(&obj.start),
        end: lsp_pos(&obj.end),
    };
    DocumentSymbol {
        name: obj.name.clone(),
        kind,
        range,
        selection_range: range,
        detail: Some(obj.ty.ty_str()),
        tags: None,
        children: None,
        deprecated: None,
    }
}

fn scope_obj_kind_to_document_symbol_kind(kind: ScopeObjectKind) -> SymbolKind {
    match kind {
        ScopeObjectKind::Variable => SymbolKind::VARIABLE,
        ScopeObjectKind::Attribute => SymbolKind::PROPERTY,
        ScopeObjectKind::Definition => SymbolKind::STRUCT,
        ScopeObjectKind::Parameter => SymbolKind::VARIABLE,
        ScopeObjectKind::TypeAlias => SymbolKind::TYPE_PARAMETER,
        ScopeObjectKind::Module => SymbolKind::MODULE,
    }
}
