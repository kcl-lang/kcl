mod config;
mod identifier;
mod lit_ty_default_value;
mod multi_assign;

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
    for (pkgpath, modules) in program.pkgs.iter() {
        let mut import_names = IndexMap::default();
        if pkgpath == kclvm_ast::MAIN_PKG {
            for module in modules.iter() {
                let module = program
                    .get_module(module)
                    .expect("Failed to acquire module lock")
                    .expect(&format!("module {:?} not found in program", module));
                for stmt in &module.body {
                    if let ast::Stmt::Import(import_stmt) = &stmt.node {
                        import_names
                            .insert(import_stmt.name.clone(), import_stmt.path.node.clone());
                    }
                }
            }
        }
        for module in modules.iter() {
            let mut module = program
                .get_module_mut(module)
                .expect("Failed to acquire module lock")
                .expect(&format!("module {:?} not found in program", module));
            if pkgpath != kclvm_ast::MAIN_PKG {
                import_names.clear();
            }
            // First we should transform the raw identifier to avoid raw identifier that happens to be a package path.
            fix_raw_identifier_prefix(&mut module);
            fix_qualified_identifier(&mut module, &mut import_names);
            fix_config_expr_nest_attr(&mut module);
            fix_lit_ty_default_value(&mut module);
        }
    }
    if opts.merge_program {
        merge_program(program);
    }
}
