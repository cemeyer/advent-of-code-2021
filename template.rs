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

fn part1(input: &Vec<Vec<u8>>) -> i64 {
    0
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 9999)?;
    let data = puzzle.get_data()?;
    let lines = data.lines().collect::<Vec<_>>();
    //let grid = lines.iter().map(|line| {
    //    line.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
    //}).collect::<Vec<_>>();
    //let nrows = grid.len();
    //let ncols = grid[0].len();
    //let mut matrix = Array::zeros((nrows, ncols));
    //for (r, line) in grid.iter().enumerate() {
    //    for (c, val) in line.iter().enumerate() {
    //        matrix[[r, c]] = *val;
    //    }
    //}

    let answ1 = part1(&grid);
    dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    //let answ2 = part2(&grid);
    //dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
