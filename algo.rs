#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::fmt::Debug;
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

/// Construct a 2d grid graph of `m` rows by `n` columns.
///
/// Returns a mapping from row, column to `VertexId`s, in addition to the constructed graph.
///
/// Really graphlib should internalize this.
pub fn grid_2d_graph(m: usize, n: usize) -> (Vec<Vec<VertexId>>, Graph<(usize, usize)>) {
    let mut key = Vec::new();
    let mut graph = Graph::new();

    for y in 0..m {
        let mut key_row = Vec::new();
        for x in 0..n {
            let v = graph.add_vertex((y, x));
            key_row.push(v);
        }
        key.push(key_row);
    }

    // Drop mut.
    let key = key;

    for y in 0..m {
        for x in 0..n {
            // Nodes are connected to the node in the preceeding row of the same column (if any).
            if y > 0 {
                graph.add_edge(&key[y-1][x], &key[y][x]).unwrap();
                graph.add_edge(&key[y][x], &key[y-1][x]).unwrap();
            }
            // Nodes are connected to the node in the preceeding column of the same row (if any).
            if x > 0 {
                graph.add_edge(&key[y][x-1], &key[y][x]).unwrap();
                graph.add_edge(&key[y][x], &key[y][x-1]).unwrap();
            }
        }
    }

    (key, graph)
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn graph2d_components() {
        // (0, 0) <-> (0, 1)
        //   |          |
        // (1, 0) <-> (1, 1)
        let (vertices, graph) = grid_2d_graph(2, 2);
        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].len(), 2);

        let mut forest = DisjointSetBuilder::from_graph(&graph);
        let connected_values = forest.connected_components();

        assert_eq!(connected_values.len(), 1);
        assert_eq!(connected_values[0].len(), 4);
    }
}

/// Convert an iterator of values to a histogram of those value frequencies.
///
/// Generic in type of values and type of counter.
///
/// BTreeMaps form hashable histograms, so these can be used as keys in, e.g., hashsets of explored
/// search space.
pub fn histo<'a, T, C, I>(vals: I) -> BTreeMap<T, C>
where
    T: 'a + Copy + Ord,
    C: std::ops::AddAssign + TryFrom<usize> + Copy,
    I: Iterator<Item = &'a T>,
{
    let mut res: BTreeMap<T, C> = BTreeMap::new();
    let zero = C::try_from(0).ok().unwrap();
    let one = C::try_from(1).ok().unwrap();
    for v in vals {
        *res.entry(*v).or_insert(zero) += one;
    }
    res
}

/// Update a histogram by adding `n` elements `elm`.
#[inline]
pub fn histo_count_n<T, C>(histo: &mut BTreeMap<T, C>, elm: T, n: C)
where
    T: Copy + Ord,
    C: std::ops::AddAssign + TryFrom<usize> + Copy,
{
    *histo.entry(elm).or_insert(C::try_from(0).ok().unwrap()) += n;
}

/// Expand histogram to an ordered list.
pub fn histo_explode<T, C>(histo: &BTreeMap<T, C>) -> impl Iterator<Item = &T> + Debug
where
    T: Copy + Ord + Debug,
    C: std::ops::AddAssign + TryFrom<usize> + Copy + Debug + TryInto<usize>,
{
    histo.iter()
        .map(|(elm, count)| {
            std::iter::repeat(elm).take((*count).try_into().ok().unwrap())
        })
        .flatten()
}

#[cfg(test)]
mod tests3 {
    use super::*;

    #[test]
    fn histo_basic() {
        let mut myhisto: BTreeMap<_, i64> = histo([1,2,3,4,1,1].iter());
        assert_eq!(histo_explode(&myhisto).copied().collect::<Vec<_>>(), [1,1,1,2,3,4]);
        println!("{:?}", &myhisto);

        assert_eq!(myhisto[&1], 3);
        assert_eq!(myhisto[&2], 1);
        assert_eq!(myhisto[&3], 1);
        assert_eq!(myhisto[&4], 1);

        histo_count_n(&mut myhisto, 4, 3);
        assert_eq!(myhisto[&4], 4);
        assert_eq!(histo_explode(&myhisto).copied().collect::<Vec<_>>(), [1,1,1,2,3,4,4,4,4]);

        histo_count_n(&mut myhisto, 4, -3);
        assert_eq!(myhisto[&4], 1);
        assert_eq!(histo_explode(&myhisto).copied().collect::<Vec<_>>(), [1,1,1,2,3,4]);
    }
}
