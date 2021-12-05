#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::collections::*;

type Num = u16;

fn find(ds: &mut HashMap<Num, Num>, v: Num) -> Num {
    let x = *ds.get(&v).unwrap();
    if x != v {
        let x = find(ds, x);
        ds.insert(v, x);
    }
    *ds.get(&v).unwrap()
}

fn union(ds: &mut HashMap<Num, Num>, v1: Num, v2: Num) {
    let r1 = find(ds, v1);
    let r2 = find(ds, v2);
    if r1 != r2 {
        ds.insert(r1, r2);
    }
}

fn disjoint_sets(input: &str) -> Vec<HashSet<Num>> {
    let mut ds = HashMap::new();
    let mut vertices = Vec::new();

    for line in input.lines() {
        let words = line.split_ascii_whitespace().collect::<Vec<_>>();
        let left = words[0].parse::<Num>().unwrap();
        ds.insert(left, left);
        vertices.push(left);
    }

    for line in input.lines() {
        let words = line.split_ascii_whitespace().collect::<Vec<_>>();
        let left = words[0].parse::<Num>().unwrap();

        let right = words[2..].iter().map(|w| {
            w.trim_end_matches(",").parse::<Num>().unwrap()
        });
        for val in right {
            union(&mut ds, left, val);
        }
    }

    let mut components = HashMap::new();
    for v in vertices {
        let r = find(&mut ds, v);
        let rootset = components.entry(r).or_insert_with(|| HashSet::new());
        rootset.insert(v);
    }

    components.into_values().collect()
}

fn part1(graph: &str) -> usize {
    let connected_comps = disjoint_sets(graph);
    for comp in connected_comps.iter() {
        if comp.contains(&0) {
            return comp.len();
        }
    }
    unreachable!();
}

fn part2(graph: &str) -> usize {
    disjoint_sets(graph).len()
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2017, 12)?;
    let data = puzzle.get_data()?;

    let answ1 = part1(data);
    assert_eq!(answ1, 175);
    let answ2 = part2(data);
    assert_eq!(answ2, 213);

    Ok(())
}
