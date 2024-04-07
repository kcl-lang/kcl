use anyhow::Result;
use kclvm_ast::{ast, walker::MutSelfWalker};
use kclvm_sema::builtin::BUILTIN_FUNCTIONS;
use kclvm_sema::{builtin::option::OptionHelp, resolver::scope::NodeKey};

use crate::util::{get_call_args_string, get_call_args_strip_string};
use crate::{load_packages, util::get_call_args_bool, LoadPackageOptions, Packages};

#[derive(Debug)]
struct OptionHelpExtractor<'ctx> {
    pkgpath: String,
    options: Vec<OptionHelp>,
    packages: &'ctx Packages,
}

impl<'ctx> MutSelfWalker for OptionHelpExtractor<'ctx> {
    fn walk_call_expr(&mut self, call_expr: &ast::CallExpr) {
        if let ast::Expr::Identifier(identifier) = &call_expr.func.node {
            if identifier.names.len() == 1 {
                let node_key = NodeKey {
                    pkgpath: self.pkgpath.clone(),
                    id: identifier.names[0].id.clone(),
                };
                let symbol_ref = self.packages.node_symbol_map.get(&node_key).unwrap();
                let symbol = self.packages.symbols.get(symbol_ref).unwrap();
                let binding = BUILTIN_FUNCTIONS;
                let builtin_option_type = binding.get("option").unwrap();
                if !symbol.is_global
                    && symbol.ty.is_func()
                    && symbol.ty.ty_str() == builtin_option_type.ty_str()
                {
                    self.options.push(OptionHelp {
                        name: get_call_args_strip_string(call_expr, 0, Some("key")),
                        ty: get_call_args_strip_string(call_expr, 1, Some("type")),
                        required: get_call_args_bool(call_expr, 2, Some("required")),
                        default_value: get_call_args_string(call_expr, 3, Some("default")),
                        help: get_call_args_strip_string(call_expr, 3, Some("help")),
                    })
                }
            }
        }
    }
}

/// list_options provides users with the ability to parse kcl program and get all option
/// calling information.
pub fn list_options(opts: &LoadPackageOptions) -> Result<Vec<OptionHelp>> {
    let packages = load_packages(opts)?;
    let mut extractor = OptionHelpExtractor {
        pkgpath: String::new(),
        options: vec![],
        packages: &packages,
    };

    for (pkgpath, modules) in &packages.program.pkgs {
        extractor.pkgpath = pkgpath.clone();
        for module in modules {
            extractor.walk_module(module)
        }
    }
    Ok(extractor.options)
}
