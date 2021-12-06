#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::cmp::{min, max};
use std::convert::TryFrom;
use std::collections::*;
use std::hash::Hash;

fn day(wheel: &mut[u64; 9], day: usize) {
    // first, process 7-8 day countdowns:
    //wheel[(day + 6) % 7] += wheel[7];
    //wheel[7] = wheel[8];

    let spawn = wheel[day];
    wheel[day] += wheel[7];
    wheel[7] = wheel[8];
    // Then, spawn the current day.
    wheel[8] = spawn;
}

fn print_wheel(wheel: &[u64; 9], day: usize) {
    let mut x = Vec::new();

    for days in 0..7 {
        let y = wheel[(day + days) % 7];
        x.push((days, y));
    }
    x.push((7, wheel[7]));
    x.push((8, wheel[8]));
    println!("{}: {:?}", day, x);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sixseven() {
        let mut wheel = [0; 9];
        wheel[2] = 1;
        wheel[3] = 2;
        wheel[4] = 1;
        wheel[1] = 1;
        wheel[8] = 1;
        day(&mut wheel, 2);
        assert_eq!(&wheel, &[0, 1, 1, 2, 1, 0, 0, 1, 1]);
    }
}

fn part1(input: &str, days: usize) -> u64 {
    let mut wheel = [0; 9];

    for fish in input.trim_end().split(',').map(|w| w.parse::<usize>().unwrap()) {
        assert!(fish > 0 && fish < 7);
        wheel[fish] += 1;
    }

    let mut dayi = 0;
    for _i in 0..days {
        day(&mut wheel, dayi);
        dayi = (dayi + 1) % 7;

        //dbg!(_i + 1, &wheel);
        print_wheel(&wheel, dayi);
    }

    wheel.iter().sum()
}

#[cfg(test)]
mod test2 {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(part1("3,4,3,1,2", 1), 5);
        assert_eq!(part1("3,4,3,1,2", 2), 6);
        assert_eq!(part1("3,4,3,1,2", 3), 7);
        assert_eq!(part1("3,4,3,1,2", 4), 9);
        assert_eq!(part1("3,4,3,1,2", 5), 10);

        assert_eq!(part1("3,4,3,1,2", 8), 10);
        assert_eq!(part1("3,4,3,1,2", 9), 11);
        assert_eq!(part1("3,4,3,1,2", 10), 12);

        assert_eq!(part1("3,4,3,1,2", 18), 26);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part1("3,4,3,1,2", 256), 26984457539);
    }
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 6)?;
    let data = puzzle.get_data()?;

    let answ1 = part1(data, 80);
    dbg!(&answ1);
    assert!(answ1 < 606943); // TooHigh
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    let answ2 = part1(data, 256);
    dbg!(&answ2);
    puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
