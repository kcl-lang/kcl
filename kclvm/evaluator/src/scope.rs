use crate::func::ClosureMap;
use crate::lazy::merge_setters;
use crate::{
    error as kcl_error, lazy::LazyEvalScope, rule::RuleEvalContextRef, schema::SchemaEvalContextRef,
};
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast;
use kclvm_runtime::{ValueRef, _kclvm_get_fn_ptr_by_name, MAIN_PKG_PATH};
use kclvm_sema::{builtin, plugin};

use crate::{Evaluator, GLOBAL_LEVEL, INNER_LEVEL};

/// The evaluator scope.
#[derive(Debug, Default)]
pub struct SchemaSelf {
    pub value: ValueRef,
    pub config: ValueRef,
}

/// The evaluator scope.
#[derive(Debug, Default)]
pub struct Scope {
    /// Scalars denotes the expression statement values without attribute.
    pub scalars: Vec<ValueRef>,
    /// schema_scalar_idx denotes whether a schema exists in the scalar list.
    pub schema_scalar_idx: usize,
    /// Scope normal variables
    pub variables: IndexMap<String, ValueRef>,
    /// Potential arguments in the current scope, such as schema/lambda arguments.
    pub arguments: IndexSet<String>,
}

impl<'ctx> Evaluator<'ctx> {
    /// Init a scope named `pkgpath` with all builtin functions
    pub(crate) fn init_scope(&self, pkgpath: &str) {
        {
            let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
            if pkg_scopes.contains_key(pkgpath) {
                return;
            }
            let scopes = vec![Scope::default()];
            pkg_scopes.insert(String::from(pkgpath), scopes);
        }
        let msg = format!("pkgpath {} is not found", pkgpath);
        // Get the AST module list in the package path.
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
        // Init all global types including schema and rule.
        for module in module_list {
            for stmt in &module.body {
                let name = match &stmt.node {
                    ast::Stmt::Schema(schema_stmt) => schema_stmt.name.node.clone(),
                    ast::Stmt::Rule(rule_stmt) => rule_stmt.name.node.clone(),
                    _ => "".to_string(),
                };
                if !name.is_empty() {
                    self.add_variable(&name, self.undefined_value());
                }
            }
        }
        // Init all builtin functions
        for symbol in builtin::BUILTIN_FUNCTION_NAMES {
            let function_name =
                format!("{}_{}", builtin::KCL_BUILTIN_FUNCTION_MANGLE_PREFIX, symbol);
            let function_ptr = _kclvm_get_fn_ptr_by_name(&function_name);
            self.add_variable(symbol, self.function_value_with_ptr(function_ptr));
        }
        // Init lazy scopes.
        {
            let mut lazy_scopes = self.lazy_scopes.borrow_mut();
            let mut setters = IndexMap::new();
            for (index, module) in module_list.iter().enumerate() {
                let index = self.add_global_body(index);
                merge_setters(&mut setters, &self.emit_setters(&module.body, Some(index)))
            }
            if !lazy_scopes.contains_key(pkgpath) {
                lazy_scopes.insert(
                    pkgpath.to_string(),
                    LazyEvalScope {
                        setters,
                        ..Default::default()
                    },
                );
            }
        }
        // Enter the global scope.
        self.enter_scope();
    }

    /// Get the scope level
    pub(crate) fn scope_level(&self) -> usize {
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &self.pkg_scopes.borrow();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        // Sub the builtin global scope
        scopes.len() - 1
    }

