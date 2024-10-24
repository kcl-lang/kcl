mod config;
mod identifier;
mod lit_ty_default_value;
mod multi_assign;

use std::sync::Arc;

use indexmap::IndexMap;
use kclvm_ast::ast;

#[cfg(test)]
mod tests;

pub use config::{fix_config_expr_nest_attr, merge_program};
pub use identifier::{fix_qualified_identifier, fix_raw_identifier_prefix};
pub use lit_ty_default_value::fix_lit_ty_default_value;
pub use multi_assign::transform_multi_assign;

use crate::resolver::Options;

/// Pre-process AST program.
pub fn pre_process_program(program: &mut ast::Program, opts: &Options) {
    for (pkgpath, modules) in program.pkgs.iter_mut() {
        let mut import_names = IndexMap::default();
        if pkgpath == kclvm_ast::MAIN_PKG {
            for module in modules.iter_mut() {
                for stmt in &module.body {
                    if let ast::Stmt::Import(import_stmt) = &stmt.node {
                        import_names
                            .insert(import_stmt.name.clone(), import_stmt.path.node.clone());
                    }
                }
            }
        }
        for module in modules.iter_mut() {
            if pkgpath != kclvm_ast::MAIN_PKG {
                import_names.clear();
            }
            let module = Arc::make_mut(module);
            // First we should transform the raw identifier to avoid raw identifier that happens to be a package path.
            fix_raw_identifier_prefix(module);
            fix_qualified_identifier(module, &mut import_names);
            fix_config_expr_nest_attr(module);
            fix_lit_ty_default_value(module);
        }
    }
    if opts.merge_program {
        merge_program(program);
    }
}
