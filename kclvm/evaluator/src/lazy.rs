use std::ops::Range;

use indexmap::IndexMap;
use kclvm_runtime::ValueRef;

pub type EvalBodyRange = Range<usize>;

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
    pub setters: IndexMap<String, Vec<EvalBodyRange>>,
    /// Calculate times without backtracking.
    pub cal_times: IndexMap<String, usize>,
    // Scope statement
    // pub body: &'ctx [Box<ast::Node<ast::Stmt>>],
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

    /// Set value to the context.
    #[inline]
    pub fn set_value(&mut self, key: &str, value: &ValueRef) {
        self.vars.insert(key.to_string(), value.clone());
        if self.cal_increment(key) && self.cache.get(key).is_none() {
            self.cache.insert(key.to_string(), value.clone());
        }
    }
}