    /// Enter scope
    pub(crate) fn enter_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let scope = Scope::default();
        scopes.push(scope);
    }

    /// Leave scope
    pub(crate) fn leave_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        scopes.pop();
    }

    pub(crate) fn get_schema_eval_context(&self) -> Option<SchemaEvalContextRef> {
        match self.schema_stack.borrow().last() {
            Some(ctx) => match ctx {
                crate::EvalContext::Schema(schema) => Some(schema.clone()),
                crate::EvalContext::Rule(_) => None,
            },
            None => None,
        }
    }

    pub(crate) fn get_rule_eval_context(&self) -> Option<RuleEvalContextRef> {
        match self.schema_stack.borrow().last() {
            Some(ctx) => match ctx {
                crate::EvalContext::Schema(_) => None,
                crate::EvalContext::Rule(rule) => Some(rule.clone()),
            },
            None => None,
        }
    }

    /// Returns (value, config, config_meta)
    #[inline]
    pub(crate) fn get_schema_or_rule_config_info(&self) -> Option<(ValueRef, ValueRef, ValueRef)> {
        match self.get_schema_eval_context() {
            Some(v) => Some((
                v.borrow().value.clone(),
                v.borrow().config.clone(),
                v.borrow().config_meta.clone(),
            )),
            None => self.get_rule_eval_context().map(|v| {
                (
                    v.borrow().value.clone(),
                    v.borrow().config.clone(),
                    v.borrow().config_meta.clone(),
                )
            }),
        }
    }

    /// Append a scalar value into the scope.
    pub fn add_scalar(&self, scalar: ValueRef, is_schema: bool) {
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let scopes = pkg_scopes
            .get_mut(&current_pkgpath)
            .unwrap_or_else(|| panic!("pkgpath {} is not found", current_pkgpath));
        if let Some(last) = scopes.last_mut() {
            let scalars = &mut last.scalars;
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
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        if let Some(last) = scopes.last_mut() {
            let variables = &mut last.variables;
            variables.insert(name.to_string(), pointer);
        }
    }

    /// Store the argument named `name` in the current scope.
    pub(crate) fn store_argument_in_current_scope(&self, name: &str) {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let index = scopes.len() - 1;
        let arguments_mut = &mut scopes[index].arguments;
        arguments_mut.insert(name.to_string());
    }

    /// Store the variable named `name` with `value` from the current scope, return false when not found
    pub fn store_variable_in_current_scope(&self, name: &str, value: ValueRef) -> bool {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let index = scopes.len() - 1;
        let variables = &mut scopes[index].variables;
        // If exists and update it
        if variables.get(&name.to_string()).is_some() {
            variables.insert(name.to_string(), value);
            return true;
        }
        false
    }

    /// Store the variable named `name` with `value` from the scope, return false when not found
    pub fn store_variable(&self, name: &str, value: ValueRef) -> bool {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            let variables = &mut scopes[index].variables;
            // If exists and update it
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
        let pkg_scopes = &self.pkg_scopes.borrow();
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
        let is_local_var = self.is_local_var(name);
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
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
    pub fn add_or_update_global_variable(
        &self,
        name: &str,
        value: ValueRef,
        save_lazy_scope: bool,
    ) {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let pkg_scopes = &mut self.pkg_scopes.borrow_mut();
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let mut existed = false;
        if let Some(last) = scopes.last_mut() {
            let variables = &mut last.variables;
            if variables.get(&name.to_string()).is_some() {
                variables.insert(name.to_string(), value.clone());
                if save_lazy_scope {
                    self.set_value_to_lazy_scope(&current_pkgpath, name, &value)
                }
                existed = true;
            }
        }
        if !existed {
            if let Some(last) = scopes.last_mut() {
                let variables = &mut last.variables;
                if !variables.contains_key(name) {
                    variables.insert(name.to_string(), value.clone());
                    if save_lazy_scope {
                        self.set_value_to_lazy_scope(&current_pkgpath, name, &value)
                    }
                }
            }
        }
    }

    /// Get the variable value named `name` from the scope, return Err when not found
    pub fn get_variable(&self, name: &str) -> ValueRef {
        let current_pkgpath = self.current_pkgpath();
        self.get_variable_in_pkgpath(name, &current_pkgpath)
    }

    /// Get the variable value named `name` from the scope, return Err when not found
    pub fn get_variable_in_schema_or_rule(&self, name: &str) -> ValueRef {
        let pkgpath = self.current_pkgpath();
        if let Some(schema_ctx) = self.get_schema_eval_context() {
            return schema_ctx
                .borrow()
                .get_value(self, name, &pkgpath, &self.get_target_var());
        } else if let Some(rule_ctx) = self.get_rule_eval_context() {
            let rule_value: ValueRef = rule_ctx.borrow().value.clone();
            return if let Some(value) = rule_value.dict_get_value(name) {
                value
            } else {
                self.get_variable_in_pkgpath(name, &pkgpath)
            };
        } else {
            self.get_variable_in_pkgpath(name, &pkgpath)
        }
    }

    /// Get the variable value named `name` from the scope named `pkgpath`, return Err when not found
    pub fn get_variable_in_pkgpath(&self, name: &str, pkgpath: &str) -> ValueRef {
        let pkg_scopes = self.pkg_scopes.borrow();
        let pkgpath =
            if !pkgpath.starts_with(kclvm_runtime::PKG_PATH_PREFIX) && pkgpath != MAIN_PKG_PATH {
                format!("{}{}", kclvm_runtime::PKG_PATH_PREFIX, pkgpath)
            } else {
                pkgpath.to_string()
            };
        let mut result = self.undefined_value();
        // System module
        if builtin::STANDARD_SYSTEM_MODULE_NAMES_WITH_AT.contains(&pkgpath.as_str()) {
            let pkgpath = &pkgpath[1..];

            if pkgpath == builtin::system_module::UNITS
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
                let func_name = format!(
                    "{}{}_{}",
                    builtin::KCL_SYSTEM_MODULE_MANGLE_PREFIX,
                    pkgpath,
                    name
                );
                let function_ptr = _kclvm_get_fn_ptr_by_name(&func_name);
                self.function_value_with_ptr(function_ptr)
            }
        }
        // Plugin pkgpath
        else if pkgpath.starts_with(plugin::PLUGIN_PREFIX_WITH_AT) {
            // Strip the @kcl_plugin to kcl_plugin.
            let name = format!("{}.{}", &pkgpath[1..], name);
            ValueRef::func(0, 0, self.undefined_value(), &name, "", true)
        // User pkgpath
        } else {
            // Global or local variables.
            let scopes = pkg_scopes
                .get(&pkgpath)
                .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
            // Scopes 0 is builtin scope, Scopes 1 is the global scope, Scopes 2~ are the local scopes
            let scopes_len = scopes.len();
            let mut found = false;
            for i in 0..scopes_len {
                let index = scopes_len - i - 1;
                let variables = &scopes[index].variables;
                if let Some(var) = variables.get(name) {
                    // Closure vars, 2 denotes the builtin scope and the global scope, here is a closure scope.
                    result = if let Some(lambda_ctx) = self.last_lambda_ctx() {
                        let last_lambda_scope = lambda_ctx.level;
                        // Local scope variable or lambda closure variable.
                        let ignore = if let Some((start, end)) = self.scope_covers.borrow().last() {
                            *start <= index && index <= *end
                        } else {
                            false
                        };
                        if index >= last_lambda_scope && !ignore {
                            var.clone()
                        } else {
                            lambda_ctx.closure.get(name).unwrap_or(var).clone()
                        }
                    } else {
                        // Not in the lambda, maybe a local variable.
                        var.clone()
                    };
                    found = true;
                    break;
                }
            }
            if found {
                result
            } else {
                // Not found variable in the scope, maybe lambda closures captured in other package scopes.
                self.last_lambda_ctx()
                    .map(|ctx| ctx.closure.get(name).cloned().unwrap_or(result.clone()))
                    .unwrap_or(result)
            }
        }
    }

    /// Get closure map in the current inner scope.
    pub(crate) fn get_current_closure_map(&self) -> ClosureMap {
        // Get variable map in the current scope.
        let pkgpath = self.current_pkgpath();
        let pkg_scopes = self.pkg_scopes.borrow();
        let scopes = pkg_scopes
            .get(&pkgpath)
            .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
        let last_lambda_ctx = self.last_lambda_ctx();
        // Get current closure map.
        let mut closure_map = last_lambda_ctx
            .as_ref()
            .map(|ctx| ctx.closure.clone())
            .unwrap_or_default();
        // Get variable map including schema  in the current scope.
        for i in INNER_LEVEL..scopes.len() {
            let variables = &scopes
                .get(i)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .variables;
            for (k, v) in variables {
                closure_map.insert(k.to_string(), v.clone());
            }
        }
        closure_map
    }

    /// Load value from name.
    pub fn load_value(&self, pkgpath: &str, names: &[&str]) -> ValueRef {
        if names.is_empty() {
            return self.undefined_value();
        }
        let name = names[0];
        // Get variable from the scope.
        let get = |name: &str| {
            match (
                self.is_in_schema(),
                self.is_in_lambda(),
                self.is_local_var(name),
            ) {
                // Get variable from the global lazy scope.
                (false, false, false) => {
                    let variable = self.get_variable(name);
                    match self.resolve_variable_level(name) {
                        // Closure variable or local variables
                        Some(level) if level <= GLOBAL_LEVEL => self.get_value_from_lazy_scope(
                            &self.current_pkgpath(),
                            name,
                            &self.get_target_var(),
                            variable,
                        ),
                        // Schema closure or global variables
                        _ => variable,
                    }
                }
                // Get variable from the local or global scope.
                (false, _, _) | (_, _, true) => self.get_variable(name),
                // Get variable from the current schema scope.
                (true, false, false) => self.get_variable_in_schema_or_rule(name),
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
                        _ => self.get_variable_in_schema_or_rule(name),
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
                self.undefined_value()
            };
            for i in 0..names.len() - 1 {
                let attr = names[i + 1];
                if i == 0 && !pkgpath.is_empty() {
                    value = self.get_variable_in_pkgpath(attr, pkgpath);
                } else {
                    value = value.load_attr(attr)
                }
            }
            value
        }
    }
}
