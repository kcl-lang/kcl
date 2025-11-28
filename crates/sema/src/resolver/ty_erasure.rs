use kcl_ast::walker::MutSelfMutWalker;
use kcl_ast::{ast, walk_if_mut, walk_list_mut};

#[derive(Default)]
struct TypeErasureTransformer;
const FUNCTION: &str = "function";

impl<'ctx> MutSelfMutWalker<'ctx> for TypeErasureTransformer {
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx mut ast::SchemaStmt) {
        if let Some(schema_index_signature) = schema_stmt.index_signature.as_deref_mut()
            && let kcl_ast::ast::Type::Function(_) = &mut schema_index_signature.node.value_ty.node
        {
            schema_index_signature.node.value_ty.node = FUNCTION.to_string().into();
        }
        walk_if_mut!(self, walk_arguments, schema_stmt.args);
        walk_list_mut!(self, walk_call_expr, schema_stmt.decorators);
        walk_list_mut!(self, walk_check_expr, schema_stmt.checks);
        walk_list_mut!(self, walk_stmt, schema_stmt.body);
    }
    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        walk_list_mut!(self, walk_call_expr, schema_attr.decorators);
        walk_if_mut!(self, walk_expr, schema_attr.value);
        if let kcl_ast::ast::Type::Function(_) = schema_attr.ty.as_ref().node {
            schema_attr.ty.node = FUNCTION.to_string().into();
        }
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if let Some(ty) = &mut assign_stmt.ty
            && let kcl_ast::ast::Type::Function(_) = ty.as_ref().node
            && let Some(ty_anno) = &mut assign_stmt.ty
        {
            ty_anno.node = FUNCTION.to_string().into();
        }
        self.walk_expr(&mut assign_stmt.value.node);
    }
    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx mut ast::TypeAliasStmt) {
        if let kcl_ast::ast::Type::Function(_) = type_alias_stmt.ty.as_ref().node {
            type_alias_stmt.type_value.node = FUNCTION.to_string();
        }
    }
    fn walk_arguments(&mut self, arguments: &'ctx mut ast::Arguments) {
        for ty in (&mut arguments.ty_list.iter_mut()).flatten() {
            if let kcl_ast::ast::Type::Function(_) = ty.as_ref().node {
                ty.node = FUNCTION.to_string().into();
            }
        }
        for default in arguments.defaults.iter_mut() {
            if let Some(d) = default.as_deref_mut() {
                self.walk_expr(&mut d.node)
            }
        }
    }
    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx mut ast::LambdaExpr) {
        walk_if_mut!(self, walk_arguments, lambda_expr.args);
        walk_list_mut!(self, walk_stmt, lambda_expr.body);
        if let Some(ty) = lambda_expr.return_ty.as_mut()
            && let kcl_ast::ast::Type::Function(_) = ty.as_ref().node
        {
            ty.node = FUNCTION.to_string().into();
        }
    }
}

/// Run a pass on AST and change the function type to the `Named("function")` type
pub fn type_func_erasure_pass(program: &mut ast::Program) {
    for (_, modules) in program.pkgs.iter() {
        for module in modules.iter() {
            let mut module = program
                .get_module_mut(module)
                .expect("Failed to acquire module lock")
                .unwrap_or_else(|| panic!("module {:?} not found in program", module));
            TypeErasureTransformer.walk_module(&mut module);
        }
    }
}
