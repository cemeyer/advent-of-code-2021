#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::cmp::{min, max};
use std::convert::TryFrom;
use std::collections::*;
use std::hash::Hash;

mod day10;

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2017, 14)?;
    let data = puzzle.get_data()?.trim_end();

    let (vertices, mut graph) = aoc::grid_2d_graph(128, 128);

    let mut count = 0;
    for i in 0..128 {
        let rowhash = day10::knothash(format!("{}-{}", data, i).as_bytes());
        for (idx, nibble) in rowhash.chars().enumerate() {
            let nibble = nibble.to_digit(16).unwrap();
            for bit in (0..4).rev() {
                // Block present
                if (nibble & (1 << bit)) != 0 {
                    count += 1;
                } else {
                    let vid = &vertices[i][idx*4 + (3 - bit)];
                    graph.remove(vid);
                }
            }

        }
    }

    // Part 1
    dbg!(count);
    assert_eq!(count, 8214);

    // Part 2
    let mut forest = aoc::DisjointSetBuilder::from_graph(&graph);
    let num_regions = forest.connected_components().len();
    dbg!(num_regions);
    assert_eq!(num_regions, 1093);

    println!("Ok");
    Ok(())
}
