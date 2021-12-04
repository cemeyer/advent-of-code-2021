#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::collections::*;

fn part1(input: &str) -> String {
    format!("{}", 1)
}

fn part2(input: &str) -> String {
    format!("{}", 2)
}

fn submit(puzzle: &mut aoc::Puzzle, part: aoc::Part, answ: &str) -> Result<()> {
    println!("Submitting: {} for part {:?}", answ, part);
    puzzle.submit_answer(part, answ)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new2021(9999)?;
    let data = puzzle.get_data()?;

    let answ1 = part1(data);
    //submit(&mut puzzle, aoc::Part::One, &answ1)?;
    //let answ2 = part2(data);
    //submit(&mut puzzle, aoc::Part::Two, &answ2)?;

    Ok(())
}
