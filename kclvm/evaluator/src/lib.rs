//! Copyright The KCL Authors. All rights reserved.

#[cfg(test)]
mod tests;

mod error;
pub mod node;

extern crate kclvm_error;

use indexmap::{IndexMap, IndexSet};
use kclvm_ast::walker::TypedResultWalker;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::str;

use crate::error as kcl_error;
use anyhow::Result;
use kclvm_ast::ast;
use kclvm_error::*;
use kclvm_runtime::{
    Context, ValueRef, _kclvm_get_fn_ptr_by_name, type_pack_and_check, ConfigEntryOperationKind,
    UnionOptions, Value, MAIN_PKG_PATH, PKG_PATH_PREFIX,
};
use kclvm_sema::builtin;
use kclvm_sema::pkgpath_without_prefix;
use kclvm_sema::plugin;

/// SCALAR_KEY denotes the temp scalar key for the global variable json plan process.
const SCALAR_KEY: &str = "";
/// Global level
const GLOBAL_LEVEL: usize = 1;
/// Inner level
const INNER_LEVEL: usize = 2;

/// The evaluator function result
pub type EvalResult = Result<ValueRef>;

/// The evaluator scope.
#[derive(Debug, Default)]
pub struct Scope {
    /// Scalars denotes the expression statement values without attribute.
    pub scalars: Vec<ValueRef>,
    /// schema_scalar_idx denotes whether a schema exists in the scalar list.
    pub schema_scalar_idx: usize,
    /// Scope normal variables
    pub variables: IndexMap<String, ValueRef>,
    /// Scope closures referenced by internal scope.
    pub closures: IndexMap<String, ValueRef>,
    /// Potential arguments in the current scope, such as schema/lambda arguments.
    pub arguments: IndexSet<String>,
    /// Schema self denotes the scope that is belonged to a schema.
    pub schema_self: Option<SchemaSelf>,
}

/// The evaluator scope.
#[derive(Debug, Default)]
pub struct SchemaSelf {
    pub value: ValueRef,
    pub config: ValueRef,
}

type FuncPtr = fn(&mut Context, &ValueRef, &ValueRef) -> ValueRef;

#[derive(Debug)]
pub struct FunctionValue {
    _inner: FuncPtr,
}

/// The evaluator for the program
pub struct Evaluator<'ctx> {
    pub program: &'ctx ast::Program,
    pub ctx: RefCell<EvaluatorContext>,
    pub runtime_ctx: RefCell<Context>,
}

#[derive(Debug)]
pub struct EvaluatorContext {
    pub functions: Vec<FunctionValue>,
    pub imported: HashSet<String>,
    pub lambda_stack: Vec<usize>,
    pub schema_stack: Vec<()>,
    pub schema_expr_stack: Vec<()>,
    pub pkgpath_stack: Vec<String>,
    pub filename_stack: Vec<String>,
    /// Import names mapping
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    /// Package scope to store variable pointers.
    pub pkg_scopes: HashMap<String, Vec<Scope>>,
    /// Local variables in the loop.
    pub local_vars: HashSet<String>,
    /// The names of possible assignment objects for the current instruction.
    pub target_vars: Vec<String>,
    /// Global string caches
    pub global_strings: IndexMap<String, IndexMap<String, ValueRef>>,
    /// Global variable pointers cross different packages.
    pub global_vars: IndexMap<String, IndexMap<String, ValueRef>>,
    /// The filename of the source file corresponding to the current instruction
    pub current_filename: String,
    /// The line number of the source file corresponding to the current instruction
    pub current_line: u64,
    /// Error handler to store compile errors.
    pub handler: Handler,
    /// Debug mode
    pub debug: bool,
    /// Program work directory
    pub workdir: String,
}

impl Default for EvaluatorContext {
    fn default() -> Self {
        Self {
            functions: Default::default(),
            imported: Default::default(),
            lambda_stack: vec![GLOBAL_LEVEL],
            schema_stack: Default::default(),
            schema_expr_stack: Default::default(),
            pkgpath_stack: vec![kclvm_ast::MAIN_PKG.to_string()],
            filename_stack: Default::default(),
            import_names: Default::default(),
            pkg_scopes: Default::default(),
            local_vars: Default::default(),
            target_vars: Default::default(),
            global_strings: Default::default(),
            global_vars: Default::default(),
            current_filename: Default::default(),
            current_line: Default::default(),
            handler: Default::default(),
            debug: Default::default(),
            workdir: Default::default(),
        }
    }
}

