use indexmap::IndexSet;
use kclvm_ast::ast::{
    ConfigEntry, Expr, Identifier, Node, NodeRef, PosTuple, Program, SchemaExpr, SchemaStmt, Stmt,
    Type,
};
use kclvm_ast::node_ref;
use kclvm_ast::pos::ContainsPos;

use kclvm_driver::lookup_compile_unit;
use kclvm_error::Diagnostic;
use kclvm_error::Position as KCLPos;
use kclvm_parser::entry::get_dir_files;
use kclvm_parser::{load_program, KCLModuleCache, ParseSessionRef};
use kclvm_sema::advanced_resolver::AdvancedResolver;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::namer::Namer;

use kclvm_sema::resolver::resolve_program_with_opts;
use kclvm_sema::resolver::scope::KCLScopeCache;

use crate::from_lsp;
use crate::state::KCLVfs;
use lsp_types::Url;
use parking_lot::RwLockReadGuard;
use ra_ap_vfs::{FileId, Vfs};
use serde::{de::DeserializeOwned, Serialize};

use std::path::Path;
use std::{fs, panic};

#[allow(unused)]
/// Deserializes a `T` from a json value.
pub(crate) fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: serde_json::Value,
) -> anyhow::Result<T> {
    T::deserialize(&json)
        .map_err(|e| anyhow::anyhow!("could not deserialize {}: {}: {}", what, e, json))
}

/// Converts the `T` to a json value
pub(crate) fn to_json<T: Serialize>(value: T) -> anyhow::Result<serde_json::Value> {
    serde_json::to_value(value).map_err(|e| anyhow::anyhow!("could not serialize to json: {}", e))
}

/// Get the filename from the given VFS using the file id.
pub(crate) fn get_file_name(vfs: RwLockReadGuard<Vfs>, file_id: FileId) -> anyhow::Result<String> {
    if let Some(path) = vfs.file_path(file_id).as_path() {
        Ok(path
            .as_ref()
            .to_str()
            .ok_or(anyhow::anyhow!("Failed to get file file"))?
            .to_string())
    } else {
        Err(anyhow::anyhow!(
            "{} isn't on the file system.",
            vfs.file_path(file_id)
        ))
    }
}

pub(crate) struct Params {
    pub file: String,
    pub module_cache: Option<KCLModuleCache>,
    pub scope_cache: Option<KCLScopeCache>,
    pub vfs: Option<KCLVfs>,
}

pub(crate) fn compile_with_params(
    params: Params,
) -> anyhow::Result<(Program, IndexSet<Diagnostic>, GlobalState)> {
    // Lookup compile unit (kcl.mod or kcl.yaml) from the entry file.
    let (mut files, opt) = lookup_compile_unit(&params.file, true);
    if !files.contains(&params.file) {
        files.push(params.file.clone());
    }
    let files: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let mut opt = opt.unwrap_or_default();
    opt.load_plugins = true;
    // Update opt.k_code_list
    if let Some(vfs) = params.vfs {
        let mut k_code_list = load_files_code_from_vfs(&files, vfs)?;
        opt.k_code_list.append(&mut k_code_list);
    }
    match panic::catch_unwind(move || {
        // Parser
        let sess = ParseSessionRef::default();
        let mut program = load_program(sess.clone(), &files, Some(opt), params.module_cache)
            .unwrap()
            .program;
        // Resolver
        let prog_scope = resolve_program_with_opts(
            &mut program,
            kclvm_sema::resolver::Options {
                merge_program: false,
                type_erasure: false,
                ..Default::default()
            },
            params.scope_cache,
        );
        // Please note that there is no global state cache at this stage.
        let gs = GlobalState::default();
        let gs = Namer::find_symbols(&program, gs);
        let gs = AdvancedResolver::resolve_program(&program, gs, prog_scope.node_ty_map.clone());
        // Merge parse diagnostic and resolve diagnostic
        sess.append_diagnostic(prog_scope.handler.diagnostics.clone());
        let diags = sess.1.borrow().diagnostics.clone();
        Ok((program, diags, gs))
    }) {
        Ok(res) => return res,
        Err(e) => Err(anyhow::anyhow!("Compile failed: {:?}", e)),
    }
}

