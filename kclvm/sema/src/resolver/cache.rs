use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use indexmap::IndexMap;

use crate::ty::TypeRef;

use super::scope::{ProgramScope, Scope};
use kclvm_ast::ast;

/// For CachedScope, we assume that all changed files must be located in kclvm_ast::MAIN_PKG ,
/// if this is not the case, please clear the cache directly
#[derive(Debug, Clone, Default)]
pub struct CachedScope {
    pub program_root: String,
    pub scope_map: IndexMap<String, Rc<RefCell<Scope>>>,
    pub node_ty_map: IndexMap<ast::AstIndex, TypeRef>,
    dependency_graph: DependencyGraph,
}
#[derive(Debug, Clone, Default)]

struct DependencyGraph {
    /// map filename to pkgpath
    pub module_map: HashMap<String, HashSet<String>>,
    /// map pkgpath to node
    pub node_map: HashMap<String, DependencyNode>,
}

impl DependencyGraph {
    pub fn clear(&mut self) {
        self.module_map.clear();
        self.node_map.clear();
    }

    pub fn update(&mut self, program: &ast::Program) -> Result<HashSet<String>, String> {
        let mut new_modules = HashMap::new();
        for (pkgpath, modules) in program.pkgs.iter() {
            if pkgpath == kclvm_ast::MAIN_PKG {
                continue;
            }
            if !self.node_map.contains_key(pkgpath) {
                self.node_map.insert(
                    pkgpath.to_string(),
                    DependencyNode {
                        pkgpath: pkgpath.to_string(),
                        parents: HashSet::new(),
                        children: HashSet::new(),
                    },
                );
            }
            for module in modules {
                if !self.module_map.contains_key(&module.filename) {
                    new_modules.insert(module.filename.to_string(), module);
                    self.module_map
                        .insert(module.filename.to_string(), HashSet::new());
                }
                self.module_map
                    .get_mut(&module.filename)
                    .unwrap()
                    .insert(pkgpath.to_string());
            }
        }

        for new_module in new_modules.values() {
            self.add_new_module(new_module);
        }
        let mut invalidated_set = HashSet::new();
        if let Some(main_modules) = program.pkgs.get(kclvm_ast::MAIN_PKG) {
            for module in main_modules {
                let result = self.invalidate_module(module)?;
                let _ = result.into_iter().map(|pkg| invalidated_set.insert(pkg));
                self.remove_dependency_from_pkg(&module.filename);
                self.add_new_module(module);
            }
        }
        Ok(invalidated_set)
    }

    fn add_new_module(&mut self, new_module: &ast::Module) {
        let module_file = new_module.filename.clone();
        if let Some(pkgpaths) = self.module_map.get(&module_file) {
            for stmt in &new_module.body {
                if let ast::Stmt::Import(import_stmt) = &stmt.node {
                    let parent_pkg = &import_stmt.path;
                    if let Some(parent_node) = self.node_map.get_mut(parent_pkg) {
                        parent_node.children.insert(new_module.filename.clone());
                    }
                    for pkgpath in pkgpaths {
                        let cur_node = self.node_map.get_mut(pkgpath).unwrap();
                        cur_node.parents.insert(parent_pkg.clone());
                    }
                }
            }
        }
    }

    fn invalidate_module(
        &mut self,
        changed_module: &ast::Module,
    ) -> Result<HashSet<String>, String> {
        let module_file = changed_module.filename.clone();
        let mut invalidated_set = HashSet::new();
        if let Some(pkgpaths) = self.module_map.get(&module_file).cloned() {
            let mut pkg_queue = VecDeque::new();
            for pkgpath in pkgpaths.iter() {
                invalidated_set.insert(pkgpath.clone());
                pkg_queue.push_back(self.node_map.get(pkgpath));
            }

            let mut old_size = 0;
            while old_size < invalidated_set.len() {
                old_size = invalidated_set.len();
                let cur_node = loop {
                    match pkg_queue.pop_front() {
                        Some(cur_node) => match cur_node {
                            None => continue,
                            Some(cur_node) => {
                                if invalidated_set.contains(&cur_node.pkgpath) {
                                    continue;
                                }
                                invalidated_set.insert(cur_node.pkgpath.clone());
                                break Some(cur_node);
                            }
                        },
                        None => break None,
                    }
                };
                if let Some(cur_node) = cur_node {
                    for child in cur_node.children.iter() {
                        if let Some(child_pkgs) = self.module_map.get(child) {
                            for child_pkg in child_pkgs {
                                if invalidated_set.contains(child_pkg) {
                                    continue;
                                }
                                pkg_queue.push_back(self.node_map.get(child_pkg));
                            }
                        }
                    }
                }
            }
        };
        Ok(invalidated_set)
    }

    fn remove_dependency_from_pkg(&mut self, filename: &str) {
        if let Some(pkgpaths) = self.module_map.get(filename).cloned() {
            for pkgpath in pkgpaths {
                if let Some(node) = self.node_map.get(&pkgpath).cloned() {
                    for parent in node.parents {
                        if let Some(parent_node) = self.node_map.get_mut(&parent) {
                            parent_node.children.remove(filename);
                        }
                    }
                }
            }
        }
    }
}
#[derive(Debug, Clone, Default)]
struct DependencyNode {
    pkgpath: String,
    //the pkgpath which is imported by this pkg
    parents: HashSet<String>,
    //the files which import this pkg
    children: HashSet<String>,
}

impl CachedScope {
    pub fn new(scope: &ProgramScope, program: &ast::Program) -> Self {
        let mut cached_scope = Self {
            program_root: program.root.to_string(),
            scope_map: scope.scope_map.clone(),
            node_ty_map: scope.node_ty_map.clone(),
            dependency_graph: DependencyGraph::default(),
        };
        let invalidated_pkgs = cached_scope.dependency_graph.update(program);
        cached_scope.invalidte_cache(invalidated_pkgs.as_ref());
        cached_scope
    }

    pub fn clear(&mut self) {
        self.scope_map.clear();
        self.node_ty_map.clear();
        self.dependency_graph.clear();
    }

    pub fn invalidte_cache(&mut self, invalidated_pkgs: Result<&HashSet<String>, &String>) {
        match invalidated_pkgs {
            Ok(invalidated_pkgs) => {
                for invalidated_pkg in invalidated_pkgs.iter() {
                    self.scope_map.remove(invalidated_pkg);
                }
            }
            Err(_) => self.clear(),
        }
    }

    pub fn update(&mut self, program: &ast::Program) {
        if self.program_root != program.root {
            self.clear();
            self.program_root = program.root.clone();
        }
        let invalidated_pkgs = self.dependency_graph.update(program);
        self.invalidte_cache(invalidated_pkgs.as_ref());
    }
}