/* Value methods */

impl<'ctx> Evaluator<'ctx> {
    /// Construct a 64-bit int value using i64
    #[inline]
    fn int_value(&self, v: i64) -> ValueRef {
        ValueRef::int(v)
    }

    /// Construct a 64-bit float value using f64
    #[inline]
    fn float_value(&self, v: f64) -> ValueRef {
        ValueRef::float(v)
    }

    /// Construct a string value using &str
    #[inline]
    fn string_value(&self, v: &str) -> ValueRef {
        ValueRef::str(v)
    }

    /// Construct a bool value
    #[inline]
    fn bool_value(&self, v: bool) -> ValueRef {
        ValueRef::bool(v)
    }

    /// Construct a None value
    #[inline]
    fn none_value(&self) -> ValueRef {
        ValueRef::none()
    }

    /// Construct a Undefined value
    #[inline]
    fn undefined_value(&self) -> ValueRef {
        ValueRef::undefined()
    }

    /// Construct a empty kcl list value
    #[inline]
    fn list_value(&self) -> ValueRef {
        ValueRef::list(None)
    }

    /// Construct a list value with `n` elements
    fn _list_values(&self, values: &[&ValueRef]) -> ValueRef {
        ValueRef::list(Some(values))
    }

    /// Construct a empty kcl dict value.
    #[inline]
    fn dict_value(&self) -> ValueRef {
        ValueRef::dict(None)
    }

