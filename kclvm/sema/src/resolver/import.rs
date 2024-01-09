use crate::plugin::PLUGIN_MODULE_PREFIX;
use crate::resolver::scope::Module;
use crate::resolver::Resolver;
use crate::ty::ModuleKind;
use crate::{
    builtin::system_module::STANDARD_SYSTEM_MODULES,
    ty::{Type, TypeKind},
};
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast;
use kclvm_error::*;
use std::rc::Rc;
use std::sync::Arc;
use std::{cell::RefCell, path::Path};

use super::scope::{Scope, ScopeKind, ScopeObject, ScopeObjectKind};
use kclvm_ast::pos::GetPos;
use kclvm_utils::pkgpath::parse_external_pkg_name;

impl<'ctx> Resolver<'ctx> {
    /// Check import error
    pub fn resolve_import(&mut self) {
        let main_files = self.program.get_main_files();
        for modules in self.program.pkgs.values() {
            for m in modules {
                for stmt in &m.body {
                    if let ast::Stmt::Import(import_stmt) = &stmt.node {
                        let pkgpath = &import_stmt.path.node;
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
                                    range: stmt.get_span_pos(),
                                    style: Style::Line,
                                    message: format!(
                                        "Cannot find the module {} from {}",
                                        import_stmt.rawpath,
                                        real_path.to_str().unwrap()
                                    ),
                                    note: None,
                                    suggested_replacement: None,
                                }],
                            );

                            let mut suggestions =
                                vec![format!("find more package on 'https://artifacthub.io'")];

                            if let Ok(pkg_name) = parse_external_pkg_name(pkgpath) {
                                suggestions.insert(
                                    0,
                                    format!(
                                        "try 'kcl mod add {}' to download the package not found",
                                        pkg_name
                                    ),
                                );
                            }
                            self.handler.add_suggestions(suggestions);
                        } else {
                            let file = real_path.to_str().unwrap().to_string();
                            if real_path.is_file() && main_files.contains(&file) {
                                self.handler.add_error(
                                    ErrorKind::CompileError,
                                    &[Message {
                                        range: stmt.get_span_pos(),
                                        style: Style::Line,
                                        message: format!(
                                            "Cannot import {} in the main package",
                                            file
                                        ),
                                        note: None,
                                        suggested_replacement: None,
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
                                        // 'import sub as s' and 'import sub.sub as s' will raise this error.
                                        // 'import sub' and 'import sub' will not raise this error.
                                        // 'import sub as s' and 'import sub as s' will not raise this error.
                                        if let Some(path) = mapping.get(&import_stmt.name) {
                                            if path != &import_stmt.path.node {
                                                self.handler.add_compile_error(
                                                    &format!(
                                                        "the name '{}' is defined multiple times, '{}' must be defined only once",
                                                        import_stmt.name, import_stmt.name
                                                    ),
                                                    stmt.get_span_pos(),
                                                );
                                            }
                                        }
                                        mapping.insert(
                                            import_stmt.name.to_string(),
                                            import_stmt.path.node.to_string(),
                                        );
                                    }
                                    None => {
                                        let mut mapping = IndexMap::default();
                                        mapping.insert(
                                            import_stmt.name.to_string(),
                                            import_stmt.path.node.to_string(),
                                        );
                                        self.ctx
                                            .import_names
                                            .insert(self.ctx.filename.clone(), mapping);
                                    }
                                }
                                let mut scope = self.scope.borrow_mut();
                                let is_user_module = match scope.elems.get(&import_stmt.path.node) {
                                    Some(scope_obj) => {
                                        let mut obj = scope_obj.borrow_mut();
                                        match &mut obj.kind {
                                                    ScopeObjectKind::Module(m) => {
                                                        m.import_stmts.push((stmt.clone(), false))
                                                    },
                                                    _ => bug!(
                                                        "invalid module type in the import check function {}",
                                                        scope_obj.borrow().ty.ty_str()
                                                    )
                                                }
                                        match &obj.ty.kind {
                                            TypeKind::Module(module_ty) => {
                                                let mut module_ty = module_ty.clone();
                                                module_ty
                                                    .imported
                                                    .push(self.ctx.filename.to_string());
                                                obj.ty = Arc::new(Type::module(
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
                                        let kind = if import_stmt
                                            .path
                                            .node
                                            .starts_with(PLUGIN_MODULE_PREFIX)
                                        {
                                            ModuleKind::Plugin
                                        } else if STANDARD_SYSTEM_MODULES
                                            .contains(&import_stmt.path.node.as_str())
                                        {
                                            ModuleKind::System
                                        } else {
                                            ModuleKind::User
                                        };
                                        let ty = Type::module(
                                            &import_stmt.path.node,
                                            &[self.ctx.filename.clone()],
                                            kind.clone(),
                                        );
                                        let (start, end) = stmt.get_span_pos();

                                        scope.elems.insert(
                                            import_stmt.path.node.to_string(),
                                            Rc::new(RefCell::new(ScopeObject {
                                                name: import_stmt.path.node.to_string(),
                                                start,
                                                end,
                                                ty: Arc::new(ty),
                                                kind: ScopeObjectKind::Module(Module {
                                                    import_stmts: vec![(stmt.clone(), false)],
                                                }),
                                                doc: None,
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
                                .add_dependencies(&self.ctx.pkgpath, &import_stmt.path.node);
                            if self.ctx.ty_ctx.is_cyclic() {
                                self.handler.add_compile_error(
                                    &format!(
                                        "There is a circular import reference between module {} and {}",
                                        self.ctx.pkgpath, import_stmt.path.node,
                                    ),
                                    stmt.get_span_pos(),
                                );
                            }
                            // Switch pkgpath context
                            if !self.scope_map.contains_key(&import_stmt.path.node) {
                                self.check(&import_stmt.path.node);
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
                kind: ScopeKind::Package(IndexSet::new()),
            }));
            self.scope_map
                .insert(pkgpath.to_string(), Rc::clone(&scope));
            self.scope = scope;
        }
        self.ctx.pkgpath = pkgpath.to_string();
        self.ctx.filename = filename.to_string();
        let scope = self.scope_map.get(pkgpath).unwrap().clone();
        self.scope = scope;
    }
}
