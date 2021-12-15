#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use itertools::iproduct;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use aoc::{dbg2, byte, ByteString};

// Basic "best-first" graph search, find the optimal path score.
fn part1(input: &ParseResult) -> i64 {
    let grid = input;

    // Since we will always visit each element along some optimal path, there is no point ever
    // revisiting a node.
    let mut visited = HashSet::new();
    let start = (0, 0);
    let end = (grid.nrows() - 1, grid.ncols() - 1);
    visited.insert(start.clone());

    let mut queue = BinaryHeap::new();
    queue.push((0i64, start));

    // Ended up not being necessary, but this is how we know where the optimal route went.  It maps
    // a single point of coordinates to the previous coordinate in the path.
    let mut prev = HashMap::new();

    while queue.len() != 0 {
        let (sco, next) = queue.pop().unwrap();
        if next == end {
            //let mut x = end;
            //print!("{:?}", x);
            //while let Some(y) = prev.get(&x) {
            //    print!(" <- {:?}", y);
            //    x = *y;
            //}
            //println!("");

            // We use negative scores because Rust's BinaryHeap is a max-heap, but we want
            // minimum-score routes first.
            return -sco;
        }
        // Adjacent nodes in the grid by (dx, dy).
        for ndelta in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let (dr, dc) = ndelta;
            let rr = next.0 as i64 + dr;
            let cc = next.1 as i64 + dc;
            if rr < 0 || rr >= grid.nrows() as i64 {
                continue;
            }
            if cc < 0 || cc >= grid.ncols() as i64 {
                continue;
            }
            let (rr, cc) = (rr as usize, cc as usize);
            if visited.contains(&(rr, cc)) {
                continue;
            }

            // In-bounds coordinates and not yet visited?  Throw it in.
            visited.insert((rr, cc));
            prev.insert((rr, cc), next);
            queue.push((sco - grid[[rr, cc]] as i64, (rr, cc)));
        }
    }
    unreachable!();
}

// Part2 was just expanding the input and then throwing it at part 1.  Or at least, it was for me.
fn part2(grid: &Array2<u8>) -> i64 {
    let nrows = grid.nrows();
    let ncols = grid.ncols();

    let new_nrows = nrows * 5;
    let new_ncols = ncols * 5;
    let mut matrix = Array::zeros((new_nrows, new_ncols));

    for rset in 0..5 {
        for cset in 0..5 {
            for r in 0..nrows {
                for c in 0..ncols {
                    let mut val = grid[[r, c]];
                    val += (rset + cset) as u8;
                    if val > 9 {
                        val -= 9;
                    }
                    matrix[[r + rset * nrows, c + cset * ncols]] = val;
                }
            }
        }
    }
    part1(&matrix)
}

type ParseResult = Array2<u8>;

fn parse(data: &str) -> ParseResult {
    let lines = data.lines().collect::<Vec<_>>();
    let grid = lines.iter().map(|line| {
        line.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    let nrows = grid.len();
    let ncols = grid[0].len();
    let mut matrix = Array::zeros((nrows, ncols));
    for (r, line) in grid.iter().enumerate() {
        for (c, val) in line.iter().enumerate() {
            matrix[[r, c]] = *val;
        }
    }
    matrix
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 15)?;
    let data = puzzle.get_data()?;
//    let data =
//"1163751742
//1381373672
//2136511328
//3694931569
//7463417111
//1319128137
//1359912421
//3125421639
//1293138521
//2311944581";
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 447);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 2825);
    Ok(())
}
