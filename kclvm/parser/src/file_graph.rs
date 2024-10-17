use std::{collections::HashMap, path::PathBuf};

use indexmap::IndexMap;
use kclvm_ast::ast::Module;
use petgraph::{prelude::StableDiGraph, visit::EdgeRef};
use std::hash::Hash;
/// File with package info
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PkgFile {
    pub path: PathBuf,
    pub pkg_path: String,
}

impl PkgFile {
    pub fn canonicalize(&self) -> PathBuf {
        match self.path.canonicalize() {
            Ok(p) => p.clone(),
            _ => self.path.clone(),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Pkg {
    pub pkg_name: String,
    pub pkg_root: String,
}

pub type PkgMap = HashMap<PkgFile, Pkg>;

/// A graph of files, where each file depends on zero or more other files.
#[derive(Default)]
pub struct PkgFileGraph {
    graph: StableDiGraph<PkgFile, ()>,
    path_to_node_index: IndexMap<PkgFile, petgraph::graph::NodeIndex>,
}

impl PkgFileGraph {
    /// Sets a file to depend on the given other files.
    ///
    /// For example, if the current graph has file A depending on B, and
    /// `update_file(pathA, &[pathC])` was called, then this function will remove the edge
    /// from A to B, and add an edge from A to C.
    pub fn update_file<'a, I: IntoIterator<Item = &'a PkgFile>>(
        &mut self,
        from_path: &PkgFile,
        to_paths: I,
    ) {
        let from_node_index = self.get_or_insert_node_index(from_path);

        // remove all current out coming edges from this node
        self.graph.retain_edges(|g, edge| {
            if let Some((source, _)) = g.edge_endpoints(edge) {
                if source == from_node_index {
                    return false;
                }
            }
            true
        });

        for to_path in to_paths {
            let to_node_index = self.get_or_insert_node_index(to_path);
            self.graph.add_edge(from_node_index, to_node_index, ());
        }
    }

    /// Returns true if the given file is in the graph
    pub fn contains_file(&self, file: &PkgFile) -> bool {
        contains_file(file, &self.path_to_node_index)
    }

    /// Returns a list of the direct dependencies of the given file.
    /// (does not include all transitive dependencies)
    /// The file path must be relative to the root of the file graph.
    pub fn dependencies_of(&self, file: &PkgFile) -> Vec<PkgFile> {
        dependencies_of(file, &self.graph, &self.path_to_node_index)
    }

    pub fn toposort(&self) -> Result<Vec<PkgFile>, Vec<PkgFile>> {
        toposort(&self.graph)
    }

    /// Returns all paths.
    #[inline]
    pub fn paths(&self) -> Vec<PkgFile> {
        self.path_to_node_index.keys().cloned().collect::<Vec<_>>()
    }

    fn get_or_insert_node_index(&mut self, file: &PkgFile) -> petgraph::graph::NodeIndex {
        if let Some(node_index) = self.path_to_node_index.get(file) {
            return *node_index;
        }

        let node_index = self.graph.add_node(file.to_owned());
        self.path_to_node_index.insert(file.to_owned(), node_index);
        node_index
    }

    pub fn file_path_graph(
        &self,
    ) -> (
        StableDiGraph<PathBuf, ()>,
        IndexMap<PathBuf, petgraph::prelude::NodeIndex>,
    ) {
        let mut graph = StableDiGraph::new();
        let mut node_map = IndexMap::new();
        for node in self.graph.node_indices() {
            let path = self.graph[node].path.clone();
            let idx = graph.add_node(path.clone());
            node_map.insert(path, idx);
        }
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                let source_path = self.graph[source].path.clone();
                let target_path = self.graph[target].path.clone();
                match (node_map.get(&source_path), node_map.get(&target_path)) {
                    (Some(source), Some(target)) => {
                        graph.add_edge(source.clone(), target.clone(), ());
                    }
                    _ => {}
                }
            }
        }
        (graph, node_map)
    }

    pub fn pkg_graph(
        &self,
        pkgs: &HashMap<String, Vec<Module>>,
    ) -> (
        StableDiGraph<String, ()>,
        IndexMap<String, petgraph::prelude::NodeIndex>,
    ) {
        let mut graph = StableDiGraph::new();
        let mut node_map = IndexMap::new();

        for pkg in pkgs.keys() {
            let idx = graph.add_node(pkg.clone());
            node_map.insert(pkg.clone(), idx);
        }

        for node in self.graph.node_indices() {
            let path = self.graph[node].pkg_path.clone();
            let idx = graph.add_node(path.clone());
            node_map.insert(path, idx);
        }
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                let source_path = self.graph[source].pkg_path.clone();
                let target_path = self.graph[target].pkg_path.clone();
                graph.add_edge(
                    node_map.get(&source_path).unwrap().clone(),
                    node_map.get(&target_path).unwrap().clone(),
                    (),
                );
            }
        }
        (graph, node_map)
    }
}

/// Returns a list of files in the order they should be compiled
/// Or a list of files that are part of a cycle, if one exists
pub fn toposort<T>(graph: &StableDiGraph<T, ()>) -> Result<Vec<T>, Vec<T>>
where
    T: Clone,
{
    match petgraph::algo::toposort(graph, None) {
        Ok(indices) => Ok(indices
            .into_iter()
            .rev()
            .map(|n| graph[n].clone())
            .collect::<Vec<_>>()),
        Err(err) => {
            // toposort function in the `petgraph` library doesn't return the cycle itself,
            // so we need to use Tarjan's algorithm to find one instead
            let strongly_connected_components = petgraph::algo::tarjan_scc(&graph);
            // a strongly connected component is a cycle if it has more than one node
            // let's just return the first one we find
            let cycle = match strongly_connected_components
                .into_iter()
                .find(|component| component.len() > 1)
            {
                Some(vars) => vars,
                None => vec![err.node_id()],
            };
            Err(cycle.iter().map(|n| graph[*n].clone()).collect::<Vec<_>>())
        }
    }
}

/// Returns a list of the direct dependencies of the given file.
/// (does not include all transitive dependencies)
/// The file path must be relative to the root of the file graph.
pub fn dependencies_of<T>(
    node: &T,
    graph: &StableDiGraph<T, ()>,
    id_map: &IndexMap<T, petgraph::prelude::NodeIndex>,
) -> Vec<T>
where
    T: Clone + Hash + Eq + PartialEq,
{
    let node_index = id_map.get(node).expect("node not in graph");
    graph
        .edges(*node_index)
        .map(|edge| &graph[edge.target()])
        .map(|node| node.clone())
        .collect::<Vec<_>>()
}

/// Returns true if the given file is in the graph
pub fn contains_file<T>(node: &T, id_map: &IndexMap<T, petgraph::prelude::NodeIndex>) -> bool
where
    T: Clone + Hash + Eq + PartialEq,
{
    id_map.contains_key(node)
}
