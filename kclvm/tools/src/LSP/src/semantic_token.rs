use kclvm_error::Position;
use kclvm_parser::{load_program, LoadProgramOptions};
use kclvm_sema::{
    resolver::{
        resolve_program,
        scope::{Scope, ScopeObject},
    },
};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::STRUCT,
    SemanticTokenType::COMMENT,
    SemanticTokenType::NUMBER,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::PROPERTY,
];

#[derive(Clone, Debug)]
pub struct ImCompleteSemanticToken {
    pub start: Position,
    pub end: Position,
    pub kind: u32,
    pub length: u32,
}

pub fn kcl_scopeobj_to_imcomplete_semantic_token(
    obj: &ScopeObject,
) -> Option<ImCompleteSemanticToken> {
    match obj.kind {
        kclvm_sema::resolver::scope::ScopeObjectKind::Variable => Some(ImCompleteSemanticToken {
            start: obj.start.clone(),
            end: obj.end.clone(),
            kind: LEGEND_TYPE
                .iter()
                .position(|item| item == &SemanticTokenType::VARIABLE)
                .unwrap() as u32,
            length: obj.name.len() as u32,
        }),

        kclvm_sema::resolver::scope::ScopeObjectKind::Attribute => Some(ImCompleteSemanticToken {
            start: obj.start.clone(),
            end: obj.end.clone(),
            kind: LEGEND_TYPE
                .iter()
                .position(|item| item == &SemanticTokenType::PROPERTY)
                .unwrap() as u32,
            length: obj.name.len() as u32,
        }),

        kclvm_sema::resolver::scope::ScopeObjectKind::Definition => {
            let mut start_pos = obj.start.clone();
            start_pos.column = Some(start_pos.column.unwrap_or(0) + "schema ".len() as u64);
            Some(ImCompleteSemanticToken {
                start: start_pos,
                end: obj.end.clone(),
                kind: LEGEND_TYPE
                    .iter()
                    .position(|item| item == &SemanticTokenType::STRUCT)
                    .unwrap() as u32,
                length: obj.name.len() as u32,
            })
        }
        kclvm_sema::resolver::scope::ScopeObjectKind::Parameter => Some(ImCompleteSemanticToken {
            start: obj.start.clone(),
            end: obj.end.clone(),
            kind: LEGEND_TYPE
                .iter()
                .position(|item| item == &SemanticTokenType::PARAMETER)
                .unwrap() as u32,
            length: obj.name.len() as u32,
        }),
        kclvm_sema::resolver::scope::ScopeObjectKind::TypeAlias => None,
        kclvm_sema::resolver::scope::ScopeObjectKind::Module => None,
    }
}

pub fn get_scope_imcomplete_sema_token(
    scope: &Scope,
    file_name: &str,
) -> Vec<ImCompleteSemanticToken> {
    let mut tokens = vec![];
    for (_, obj) in &scope.elems {
        let obj = obj.borrow();
        if obj.start.filename == file_name {
            if let Some(token) = kcl_scopeobj_to_imcomplete_semantic_token(&obj) {
                tokens.push(token);
            }
        }
    }
    for child in &scope.children {
        let child = child.borrow();
        tokens.append(&mut get_scope_imcomplete_sema_token(&child, file_name));
    }
    tokens
}

pub fn get_imcomplete_semantic_tokens(
    files: &[&str],
    ops: Option<LoadProgramOptions>,
    file: &str,
) -> Vec<ImCompleteSemanticToken> {
    let mut program = load_program(&files, ops).unwrap();

    let prog_scope = resolve_program(&mut program);

    let mut tokens = vec![];
    let scope_map = prog_scope.scope_map.clone();
    for (_, scope) in scope_map.iter() {
        let s = scope.borrow();
        tokens.append(&mut get_scope_imcomplete_sema_token(&s, file));
    }
    tokens
}

pub fn imcomplete_semantic_tokens_to_semantic_tokens(
    tokens: &mut Vec<ImCompleteSemanticToken>,
) -> Vec<SemanticToken> {
    tokens.sort_by(|a, b| {
        if a.start.line == b.start.line {
            a.start
                .column
                .unwrap_or(0)
                .cmp(&b.start.column.unwrap_or(0))
        } else {
            a.start.line.cmp(&b.start.line)
        }
    });

    let mut pre_line = 0;
    let mut pre_start = 0;

    let semantic_tokens: Vec<SemanticToken> = tokens
        .iter()
        .map(|obj| {
            let line = obj.start.line - 1;
            let start = obj.start.column.unwrap_or(0);

            let delta_line: u32 = (line - pre_line) as u32;
            let delta_start: u32 = (if delta_line == 0 {
                start - pre_start
            } else {
                start
            }) as u32;
            let length = obj.length;
            let ret = SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type: obj.kind,
                token_modifiers_bitset: 0,
            };
            pre_line = line;
            pre_start = start;
            ret
        })
        .collect();
    semantic_tokens
}
