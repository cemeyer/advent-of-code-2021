#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
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
