use generational_arena::Index;

use crate::ValueRef;

impl ValueRef {
    /// Try get the proxy function index
    pub fn try_get_proxy(&self) -> Option<Index> {
        match &*self.rc.borrow() {
            crate::Value::func_value(func) => func.proxy,
            _ => None,
        }
    }
}
