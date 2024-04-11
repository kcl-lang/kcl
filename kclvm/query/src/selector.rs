use super::util::{invalid_symbol_selector_spec_error, split_field_path};
use anyhow::Result;
use kclvm_ast::ast;

use std::collections::{HashMap, VecDeque};

use kclvm_ast::path::get_key_path;

use kclvm_ast::walker::MutSelfWalker;
use kclvm_ast_pretty::{print_ast_node, ASTNode};
use kclvm_parser::parse_file;

use kclvm_sema::resolver::Options;

use kclvm_ast::MAIN_PKG;
use kclvm_sema::pre_process::pre_process_program;
use maplit::hashmap;
#[derive(Debug, Default)]
/// UnsupportedSelectee is used to store the unsupported selectee, such as if, for, etc.
pub struct UnsupportedSelectee {
    pub code: String,
}

#[derive(Debug)]
/// Selector is used to select the target variable from the kcl program.
pub struct Selector {
    select_specs: Vec<String>,
    select_result: HashMap<String, String>,
    unsupported: Vec<UnsupportedSelectee>,
    inner: SelectorInner,
}

#[derive(Debug)]
struct SelectorInner {
    current_spec: String,
    current_spec_items: VecDeque<String>,
    spec_store: Vec<Vec<String>>,
    has_err: bool,
}

impl SelectorInner {
    fn default() -> Self {
        Self {
            current_spec: String::new(),
            current_spec_items: VecDeque::new(),
            spec_store: vec![vec![]],
            has_err: false,
        }
    }

    fn pop_front(&mut self) -> Option<String> {
        let selector = self.current_spec_items.pop_front();
        if let Some(selector) = &selector {
            if let Some(store) = self.spec_store.last_mut() {
                store.push(selector.to_string());
            } else {
                return None;
            }
        }
        return selector;
    }

    fn init(&mut self) {
        self.spec_store.push(vec![]);
    }

    fn restore(&mut self) {
        let store_items = self.spec_store.pop();

        if let Some(store_items) = store_items {
            for item in store_items.iter().rev() {
                self.current_spec_items.push_front(item.to_string());
            }
        }
    }
}

impl Selector {
    fn new(select_specs: Vec<String>) -> Result<Self> {
        Ok(Self {
            select_specs,
            select_result: HashMap::new(),
            unsupported: vec![],
            inner: SelectorInner::default(),
        })
    }

    // check_node_supported is used to check if the node is supported.
    fn check_node_supported(&mut self, expr: &ast::Expr) -> bool {
        self.inner.has_err = false;
        match expr {
            ast::Expr::If(if_expr) => self.walk_if_expr(if_expr),
            ast::Expr::ListIfItem(list_if_item_expr) => {
                self.walk_list_if_item_expr(list_if_item_expr)
            }
            ast::Expr::ListComp(list_comp) => self.walk_list_comp(list_comp),
            ast::Expr::DictComp(dict_comp) => self.walk_dict_comp(dict_comp),
            ast::Expr::ConfigIfEntry(config_if_entry_expr) => {
                self.walk_config_if_entry_expr(config_if_entry_expr)
            }
            ast::Expr::CompClause(comp_clause) => self.walk_comp_clause(comp_clause),
            ast::Expr::Lambda(lambda) => self.walk_lambda_expr(lambda),
            _ => {}
        }

        return self.inner.has_err;
    }
}

