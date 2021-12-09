#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

fn part1(input: &Vec<Vec<u32>>) -> i64 {
    let nrows = input.len();
    let ncols = input[0].len();

    let (v, graph) = aoc::grid_2d_graph(nrows, ncols);

    let mut risktot = 0;
    for node in graph.vertices() {
        let (r, c) = graph.fetch(node).unwrap();
        let node_height = input[*r][*c];
        let local_min = graph.neighbors(node).all(|n| {
            let (nr, nc) = graph.fetch(n).unwrap();
            input[*nr][*nc] > node_height
        });
        if local_min {
            risktot += 1 + node_height as i64;
        }
    }
    risktot
}

fn part2(input: &Vec<Vec<u32>>) -> usize {
    let nrows = input.len();
    let ncols = input[0].len();

    let (v, mut graph) = aoc::grid_2d_graph(nrows, ncols);

    // Create "basins" (disconnected subgraphs)
    for (r, row) in input.iter().enumerate() {
        for (c, ht) in row.iter().enumerate() {
            if *ht == 9 {
                let vtx = &v[r][c];
                graph.remove(vtx);
            }
        }
    }

    let mut forest = aoc::DisjointSetBuilder::from_graph(&graph);
    let connected_values = forest.connected_components();

    let mut sizes = connected_values.iter().map(|basin| basin.len()).collect::<Vec<_>>();
    sizes.sort();

    let mut res = 1;
    for sz in sizes.iter().rev().take(3) {
        dbg!(sz);
        res *= sz;
    }

    res
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 9)?;
    let data = puzzle.get_data()?;
    let lines = data.lines().collect::<Vec<_>>();
    let grid = lines.iter().map(|line| {
        line.chars().map(|c| c.to_digit(10).unwrap()).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    //let answ1 = part1(&grid);
    //dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    let answ2 = part2(&grid);
    dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
