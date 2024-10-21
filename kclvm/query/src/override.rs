use std::collections::HashSet;

use anyhow::{anyhow, Result};

use compiler_base_macros::bug;
use kclvm_ast::config::try_get_config_expr_mut;
use kclvm_ast::path::{get_key_parts, get_key_path};
use kclvm_ast::walk_list_mut;
use kclvm_ast::walker::MutSelfMutWalker;
use kclvm_ast::MAIN_PKG;
use kclvm_ast::{ast, path::get_target_path};
use kclvm_ast_pretty::print_ast_module;
use kclvm_parser::parse_expr;
use kclvm_sema::pre_process::{fix_config_expr_nest_attr, transform_multi_assign};

use crate::{node::AstNodeMover, path::parse_attribute_path};

use super::util::invalid_spec_error;

/// Import statement column offset always start with 1.
/// todo: The (1-based) column offset needs to be constrained by specifications.
const IMPORT_STMT_COLUMN_OFFSET: u64 = 1;

/// Apply overrides on the AST program with the override specifications.
///
/// Please note that this a low level internal API used by compiler itself,
/// The parameters of the method are all compiler internal concepts such as
/// AST, etc.
///
/// # Examples
///
/// ```no_check
/// use kclvm_parser::load_program;
/// use kclvm_tools::query::r#override::apply_overrides;
///
/// let mut prog = load_program(&["config.k"], None, None).unwrap();
/// let overrides = vec![parse_override_spec("config.id=1").unwrap()];
/// let import_paths = vec!["path.to.pkg".to_string()];
/// let result = apply_overrides(&mut prog, &overrides, &import_paths, true).unwrap();
/// ```
pub fn apply_overrides(
    prog: &mut ast::Program,
    overrides: &[String],
    import_paths: &[String],
    print_ast: bool,
) -> Result<()> {
    for o in overrides {
        if let Some(modules) = prog.pkgs.get_mut(MAIN_PKG) {
            for m in modules.iter_mut() {
                if apply_override_on_module(m, o, import_paths)? && print_ast {
                    let code_str = print_ast_module(m);
                    std::fs::write(&m.filename, &code_str)?
                }
            }
        }
    }
    Ok(())
}

/// Build a expression from string.
pub fn build_expr_from_string(value: &str) -> Option<ast::NodeRef<ast::Expr>> {
    let expr: Option<ast::NodeRef<ast::Expr>> = parse_expr(value);
    match &expr {
        Some(e) => match &e.node {
            // fix attr=value to attr="value"
            ast::Expr::Unary(_) | ast::Expr::Binary(_) => {
                Some(ast::NodeRef::new(ast::Node::node_with_pos(
                    ast::Expr::StringLit(ast::StringLit {
                        is_long_string: false,
                        raw_value: format!("{value:?}"),
                        value: value.to_string(),
                    }),
                    e.pos(),
                )))
            }
            _ => expr,
        },
        None => None,
    }
}