impl<'ctx> MutSelfWalker for Selector {
    fn walk_module(&mut self, module: &ast::Module) {
        let select_paths = self.select_specs.clone();
        // If there is no select path, walk the entire module
        // And return all the variables in the top level.
        if select_paths.is_empty() {
            for stmt in &module.body {
                self.walk_stmt(&stmt.node);
            }
        }

        for path in &select_paths {
            // split the spec with '.'
            // put the spec into a queue to select the target
            self.inner.current_spec = path.clone();
            self.inner.current_spec_items = path
                .split('.')
                .map(|s| s.to_string())
                .collect::<VecDeque<String>>();

            // walk the module to find the target
            for stmt in &module.body {
                self.walk_stmt(&stmt.node);
            }
        }
    }

    fn walk_unification_stmt(&mut self, unification_stmt: &ast::UnificationStmt) {
        self.inner.init();
        let target = &unification_stmt.target;
        let target = &Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
            target.node.clone(),
        ))));
        let target = get_key_path(&target);

        // If the spec is empty, all the top level variables are returned.
        if self.inner.current_spec.is_empty() {
            let kcode = print_ast_node(ASTNode::Expr(&Box::new(ast::Node::dummy_node(
                ast::Expr::Schema(unification_stmt.value.node.clone()),
            ))));
            self.select_result.insert(target.to_string(), kcode);
        } else {
            // if length of spec is largr or equal to target
            let selector = self.inner.pop_front();
            if let Some(selector) = selector {
                if selector == target.to_string() {
                    if self.inner.current_spec_items.is_empty() {
                        // matched
                        let kcode =
                            print_ast_node(ASTNode::Expr(&Box::new(ast::Node::dummy_node(
                                ast::Expr::Schema(unification_stmt.value.node.clone()),
                            ))));
                        self.select_result.insert(target.to_string(), kcode);
                    } else {
                        // walk ahead
                        self.walk_schema_expr(&unification_stmt.value.node);
                    }
                }
            }
            // the spec is still used up
            // Unmatched, return
            self.inner.restore();
        }
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &ast::AssignStmt) {
        self.inner.init();
        // If the spec is empty, all the top level variables are returned.
        if self.inner.current_spec.is_empty() {
            // check the value of the assign statement is supported
            if self.check_node_supported(&assign_stmt.value.node) {
                return;
            }
            // get the value source code of the assign statement
            let kcode = print_ast_node(ASTNode::Expr(&assign_stmt.value));
            // The length of name for variable in top level is 1
            if assign_stmt.targets.len() == 1 {
                let target = &assign_stmt.targets[0];
                let target = &Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
                    target.node.clone(),
                ))));
                let key = get_key_path(&target);
                self.select_result.insert(key.to_string(), kcode);
            }
        } else {
            // Compare the target with the spec
            if assign_stmt.targets.len() == 1 {
                let target = &assign_stmt.targets[0];
                let target = &Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
                    target.node.clone(),
                ))));
                let target = get_key_path(target);
                let selector = self.inner.pop_front();
                if let Some(selector) = selector {
                    if selector == target.to_string() {
                        if self.inner.current_spec_items.is_empty() {
                            // check the value of the assign statement is supported
                            if self.check_node_supported(&assign_stmt.value.node) {
                                self.inner.restore();
                                return;
                            }

                            // matched
                            let kcode = print_ast_node(ASTNode::Expr(&assign_stmt.value));
                            self.select_result.insert(target.to_string(), kcode);
                        } else {
                            // walk ahead
                            self.walk_expr(&assign_stmt.value.node)
                        }
                    }
                }
                // if lentgh of spec is less than target
                // Unmatched, return
                self.inner.restore();
            }
        }
    }

    fn walk_config_expr(&mut self, config_expr: &ast::ConfigExpr) {
        self.inner.init();
        let selector = self.inner.pop_front();

        if let Some(selector) = selector {
            for item in &config_expr.items {
                let key = get_key_path(&item.node.key);
                // key is empty, the value of the config entry may be supported action. e.g. if, for
                if key.is_empty() {
                    // chack the value of the config entry is supported
                    if self.check_node_supported(&item.node.value.node) {
                        self.inner.restore();
                        return;
                    }
                }
                // match the key with the selector
                if key == selector {
                    if self.inner.current_spec_items.is_empty() {
                        // If all the spec items are matched
                        // check and return
                        if self.check_node_supported(&item.node.value.node) {
                            return;
                        }
                        let kcode = print_ast_node(ASTNode::Expr(&item.node.value));
                        self.select_result
                            .insert(self.inner.current_spec.to_string(), kcode);
                    } else {
                        // the spec is still not used up
                        // walk ahead
                        self.walk_expr(&item.node.value.node);
                    }
                }
            }
            self.inner.restore();
        }
    }

    fn walk_if_expr(&mut self, if_expr: &ast::IfExpr) {
        self.unsupported.push(UnsupportedSelectee::default());
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::If(if_expr.clone())),
        )));
        self.unsupported.push(un_supported_selectee);
        self.inner.has_err = true;
    }

    fn walk_list_if_item_expr(&mut self, list_if_item_expr: &ast::ListIfItemExpr) {
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::ListIfItem(list_if_item_expr.clone())),
        )));
        self.unsupported.push(un_supported_selectee);

        self.inner.has_err = true;
    }

    fn walk_list_comp(&mut self, list_comp: &ast::ListComp) {
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::ListComp(list_comp.clone())),
        )));
        self.unsupported.push(un_supported_selectee);

        self.inner.has_err = true;
    }

    fn walk_dict_comp(&mut self, dict_comp: &ast::DictComp) {
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::DictComp(dict_comp.clone())),
        )));
        self.unsupported.push(un_supported_selectee);

        self.inner.has_err = true;
    }

    fn walk_config_if_entry_expr(&mut self, config_if_entry_expr: &ast::ConfigIfEntryExpr) {
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::ConfigIfEntry(config_if_entry_expr.clone())),
        )));
        self.unsupported.push(un_supported_selectee);

        self.inner.has_err = true;
    }

    fn walk_comp_clause(&mut self, comp_clause: &ast::CompClause) {
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::CompClause(comp_clause.clone())),
        )));
        self.unsupported.push(un_supported_selectee);

        self.inner.has_err = true;
    }

    fn walk_lambda_expr(&mut self, lambda_expr: &ast::LambdaExpr) {
        let mut un_supported_selectee = UnsupportedSelectee::default();
        un_supported_selectee.code = print_ast_node(ASTNode::Expr(&Box::new(
            ast::Node::dummy_node(ast::Expr::Lambda(lambda_expr.clone())),
        )));
        self.unsupported.push(un_supported_selectee);
        self.inner.has_err = true;
    }
}

