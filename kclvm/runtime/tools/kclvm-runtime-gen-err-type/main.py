# Copyright 2021 The KCL Authors. All rights reserved.

import kclvm.kcl.error as kcl_error

# enum -> code
# kcl_error.ErrType.EvaluationError_TYPE.value[0]

# code -> type
# x = kcl_error.ErrType((6,))

print(
    """// Copyright 2021 The KCL Authors. All rights reserved.

// Auto generated, DONOT EDIT!!!

// python: kclvm.kcl.error.ErrType

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum ErrType {"""
)
for x in kcl_error.ErrType:
    print(f"    {x.name} = {x.value[0]},")
print("}")
