#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::collections::*;
use std::hash::Hash;

/// Unidirectional graph representation for finding disjoint sets / connected components.
pub struct DisjointSetBuilder<T> {
    state: HashMap<T, T>,
}

impl<T: Clone + Eq + Hash> DisjointSetBuilder<T> {
    pub fn new() -> Self {
        Self { state: HashMap::new(), }
    }

    /// Add a vertex to the Forest
    #[inline]
    pub fn add_vertex(&mut self, v: &T) {
        self.state.insert(v.clone(), v.clone());
    }

    fn find(&mut self, v: &T) -> T {
        let x = self.state.get(v).unwrap();
        if *x != *v {
            let x = x.clone();
            let x = self.find(&x);
            self.state.insert(v.clone(), x);
        }
        self.state.get(v).unwrap().clone()
    }

    /// Add a unidirectional edge between two vertices in the Forest.
    ///
    /// Both must already be present (see [`add_vertex`]).
    #[inline]
    pub fn add_edge(&mut self, v1: &T, v2: &T) {
        let r1 = self.find(v1);
        let r2 = self.find(v2);
        if r1 != r2 {
            self.state.insert(r1, r2);
        }
    }

    /// Compute connected components / disjoint sets.
    ///
    /// Each graph node will be present in exactly one of the [`HashSet`]s.
    pub fn connected_components(&mut self) -> Vec<HashSet<T>> {
        let mut components = HashMap::new();
        for v in self.state.keys().cloned().collect::<Vec<_>>() {
            let r = self.find(&v);
            let rootset = components.entry(r).or_insert_with(|| HashSet::new());
            rootset.insert(v);
        }

        components.into_values().collect()
    }
}

impl DisjointSetBuilder<VertexId> {
    /// Compute the disjoint sets in the provided graph.
    ///
    /// Vertices are identified by VertexId, because graphlib only provides one-directional lookup
    /// and that option gives more flexibility.
    ///
    /// These are "weakly" connected components -- we treat any edge as connecting two sub-graphs.
    pub fn from_graph<T>(graph: &Graph<T>) -> Self {
        let mut res = Self::new();
        for v in graph.vertices() {
            res.add_vertex(v);
        }
        // N.B., graphlib considers these directional, while for this algorithm, we treat them as
        // unidirectional.
        for e in graph.edges() {
            let (v1, v2) = e;
            res.add_edge(v1, v2);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphlib_disjoint_set_example() {
        let mut graph = Graph::<usize>::new();
        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        let v3 = graph.add_vertex(3);
        graph.add_edge(&v1, &v2).unwrap();
        graph.add_edge(&v2, &v1).unwrap();

        // Forest is:
        // (1) <-> (2)     (3)


        let mut forest = DisjointSetBuilder::from_graph(&graph);
        let connected_values = forest.connected_components()
            .iter()
            .map(|hs| {
                hs.iter()
                    .map(|vid| graph.fetch(&vid).unwrap().clone())
                    .collect::<HashSet<_>>()
            })
            .collect::<Vec<_>>();

        assert_eq!(connected_values.len(), 2);
        let (s1, s2) = if connected_values[0].len() == 1 {
            (&connected_values[0], &connected_values[1])
        } else {
            (&connected_values[1], &connected_values[0])
        };
        assert_eq!(s1, &[3].iter().copied().collect::<HashSet<usize>>());
        assert_eq!(s2, &[1, 2].iter().copied().collect::<HashSet<usize>>());
    }
}