/// Apply overrides on the AST module with the override specifications.
///
/// Please note that this a low level internal API used by compiler itself,
/// The parameters of the method are all compiler internal concepts such as
/// AST, etc.
///
/// # Examples
///
/// ```no_check
/// use kclvm_parser::parse_file_force_errors;
/// use kclvm_tools::query::apply_override_on_module;
///
/// let mut module = parse_file_force_errors("", None).unwrap();
/// let override_spec = parse_override_spec("config.id=1").unwrap();
/// let import_paths = vec!["path.to.pkg".to_string()];
/// let result = apply_override_on_module(&mut module, override_spec, &import_paths).unwrap();
/// ```
pub fn apply_override_on_module(
    m: &mut ast::Module,
    o: &str,
    import_paths: &[String],
) -> Result<bool> {
    // Apply import paths on AST module.
    apply_import_paths_on_module(m, import_paths)?;
    let o = parse_override_spec(o)?;
    let ss = parse_attribute_path(&o.field_path)?;
    let default = String::default();
    let target_id = ss.get(0).unwrap_or(&default);
    let value = &o.field_value;
    let key = ast::Identifier {
        names: ss[1..]
            .iter()
            .map(|s| ast::Node::dummy_node(s.to_string()))
            .collect(),
        ctx: ast::ExprContext::Store,
        pkgpath: "".to_string(),
    };
    // Transform config expr to simplify the config path query and override.
    fix_config_expr_nest_attr(m);
    // When there is a multi-target assignment statement of the form `a = b = Config {}`,
    // it needs to be transformed into the following form first to prevent the configuration
    // from being incorrectly modified.
    // ```kcl
    // a = Config {}
    // b = Config {}
    // ```
    transform_multi_assign(m);
    let mut transformer = OverrideTransformer {
        target_id: target_id.to_string(),
        field_paths: ss[1..].to_vec(),
        override_key: key,
        override_value: build_expr_from_string(value),
        override_target_count: 0,
        has_override: false,
        action: o.action,
        operation: o.operation,
    };
    transformer.walk_module(m);
    Ok(transformer.has_override)
}

/// Parse override spec string to override structure.
///
/// parse_override_spec("alice.age=10") -> ast::OverrideSpec {
///     pkgpath: "".to_string(),
///     field_path: "alice.age".to_string(),
///     field_value: "10".to_string(),
///     action: ast::OverrideAction::CreateOrUpdate,
/// }
pub fn parse_override_spec(spec: &str) -> Result<ast::OverrideSpec> {
    if let Some((path, value, operation)) = split_override_spec_op(spec) {
        // Create or update the override value.
        let field_path = path.trim().to_string();
        let field_value = value.trim().to_string();
        if field_path.is_empty() || field_value.is_empty() {
            Err(invalid_spec_error(spec))
        } else {
            Ok(ast::OverrideSpec {
                field_path,
                field_value,
                action: ast::OverrideAction::CreateOrUpdate,
                operation,
            })
        }
    } else if let Some(stripped_spec) = spec.strip_suffix('-') {
        // Delete the override value.
        let field_path = stripped_spec.trim().to_string();
        if field_path.is_empty() {
            Err(invalid_spec_error(spec))
        } else {
            Ok(ast::OverrideSpec {
                field_path: stripped_spec.trim().to_string(),
                field_value: "".to_string(),
                action: ast::OverrideAction::Delete,
                operation: ast::ConfigEntryOperation::Override,
            })
        }
    } else {
        Err(invalid_spec_error(spec))
    }
}

