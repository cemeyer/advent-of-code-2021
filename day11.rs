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

fn step(grid: &mut Vec<Vec<u8>>) -> HashSet<(usize, usize)> {
    let mut flashes = HashSet::new();

    for (r, row) in grid.iter_mut().enumerate() {
        for (c, octopus) in row.iter_mut().enumerate() {
            *octopus += 1;
        }
    }

    let mut any = true;
    while any {
        any = false;
        for r in 0..grid.len() {
            for c in 0..grid[0].len() {
                let octo = grid[r][c];
                if octo > 9 && !flashes.contains(&(r, c)) {
                    any = true;
                    flashes.insert((r, c));
                    for dr in -1i64..=1 {
                        let rr = r as i64 + dr;
                        if rr < 0 || rr >= grid.len() as i64 {
                            continue;
                        }
                        for dc in -1i64..=1 {
                            let cc = c as i64 +dc;
                            if cc < 0 || cc >= grid[0].len() as i64 {
                                continue;
                            }
                            if dc == 0 && dr == 0 {
                                continue;
                            }
                            grid[rr as usize][cc as usize] += 1;
                        }
                    }
                }
            }
        }
    }

    for (r, row) in grid.iter_mut().enumerate() {
        for (c, octo) in row.iter_mut().enumerate() {
            if *octo > 9 {
                *octo = 0;
            }
        }
    }

    flashes
}

fn part1(grid: &mut Vec<Vec<u8>>) -> usize {
    let mut totalflash = 0;
    for _ in 0..100 {
        let flashes = step(grid);
        totalflash += flashes.len();
    }

    totalflash
}

fn part2(grid: &mut Vec<Vec<u8>>) -> u64 {
    let mut n = 0;
    loop {
        let flashes = step(grid);
        n += 1;
        if flashes.len() == 100 {
            break;
        }
    }

    n
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 11)?;
    let data = puzzle.get_data()?;
//    let data =
//"5483143223
//2745854711
//5264556173
//6141336146
//6357385478
//4167524645
//2176841721
//6882881134
//4846848554
//5283751526";
    let lines = data.lines().collect::<Vec<_>>();
    let grid = lines.iter().map(|line| {
        line.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut grid1 = grid.clone();
    let answ1 = part1(&mut grid1);
    dbg!(&answ1);
    assert_eq!(answ1, 1747);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    let mut grid2 = grid.clone();
    let answ2 = part2(&mut grid2);
    dbg!(&answ2);
    assert_eq!(answ2, 505);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