    /// Construct a unit value
    #[inline]
    fn unit_value(&self, v: f64, raw: i64, unit: &str) -> ValueRef {
        ValueRef::unit(v, raw, unit)
    }
    /// Construct a function value using a native function.
    fn _function_value(&self, function: FunctionValue) -> ValueRef {
        ValueRef::func(function._inner as u64, 0, self.list_value(), "", "", false)
    }
    /// Construct a function value using a native function.
    fn _function_value_with_ptr(&self, function_ptr: u64) -> ValueRef {
        ValueRef::func(function_ptr, 0, self.list_value(), "", "", false)
    }
    /// Construct a closure function value with the closure variable.
    fn _closure_value(&self, function: FunctionValue, closure: ValueRef) -> ValueRef {
        ValueRef::func(function._inner as u64, 0, closure, "", "", false)
    }
    /// Construct a builtin function value using the function name.
    fn _builtin_function_value(&self, name: &str) -> ValueRef {
        let func = _kclvm_get_fn_ptr_by_name(name);
        ValueRef::func(func, 0, self.list_value(), "", "", false)
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// lhs + rhs
    #[inline]
    fn add(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_add(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs - rhs
    #[inline]
    fn sub(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_sub(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs * rhs
    #[inline]
    fn mul(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_mul(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs / rhs
    #[inline]
    fn div(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_div(&rhs)
    }
    /// lhs // rhs
    #[inline]
    fn floor_div(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_floor_div(&rhs)
    }
    /// lhs % rhs
    #[inline]
    fn r#mod(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_mod(&rhs)
    }
    /// lhs ** rhs
    #[inline]
    fn pow(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_pow(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs << rhs
    #[inline]
    fn bit_lshift(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_lshift(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs >> rhs
    #[inline]
    fn bit_rshift(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_rshift(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs & rhs
    #[inline]
    fn bit_and(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_and(&rhs)
    }
    /// lhs | rhs
    #[inline]
    fn bit_or(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_or(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs ^ rhs
    #[inline]
    fn bit_xor(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_xor(&rhs)
    }
    /// lhs and rhs
    #[inline]
    fn logic_and(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.logic_and(&rhs).into()
    }
    /// lhs or rhs
    #[inline]
    fn logic_or(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.logic_or(&rhs).into()
    }
    /// lhs == rhs
    #[inline]
    fn cmp_equal_to(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_equal(&rhs).into()
    }
    /// lhs != rhs
    #[inline]
    fn cmp_not_equal_to(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_not_equal(&rhs).into()
    }
    /// lhs > rhs
    #[inline]
    fn cmp_greater_than(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_greater_than(&rhs).into()
    }
    /// lhs >= rhs
    #[inline]
    fn cmp_greater_than_or_equal(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_greater_than_or_equal(&rhs).into()
    }
    /// lhs < rhs
    #[inline]
    fn cmp_less_than(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_less_than(&rhs).into()
    }
    /// lhs <= rhs
    #[inline]
    fn cmp_less_than_or_equal(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_greater_than_or_equal(&rhs).into()
    }
    /// lhs as rhs
    #[inline]
    fn r#as(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        type_pack_and_check(
            &mut self.runtime_ctx.borrow_mut(),
            &lhs,
            vec![&rhs.as_str()],
        )
    }
    /// lhs is rhs
    #[inline]
    fn is(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        (lhs == rhs).into()
    }
    /// lhs is not rhs
    #[inline]
    fn is_not(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        (lhs != rhs).into()
    }
    /// lhs in rhs
    #[inline]
    fn r#in(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.r#in(&rhs).into()
    }
    /// lhs not in rhs
    #[inline]
    fn not_in(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.not_in(&rhs).into()
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// Value subscript a[b]
    #[inline]
    fn _value_subscript(&self, value: &ValueRef, item: &ValueRef) -> ValueRef {
        value.bin_subscr(item)
    }
    /// Value is truth function, return i1 value.
    #[inline]
    fn value_is_truthy(&self, value: &ValueRef) -> bool {
        value.is_truthy()
    }
    /// Value deep copy
    #[inline]
    fn value_deep_copy(&self, value: &ValueRef) -> ValueRef {
        value.deep_copy()
    }
    /// value_union unions two collection elements.
    fn _value_union(&self, lhs: &mut ValueRef, rhs: &ValueRef) -> ValueRef {
        let attr_map = match &*lhs.rc.borrow() {
            Value::dict_value(dict) => dict.attr_map.clone(),
            Value::schema_value(schema) => schema.config.attr_map.clone(),
            _ => panic!("invalid object '{}' in attr_map", lhs.type_str()),
        };
        let opts = UnionOptions {
            list_override: false,
            idempotent_check: false,
            config_resolve: true,
        };
        let ctx = &mut self.runtime_ctx.borrow_mut();
        if rhs.is_config() {
            let dict = rhs.as_dict_ref();
            for k in dict.values.keys() {
                let entry = rhs.dict_get_entry(k).unwrap();
                lhs.union_entry(ctx, &entry, true, &opts);
                // Has type annotation
                if let Some(ty) = attr_map.get(k) {
                    let value = lhs.dict_get_value(k).unwrap();
                    lhs.dict_update_key_value(k, type_pack_and_check(ctx, &value, vec![ty]));
                }
            }
            lhs.clone()
        } else {
            lhs.union_entry(ctx, rhs, true, &opts)
        }
    }
    // List get the item using the index.
    #[inline]
    fn _list_get(&self, list: &ValueRef, index: ValueRef) -> ValueRef {
        list.list_get(index.as_int() as isize).unwrap()
    }
    // List set the item using the index.
    #[inline]
    fn _list_set(&self, list: &mut ValueRef, index: ValueRef, value: &ValueRef) {
        list.list_set(index.as_int() as usize, value)
    }
    // List slice.
    #[inline]
    fn _list_slice(
        &self,
        list: &ValueRef,
        start: &ValueRef,
        stop: &ValueRef,
        step: &ValueRef,
    ) -> ValueRef {
        list.list_slice(start, stop, step)
    }
    /// Append a item into the list.
    #[inline]
    fn list_append(&self, list: &mut ValueRef, item: &ValueRef) {
        list.list_append(item)
    }
    /// Append a list item and unpack it into the list.
    #[inline]
    fn list_append_unpack(&self, list: &mut ValueRef, item: &ValueRef) {
        list.list_append_unpack(item)
    }
    /// Runtime list value pop
    #[inline]
    fn _list_pop(&self, list: &mut ValueRef) -> Option<ValueRef> {
        list.list_pop()
    }
    /// Runtime list pop the first value
    #[inline]
    fn _list_pop_first(&self, list: &mut ValueRef) -> Option<ValueRef> {
        list.list_pop_first()
    }
    /// List clear value.
    #[inline]
    fn _list_clear(&self, list: &mut ValueRef) {
        list.list_clear()
    }
    /// Return number of occurrences of the list value.
    #[inline]
    fn _list_count(&self, list: &ValueRef, item: &ValueRef) -> ValueRef {
        ValueRef::int(list.list_count(item) as i64)
    }
    /// Return first index of the list value. Panic if the value is not present.
    #[inline]
    fn _list_find(&self, list: &ValueRef, item: &ValueRef) -> isize {
        list.list_find(item)
    }
    /// Insert object before index of the list value.
    #[inline]
    fn _list_insert(&self, list: &mut ValueRef, index: &ValueRef, value: &ValueRef) {
        list.list_insert_at(index.as_int() as usize, value)
    }
    /// List length.
    #[inline]
    fn _list_len(&self, list: &ValueRef) -> usize {
        list.len()
    }
    /// Dict get the value of the key.
    #[inline]
    fn _dict_get(&self, dict: &ValueRef, key: &ValueRef) -> ValueRef {
        dict.dict_get(key).unwrap()
    }
    #[inline]
    fn dict_get_value(&self, dict: &ValueRef, key: &str) -> ValueRef {
        dict.dict_get_value(key).unwrap()
    }
    /// Dict clear value.
    #[inline]
    fn _dict_clear(&self, dict: &mut ValueRef) {
        dict.dict_clear()
    }
    /// Dict length.
    #[inline]
    fn _dict_len(&self, dict: &ValueRef) -> usize {
        dict.len()
    }
    /// Insert a dict entry including key, value, op and insert_index into the dict,
    /// and the type of key is `&str`
    #[inline]
    fn dict_insert(
        &self,
        dict: &mut ValueRef,
        key: &str,
        value: &ValueRef,
        op: &ast::ConfigEntryOperation,
        insert_index: i32,
    ) {
        let op = match op {
            ast::ConfigEntryOperation::Union => ConfigEntryOperationKind::Union,
            ast::ConfigEntryOperation::Override => ConfigEntryOperationKind::Override,
            ast::ConfigEntryOperation::Insert => ConfigEntryOperationKind::Insert,
        };
        dict.dict_insert(
            &mut self.runtime_ctx.borrow_mut(),
            key,
            value,
            op,
            insert_index,
        );
    }
    /// Insert a dict entry including key, value, op and insert_index into the dict,
    /// and the type of key is `&str`
    #[inline]
    fn dict_insert_value(&self, dict: &mut ValueRef, key: &str, value: &ValueRef) {
        dict.dict_insert(
            &mut self.runtime_ctx.borrow_mut(),
            key,
            value,
            ConfigEntryOperationKind::Union,
            -1,
        );
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// Current package path
    #[inline]
    fn current_pkgpath(&self) -> String {
        self.ctx
            .borrow()
            .pkgpath_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }

    /// Current filename
    #[inline]
    fn current_filename(&self) -> String {
        self.ctx
            .borrow()
            .filename_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }
    /// Current line
    #[inline]
    fn current_line(&self) -> u64 {
        self.ctx.borrow().current_line
    }
    /// Init a scope named `pkgpath` with all builtin functions
    fn init_scope(&self, pkgpath: &str) {
        {
            let mut ctx = self.ctx.borrow_mut();
            let pkg_scopes = &mut ctx.pkg_scopes;
            if pkg_scopes.contains_key(pkgpath) {
                return;
            }
            let scopes = vec![Scope::default()];
            pkg_scopes.insert(String::from(pkgpath), scopes);
        }
        let msg = format!("pkgpath {} is not found", pkgpath);
        // Init all global types including schema and rule
        let module_list: &Vec<ast::Module> = if self.program.pkgs.contains_key(pkgpath) {
            self.program.pkgs.get(pkgpath).expect(&msg)
        } else if pkgpath.starts_with(kclvm_runtime::PKG_PATH_PREFIX)
            && self.program.pkgs.contains_key(&pkgpath[1..])
        {
            self.program
                .pkgs
                .get(&pkgpath[1..])
                .expect(kcl_error::INTERNAL_ERROR_MSG)
        } else {
            panic!("pkgpath {} not found", pkgpath);
        };
        for module in module_list {
            for stmt in &module.body {
                let name = match &stmt.node {
                    ast::Stmt::Schema(schema_stmt) => schema_stmt.name.node.clone(),
                    ast::Stmt::Rule(rule_stmt) => rule_stmt.name.node.clone(),
                    _ => "".to_string(),
                };
                if !name.is_empty() {
                    let name = name.as_str();
                    let _var_name = format!("${}.${}", pkgpath_without_prefix!(pkgpath), name);
                    let global_var_ptr = self.undefined_value();
                    self.add_variable(name, global_var_ptr);
                }
            }
        }
        // Init all builtin functions
        for symbol in builtin::BUILTIN_FUNCTION_NAMES {
            let function_name =
                format!("{}_{}", builtin::KCL_BUILTIN_FUNCTION_MANGLE_PREFIX, symbol);
            let function_ptr = _kclvm_get_fn_ptr_by_name(&function_name);
            self.add_variable(symbol, self._function_value_with_ptr(function_ptr));
        }
        self.enter_scope();
    }

    /// Get the scope level
    fn scope_level(&self) -> usize {
        let current_pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        // Sub the builtin global scope
        scopes.len() - 1
    }

    /// Enter scope
    fn enter_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let scope = Scope::default();
        scopes.push(scope);
    }

    /// Leave scope
    fn leave_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        scopes.pop();
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// New aa Evaluator using the LLVM Context and AST Program
    pub fn new(program: &'ctx ast::Program) -> Evaluator<'ctx> {
        Evaluator {
            ctx: RefCell::new(EvaluatorContext::default()),
            runtime_ctx: RefCell::new(Context::new()),
            program,
        }
    }

    /// Generate LLVM IR of ast module.
    pub fn run(self: &Evaluator<'ctx>) -> Result<(String, String)> {
        self.init_scope(kclvm_ast::MAIN_PKG);
        for module in self
            .program
            .pkgs
            .get(kclvm_ast::MAIN_PKG)
            .unwrap_or(&vec![])
        {
            self.walk_module(module)?;
        }
        Ok(self.globals_to_plan_str())
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// Get compiler default ok result
    #[inline]
    pub fn ok_result(&self) -> EvalResult {
        Ok(self.undefined_value())
    }

    pub(crate) fn clear_local_vars(&self) {
        self.ctx.borrow_mut().local_vars.clear();
    }

    /// Reset target vars
    pub(crate) fn reset_target_vars(&self) {
        let target_vars = &mut self.ctx.borrow_mut().target_vars;
        target_vars.clear();
        target_vars.push("".to_string());
    }

    #[inline]
    pub(crate) fn last_lambda_scope(&self) -> usize {
        *self
            .ctx
            .borrow()
            .lambda_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
    }

    /// Append a scalar value into the scope.
    pub fn add_scalar(&self, scalar: ValueRef, is_schema: bool) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let scopes = pkg_scopes
            .get_mut(&current_pkgpath)
            .unwrap_or_else(|| panic!("pkgpath {} is not found", current_pkgpath));
        if let Some(last) = scopes.last_mut() {
            let scalars = &mut last.scalars;
            // TODO: To avoid conflicts, only the last schema scalar expressions are allowed.
            let schema_scalar_idx = &mut last.schema_scalar_idx;
            if is_schema {
                // Remove the last schema scalar.
                if *schema_scalar_idx < scalars.len() {
                    scalars.remove(*schema_scalar_idx);
                }
                // Override the last schema scalar.
                scalars.push(scalar);
                *schema_scalar_idx = scalars.len() - 1;
            } else {
                scalars.push(scalar);
            }
        }
    }

    /// Append a variable into the scope
    pub fn add_variable(&self, name: &str, pointer: ValueRef) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        if let Some(last) = scopes.last_mut() {
            let variables = &mut last.variables;
            if !variables.contains_key(name) {
                variables.insert(name.to_string(), pointer);
            }
        }
    }

    /// Store the variable named `name` with `value` from the current scope, return false when not found
    pub fn store_variable_in_current_scope(&self, name: &str, value: ValueRef) -> bool {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let index = scopes.len() - 1;
        let variables = &mut scopes[index].variables;
        if let Some(_var) = variables.get(&name.to_string()) {
            variables.insert(name.to_string(), value);
            return true;
        }
        false
    }

    /// Store the variable named `name` with `value` from the scope, return false when not found
    pub fn store_variable(&self, name: &str, value: ValueRef) -> bool {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let variables = &mut scopes[index].variables;
            if variables.get(&name.to_string()).is_some() {
                variables.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    /// Resolve variable in scope, return false when not found.
    #[inline]
    pub fn resolve_variable(&self, name: &str) -> bool {
        self.resolve_variable_level(name).is_some()
    }

    /// Resolve variable level in scope, return None when not found.
    pub fn resolve_variable_level(&self, name: &str) -> Option<usize> {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        let mut level = None;
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let variables = &scopes[index].variables;
            let arguments = &scopes[index].arguments;
            if variables.get(name).is_some() {
                level = Some(index);
                break;
            }
            if arguments.contains(name) {
                level = Some(index);
                break;
            }
        }
        level
    }

    /// Append a variable or update the existed local variable.
    pub fn add_or_update_local_variable(&self, name: &str, value: ValueRef) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let is_local_var = self.is_local_var(name);
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let mut existed = false;
        // Query the variable in all scopes.
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let is_argument = scopes[index].arguments.contains(name);
            let variables_mut = &mut scopes[index].variables;
            match variables_mut.get(&name.to_string()) {
                // If the local variable is found, store the new value for the variable.
                // We cannot update rule/lambda/schema arguments because they are read-only.
                Some(_) if index > GLOBAL_LEVEL && !is_local_var && !is_argument => {
                    variables_mut.insert(name.to_string(), value.clone());
                    existed = true;
                }
                _ => {}
            }
        }
        // If not found, alloc a new variable.
        if !existed {
            // Store the value for the variable and add the variable into the current scope.
            if let Some(last) = scopes.last_mut() {
                last.variables.insert(name.to_string(), value);
            }
        }
    }

    /// Append a variable or update the existed variable
    pub fn add_or_update_global_variable(&self, name: &str, value: ValueRef) {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let mut existed = false;
        if let Some(last) = scopes.last_mut() {
            let variables = &mut last.variables;
            if variables.get(&name.to_string()).is_some() {
                variables.insert(name.to_string(), value.clone());
                existed = true;
            }
        }
        if !existed {
            if let Some(last) = scopes.last_mut() {
                let variables = &mut last.variables;
                if !variables.contains_key(name) {
                    variables.insert(name.to_string(), value);
                }
            }
        }
    }

    /// Get the variable value named `name` from the scope, return Err when not found
    pub fn get_variable(&self, name: &str) -> EvalResult {
        let current_pkgpath = self.current_pkgpath();
        self.get_variable_in_pkgpath(name, &current_pkgpath)
    }

    /// Get the variable value named `name` from the scope, return Err when not found
    pub fn get_variable_in_schema(&self, name: &str) -> EvalResult {
        let pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let scope = ctx.pkg_scopes.get(&pkgpath).unwrap().last().unwrap();
        let schema_self = scope.schema_self.as_ref().unwrap();
        let schema_value = &schema_self.value;
        if let Some(value) = schema_value.dict_get_value(name) {
            Ok(value)
        } else {
            let current_pkgpath = self.current_pkgpath();
            self.get_variable_in_pkgpath(name, &current_pkgpath)
        }
    }

    /// Get the variable value named `name` from the scope named `pkgpath`, return Err when not found
    pub fn get_variable_in_pkgpath(&self, name: &str, pkgpath: &str) -> EvalResult {
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        let pkgpath =
            if !pkgpath.starts_with(kclvm_runtime::PKG_PATH_PREFIX) && pkgpath != MAIN_PKG_PATH {
                format!("{}{}", kclvm_runtime::PKG_PATH_PREFIX, pkgpath)
            } else {
                pkgpath.to_string()
            };
        let mut result = Err(anyhow::anyhow!("name '{}' is not defined", name));
        let is_in_schema = self.is_in_schema();
        // System module
        if builtin::STANDARD_SYSTEM_MODULE_NAMES_WITH_AT.contains(&pkgpath.as_str()) {
            let pkgpath = &pkgpath[1..];
            let _mangle_func_name = format!(
                "{}{}_{}",
                builtin::KCL_SYSTEM_MODULE_MANGLE_PREFIX,
                pkgpath_without_prefix!(pkgpath),
                name
            );
            let value = if pkgpath == builtin::system_module::UNITS
                && builtin::system_module::UNITS_FIELD_NAMES.contains(&name)
            {
                let value_float: f64 = kclvm_runtime::f64_unit_value(name);
                let value_int: u64 = kclvm_runtime::u64_unit_value(name);
                if value_int != 1 {
                    self.int_value(value_int as i64)
                } else {
                    self.float_value(value_float)
                }
            } else {
                todo!()
            };
            Ok(value)
        }
        // Plugin pkgpath
        else if pkgpath.starts_with(plugin::PLUGIN_PREFIX_WITH_AT) {
            let _null_fn_ptr = 0;
            let name = format!("{}.{}", &pkgpath[1..], name);
            let _none_value = self.none_value();
            return Ok(ValueRef::func(
                0,
                0,
                self.undefined_value(),
                &name,
                "",
                true,
            ));
        // User pkgpath
        } else {
            // Global or local variables.
            let scopes = pkg_scopes
                .get(&pkgpath)
                .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
            // Scopes 0 is builtin scope, Scopes 1 is the global scope, Scopes 2~ are the local scopes
            let scopes_len = scopes.len();
            for i in 0..scopes_len {
                let index = scopes_len - i - 1;
                let variables = &scopes[index].variables;
                if let Some(var) = variables.get(&name.to_string()) {
                    // Closure vars, 2 denotes the builtin scope and the global scope, here is a closure scope.
                    let value = if i >= 1 && i < scopes_len - 2 {
                        let last_lambda_scope = self.last_lambda_scope();
                        // Local scope variable
                        if index >= last_lambda_scope {
                            var.clone()
                        } else {
                            // Outer lambda closure
                            let _variables = &scopes[last_lambda_scope].variables;
                            let ptr: Option<&ValueRef> = None;
                            // Lambda closure
                            match ptr {
                                Some(closure_map) => closure_map.dict_get_value(name).unwrap(),
                                None => var.clone(),
                            }
                        }
                    } else {
                        var.clone()
                    };
                    result = Ok(value);
                    break;
                }
            }
            match result {
                Ok(_) => result,
                Err(ref err) => {
                    if !is_in_schema {
                        let mut ctx = self.ctx.borrow_mut();
                        let handler = &mut ctx.handler;
                        let pos = Position {
                            filename: self.current_filename(),
                            line: self.current_line(),
                            column: None,
                        };
                        handler.add_compile_error(&err.to_string(), (pos.clone(), pos));
                        handler.abort_if_any_errors()
                    }
                    result
                }
            }
        }
    }

    /// Get closure map in the current inner scope.
    pub(crate) fn get_current_inner_scope_variable_map(&self) -> ValueRef {
        let var_map = {
            let last_lambda_scope = self.last_lambda_scope();
            // Get variable map in the current scope.
            let pkgpath = self.current_pkgpath();
            let pkgpath = if !pkgpath.starts_with(PKG_PATH_PREFIX) && pkgpath != MAIN_PKG_PATH {
                format!("{}{}", PKG_PATH_PREFIX, pkgpath)
            } else {
                pkgpath
            };
            let ctx = self.ctx.borrow();
            let pkg_scopes = &ctx.pkg_scopes;
            let scopes = pkg_scopes
                .get(&pkgpath)
                .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
            let current_scope = scopes.len() - 1;
            // Get last closure map.

            if current_scope >= last_lambda_scope && last_lambda_scope > 0 {
                let _variables = &scopes[last_lambda_scope].variables;
                // todo: lambda closure in the lambda.
                let ptr: Option<ValueRef> = None;
                let var_map = match ptr {
                    Some(ptr) => ptr.clone(),
                    None => self.dict_value(),
                };
                // Get variable map including schema  in the current scope.
                for i in last_lambda_scope..current_scope + 1 {
                    let variables = &scopes
                        .get(i)
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                        .variables;
                    for (_key, _ptr) in variables {
                        todo!()
                    }
                }
                var_map
            } else {
                self.dict_value()
            }
        };
        // Capture schema `self` closure.
        if self.is_in_schema() {
            todo!()
        }
        var_map
    }

    /// Load value from name.
    pub fn load_value(&self, pkgpath: &str, names: &[&str]) -> EvalResult {
        if names.is_empty() {
            return Err(anyhow::anyhow!("error: read value from empty name"));
        }
        let name = names[0];
        // Get variable from the scope.
        let get = |name: &str| {
            match (
                self.is_in_schema(),
                self.is_in_lambda(),
                self.is_local_var(name),
            ) {
                // Get from local or global scope
                (false, _, _) | (_, _, true) => self.get_variable(name),
                // Get variable from the current schema scope.
                (true, false, false) => self.get_variable_in_schema(name),
                // Get from local scope including lambda arguments, lambda variables,
                // loop variables or global variables.
                (true, true, _) =>
                // Get from local scope including lambda arguments, lambda variables,
                // loop variables or global variables.
                {
                    match self.resolve_variable_level(name) {
                        // Closure variable or local variables
                        Some(level) if level > GLOBAL_LEVEL => self.get_variable(name),
                        // Schema closure or global variables
                        _ => self.get_variable_in_schema(name),
                    }
                }
            }
        };
        if names.len() == 1 {
            get(name)
        } else {
            let mut value = if pkgpath.is_empty() {
                get(name)
            } else {
                self.ok_result()
            }
            .expect(kcl_error::INTERNAL_ERROR_MSG);
            for i in 0..names.len() - 1 {
                let attr = names[i + 1];
                if i == 0 && !pkgpath.is_empty() {
                    value = self
                        .get_variable_in_pkgpath(attr, pkgpath)
                        .expect(kcl_error::INTERNAL_ERROR_MSG)
                } else {
                    value = value.load_attr(attr)
                }
            }
            Ok(value)
        }
    }

    pub fn build_call(&self, _name: &str, _args: &[ValueRef]) -> ValueRef {
        todo!()
    }

    /// Push a lambda definition scope into the lambda stack
    #[inline]
    pub fn push_lambda(&self, scope: usize) {
        self.ctx.borrow_mut().lambda_stack.push(scope);
    }

    /// Pop a lambda definition scope.
    #[inline]
    pub fn pop_lambda(&self) {
        self.ctx.borrow_mut().lambda_stack.pop();
    }

    #[inline]
    pub fn is_in_lambda(&self) -> bool {
        *self
            .ctx
            .borrow()
            .lambda_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            > GLOBAL_LEVEL
    }

    #[inline]
    pub fn is_in_schema(&self) -> bool {
        !self.ctx.borrow().schema_stack.is_empty()
    }

    #[inline]
    pub fn is_in_schema_expr(&self) -> bool {
        !self.ctx.borrow().schema_expr_stack.is_empty()
    }

    #[inline]
    pub fn is_local_var(&self, name: &str) -> bool {
        self.ctx.borrow().local_vars.contains(name)
    }

    /// Plan globals to a planed json and yaml string
    pub fn globals_to_plan_str(&self) -> (String, String) {
        let current_pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        let scopes = pkg_scopes
            .get(&current_pkgpath)
            .unwrap_or_else(|| panic!("pkgpath {} is not found", current_pkgpath));
        // The global scope.
        let scope = scopes.last().expect(kcl_error::INTERNAL_ERROR_MSG);
        let scalars = &scope.scalars;
        let globals = &scope.variables;
        // Construct a plan object.
        let mut global_dict = self.dict_value();
        // Plan empty dict result.
        if scalars.is_empty() && globals.is_empty() {
            return global_dict.plan(&self.runtime_ctx.borrow());
        }
        // Deal scalars
        for scalar in scalars.iter() {
            self.dict_insert_value(&mut global_dict, SCALAR_KEY, scalar);
        }
        // Deal global variables
        for (name, value) in globals.iter() {
            let mut value_dict = self.dict_value();
            self.dict_insert_value(&mut value_dict, name.as_str(), value);
            self.dict_insert_value(&mut global_dict, SCALAR_KEY, &value_dict);
        }
        // Plan result to JSON and YAML string.
        match global_dict.dict_get_value(SCALAR_KEY) {
            Some(value) => value.plan(&self.runtime_ctx.borrow()),
            None => self.dict_value().plan(&self.runtime_ctx.borrow()),
        }
    }
}
