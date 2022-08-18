use crate::lint::lint::{LintArray, LintContext};
use crate::lint::lintpass::LintPass;
use crate::lint::lints_def::ImportPosition;
use crate::lint::lints_def::ReImport;
use crate::lint::lints_def::UnusedImport;
use crate::lint_methods;
use crate::resolver::scope::Scope;
use kclvm_ast::ast;
use kclvm_error::Handler;

/// Call the `check_*` method of each lintpass in CombinedLintLass.check_*.
/// ```ignore
///     fn check_ident(&mut self, handler: &mut Handler, ctx: &mut LintContext, id: &ast::Identifier, ){
///         self.LintPassA.check_ident(handler, ctx, id);
///         self.LintPassB.check_ident(handler, ctx, id);
///         ...
///     }
/// ```
macro_rules! expand_combined_lint_pass_method {
    ([$($passes:ident),*], $self: ident, $name: ident, $params:tt) => ({
        $($self.$passes.$name $params;)*
    })
}

/// Expand all methods defined in macro `lint_methods` in the `CombinedLintLass`.
///
/// ```ignore
///     fn check_ident(&mut self, handler: &mut Handler, ctx: &mut LintContext, id: &ast::Identifier){};
///     fn check_stmt(&mut self, handler: &mut Handler, ctx: &mut LintContext, module: &ast::Module){};
///     ...
///  ```
macro_rules! expand_combined_lint_pass_methods {
    ($handler:ty, $ctx:ty, $passes:tt, [$($(#[$attr:meta])* fn $name:ident($($param:ident: $arg:ty),*);)*]) => (
        $(fn $name(&mut self, handler: &mut $handler, ctx: &mut $ctx, $($param: $arg),*) {
            expand_combined_lint_pass_method!($passes, self, $name, (handler, ctx, $($param),*));
        })*
    )
}

/// Expand all definitions of `CombinedLintPass`. The results are as follows：
///
/// ```ignore
/// pub struct CombinedLintPass {
///     LintPassA: LintPassA;
///     LintPassB: LintPassB;
///     ...
/// }
///
/// impl CombinedLintPass{
///     pub fn new() -> Self {
///        Self {
///            LintPassA: LintPassA,
///            LintPassB: LintPassB,
///            ...
///        }
///     }
///     pub fn get_lints() -> LintArray {
///         let mut lints = Vec::new();
///         lints.extend_from_slice(&LintPassA::get_lints());
///         lints.extend_from_slice(&LintPassB::get_lints());
///         ...
///         lints
///      }
///  }
///
/// impl LintPass for CombinedLintPass {
///     fn check_ident(&mut self, handler: &mut Handler, ctx: &mut LintContext, id: &ast::Identifier, ){
///         self.LintPassA.check_ident(handler, ctx, id);
///         self.LintPassB.check_ident(handler, ctx, id);
///         ...
///     }
///     fn check_stmt(&mut self, handler: &mut Handler ctx: &mut LintContext, module: &ast::Module){
///         self.LintPassA.check_stmt(handler, ctx, stmt);
///         self.LintPassB.check_stmt(handler, ctx, stmt);
///         ...
///     }
///     ...
/// }
/// ```
macro_rules! declare_combined_lint_pass {
    ([$v:vis $name:ident, [$($passes:ident: $constructor:expr,)*]], $methods:tt) => (
        #[allow(non_snake_case)]
        $v struct $name {
            $($passes: $passes,)*
        }

        impl $name {
            $v fn new() -> Self {
                Self {
                    $($passes: $constructor,)*
                }
            }

            $v fn get_lints() -> LintArray {
                let mut lints = Vec::new();
                $(lints.extend_from_slice(&$passes::get_lints());)*
                lints
            }
        }

        impl LintPass for $name {
            expand_combined_lint_pass_methods!(Handler, LintContext,[$($passes),*], $methods);
        }
    )
}

macro_rules! default_lint_passes {
    ($macro:path, $args:tt) => {
        $macro!(
            $args,
            [
                ImportPosition: ImportPosition,
                UnusedImport: UnusedImport,
                ReImport: ReImport,
            ]
        );
    };
}

macro_rules! declare_combined_default_pass {
    ([$name:ident], $passes:tt) => (
        lint_methods!(declare_combined_lint_pass, [pub $name, $passes]);
    )
}

// Define CombinedLintPass
default_lint_passes!(declare_combined_default_pass, [CombinedLintPass]);
