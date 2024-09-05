use indexmap::IndexMap;
use petgraph::visit::EdgeRef;

use crate::File;

/// A graph of files, where each file depends on zero or more other files.
#[derive(Default)]
pub struct FileGraph {
    graph: petgraph::stable_graph::StableDiGraph<File, ()>,
    path_to_node_index: IndexMap<File, petgraph::graph::NodeIndex>,
}

impl FileGraph {
    /// Sets a file to depend on the given other files.
    ///
    /// For example, if the current graph has file A depending on B, and
    /// `update_file(pathA, &[pathC])` was called, then this function will remove the edge
    /// from A to B, and add an edge from A to C.
    pub fn update_file<'a, I: IntoIterator<Item = &'a File>>(
        &mut self,
        from_path: &File,
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
    pub fn contains_file(&self, file: &File) -> bool {
        self.path_to_node_index.contains_key(file)
    }

    /// Returns a list of the direct dependencies of the given file.
    /// (does not include all transitive dependencies)
    /// The file path must be relative to the root of the file graph.
    pub fn dependencies_of(&self, file: &File) -> Vec<&File> {
        let node_index = self
            .path_to_node_index
            .get(file)
            .expect("file not in graph");
        self.graph
            .edges(*node_index)
            .map(|edge| &self.graph[edge.target()])
            .collect::<Vec<_>>()
    }

    /// Returns a list of files in the order they should be compiled
    /// Or a list of files that are part of a cycle, if one exists
    pub fn toposort(&self) -> Result<Vec<File>, Vec<File>> {
        match petgraph::algo::toposort(&self.graph, None) {
            Ok(indices) => Ok(indices
                .into_iter()
                .rev()
                .map(|n| self.graph[n].clone())
                .collect::<Vec<_>>()),
            Err(err) => {
                // toposort function in the `petgraph` library doesn't return the cycle itself,
                // so we need to use Tarjan's algorithm to find one instead
                let strongly_connected_components = petgraph::algo::tarjan_scc(&self.graph);

                // a strongly connected component is a cycle if it has more than one node
                // let's just return the first one we find
                let cycle = match strongly_connected_components
                    .into_iter()
                    .find(|component| component.len() > 1)
                {
                    Some(vars) => vars,
                    None => vec![err.node_id()],
                };
                Err(cycle
                    .iter()
                    .map(|n| self.graph[*n].clone())
                    .collect::<Vec<_>>())
            }
        }
    }

    /// Returns all paths.
    #[inline]
    pub fn paths(&self) -> Vec<File> {
        self.path_to_node_index.keys().cloned().collect::<Vec<_>>()
    }

    fn get_or_insert_node_index(&mut self, file: &File) -> petgraph::graph::NodeIndex {
        if let Some(node_index) = self.path_to_node_index.get(file) {
            return *node_index;
        }

        let node_index = self.graph.add_node(file.to_owned());
        self.path_to_node_index.insert(file.to_owned(), node_index);
        node_index
    }
}
