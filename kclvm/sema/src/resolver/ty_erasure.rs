use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfMutWalker;

#[derive(Default)]
struct TypeErasureTransformer;
const FUNCTION: &str = "function";

impl<'ctx> MutSelfMutWalker<'ctx> for TypeErasureTransformer {
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx mut ast::SchemaStmt) {
        if let Some(schema_index_signature) = schema_stmt.index_signature.as_deref_mut() {
            if let kclvm_ast::ast::Type::Function(_) =
                &mut schema_index_signature.node.value_ty.node
            {
                schema_index_signature.node.value_ty.node = FUNCTION.to_string().into();
            }
        }
        for item in schema_stmt.body.iter_mut() {
            if let kclvm_ast::ast::Stmt::SchemaAttr(attr) = &mut item.node {
                self.walk_schema_attr(attr);
            }
        }
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        if let kclvm_ast::ast::Type::Function(_) = schema_attr.ty.as_ref().node {
            schema_attr.ty.node = FUNCTION.to_string().into();
        }
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if let Some(ty) = &mut assign_stmt.ty {
            if let kclvm_ast::ast::Type::Function(_) = ty.as_ref().node {
                if let Some(ty_anno) = &mut assign_stmt.ty {
                    ty_anno.node = FUNCTION.to_string().into();
                }
            }
        }
    }
    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx mut ast::TypeAliasStmt) {
        if let kclvm_ast::ast::Type::Function(_) = type_alias_stmt.ty.as_ref().node {
            type_alias_stmt.type_value.node = FUNCTION.to_string();
        }
    }
    fn walk_arguments(&mut self, arguments: &'ctx mut ast::Arguments) {
        for ty in (&mut arguments.ty_list.iter_mut()).flatten() {
            if let kclvm_ast::ast::Type::Function(_) = ty.as_ref().node {
                ty.node = FUNCTION.to_string().into();
            }
        }
    }
}

/// Run a pass on AST and change the function type to the `Named("function")` type
pub fn type_func_erasure_pass<'ctx>(program: &'ctx mut ast::Program) {
    for (_, modules) in program.pkgs.iter_mut() {
        for module in modules.iter_mut() {
            TypeErasureTransformer::default().walk_module(module);
        }
    }
}
