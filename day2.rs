#![allow(dead_code, unused_imports, unused_variables)]

use anyhow::{anyhow, Result};

fn part1(input: &str) -> String {
    let mut hor = 0;
    let mut depth = 0;
    for line in input.lines() {
        let mut vals = line.split(' ');
        let cmd = vals.next().unwrap();
        let n = vals.next().unwrap().parse::<i64>().unwrap();
        match cmd {
            "forward" => {
                hor += n;
            }
            "down" => {
                depth += n;
            }
            "up" => {
                depth -= n;
            }
            _ => {
                panic!();
            }
        }
    }
    dbg!(hor, depth);

    format!("{}", hor*depth)
}

fn part2(input: &str) -> String {
    let mut hor = 0;
    let mut depth = 0;
    let mut aim = 0;

    for line in input.lines() {
        let mut vals = line.split(' ');
        let cmd = vals.next().unwrap();
        let n = vals.next().unwrap().parse::<i64>().unwrap();
        match cmd {
            "forward" => {
                hor += n;
                depth += aim * n;
            }
            "down" => {
                aim += n;
            }
            "up" => {
                aim -= n;
            }
            _ => {
                panic!();
            }
        }
    }
    dbg!(hor, depth);

    format!("{}", hor*depth)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new2021(2)?;
    let data = puzzle.get_data()?;

    //println!("{}", part1(data));
    println!("{}", part2(data));

    Ok(())
}
