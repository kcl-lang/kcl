use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfMutWalker;

#[derive(Default)]
struct TypeErasureTransformer {}

impl<'ctx> MutSelfMutWalker<'ctx> for TypeErasureTransformer {
    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        if let kclvm_ast::ast::Type::Function(_) = schema_attr.ty.as_ref().node {
            schema_attr.type_str.node = "function".to_string();
        }
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if let Some(ty) = &mut assign_stmt.ty {
            if let kclvm_ast::ast::Type::Function(_) = ty.as_ref().node {
                if let Some(ty_anno) = &mut assign_stmt.type_annotation {
                    ty_anno.node = "function".to_string();
                }
            }
        }
    }
}

pub fn type_erasure<'ctx>(program: &'ctx mut ast::Program) {
    for (_, modules) in program.pkgs.iter_mut() {
        for module in modules.iter_mut() {
            TypeErasureTransformer::default().walk_module(module);
        }
    }
}