/// split_override_spec_op split the override_spec and do not split the override_op in list
/// expr, dict expr and string e.g., "a.b=1" -> (a.b, 1, =), "a["a=1"]=1" -> (a["a=1"], =, 1)
pub fn split_override_spec_op(spec: &str) -> Option<(String, String, ast::ConfigEntryOperation)> {
    let mut i = 0;
    let mut stack = String::new();
    while i < spec.chars().count() {
        let (c_idx, c) = spec.char_indices().nth(i).unwrap();
        if c == '=' && stack.is_empty() {
            return Some((
                spec[..c_idx].to_string(),
                spec[c_idx + 1..].to_string(),
                ast::ConfigEntryOperation::Override,
            ));
        } else if c == ':' && stack.is_empty() {
            return Some((
                spec[..c_idx].to_string(),
                spec[c_idx + 1..].to_string(),
                ast::ConfigEntryOperation::Union,
            ));
        } else if c == '+' && stack.is_empty() {
            if let Some((c_next_idx, c_next)) = spec.char_indices().nth(i + 1) {
                if c_next == '=' {
                    return Some((
                        spec[..c_idx].to_string(),
                        spec[c_next_idx + 1..].to_string(),
                        ast::ConfigEntryOperation::Insert,
                    ));
                }
            }
        }
        // List/Dict type
        else if c == '[' || c == '{' {
            stack.push(c);
        }
        // List/Dict type
        else if c == ']' || c == '}' {
            stack.pop();
        }
        // String literal type
        else if c == '\"' {
            let t: String = spec.chars().skip(i).collect();
            let re = fancy_regex::Regex::new(r#""(?!"").*?(?<!\\)(\\\\)*?""#).unwrap();
            if let Ok(Some(v)) = re.find(&t) {
                i += v.as_str().chars().count() - 1;
            }
        }
        // String literal type
        else if c == '\'' {
            let t: String = spec.chars().skip(i).collect();
            let re = fancy_regex::Regex::new(r#"'(?!'').*?(?<!\\)(\\\\)*?'"#).unwrap();
            if let Ok(Some(v)) = re.find(&t) {
                i += v.as_str().chars().count() - 1;
            }
        }
        i += 1;
    }
    None
}

// Transform the AST module with the import path list.
fn apply_import_paths_on_module(m: &mut ast::Module, import_paths: &[String]) -> Result<()> {
    if import_paths.is_empty() {
        return Ok(());
    }
    let mut exist_import_set: HashSet<String> = HashSet::new();
    for stmt in &m.body {
        if let ast::Stmt::Import(import_stmt) = &stmt.node {
            if let Some(asname) = &import_stmt.asname {
                exist_import_set.insert(format!("{} as {}", import_stmt.path.node, asname.node));
            } else {
                exist_import_set.insert(import_stmt.rawpath.to_string());
            }
        }
    }

    let mut new_imports_count = 0;

    for (i, path) in import_paths.iter().enumerate() {
        let line: u64 = i as u64 + 1;
        if exist_import_set.contains(path) {
            continue;
        }
        let name = path
            .split('.')
            .last()
            .ok_or_else(|| anyhow!("Invalid import path {}", path))?;
        let import_node = ast::ImportStmt {
            path: ast::Node::dummy_node(path.to_string()),
            rawpath: path.to_string(),
            name: name.to_string(),
            asname: None,
            pkg_name: String::new(),
        };
        let import_stmt = Box::new(ast::Node::new(
            ast::Stmt::Import(import_node),
            m.filename.clone(),
            line,
            IMPORT_STMT_COLUMN_OFFSET,
            line,
            // i denotes the space len between the `import` keyword and the path.
            ("import".len() + path.len() + 1) as u64,
        ));
        new_imports_count += 1;
        m.body.insert((line - 1) as usize, import_stmt)
    }

    // Walk the AST module to update the line number of the all the nodes except the import statement.
    let mut nlw = AstNodeMover {
        line_offset: new_imports_count,
    };

    nlw.walk_module(m);

    Ok(())
}

macro_rules! override_top_level_stmt {
    ($self:expr, $stmt: expr) => {
        let item = $stmt.value.clone();
        let mut value = $self.clone_override_value();
        // Use position information that needs to override the expression.
        value.set_pos(item.pos());
        match &$self.operation {
            ast::ConfigEntryOperation::Union => {
                if let ast::Expr::Config(merged_config_expr) = &value.node {
                    match &mut $stmt.value.node {
                        ast::Expr::Schema(schema_expr) => {
                            if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
                                $self.has_override = merge_config_expr(
                                    config_expr,
                                    merged_config_expr,
                                    &$self.action,
                                );
                            }
                        }
                        ast::Expr::Config(config_expr) => {
                            $self.has_override =
                                merge_config_expr(config_expr, merged_config_expr, &$self.action);
                        }
                        _ => {}
                    }
                } else if let ast::Expr::Schema(merged_schema_expr) = &value.node {
                    if let ast::Expr::Schema(schema_expr) = &mut $stmt.value.node {
                        if schema_expr.name.node.get_name()
                            == merged_schema_expr.name.node.get_name()
                        {
                            if let (
                                ast::Expr::Config(merged_config_expr),
                                ast::Expr::Config(config_expr),
                            ) = (
                                &merged_schema_expr.config.node,
                                &mut schema_expr.config.node,
                            ) {
                                $self.has_override = merge_config_expr(
                                    config_expr,
                                    merged_config_expr,
                                    &$self.action,
                                );
                            }
                        }
                    }
                } else {
                    // Override the node value.
                    $stmt.value = value;
                    $self.has_override = true;
                }
            }
            ast::ConfigEntryOperation::Insert => {
                if let ast::Expr::List(insert_list_expr) = &value.node {
                    if let ast::Expr::List(list_expr) = &mut $stmt.value.node {
                        for value in &insert_list_expr.elts {
                            list_expr.elts.push(value.clone());
                        }
                        $self.has_override = true;
                    }
                }
            }
            ast::ConfigEntryOperation::Override => {
                // Override the node value.
                $stmt.value = value;
                $self.has_override = true;
            }
        }
    };
}

/// OverrideTransformer is used to walk AST and transform it with the override values.
struct OverrideTransformer {
    pub target_id: String,
    pub field_paths: Vec<String>,
    pub override_key: ast::Identifier,
    pub override_value: Option<ast::NodeRef<ast::Expr>>,
    pub override_target_count: usize,
    pub has_override: bool,
    pub action: ast::OverrideAction,
    pub operation: ast::ConfigEntryOperation,
}

impl<'ctx> MutSelfMutWalker<'ctx> for OverrideTransformer {
    // When override the global variable, it should be updated in the module level.
    // Because the delete action may delete the global variable.
    // TODO: combine the code of walk_module, walk_assign_stmt and walk_unification_stmt
    fn walk_module(&mut self, module: &'ctx mut ast::Module) {
        if self.has_override {
            return;
        }
        match self.action {
            // Walk the module body to find the target and override it.
            ast::OverrideAction::CreateOrUpdate => {
                module.body.iter_mut().for_each(|stmt| {
                    if let ast::Stmt::Assign(assign_stmt) = &mut stmt.node {
                        if assign_stmt.targets.len() == 1 && self.field_paths.len() == 0 {
                            let target = assign_stmt.targets.get(0).unwrap().node.clone();
                            let target = get_target_path(&target);
                            if target == self.target_id {
                                override_top_level_stmt!(self, assign_stmt);
                            }
                        }
                    } else if let ast::Stmt::AugAssign(aug_assign_stmt) = &mut stmt.node {
                        if self.field_paths.len() == 0 {
                            let target = aug_assign_stmt.target.node.clone();
                            let target = get_target_path(&target);
                            if target == self.target_id {
                                override_top_level_stmt!(self, aug_assign_stmt);
                            }
                        }
                    } else if let ast::Stmt::Unification(unification_stmt) = &mut stmt.node {
                        if self.field_paths.len() == 0 {
                            let target = match unification_stmt.target.node.names.get(0) {
                                Some(name) => name,
                                None => bug!(
                                    "Invalid AST unification target names {:?}",
                                    unification_stmt.target.node.names
                                ),
                            };
                            if target.node == self.target_id {
                                let item = unification_stmt.value.clone();
                                let mut value = self.clone_override_value();
                                // Use position information that needs to override the expression.
                                value.set_pos(item.pos());
                                let schema_expr = &mut unification_stmt.value.node;
                                match &self.operation {
                                    ast::ConfigEntryOperation::Union => {
                                        if let ast::Expr::Config(merged_config_expr) = &value.node {
                                            if let ast::Expr::Config(config_expr) =
                                                &mut schema_expr.config.node
                                            {
                                                self.has_override = merge_config_expr(
                                                    config_expr,
                                                    merged_config_expr,
                                                    &self.action,
                                                );
                                            }
                                        } else if let ast::Expr::Schema(merged_schema_expr) =
                                            &value.node
                                        {
                                            if schema_expr.name.node.get_name()
                                                == merged_schema_expr.name.node.get_name()
                                            {
                                                if let (
                                                    ast::Expr::Config(merged_config_expr),
                                                    ast::Expr::Config(config_expr),
                                                ) = (
                                                    &merged_schema_expr.config.node,
                                                    &mut schema_expr.config.node,
                                                ) {
                                                    self.has_override = merge_config_expr(
                                                        config_expr,
                                                        merged_config_expr,
                                                        &self.action,
                                                    );
                                                }
                                            }
                                        } else {
                                            // Unification is only support to override the schema expression.
                                            if let ast::Expr::Schema(schema_expr) = value.node {
                                                if self.field_paths.len() == 0 {
                                                    unification_stmt.value = Box::new(
                                                        ast::Node::dummy_node(schema_expr),
                                                    );
                                                    self.has_override = true;
                                                }
                                            }
                                        }
                                    }
                                    ast::ConfigEntryOperation::Insert
                                    | ast::ConfigEntryOperation::Override => {
                                        // Unification is only support to override the schema expression.
                                        if let ast::Expr::Schema(schema_expr) = value.node {
                                            if self.field_paths.len() == 0 {
                                                unification_stmt.value =
                                                    Box::new(ast::Node::dummy_node(schema_expr));
                                                self.has_override = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
            ast::OverrideAction::Delete => {
                // Delete the override target when the action is DELETE.
                module.body.retain(|stmt| {
                    if let ast::Stmt::Assign(assign_stmt) = &stmt.node {
                        if assign_stmt.targets.len() == 1 && self.field_paths.len() == 0 {
                            let target = get_target_path(&assign_stmt.targets.get(0).unwrap().node);
                            if target == self.target_id {
                                self.has_override = true;
                                return false;
                            }
                        }
                    }
                    if let ast::Stmt::Unification(unification_stmt) = &stmt.node {
                        let target = match unification_stmt.target.node.names.get(0) {
                            Some(name) => name,
                            None => bug!(
                                "Invalid AST unification target names {:?}",
                                unification_stmt.target.node.names
                            ),
                        };
                        if target.node == self.target_id && self.field_paths.len() == 0 {
                            self.has_override = true;
                            return false;
                        }
                    }
                    true
                });
            }
        }

        walk_list_mut!(self, walk_stmt, module.body);

        // If the variable is not found, add a new variable with the override value.
        if !self.has_override {
            match self.action {
                // Walk the module body to find the target and override it.
                ast::OverrideAction::CreateOrUpdate => {
                    let value = if self.field_paths.len() == 0 {
                        self.clone_override_value()
                    } else {
                        // if the spec is b.c.d=1 and the b is not found, add config b: {c: {d: 1}}
                        Box::new(ast::Node::dummy_node(ast::Expr::Config(ast::ConfigExpr {
                            items: vec![Box::new(ast::Node::dummy_node(ast::ConfigEntry {
                                key: Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
                                    ast::Identifier {
                                        names: self
                                            .field_paths
                                            .iter()
                                            .map(|s| ast::Node::dummy_node(s.to_string()))
                                            .collect(),
                                        ctx: ast::ExprContext::Store,
                                        pkgpath: "".to_string(),
                                    },
                                )))),
                                value: self.clone_override_value(),
                                operation: self.operation.clone(),
                            }))],
                        })))
                    };
                    match &self.operation {
                        ast::ConfigEntryOperation::Override => {
                            let assign = ast::AssignStmt {
                                targets: vec![Box::new(ast::Node::dummy_node(ast::Target {
                                    name: ast::Node::dummy_node(self.target_id.clone()),
                                    paths: vec![],
                                    pkgpath: "".to_string(),
                                }))],
                                ty: None,
                                value,
                            };
                            module
                                .body
                                .push(Box::new(ast::Node::dummy_node(ast::Stmt::Assign(assign))));
                        }
                        ast::ConfigEntryOperation::Union => {
                            let schema_expr: Result<ast::Node<ast::SchemaExpr>, _> =
                                value.as_ref().clone().try_into();
                            match schema_expr {
                                Ok(schema_expr) => {
                                    let stmt = ast::UnificationStmt {
                                        target: Box::new(ast::Node::dummy_node(ast::Identifier {
                                            names: vec![ast::Node::dummy_node(
                                                self.target_id.clone(),
                                            )],
                                            ctx: ast::ExprContext::Store,
                                            pkgpath: "".to_string(),
                                        })),
                                        value: Box::new(schema_expr),
                                    };
                                    module.body.push(Box::new(ast::Node::dummy_node(
                                        ast::Stmt::Unification(stmt),
                                    )));
                                }
                                Err(_) => {
                                    let stmt = ast::AssignStmt {
                                        targets: vec![Box::new(ast::Node::dummy_node(
                                            ast::Target {
                                                name: ast::Node::dummy_node(self.target_id.clone()),
                                                paths: vec![],
                                                pkgpath: "".to_string(),
                                            },
                                        ))],
                                        ty: None,
                                        value,
                                    };
                                    module.body.push(Box::new(ast::Node::dummy_node(
                                        ast::Stmt::Assign(stmt),
                                    )));
                                }
                            }
                        }
                        ast::ConfigEntryOperation::Insert => {
                            let stmt = ast::AugAssignStmt {
                                target: Box::new(ast::Node::dummy_node(ast::Target {
                                    name: ast::Node::dummy_node(self.target_id.clone()),
                                    paths: vec![],
                                    pkgpath: "".to_string(),
                                })),
                                op: ast::AugOp::Add,
                                value,
                            };
                            module
                                .body
                                .push(Box::new(ast::Node::dummy_node(ast::Stmt::AugAssign(stmt))));
                        }
                    }

                    self.has_override = true;
                }
                ast::OverrideAction::Delete => {
                    return;
                }
            }
        }
    }

    fn walk_unification_stmt(&mut self, unification_stmt: &'ctx mut ast::UnificationStmt) {
        if self.has_override {
            return;
        }
        let name = match unification_stmt.target.node.names.get(0) {
            Some(name) => name,
            None => bug!(
                "Invalid AST unification target names {:?}",
                unification_stmt.target.node.names
            ),
        };
        if name.node != self.target_id || self.field_paths.len() == 0 {
            return;
        }
        self.override_target_count = 1;
        self.walk_schema_expr(&mut unification_stmt.value.node);
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if self.has_override {
            return;
        }
        if let ast::Expr::Schema(_) | ast::Expr::Config(_) = &assign_stmt.value.node {
            self.override_target_count = 0;
            for target in &assign_stmt.targets {
                if !target.node.paths.is_empty() {
                    continue;
                }
                if target.node.name.node != self.target_id {
                    continue;
                }
                self.override_target_count += 1;
            }
            if self.override_target_count == 0 {
                return;
            }
            self.walk_expr(&mut assign_stmt.value.node);
        }
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx mut ast::SchemaExpr) {
        if self.has_override {
            return;
        }
        if self.override_target_count == 0 {
            return;
        }
        if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
            if !self.lookup_config_and_replace(config_expr) {
                // Not exist and append an override value when the action is CREATE_OR_UPDATE
                if let ast::OverrideAction::CreateOrUpdate = self.action {
                    if let ast::Expr::Config(config_expr) = &mut schema_expr.config.node {
                        config_expr
                            .items
                            .push(Box::new(ast::Node::dummy_node(ast::ConfigEntry {
                                key: Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(
                                    self.override_key.clone(),
                                )))),
                                value: self.clone_override_value(),
                                operation: self.operation.clone(),
                            })));
                        self.has_override = true;
                    }
                }
            } else {
                self.has_override = true;
            }
        }
        self.override_target_count = 0;
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx mut ast::ConfigExpr) {
        if self.has_override {
            return;
        }
        // Lookup config all fields and replace if it is matched with the override spec.
        if !self.lookup_config_and_replace(config_expr) {
            return;
        }
        self.has_override = true;
        self.override_target_count = 0;
    }

    fn walk_if_stmt(&mut self, _: &'ctx mut ast::IfStmt) {
        // Do not override AssignStmt in IfStmt
    }
    fn walk_schema_stmt(&mut self, _: &'ctx mut ast::SchemaStmt) {
        // Do not override AssignStmt in SchemaStmt
    }
    fn walk_lambda_expr(&mut self, _: &'ctx mut ast::LambdaExpr) {
        // Do not override AssignStmt in LambdaExpr
    }
}

impl OverrideTransformer {
    /// Lookup schema config all fields and replace if it is matched with the override spec,
    /// return whether is found a replaced one.
    fn lookup_config_and_replace(&self, config_expr: &mut ast::ConfigExpr) -> bool {
        // Split a path into multiple parts. `a.b.c` -> ["a", "b", "c"]
        let parts = self
            .field_paths
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        replace_config_with_path_parts(
            config_expr,
            &parts,
            &self.action,
            &self.operation,
            &self.override_value,
        )
    }

    /// Clone a override value
    #[inline]
    fn clone_override_value(&self) -> ast::NodeRef<ast::Expr> {
        match &self.override_value {
            Some(v) => v.clone(),
            None => bug!("Override value is None"),
        }
    }
}

fn merge_config_expr(
    config_expr: &mut ast::ConfigExpr,
    merged_config_expr: &ast::ConfigExpr,
    action: &ast::OverrideAction,
) -> bool {
    let mut changed = false;
    for item in &merged_config_expr.items {
        let parts = get_key_parts(&item.node.key);
        // Deal double star and config if expr
        if parts.is_empty() {
            config_expr.items.push(item.clone());
            changed = true;
        } else {
            if replace_config_with_path_parts(
                config_expr,
                &parts,
                action,
                &item.node.operation,
                &Some(item.node.value.clone()),
            ) {
                changed = true;
            }
        }
    }
    changed
}

/// Replace AST config expr with one part of path. The implementation of this function
/// uses recursive matching to find the config entry need to be modified.
fn replace_config_with_path_parts(
    config_expr: &mut ast::ConfigExpr,
    parts: &[&str],
    action: &ast::OverrideAction,
    operation: &ast::ConfigEntryOperation,
    value: &Option<ast::NodeRef<ast::Expr>>,
) -> bool {
    // Do not replace empty path parts and out of index parts on the config expression.
    if parts.is_empty() {
        return false;
    }
    // Always take the first part to match, because recursive search is required.
    let part = parts[0];
    let mut delete_index_set = HashSet::new();
    let mut changed = false;
    // Loop all entries in the config expression and replace, because there may be duplicate
    // configuration items in config.
    for (i, item) in config_expr.items.iter_mut().enumerate() {
        // Compare each field of the config structure one by one.
        // - `part` denotes the path entered by the user to be modified.
        // - `get_path_key` returns the real config key name.
        // For example, the real config node is `a: {b: c: {}}`. The path
        // that needs to be modified is `a.b.c`, and its parts are ["a", "b", "c"].
        if part == get_key_path(&item.node.key) {
            // When the last part of the path is successfully recursively matched,
            // it indicates that the original value that needs to be overwritten
            // is successfully found, and the new value is used to overwrite it.
            // - `parts.len() == 1` denotes the path matches exactly.
            if parts.len() == 1 {
                match action {
                    ast::OverrideAction::CreateOrUpdate => {
                        if let Some(value) = value {
                            let mut value = value.clone();
                            // Use position information that needs to override the expression.
                            value.set_pos(item.pos());
                            match operation {
                                ast::ConfigEntryOperation::Union => {
                                    if let ast::Expr::Config(merged_config_expr) = &value.node {
                                        match &mut item.node.value.node {
                                            ast::Expr::Schema(schema_expr) => {
                                                if let ast::Expr::Config(config_expr) =
                                                    &mut schema_expr.config.node
                                                {
                                                    changed = merge_config_expr(
                                                        config_expr,
                                                        merged_config_expr,
                                                        action,
                                                    );
                                                }
                                            }
                                            ast::Expr::Config(config_expr) => {
                                                changed = merge_config_expr(
                                                    config_expr,
                                                    merged_config_expr,
                                                    action,
                                                );
                                            }
                                            _ => {}
                                        }
                                    } else if let ast::Expr::Schema(merged_schema_expr) =
                                        &value.node
                                    {
                                        if let ast::Expr::Schema(schema_expr) =
                                            &mut item.node.value.node
                                        {
                                            if schema_expr.name.node.get_name()
                                                == merged_schema_expr.name.node.get_name()
                                            {
                                                if let (
                                                    ast::Expr::Config(merged_config_expr),
                                                    ast::Expr::Config(config_expr),
                                                ) = (
                                                    &merged_schema_expr.config.node,
                                                    &mut schema_expr.config.node,
                                                ) {
                                                    changed = merge_config_expr(
                                                        config_expr,
                                                        merged_config_expr,
                                                        action,
                                                    );
                                                }
                                            }
                                        }
                                    } else {
                                        // Override the node value.
                                        item.node.value = value;
                                        changed = true;
                                    }
                                }
                                ast::ConfigEntryOperation::Insert => {
                                    if let ast::Expr::List(insert_list_expr) = &value.node {
                                        if let ast::Expr::List(list_expr) =
                                            &mut item.node.value.node
                                        {
                                            for value in &insert_list_expr.elts {
                                                list_expr.elts.push(value.clone());
                                            }
                                            changed = true;
                                        }
                                    }
                                }
                                ast::ConfigEntryOperation::Override => {
                                    // Override the node value.
                                    item.node.value = value;
                                    changed = true;
                                }
                            }
                        }
                    }
                    ast::OverrideAction::Delete => {
                        // Store the config entry delete index into the delete index set.
                        // Because we can't delete the entry directly in the loop
                        delete_index_set.insert(i);
                        changed = true;
                    }
                }
            }
            // Replace value recursively using the path composed by subsequent parts.
            //
            // The reason for using recursion instead of looping for path matching
            // is that rust cannot directly hold shared references to AST nodes
            // (ast::NodeRef<T> is a Box<T>), so recursive search is performed
            // directly on AST nodes.
            else if let Some(config_expr) = try_get_config_expr_mut(&mut item.node.value.node) {
                changed = replace_config_with_path_parts(
                    config_expr,
                    &parts[1..],
                    action,
                    operation,
                    value,
                );
            }
        }
    }
    // Delete entries according delete index set.
    if !delete_index_set.is_empty() {
        let items: Vec<(usize, &ast::NodeRef<ast::ConfigEntry>)> = config_expr
            .items
            .iter()
            .enumerate()
            .filter(|(i, _)| !delete_index_set.contains(i))
            .collect();
        config_expr.items = items
            .iter()
            .map(|(_, item)| <&ast::NodeRef<ast::ConfigEntry>>::clone(item).clone())
            .collect();
    } else if let ast::OverrideAction::CreateOrUpdate = action {
        if !changed {
            if let Some(value) = value {
                let key = ast::Identifier {
                    names: parts
                        .iter()
                        .map(|s| ast::Node::dummy_node(s.to_string()))
                        .collect(),
                    ctx: ast::ExprContext::Store,
                    pkgpath: "".to_string(),
                };
                config_expr
                    .items
                    .push(Box::new(ast::Node::dummy_node(ast::ConfigEntry {
                        key: Some(Box::new(ast::Node::dummy_node(ast::Expr::Identifier(key)))),
                        value: value.clone(),
                        operation: operation.clone(),
                    })));
                changed = true;
            }
        }
    }
    return changed;
}
