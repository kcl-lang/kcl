//! Copyright 2021 The KCL Authors. All rights reserved.

mod backend;
mod builder;
mod r#type;
mod value;

pub use backend::*;
pub use builder::*;
pub use r#type::*;
pub use value::*;

pub trait ProgramCodeGen: TypeCodeGen + ValueCodeGen + BuilderMethods {
    /// Current package path
    fn current_pkgpath(&self) -> String;
    /// Current filename
    fn current_filename(&self) -> String;
    /// Init a scope named `pkgpath` with all builtin functions
    fn init_scope(&self, pkgpath: &str);
    /// Get the scope level
    fn scope_level(&self) -> usize;
    /// Enter scope
    fn enter_scope(&self);
    /// Leave scope
    fn leave_scope(&self);
}
