#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::cmp::{min, max};
use std::convert::TryFrom;
use std::collections::*;
use std::hash::Hash;

fn part1(input: &str) -> i64 {
    1
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 9999)?;
    let data = puzzle.get_data()?;

    let answ1 = part1(data);
    dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    //let answ2 = part2(data);
    //dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
