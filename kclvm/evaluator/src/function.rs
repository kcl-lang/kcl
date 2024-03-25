use kclvm_runtime::{Context, ValueRef};

pub type FuncPtr = fn(&mut Context, &ValueRef, &ValueRef) -> ValueRef;

#[derive(Debug)]
pub struct FunctionValue {
    inner: FuncPtr,
}

impl FunctionValue {
    #[inline]
    pub fn get_fn_ptr(&self) -> u64 {
        self.inner as u64
    }
}
