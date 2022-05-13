//! Copyright 2021 The KCL Authors. All rights reserved.

use std::fmt::Debug;

/// CodeGenObject constrains the behavior that types and values need to satisfy
pub trait CodeGenObject: Copy + PartialEq + Debug {}

/// BackendTypes define the value and type abstraction.
pub trait BackendTypes {
    /// Value abstraction, there may be different implementations corresponding
    /// to different compiler backends.
    type Value: CodeGenObject;
    /// Type abstraction, there may be different implementations corresponding
    /// to different compiler backends.
    type Type: CodeGenObject;
    /// BasicBlock is a SSA basic block abstraction, for the construction of branch, jump, etc. instructions.
    type BasicBlock: Copy;
    /// Function is a SSA basic function value abstraction.
    type Function: Copy;
    /// FunctionLet is SSA basic function declaration abstraction.
    type FunctionLet: Copy;
}