pub struct ListVariablesResult {
    pub select_result: HashMap<String, String>,
    pub unsupported: Vec<UnsupportedSelectee>,
}

/// list_options provides users with the ability to parse kcl program and get all option
/// calling information.
pub fn list_variables(file: String, specs: Vec<String>) -> Result<ListVariablesResult> {
    let mut selector = Selector::new(specs)?;
    let ast = parse_file(&file, None)?;

    let mut opts = Options::default();
    opts.merge_program = true;
    pre_process_program(
        &mut ast::Program {
            root: file,
            pkgs: hashmap! { MAIN_PKG.to_string() => vec![ast.module.clone()] },
        },
        &opts,
    );

    selector.walk_module(&ast.module);

    Ok(ListVariablesResult {
        select_result: selector.select_result,
        unsupported: selector.unsupported,
    })
}

/// Parse symbol selector string to symbol selector spec
///
/// # Examples
///
/// ```
/// use kclvm_query::selector::parse_symbol_selector_spec;
///
/// if let Ok(spec) = parse_symbol_selector_spec("", "alice.age") {
///     assert_eq!(spec.pkgpath, "".to_string());
///     assert_eq!(spec.field_path, "alice.age".to_string());
/// }
/// ```
pub fn parse_symbol_selector_spec(
    pkg_root: &str,
    symbol_path: &str,
) -> Result<ast::SymbolSelectorSpec> {
    if let Ok((pkgpath, field_path)) = split_field_path(symbol_path) {
        Ok(ast::SymbolSelectorSpec {
            pkg_root: pkg_root.to_string(),
            pkgpath,
            field_path,
        })
    } else {
        Err(invalid_symbol_selector_spec_error(symbol_path))
    }
}

#[test]
fn test_symbol_path_selector() {
    let spec = parse_symbol_selector_spec("", "pkg_name:alice.age").unwrap();
    assert_eq!(spec.pkgpath, "pkg_name".to_string());
    assert_eq!(spec.field_path, "alice.age".to_string());
}
