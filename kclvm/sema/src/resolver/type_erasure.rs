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
                schema_index_signature.node.value_type.node = FUNCTION.to_string();
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
            schema_attr.type_str.node = FUNCTION.to_string();
        }
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        if let Some(ty) = &mut assign_stmt.ty {
            if let kclvm_ast::ast::Type::Function(_) = ty.as_ref().node {
                if let Some(ty_anno) = &mut assign_stmt.type_annotation {
                    ty_anno.node = FUNCTION.to_string();
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
        for (ty, ty_anno) in arguments
            .ty_list
            .iter_mut()
            .zip(arguments.type_annotation_list.iter_mut())
        {
            if let Some(ty) = ty {
                if let kclvm_ast::ast::Type::Function(_) = ty.as_ref().node {
                    if let Some(ty_anno) = ty_anno {
                        ty_anno.node = FUNCTION.to_string();
                    }
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
