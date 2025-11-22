use std::collections::HashMap;

use kclvm_ast::{ast, walker::MutSelfMutWalker};

/// Transform AST and split multi target assign statements to multiple assign statements.
///
/// # Examples
///
/// ```
/// use kclvm_parser::parse_file_force_errors;
/// use kclvm_sema::pre_process::transform_multi_assign;
///
/// let mut module = parse_file_force_errors("", Some("a = b = Config {}".to_string())).unwrap();
/// assert_eq!(module.body.len(), 1);
/// transform_multi_assign(&mut module);
/// assert_eq!(module.body.len(), 2);
/// ```
pub fn transform_multi_assign(m: &mut ast::Module) {
    let mut transformer = MultiAssignTransformer::default();
    transformer.walk_module(m);
    let mut insert_count = 0;
    for (index, assign_stmt_list) in transformer.multi_assign_mapping {
        // Get the origin assign statement insert index in AST module body with offset.
        // offset denotes the sum of the number of assigned stmt has been inserted.
        let insert_index = index + insert_count;
        let pos = match m.body.get(insert_index) {
            Some(stmt) => stmt.pos().clone(),
            None => bug!("AST module body index {} out of bound", insert_index),
        };
        for (insert_offset, assign_stmt) in assign_stmt_list.iter().enumerate() {
            // Insert behind the node with the insert offset, so the index plus one.
            m.body.insert(
                insert_index + insert_offset + 1,
                Box::new(ast::Node::node_with_pos(
                    ast::Stmt::Assign(assign_stmt.clone()),
                    pos.clone(),
                )),
            );
            insert_count += 1;
        }
    }
}

/// MultiAssignTransformer is used to transform AST Module and split top level
/// multiple target assign statement to multiple assign statements
///
/// - Before
///
/// ```kcl
/// a = b = Config {}
/// ```
///
/// - After
///
/// ```kcl
/// a = Config {}
/// b = Config {}
/// ```
#[derive(Debug, Default)]
struct MultiAssignTransformer {
    pub multi_assign_mapping: HashMap<usize, Vec<ast::AssignStmt>>,
    pub index: usize,
}

impl<'ctx> MutSelfMutWalker<'ctx> for MultiAssignTransformer {
    fn walk_stmt(&mut self, stmt: &'ctx mut ast::Stmt) {
        if let ast::Stmt::Assign(assign_stmt) = stmt {
            self.walk_assign_stmt(assign_stmt)
        }
        // Statement count.
        self.index += 1;
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if assign_stmt.targets.len() <= 1 {
            return;
        }
        let mut assign_stmt_list = vec![];
        for target in &assign_stmt.targets[1..] {
            let mut new_assign_stmt = assign_stmt.clone();
            new_assign_stmt.targets = vec![target.clone()];
            assign_stmt_list.push(new_assign_stmt);
        }
        self.multi_assign_mapping
            .insert(self.index, assign_stmt_list);
        assign_stmt.targets = vec![assign_stmt.targets[0].clone()];
    }
    fn walk_if_stmt(&mut self, _: &'ctx mut ast::IfStmt) {
        // Do not fix AssignStmt in IfStmt
    }
    fn walk_schema_stmt(&mut self, _: &'ctx mut ast::SchemaStmt) {
        // Do not fix AssignStmt in SchemaStmt
    }
    fn walk_lambda_expr(&mut self, _: &'ctx mut ast::LambdaExpr) {
        // Do not fix AssignStmt in LambdaExpr
    }
}
