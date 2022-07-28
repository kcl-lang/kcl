use crate::plugin::PLUGIN_MODULE_PREFIX;
use crate::resolver::Resolver;
use crate::ty::ModuleKind;
use crate::{
    builtin::system_module::STANDARD_SYSTEM_MODULES,
    ty::{Type, TypeKind},
};
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_error::*;
use std::{cell::RefCell, path::Path, rc::Rc};

use super::scope::{Scope, ScopeKind, ScopeObject, ScopeObjectKind};
use crate::resolver::pos::GetPos;

impl<'ctx> Resolver<'ctx> {
    /// Check import error
    pub fn resolve_import(&mut self) {
        let main_files = self.program.get_main_files();
        for modules in self.program.pkgs.values() {
            for m in modules {
                for stmt in &m.body {
                    if let ast::Stmt::Import(import_stmt) = &stmt.node {
                        let pkgpath = &import_stmt.path;
                        // System module.
                        if STANDARD_SYSTEM_MODULES.contains(&pkgpath.as_str()) {
                            continue;
                        }
                        // Plugin module.
                        if pkgpath.starts_with(PLUGIN_MODULE_PREFIX) {
                            continue;
                        }
                        let real_path =
                            Path::new(&self.program.root).join(pkgpath.replace('.', "/"));
                        if !self.program.pkgs.contains_key(pkgpath) {
                            self.handler.add_error(
                                ErrorKind::CannotFindModule,
                                &[Message {
                                    pos: Position {
                                        filename: m.filename.clone(),
                                        line: stmt.line,
                                        column: None,
                                    },
                                    style: Style::Line,
                                    message: format!(
                                        "Cannot find the module {} from {}",
                                        import_stmt.rawpath,
                                        real_path.to_str().unwrap()
                                    ),
                                    note: None,
                                }],
                            );
                        } else {
                            let file = real_path.to_str().unwrap().to_string();
                            if real_path.is_file() && main_files.contains(&file) {
                                self.handler.add_error(
                                    ErrorKind::CompileError,
                                    &[Message {
                                        pos: Position {
                                            filename: self.ctx.filename.clone(),
                                            line: stmt.line,
                                            column: None,
                                        },
                                        style: Style::Line,
                                        message: format!(
                                            "Cannot import {} in the main package",
                                            file
                                        ),
                                        note: None,
                                    }],
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// The import check function.
    pub(crate) fn check_import(&mut self, pkgpath: &str) {
        self.ctx.pkgpath = pkgpath.to_string();
        let filename = self.ctx.filename.clone();
        self.change_package_context(pkgpath, &filename);
        self.init_import_list();
    }

    /// Init import list and store the module scope object into the scope map.
    fn init_import_list(&mut self) {
        let modules = self.program.pkgs.get(&self.ctx.pkgpath);
        match modules {
            Some(modules) => {
                for module in modules {
                    self.ctx.filename = module.filename.clone();
                    self.ctx.pkgpath = module.pkg.clone();
                    for stmt in &module.body {
                        if let ast::Stmt::Import(import_stmt) = &stmt.node {
                            {
                                match self.ctx.import_names.get_mut(&self.ctx.filename) {
                                    Some(mapping) => {
                                        mapping.insert(
                                            import_stmt.name.to_string(),
                                            import_stmt.path.to_string(),
                                        );
                                    }
                                    None => {
                                        let mut mapping = IndexMap::default();
                                        mapping.insert(
                                            import_stmt.name.to_string(),
                                            import_stmt.path.to_string(),
                                        );
                                        self.ctx
                                            .import_names
                                            .insert(self.ctx.filename.clone(), mapping);
                                    }
                                }
                                let mut scope = self.scope.borrow_mut();
                                let is_user_module = match scope.elems.get(&import_stmt.path) {
                                    Some(scope_obj) => {
                                        let mut obj = scope_obj.borrow_mut();
                                        match &obj.ty.kind {
                                            TypeKind::Module(module_ty) => {
                                                let mut module_ty = module_ty.clone();
                                                module_ty
                                                    .imported
                                                    .push(self.ctx.filename.to_string());
                                                obj.ty = Rc::new(Type::module(
                                                    &module_ty.pkgpath,
                                                    &module_ty.imported,
                                                    module_ty.kind.clone(),
                                                ));
                                                matches!(module_ty.kind, ModuleKind::User)
                                            }
                                            _ => bug!(
                                            "invalid module type in the import check function {}",
                                            scope_obj.borrow().ty.ty_str()
                                        ),
                                        }
                                    }
                                    None => {
                                        let kind =
                                            if import_stmt.path.starts_with(PLUGIN_MODULE_PREFIX) {
                                                ModuleKind::Plugin
                                            } else if STANDARD_SYSTEM_MODULES
                                                .contains(&import_stmt.path.as_str())
                                            {
                                                ModuleKind::System
                                            } else {
                                                ModuleKind::User
                                            };
                                        let ty = Type::module(
                                            &import_stmt.path,
                                            &[self.ctx.filename.clone()],
                                            kind.clone(),
                                        );
                                        let (start, end) = stmt.get_span_pos();
                                        scope.elems.insert(
                                            import_stmt.path.to_string(),
                                            Rc::new(RefCell::new(ScopeObject {
                                                name: import_stmt.path.to_string(),
                                                start,
                                                end,
                                                ty: Rc::new(ty),
                                                kind: ScopeObjectKind::Module,
                                                used: false,
                                            })),
                                        );
                                        matches!(kind, ModuleKind::User)
                                    }
                                };
                                if !is_user_module {
                                    continue;
                                }
                            }
                            let current_pkgpath = self.ctx.pkgpath.clone();
                            let current_filename = self.ctx.filename.clone();
                            self.ctx
                                .ty_ctx
                                .add_dependencies(&self.ctx.pkgpath, &import_stmt.path);
                            if self.ctx.ty_ctx.is_cyclic() {
                                self.handler.add_compile_error(
                                    &format!(
                                        "There is a circular import reference between module {} and {}",
                                        self.ctx.pkgpath, import_stmt.path,
                                    ),
                                    stmt.get_pos(),
                                );
                            }
                            // Switch pkgpath context
                            if !self.scope_map.contains_key(&import_stmt.path) {
                                self.check(&import_stmt.path);
                            }
                            // Restore the current context
                            self.change_package_context(&current_pkgpath, &current_filename);
                        }
                    }
                }
            }
            None => {}
        }
    }

    pub(crate) fn change_package_context(&mut self, pkgpath: &str, filename: &str) {
        if pkgpath.is_empty() {
            return;
        }
        if !self.scope_map.contains_key(pkgpath) {
            let scope = Rc::new(RefCell::new(Scope {
                parent: Some(Rc::downgrade(&self.builtin_scope)),
                children: vec![],
                elems: IndexMap::default(),
                start: Position::dummy_pos(),
                end: Position::dummy_pos(),
                kind: ScopeKind::Package,
            }));
            self.scope_map
                .insert(pkgpath.to_string(), Rc::clone(&scope));
            self.scope = scope;
        }
        self.ctx.pkgpath = pkgpath.to_string();
        self.ctx.filename = filename.to_string();
        self.scope = self.scope_map.get(pkgpath).unwrap().clone();
    }
}