/// Update text with TextDocumentContentChangeEvent param
pub(crate) fn apply_document_changes(
    old_text: &mut String,
    content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) {
    for change in content_changes {
        match change.range {
            Some(range) => {
                let range = from_lsp::text_range(old_text, range);
                old_text.replace_range(range, &change.text);
            }
            None => {
                *old_text = change.text;
            }
        }
    }
}

fn load_files_code_from_vfs(files: &[&str], vfs: KCLVfs) -> anyhow::Result<Vec<String>> {
    let mut res = vec![];
    let vfs = &mut vfs.read();
    for file in files {
        let url = Url::from_file_path(file)
            .map_err(|_| anyhow::anyhow!("can't convert file to url: {}", file))?;
        let path = from_lsp::abs_path(&url)?;
        match vfs.file_id(&path.clone().into()) {
            Some(id) => {
                // Load code from vfs if exist
                res.push(String::from_utf8(vfs.file_contents(id).to_vec()).unwrap());
            }
            None => {
                // In order to ensure that k_file corresponds to k_code, load the code from the file system if not exist
                let p: &Path = path.as_ref();
                if p.is_file() {
                    res.push(
                        fs::read_to_string(path)
                            .map_err(|_| anyhow::anyhow!("can't convert file to url: {}", file))?,
                    );
                } else if p.is_dir() {
                    let k_files = get_dir_files(p.to_str().unwrap(), false)
                        .map_err(|_| anyhow::anyhow!("can't get dir files: {} ", file))?;
                    for k_file in k_files {
                        let k_file_path = Path::new(k_file.as_str());
                        res.push(
                            fs::read_to_string(k_file_path).map_err(|_| {
                                anyhow::anyhow!("can't convert file to url: {}", file)
                            })?,
                        );
                    }
                }
            }
        }
    }
    Ok(res)
}

macro_rules! walk_if_contains {
    ($expr: expr, $pos: expr, $schema_def: expr) => {
        if $expr.contains_pos($pos) {
            return inner_most_expr(&$expr, &$pos, $schema_def);
        }
    };
}

macro_rules! walk_if_contains_with_new_expr {
    ($expr: expr, $pos: expr, $schema_def: expr, $kind: expr) => {
        if $expr.contains_pos($pos) {
            walk_if_contains!(
                Node::node_with_pos(
                    $kind($expr.node.clone()),
                    (
                        $expr.filename.clone(),
                        $expr.line,
                        $expr.column,
                        $expr.end_line,
                        $expr.end_column,
                    ),
                ),
                $pos,
                $schema_def
            );
        }
    };
}

macro_rules! walk_option_if_contains {
    ($opt: expr, $pos: expr, $schema_def: expr) => {
        if let Some(expr) = &$opt {
            walk_if_contains!(expr, $pos, $schema_def)
        }
    };
}

macro_rules! walk_list_if_contains {
    ($list: expr, $pos: expr, $schema_def: expr) => {
        for elem in &$list {
            walk_if_contains!(elem, $pos, $schema_def)
        }
    };
}

fn transfer_ident_names(names: Vec<String>, pos: &PosTuple) -> Vec<Node<String>> {
    let mut new_names = vec![];
    let mut col = pos.2;
    for name in &names {
        let mut name_pos = pos.clone();
        name_pos.2 = col;
        name_pos.4 = col + name.len() as u64;
        new_names.push(Node::node_with_pos(name.clone(), name_pos));
        col = col + name.len() as u64 + ".".len() as u64;
    }
    new_names
}

