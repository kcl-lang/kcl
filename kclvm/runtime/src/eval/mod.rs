use std::{
    mem::transmute_copy,
    panic::{RefUnwindSafe, UnwindSafe},
};

use crate::{
    kclvm_context_t, kclvm_eval_scope_t, kclvm_value_ref_t, mut_ptr_as_ref, Context, IndexMap,
    ValueRef,
};

/// Variable setter function type. fn(ctx: &mut Context, scope: &mut ScopeEval, args: ValueRef, kwargs: ValueRef) -> ValueRef.
pub type SetterFuncType =
    unsafe extern "C" fn(*mut kclvm_context_t, *mut kclvm_eval_scope_t) -> *const kclvm_value_ref_t;

/// LazyEvalScope represents a scope of sequentially independent calculations, where
/// the calculation of values is lazy and only recursively performed through
/// backtracking when needed.
#[derive(PartialEq, Clone, Default, Debug)]
pub struct LazyEvalScope {
    /// Temp variable values.
    pub vars: IndexMap<String, ValueRef>,
    /// Variable value cache.
    pub cache: IndexMap<String, ValueRef>,
    /// Backtrack levels.
    pub levels: IndexMap<String, usize>,
    /// Variable setter function pointers.
    pub setters: IndexMap<String, Vec<u64>>,
    /// Calculate times without backtracking.
    pub cal_times: IndexMap<String, usize>,
}

impl LazyEvalScope {
    #[inline]
    pub fn is_backtracking(&self, key: &str) -> bool {
        let level = self.levels.get(key).unwrap_or(&0);
        *level > 0
    }

    #[inline]
    pub fn setter_len(&self, key: &str) -> usize {
        self.setters.get(key).unwrap_or(&vec![]).len()
    }

    #[inline]
    pub fn cal_increment(&mut self, key: &str) -> bool {
        if self.is_backtracking(key) {
            false
        } else {
            let cal_time = *self.cal_times.get(key).unwrap_or(&0);
            let next_cal_time = cal_time + 1;
            self.cal_times.insert(key.to_string(), next_cal_time);
            next_cal_time >= self.setter_len(key)
        }
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    /// Get the value from the context.
    pub fn get_value(&mut self, ctx: &mut Context, key: &str, target: &str) -> ValueRef {
        let value = match self.vars.get(key) {
            Some(value) => value.clone(),
            None => ValueRef::undefined(),
        };
        // Deal in-place modify and return it self immediately.
        if key == target && (!self.is_backtracking(key) || self.setter_len(key) <= 1) {
            value
        } else {
            match self.cache.get(key) {
                Some(value) => value.clone(),
                None => {
                    match &self.setters.get(key) {
                        Some(setters) if !setters.is_empty() => {
                            // Call all setters function to calculate the value recursively.
                            let level = *self.levels.get(key).unwrap_or(&0);
                            let next_level = level + 1;
                            self.levels.insert(key.to_string(), next_level);
                            let n = setters.len();
                            let index = n - next_level;
                            if index >= n {
                                value
                            } else {
                                let fn_ptr = setters[index];
                                unsafe {
                                    let ctx_ref = mut_ptr_as_ref(ctx);
                                    let panic_info = ctx_ref.panic_info.clone();
                                    let setter_fn: SetterFuncType = transmute_copy(&fn_ptr);
                                    // Restore the panic info of current schema attribute.
                                    ctx_ref.panic_info = panic_info;
                                    // Call setter functions
                                    setter_fn(ctx, self)
                                };
                                self.levels.insert(key.to_string(), level);
                                let value = match self.vars.get(key) {
                                    Some(value) => value.clone(),
                                    None => ValueRef::undefined(),
                                };
                                self.cache.insert(key.to_string(), value.clone());
                                value
                            }
                        }
                        _ => value,
                    }
                }
            }
        }
    }

    /// Set value to the context.
    #[inline]
    pub fn set_value(&mut self, key: &str, value: &ValueRef) {
        self.vars.insert(key.to_string(), value.clone());
        if self.cal_increment(key) && self.cache.get(key).is_none() {
            self.cache.insert(key.to_string(), value.clone());
        }
    }
}

impl UnwindSafe for LazyEvalScope {}
impl RefUnwindSafe for LazyEvalScope {}
