//! The goal of this module is to translate KCL Program into LLVM IR code, where each AST corresponding to KCL
//! pkgpath corresponds to a module of LLVM. They share a global symbol table and LLVM context. Different LLVM
//! module modules pass extern and declare keys. Declare and call them in words, and finally use clang to link
//! them together.
//!
//! Copyright 2021 The KCL Authors. All rights reserved.

mod backtrack;
mod context;
mod emit;
mod metadata;
mod module;
mod node;
mod schema;
mod utils;

pub use emit::emit_code;

/// Object file type format suffix.
#[cfg(target_os = "windows")]
pub const OBJECT_FILE_SUFFIX: &str = ".obj";
#[cfg(not(target_os = "windows"))]
pub const OBJECT_FILE_SUFFIX: &str = ".o";
/// LLVM IR text format suffix .ll
pub const LL_FILE_SUFFIX: &str = ".ll";