/// Recursively finds the inner most expr and its schema_def expr if in a schema expr(e.g., schema_attr and schema_expr)
/// in a stmt according to the position.
pub(crate) fn inner_most_expr_in_stmt(
    stmt: &Stmt,
    pos: &KCLPos,
    schema_def: Option<Node<Expr>>,
) -> (Option<Node<Expr>>, Option<Node<Expr>>) {
    match stmt {
        Stmt::Assign(assign_stmt) => {
            if let Some(ty) = &assign_stmt.ty {
                if ty.contains_pos(pos) {
                    return (build_identifier_from_ty_string(ty, pos), schema_def);
                }
            }
            walk_if_contains!(assign_stmt.value, pos, schema_def);

            for expr in &assign_stmt.targets {
                walk_if_contains_with_new_expr!(expr, pos, schema_def, Expr::Identifier);
            }
            (None, schema_def)
        }
        Stmt::TypeAlias(type_alias_stmt) => {
            walk_if_contains_with_new_expr!(
                type_alias_stmt.type_name,
                pos,
                schema_def,
                Expr::Identifier
            );
            (None, schema_def)
        }
        Stmt::Expr(expr_stmt) => {
            walk_list_if_contains!(expr_stmt.exprs, pos, schema_def);
            (None, schema_def)
        }
        Stmt::Unification(unification_stmt) => {
            walk_if_contains_with_new_expr!(
                unification_stmt.target,
                pos,
                schema_def,
                Expr::Identifier
            );

            walk_if_contains_with_new_expr!(unification_stmt.value, pos, schema_def, Expr::Schema);

            (None, schema_def)
        }
        Stmt::AugAssign(aug_assign_stmt) => {
            walk_if_contains!(aug_assign_stmt.value, pos, schema_def);
            walk_if_contains_with_new_expr!(
                aug_assign_stmt.target,
                pos,
                schema_def,
                Expr::Identifier
            );
            (None, schema_def)
        }
        Stmt::Assert(assert_stmt) => {
            walk_if_contains!(assert_stmt.test, pos, schema_def);
            walk_option_if_contains!(assert_stmt.if_cond, pos, schema_def);
            walk_option_if_contains!(assert_stmt.msg, pos, schema_def);
            (None, schema_def)
        }
        Stmt::If(if_stmt) => {
            walk_if_contains!(if_stmt.cond, pos, schema_def);
            for stmt in &if_stmt.body {
                if stmt.contains_pos(pos) {
                    return inner_most_expr_in_stmt(&stmt.node, pos, schema_def);
                }
            }
            for stmt in &if_stmt.orelse {
                if stmt.contains_pos(pos) {
                    return inner_most_expr_in_stmt(&stmt.node, pos, schema_def);
                }
            }
            (None, schema_def)
        }
        Stmt::Schema(schema_stmt) => {
            walk_if_contains!(
                Node::node_with_pos(
                    Expr::Identifier(Identifier {
                        names: transfer_ident_names(
                            vec![schema_stmt.name.node.clone()],
                            &(
                                schema_stmt.name.filename.clone(),
                                schema_stmt.name.line,
                                schema_stmt.name.column,
                                schema_stmt.name.end_line,
                                schema_stmt.name.end_column,
                            ),
                        ),
                        pkgpath: "".to_string(),
                        ctx: kclvm_ast::ast::ExprContext::Load,
                    }),
                    (
                        schema_stmt.name.filename.clone(),
                        schema_stmt.name.line,
                        schema_stmt.name.column,
                        schema_stmt.name.end_line,
                        schema_stmt.name.end_column,
                    ),
                ),
                pos,
                schema_def
            );
            if let Some(parent_id) = &schema_stmt.parent_name {
                walk_if_contains_with_new_expr!(parent_id, pos, schema_def, Expr::Identifier);
            }
            if let Some(host) = &schema_stmt.for_host_name {
                walk_if_contains_with_new_expr!(host, pos, schema_def, Expr::Identifier);
            }
            for mixin in &schema_stmt.mixins {
                walk_if_contains_with_new_expr!(mixin, pos, schema_def, Expr::Identifier);
            }
            for stmt in &schema_stmt.body {
                if stmt.contains_pos(pos) {
                    return inner_most_expr_in_stmt(&stmt.node, pos, schema_def);
                }
            }
            for decorator in &schema_stmt.decorators {
                walk_if_contains_with_new_expr!(decorator, pos, schema_def, Expr::Call);
            }
            for check in &schema_stmt.checks {
                walk_if_contains_with_new_expr!(check, pos, schema_def, Expr::Check);
            }
            (None, schema_def)
        }
        Stmt::SchemaAttr(schema_attr_expr) => {
            walk_if_contains!(
                Node::node_with_pos(
                    Expr::Identifier(Identifier {
                        names: vec![*schema_attr_expr.name.clone()],
                        pkgpath: "".to_string(),
                        ctx: kclvm_ast::ast::ExprContext::Load,
                    }),
                    (
                        schema_attr_expr.name.filename.clone(),
                        schema_attr_expr.name.line,
                        schema_attr_expr.name.column,
                        schema_attr_expr.name.end_line,
                        schema_attr_expr.name.end_column,
                    ),
                ),
                pos,
                schema_def
            );
            if schema_attr_expr.ty.contains_pos(pos) {
                return (
                    build_identifier_from_ty_string(&schema_attr_expr.ty, pos),
                    schema_def,
                );
            }
            walk_option_if_contains!(schema_attr_expr.value, pos, schema_def);
            for decorator in &schema_attr_expr.decorators {
                walk_if_contains_with_new_expr!(decorator, pos, schema_def, Expr::Call);
            }
            (None, schema_def)
        }
        Stmt::Rule(rule_stmt) => {
            for parent_id in &rule_stmt.parent_rules {
                walk_if_contains_with_new_expr!(parent_id, pos, schema_def, Expr::Identifier);
            }
            for decorator in &rule_stmt.decorators {
                walk_if_contains_with_new_expr!(decorator, pos, schema_def, Expr::Call);
            }
            for check in &rule_stmt.checks {
                walk_if_contains_with_new_expr!(check, pos, schema_def, Expr::Check);
            }
            (None, schema_def)
        }
        Stmt::Import(_) => (None, schema_def),
    }
}

