#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::collections::*;

type Num = u16;

fn disjoint_sets(input: &str) -> Vec<HashSet<Num>> {
    let mut graph = aoc::DisjointSetBuilder::new();
    for line in input.lines() {
        let words = line.split_ascii_whitespace().collect::<Vec<_>>();
        let left = words[0].parse::<Num>().unwrap();
        graph.add_vertex(&left);
    }

    for line in input.lines() {
        let words = line.split_ascii_whitespace().collect::<Vec<_>>();
        let left = words[0].parse::<Num>().unwrap();

        let right = words[2..].iter().map(|w| {
            w.trim_end_matches(",").parse::<Num>().unwrap()
        });
        for val in right {
            graph.add_edge(&left, &val);
        }
    }

    graph.connected_components()
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
    dbg!(answ1);
    assert_eq!(answ1, 175);
    let answ2 = part2(data);
    dbg!(answ2);
    assert_eq!(answ2, 213);

    println!("Ok.");

    Ok(())
}
