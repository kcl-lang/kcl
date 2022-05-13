// Copyright 2021 The KCL Authors. All rights reserved.

pub mod ast;
pub mod token;
pub mod token_stream;
pub mod walker;

#[cfg(test)]
mod tests;

pub const MAIN_PKG: &str = "__main__";
