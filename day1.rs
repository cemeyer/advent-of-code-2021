#![allow(dead_code, unused_imports, unused_variables)]

use anyhow::{anyhow, Result};

fn part1(input: &str) {
    let mut first = true;
    let mut prev = None;
    let mut total = 0;

    for line in input.lines() {
        let depth = line.parse::<i64>().unwrap();

        if first {
            first = false;
            prev = Some(depth);
            continue;
        }
        if depth > prev.unwrap() {
            total += 1;
        }

        prev = Some(depth);
    }

    println!("{}", total);
}

fn part2(input: &str) {
    let mut first = true;
    let mut prev = None;
    let mut total = 0;

    let depths = input.lines().map(|line| {
        line.parse::<i64>().unwrap()
    }).collect::<Vec<_>>();

    for window in depths.windows(3) {
        let sum = window[0] + window[1] + window[2];

        if first {
            first = false;
            prev = Some(sum);
            continue;
        }
        if sum > prev.unwrap() {
            total += 1;
        }

        prev = Some(sum);
    }

    println!("{}", total);
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new2021(1)?;
    let data = puzzle.get_data()?;

    //part1(data);
    part2(data);

    Ok(())
}
