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

fn part1(input: &str) -> i64 {
    0
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 9999)?;
    let data = puzzle.get_data()?;
    let lines = data.lines().collect::<Vec<_>>();
    for line in lines.iter() {
    }

    let answ1 = part1(data);
    dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    //let answ2 = part2(data);
    //dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
