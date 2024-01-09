mod config;
mod identifier;
mod multi_assign;

use indexmap::IndexMap;
use kclvm_ast::ast;

#[cfg(test)]
mod tests;

pub use config::{fix_config_expr_nest_attr, merge_program};
pub use identifier::{fix_qualified_identifier, fix_raw_identifier_prefix};
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
            // First we should transform the raw identifier to avoid raw identifier that happens to be a package path.
            fix_raw_identifier_prefix(module);
            fix_qualified_identifier(module, &mut import_names);
            fix_config_expr_nest_attr(module);
        }
    }
    if opts.merge_program {
        merge_program(program);
    }
}
