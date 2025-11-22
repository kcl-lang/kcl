//! Copyright The KCL Authors. All rights reserved.

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum RuntimeErrorType {
    EvaluationError = 1,
    RecursiveLoad = 2,
    FloatOverflow = 3,
    FloatUnderflow = 4,
    IntOverflow = 5,
    TypeError = 6,
    AssertionError = 7,
    Deprecated = 8,
    DeprecatedWarning = 9,
    SchemaCheckFailure = 10,
}
