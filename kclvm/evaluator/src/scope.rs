use crate::{error as kcl_error, schema::SchemaEvalContextRef};
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast;
use kclvm_error::Position;
use kclvm_runtime::{ValueRef, _kclvm_get_fn_ptr_by_name, MAIN_PKG_PATH};
use kclvm_sema::{builtin, plugin};

use crate::{EvalResult, Evaluator, GLOBAL_LEVEL};

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
    /// Schema self denotes the scope that is belonged to a schema.
    pub schema_ctx: Option<SchemaEvalContextRef>,
}

impl<'ctx> Evaluator<'ctx> {
    /// Init a scope named `pkgpath` with all builtin functions
    pub(crate) fn init_scope(&self, pkgpath: &str) {
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
        self.enter_scope();
    }

    /// Get the scope level
    pub(crate) fn scope_level(&self) -> usize {
        let current_pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get(&current_pkgpath).expect(&msg);
        // Sub the builtin global scope
        scopes.len() - 1
    }

    /// Enter scope
    pub(crate) fn enter_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let scope = Scope::default();
        scopes.push(scope);
    }

    /// Leave scope
    pub(crate) fn leave_scope(&self) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        scopes.pop();
    }

    /// Enter scope with the schema eval context.
    pub(crate) fn enter_scope_with_schema_eval_context(&self, schema_ctx: &SchemaEvalContextRef) {
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
        let msg = format!("pkgpath {} is not found", current_pkgpath);
        let scopes = pkg_scopes.get_mut(&current_pkgpath).expect(&msg);
        let scope = Scope {
            schema_ctx: Some(schema_ctx.clone()),
            ..Default::default()
        };
        scopes.push(scope);
    }

    pub(crate) fn get_schema_eval_context(&self) -> Option<SchemaEvalContextRef> {
        let pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        // Global or local variables.
        let scopes = pkg_scopes
            .get(&pkgpath)
            .unwrap_or_else(|| panic!("package {} is not found", pkgpath));
        // Scopes 0 is builtin scope, Scopes 1 is the global scope, Scopes 2~ are the local scopes
        let scopes_len = scopes.len();
        for i in 0..scopes_len {
            let index = scopes_len - i - 1;
            if let Some(ctx) = &scopes[index].schema_ctx {
                return Some(ctx.clone());
            }
        }
        None
    }

    #[inline]
    pub(crate) fn get_schema_and_config(&self) -> Option<(ValueRef, ValueRef)> {
        self.get_schema_eval_context()
            .map(|v| (v.borrow().value.clone(), v.borrow().config.clone()))
    }

    #[inline]
    pub(crate) fn get_schema_config_meta(&self) -> Option<ValueRef> {
        self.get_schema_eval_context()
            .map(|v| v.borrow().config_meta.clone())
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
            variables.insert(name.to_string(), pointer);
        }
    }

    /// Store the argument named `name` in the current scope.
    pub(crate) fn store_argument_in_current_scope(&self, name: &str) {
        // Find argument name in the scope
        let current_pkgpath = self.current_pkgpath();
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
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
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
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
        let mut ctx = self.ctx.borrow_mut();
        let pkg_scopes = &mut ctx.pkg_scopes;
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
        let is_local_var = self.is_local_var(name);
        let mut ctx = self.ctx.borrow_mut();
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
        let scopes = ctx
            .pkg_scopes
            .get(&pkgpath)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        // Query the schema self value in all scopes.
        for i in 0..scopes.len() {
            let index = scopes.len() - i - 1;
            if let Some(schema_ctx) = &scopes[index].schema_ctx {
                let schema_value: ValueRef = schema_ctx.borrow().value.clone();
                return if let Some(value) = schema_value.dict_get_value(name) {
                    Ok(value)
                } else {
                    self.get_variable_in_pkgpath(name, &pkgpath)
                };
            }
        }
        self.get_variable_in_pkgpath(name, &pkgpath)
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
                let func_name = format!(
                    "{}{}_{}",
                    builtin::KCL_SYSTEM_MODULE_MANGLE_PREFIX,
                    pkgpath,
                    name
                );
                let function_ptr = _kclvm_get_fn_ptr_by_name(&func_name);
                self.function_value_with_ptr(function_ptr)
            };
            Ok(value)
        }
        // Plugin pkgpath
        else if pkgpath.starts_with(plugin::PLUGIN_PREFIX_WITH_AT) {
            // Strip the @kcl_plugin to kcl_plugin.
            let name = format!("{}.{}", &pkgpath[1..], name);
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
                    result = Ok(var.clone());
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
}