/// Recursively finds the inner most expr and its schema_def expr if in a schema expr(e.g., schema_attr in schema_expr)
/// in a expr according to the position.
pub(crate) fn inner_most_expr(
    expr: &Node<Expr>,
    pos: &KCLPos,
    schema_def: Option<Node<Expr>>,
) -> (Option<Node<Expr>>, Option<Node<Expr>>) {
    if !expr.contains_pos(pos) {
        return (None, None);
    }
    match &expr.node {
        Expr::Identifier(_) => (Some(expr.clone()), schema_def),
        Expr::Selector(select_expr) => {
            walk_if_contains!(select_expr.value, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Schema(schema_expr) => {
            walk_if_contains_with_new_expr!(schema_expr.name, pos, schema_def, Expr::Identifier);
            walk_list_if_contains!(schema_expr.args, pos, schema_def);

            for kwargs in &schema_expr.kwargs {
                walk_if_contains_with_new_expr!(kwargs, pos, schema_def, Expr::Keyword);
            }
            if schema_expr.config.contains_pos(pos) {
                return inner_most_expr(&schema_expr.config, pos, Some(expr.clone()));
            }
            (Some(expr.clone()), schema_def)
        }
        Expr::Config(config_expr) => {
            for item in &config_expr.items {
                if item.contains_pos(pos) {
                    return inner_most_expr_in_config_entry(item, pos, schema_def);
                }
            }
            (Some(expr.clone()), schema_def)
        }
        Expr::Unary(unary_expr) => {
            walk_if_contains!(unary_expr.operand, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Binary(binary_expr) => {
            walk_if_contains!(binary_expr.left, pos, schema_def);
            walk_if_contains!(binary_expr.right, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::If(if_expr) => {
            walk_if_contains!(if_expr.body, pos, schema_def);
            walk_if_contains!(if_expr.cond, pos, schema_def);
            walk_if_contains!(if_expr.orelse, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Call(call_expr) => {
            walk_list_if_contains!(call_expr.args, pos, schema_def);
            for keyword in &call_expr.keywords {
                walk_if_contains_with_new_expr!(keyword, pos, schema_def, Expr::Keyword);
            }
            walk_if_contains!(call_expr.func, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Paren(paren_expr) => {
            walk_if_contains!(paren_expr.expr, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Quant(quant_expr) => {
            walk_if_contains!(quant_expr.target, pos, schema_def);
            for var in &quant_expr.variables {
                walk_if_contains_with_new_expr!(var, pos, schema_def, Expr::Identifier);
            }
            walk_if_contains!(quant_expr.test, pos, schema_def);
            walk_option_if_contains!(quant_expr.if_cond, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::List(list_expr) => {
            walk_list_if_contains!(list_expr.elts, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::ListIfItem(list_if_item_expr) => {
            walk_if_contains!(list_if_item_expr.if_cond, pos, schema_def);
            walk_list_if_contains!(list_if_item_expr.exprs, pos, schema_def);
            walk_option_if_contains!(list_if_item_expr.orelse, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::ListComp(list_comp_expr) => {
            walk_if_contains!(list_comp_expr.elt, pos, schema_def);
            for comp_clause in &list_comp_expr.generators {
                walk_if_contains_with_new_expr!(comp_clause, pos, schema_def, Expr::CompClause);
            }
            (Some(expr.clone()), schema_def)
        }
        Expr::Starred(starred_exor) => {
            walk_if_contains!(starred_exor.value, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::DictComp(dict_comp) => {
            walk_option_if_contains!(dict_comp.entry.key, pos, schema_def);
            walk_if_contains!(dict_comp.entry.value, pos, schema_def);

            for generator in &dict_comp.generators {
                if generator.contains_pos(pos) {
                    walk_if_contains_with_new_expr!(generator, pos, schema_def, Expr::CompClause);
                }
            }
            (Some(expr.clone()), schema_def)
        }
        Expr::ConfigIfEntry(config_if_entry_expr) => {
            walk_if_contains!(config_if_entry_expr.if_cond, pos, schema_def);
            for item in &config_if_entry_expr.items {
                if item.contains_pos(pos) {
                    return inner_most_expr_in_config_entry(item, pos, schema_def);
                }
            }
            walk_option_if_contains!(config_if_entry_expr.orelse, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::CompClause(comp_clause) => {
            for target in &comp_clause.targets {
                walk_if_contains_with_new_expr!(target, pos, schema_def, Expr::Identifier);
            }
            walk_if_contains!(comp_clause.iter, pos, schema_def);
            walk_list_if_contains!(comp_clause.ifs, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Check(check_expr) => {
            walk_if_contains!(check_expr.test, pos, schema_def);
            walk_option_if_contains!(check_expr.if_cond, pos, schema_def);
            walk_option_if_contains!(check_expr.msg, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Lambda(lambda_expr) => {
            if let Some(args) = &lambda_expr.args {
                walk_if_contains_with_new_expr!(args, pos, schema_def, Expr::Arguments);
            }
            for stmt in &lambda_expr.body {
                if stmt.contains_pos(pos) {
                    return inner_most_expr_in_stmt(&stmt.node, pos, schema_def);
                }
            }

            (Some(expr.clone()), schema_def)
        }
        Expr::Subscript(subscript_expr) => {
            walk_if_contains!(subscript_expr.value, pos, schema_def);
            walk_option_if_contains!(subscript_expr.index, pos, schema_def);
            walk_option_if_contains!(subscript_expr.lower, pos, schema_def);
            walk_option_if_contains!(subscript_expr.upper, pos, schema_def);
            walk_option_if_contains!(subscript_expr.step, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Keyword(keyword) => {
            walk_if_contains_with_new_expr!(keyword.arg, pos, schema_def, Expr::Identifier);
            walk_option_if_contains!(keyword.value, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Arguments(argument) => {
            for arg in &argument.args {
                walk_if_contains_with_new_expr!(arg, pos, schema_def, Expr::Identifier);
            }
            for default in &argument.defaults {
                walk_option_if_contains!(default, pos, schema_def);
            }
            for ty in argument.ty_list.iter().flatten() {
                if ty.contains_pos(pos) {
                    return (
                        Some(build_identifier_from_string(&node_ref!(
                            ty.node.to_string(),
                            ty.pos()
                        ))),
                        schema_def,
                    );
                }
            }
            (Some(expr.clone()), schema_def)
        }
        Expr::Compare(compare_expr) => {
            walk_if_contains!(compare_expr.left, pos, schema_def);
            walk_list_if_contains!(compare_expr.comparators, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::NumberLit(_) => (Some(expr.clone()), schema_def),
        Expr::StringLit(_) => (Some(expr.clone()), schema_def),
        Expr::NameConstantLit(_) => (Some(expr.clone()), schema_def),
        Expr::JoinedString(joined_string) => {
            walk_list_if_contains!(joined_string.values, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::FormattedValue(formatted_value) => {
            walk_if_contains!(formatted_value.value, pos, schema_def);
            (Some(expr.clone()), schema_def)
        }
        Expr::Missing(_) => (Some(expr.clone()), schema_def),
    }
}

fn inner_most_expr_in_config_entry(
    config_entry: &Node<ConfigEntry>,
    pos: &KCLPos,
    schema_def: Option<Node<Expr>>,
) -> (Option<Node<Expr>>, Option<Node<Expr>>) {
    if let Some(key) = &config_entry.node.key {
        if key.contains_pos(pos) {
            return inner_most_expr(key, pos, schema_def);
        }
    }
    if config_entry.node.value.contains_pos(pos) {
        inner_most_expr(&config_entry.node.value, pos, None)
    } else {
        (None, schema_def)
    }
}

pub(crate) fn is_in_schema_expr(
    program: &Program,
    pos: &KCLPos,
) -> Option<(Node<Stmt>, SchemaExpr)> {
    match program.pos_to_stmt(pos) {
        Some(node) => {
            let parent_expr = inner_most_expr_in_stmt(&node.node, pos, None).1;
            match parent_expr {
                Some(expr) => match expr.node {
                    Expr::Schema(schema) => Some((node, schema)),
                    _ => None,
                },
                None => None,
            }
        }
        None => None,
    }
}

pub(crate) fn is_in_docstring(
    program: &Program,
    pos: &KCLPos,
) -> Option<(NodeRef<String>, SchemaStmt)> {
    match program.pos_to_stmt(pos) {
        Some(node) => match node.node.clone() {
            Stmt::Schema(schema) => match schema.doc {
                Some(ref doc) => {
                    if doc.contains_pos(pos) {
                        Some((doc.clone(), schema))
                    } else {
                        None
                    }
                }
                None => None,
            },
            _ => None,
        },
        None => None,
    }
}

/// Build a temp identifier expr with string
fn build_identifier_from_string(s: &NodeRef<String>) -> Node<Expr> {
    Node::node_with_pos(
        Expr::Identifier(Identifier {
            names: transfer_ident_names(
                vec![s.node.clone()],
                &(
                    s.filename.clone(),
                    s.line,
                    s.column,
                    s.end_line,
                    s.end_column,
                ),
            ),
            pkgpath: "".to_string(),
            ctx: kclvm_ast::ast::ExprContext::Load,
        }),
        (
            s.filename.clone(),
            s.line,
            s.column,
            s.end_line,
            s.end_column,
        ),
    )
}

/// Build a temp identifier expr with string
fn build_identifier_from_ty_string(ty: &NodeRef<Type>, pos: &KCLPos) -> Option<Node<Expr>> {
    if !ty.contains_pos(pos) {
        return None;
    }
    match &ty.node {
        Type::Any => None,
        Type::Named(id) => Some(Node::node_with_pos(
            Expr::Identifier(id.clone()),
            (
                ty.filename.clone(),
                ty.line,
                ty.column,
                ty.end_line,
                ty.end_column,
            ),
        )),
        Type::Basic(_) => None,
        Type::List(list_ty) => {
            if let Some(inner) = &list_ty.inner_type {
                if inner.contains_pos(pos) {
                    return build_identifier_from_ty_string(inner, pos);
                }
            }
            None
        }
        Type::Dict(dict_ty) => {
            if let Some(key_ty) = &dict_ty.key_type {
                if key_ty.contains_pos(pos) {
                    return build_identifier_from_ty_string(key_ty, pos);
                }
            }
            if let Some(value_ty) = &dict_ty.value_type {
                if value_ty.contains_pos(pos) {
                    return build_identifier_from_ty_string(value_ty, pos);
                }
            }
            None
        }
        Type::Union(union_ty) => {
            for ty in &union_ty.type_elements {
                if ty.contains_pos(pos) {
                    return build_identifier_from_ty_string(ty, pos);
                }
            }
            None
        }
        Type::Literal(_) => None,
        Type::Function(_) => None,
    }
}
